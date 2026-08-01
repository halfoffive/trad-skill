use crate::http::get_with_retry_headers;
use crate::market::OhlcvRow;
use chrono::NaiveDate;
use reqwest::Client;
use serde_json::Value;

/// 将 YYYY-MM-DD 转为 Unix 时间戳（UTC 零点）
fn date_to_unix(date_str: &str) -> Result<i64, String> {
    let d = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("日期解析失败 '{}': {}", date_str, e))?;
    let dt = d
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| format!("无效时间: '{}'", date_str))?;
    Ok(dt.and_utc().timestamp())
}

/// 解析 Yahoo Finance chart API 响应中的 OHLCV 数据
fn parse_yahoo_response(symbol: &str, body: &str) -> Result<Vec<OhlcvRow>, String> {
    let root: Value = serde_json::from_str(body).map_err(|e| format!("JSON 解析失败: {}", e))?;

    // 检查是否有错误信息。
    // 注意：chart.error 可能是 JSON null（区域封锁/缺 crumb 时常见），
    // 此时不视为显式错误，落到下方 "返回数据为空" 分支给出更明确的提示。
    if let Some(err) = root.get("chart").and_then(|c| c.get("error")) {
        if !err.is_null() {
            let code = err.get("code").and_then(|c| c.as_str());
            let desc = err.get("description").and_then(|d| d.as_str());
            let detail = match (code, desc) {
                (Some(c), Some(d)) => format!("{}: {}", c, d),
                (Some(c), None) => c.to_string(),
                (None, Some(d)) => d.to_string(),
                (None, None) => err.to_string(),
            };
            return Err(format!("Yahoo Finance 错误({}): {}", symbol, detail));
        }
    }

    let result = root
        .get("chart")
        .and_then(|c| c.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| {
            format!(
                "Yahoo Finance 返回数据为空({}): 可能 symbol 无效，或该地区访问受限（可尝试 --source eastmoney）",
                symbol
            )
        })?;

    let timestamps = result
        .get("timestamp")
        .and_then(|t| t.as_array())
        .ok_or("缺少 timestamp 字段")?;

    let indicators = result.get("indicators").ok_or("缺少 indicators 字段")?;

    let quote = indicators
        .get("quote")
        .and_then(|q| q.as_array())
        .and_then(|arr| arr.first())
        .ok_or("缺少 quote 数据")?;

    let opens = quote
        .get("open")
        .and_then(|v| v.as_array())
        .ok_or("缺少 open")?;
    let highs = quote
        .get("high")
        .and_then(|v| v.as_array())
        .ok_or("缺少 high")?;
    let lows = quote
        .get("low")
        .and_then(|v| v.as_array())
        .ok_or("缺少 low")?;
    let closes = quote
        .get("close")
        .and_then(|v| v.as_array())
        .ok_or("缺少 close")?;
    let volumes = quote
        .get("volume")
        .and_then(|v| v.as_array())
        .ok_or("缺少 volume")?;

    let mut rows = Vec::new();
    for (i, ts_val) in timestamps.iter().enumerate() {
        // 提取时间戳并转为日期字符串
        let ts = ts_val.as_i64().unwrap_or(0);
        let date = chrono::DateTime::from_timestamp(ts, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        // 辅助函数：从 JSON 数组中取 f64 值，null 则跳过
        let get_f64 =
            |arr: &Vec<Value>, idx: usize| -> Option<f64> { arr.get(idx).and_then(|v| v.as_f64()) };

        // 如果关键字段为 null（停牌日），跳过该行
        let (Some(o), Some(h), Some(l), Some(c)) = (
            get_f64(opens, i),
            get_f64(highs, i),
            get_f64(lows, i),
            get_f64(closes, i),
        ) else {
            continue;
        };

        let vol = get_f64(volumes, i).unwrap_or(0.0);

        rows.push(OhlcvRow {
            date,
            open: o,
            high: h,
            low: l,
            close: c,
            volume: vol,
        });
    }

    Ok(rows)
}

/// 浏览器 User-Agent：Yahoo Finance 会封锁非浏览器 UA（尤其数据中心 IP），
/// 所有 Yahoo 请求必须携带真实浏览器 UA。
const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// 简单的 URL 编码（用于 crumb 参数，镜像 news.rs 的同名实现）
fn url_encode(s: &str) -> String {
    let mut result = String::new();
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*b as char);
            }
            b' ' => result.push_str("%20"),
            _ => result.push_str(&format!("%{:02X}", b)),
        }
    }
    result
}

/// 获取 Yahoo Finance crumb token（yfinance 底层反爬握手）
///
/// 流程：先访问 fc.yahoo.com 种 cookie（cookie store 自动保存），
/// 再用同一 cookie 请求 v1/test/getcrumb 取得 crumb 字符串。
/// 任一步失败返回 None（调用方仍可尝试无 crumb 请求）。
async fn get_crumb(client: &Client) -> Option<String> {
    // 种 cookie（响应状态码无关紧要，忽略结果）
    let _ = client
        .get("https://fc.yahoo.com")
        .header("User-Agent", BROWSER_UA)
        .send()
        .await;

    let resp = client
        .get("https://query2.finance.yahoo.com/v1/test/getcrumb")
        .header("User-Agent", BROWSER_UA)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let crumb = resp.text().await.ok()?;
    let crumb = crumb.trim();
    if crumb.is_empty() {
        None
    } else {
        Some(crumb.to_string())
    }
}

/// 单次 Yahoo chart 请求（query2 端点 + 浏览器 UA，可选 crumb）
async fn fetch_yahoo_chart(
    client: &Client,
    symbol: &str,
    period1: i64,
    period2: i64,
    crumb: Option<&str>,
) -> Result<Vec<OhlcvRow>, String> {
    let mut url = format!(
        "https://query2.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval=1d",
        symbol, period1, period2
    );
    if let Some(c) = crumb {
        url.push_str(&format!("&crumb={}", url_encode(c)));
    }

    let resp = get_with_retry_headers(client, &url, &[("User-Agent", BROWSER_UA)], Some(2))
        .await
        .map_err(|e| format!("Yahoo Finance 请求失败({}): {}", symbol, e))?;
    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;
    parse_yahoo_response(symbol, &body)
}

/// 获取美股/加密货币 OHLCV 数据（Yahoo Finance v8 API）
///
/// 调用 yfinance 底层协议：
/// `GET https://query2.finance.yahoo.com/v8/finance/chart/{symbol}`
///
/// 两步策略：
/// 1. 直连 query2（浏览器 UA）——多数地区一次成功；
/// 2. 若返回空或错误（数据中心 IP / 区域封锁时 Yahoo 常返回 HTTP 200 但
///    `chart.error=null`），走 cookie + crumb 握手后带 crumb 重试。
///
/// 日期参数从 YYYY-MM-DD 转为 Unix 时间戳。
/// 返回错误字符串而非 panic（对齐 Python "never raises" 契约）。
pub async fn fetch_us_ohlcv(
    client: &Client,
    symbol: &str,
    start: &str,
    end: &str,
) -> Result<Vec<OhlcvRow>, String> {
    let period1 = date_to_unix(start)?;
    let period2 = date_to_unix(end)?;

    // 第一步：直连（无 crumb）。非空结果直接返回。
    if let Ok(rows) = fetch_yahoo_chart(client, symbol, period1, period2, None).await {
        if !rows.is_empty() {
            return Ok(rows);
        }
    }

    // 第二步：cookie + crumb 握手后重试。
    let crumb = get_crumb(client).await;
    fetch_yahoo_chart(client, symbol, period1, period2, crumb.as_deref())
        .await
        .map_err(|e| {
            format!(
                "{}（若所在地区无法访问 Yahoo，可改用 --source eastmoney）",
                e
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("AAPL"), "AAPL");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(url_encode("a+b/c"), "a%2Bb%2Fc");
    }

    #[test]
    fn test_parse_yahoo_happy_path() {
        let body = r#"{
            "chart": {
                "result": [{
                    "timestamp": [1704153600, 1704240000],
                    "indicators": {
                        "quote": [{
                            "open": [100.0, 101.0],
                            "high": [102.0, 103.0],
                            "low": [99.0, 100.0],
                            "close": [101.0, 102.0],
                            "volume": [5000, 6000]
                        }]
                    }
                }],
                "error": null
            }
        }"#;
        let rows = parse_yahoo_response("AAPL", body).unwrap();
        assert_eq!(rows.len(), 2);
        assert!((rows[0].close - 101.0).abs() < 1e-9);
        assert!((rows[1].volume - 6000.0).abs() < 1e-9);
    }

    #[test]
    fn test_parse_yahoo_error_with_code_and_desc() {
        let body = r#"{"chart":{"result":null,"error":{"code":"Bad Request","description":"Invalid symbol"}}}"#;
        let err = parse_yahoo_response("XXX", body).unwrap_err();
        assert!(err.contains("Bad Request"));
        assert!(err.contains("Invalid symbol"));
        assert!(!err.contains("未知错误"));
    }

    #[test]
    fn test_parse_yahoo_error_code_only() {
        // 只有 code 没有 description —— 旧实现会输出 "未知错误"，现在应显示 code。
        let body = r#"{"chart":{"result":null,"error":{"code":"Too Many Requests"}}}"#;
        let err = parse_yahoo_response("AAPL", body).unwrap_err();
        assert!(err.contains("Too Many Requests"));
        assert!(!err.contains("未知错误"));
    }

    #[test]
    fn test_parse_yahoo_null_error_empty_result() {
        // chart.error 为 null 且 result 为 null：区域封锁/缺 crumb 的典型响应。
        // 不应再报 "未知错误"，而是 "返回数据为空" 提示。
        let body = r#"{"chart":{"result":null,"error":null}}"#;
        let err = parse_yahoo_response("AAPL", body).unwrap_err();
        assert!(err.contains("返回数据为空"));
        assert!(!err.contains("未知错误"));
    }

    #[test]
    fn test_parse_yahoo_skips_null_ohlc() {
        // 停牌日 OHLC 为 null 应跳过该行
        let body = r#"{
            "chart": {
                "result": [{
                    "timestamp": [1704153600, 1704240000],
                    "indicators": {
                        "quote": [{
                            "open": [100.0, null],
                            "high": [102.0, null],
                            "low": [99.0, null],
                            "close": [101.0, null],
                            "volume": [5000, null]
                        }]
                    }
                }],
                "error": null
            }
        }"#;
        let rows = parse_yahoo_response("AAPL", body).unwrap();
        assert_eq!(rows.len(), 1);
    }

    // 真实网络集成测试：默认不进 CI，手动 `cargo test -- --ignored` 运行。
    #[tokio::test]
    #[ignore = "hits the live Yahoo Finance API"]
    async fn test_live_yahoo_aapl() {
        let client = crate::http::build_client().unwrap();
        let rows = fetch_us_ohlcv(&client, "AAPL", "2024-01-01", "2024-06-30")
            .await
            .expect("Yahoo AAPL 应返回数据");
        assert!(!rows.is_empty(), "Yahoo AAPL 数据不应为空");
    }
}
