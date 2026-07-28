use crate::http::{build_client, get_with_retry};
use crate::market::OhlcvRow;
use chrono::NaiveDate;
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

    // 检查是否有错误信息
    if let Some(err) = root.get("chart").and_then(|c| c.get("error")) {
        let desc = err
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("未知错误");
        return Err(format!("Yahoo Finance 错误({}): {}", symbol, desc));
    }

    let result = root
        .get("chart")
        .and_then(|c| c.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| format!("Yahoo Finance 返回数据为空: {}", symbol))?;

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

/// 获取美股/加密货币 OHLCV 数据（Yahoo Finance v8 API）
///
/// 直接调用 yfinance 底层协议：
/// `GET https://query1.finance.yahoo.com/v8/finance/chart/{symbol}`
///
/// 如果直接调用失败，尝试先获取 cookie 再重试。
/// 日期参数从 YYYY-MM-DD 转为 Unix 时间戳。
/// 返回错误字符串而非 panic（对齐 Python "never raises" 契约）。
pub async fn fetch_us_ohlcv(symbol: &str, start: &str, end: &str) -> Result<Vec<OhlcvRow>, String> {
    let period1 = date_to_unix(start)?;
    let period2 = date_to_unix(end)?;

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval=1d",
        symbol, period1, period2
    );

    let client = build_client().map_err(|e| format!("构建 HTTP 客户端失败: {}", e))?;

    // 第一次尝试：直接请求
    match get_with_retry(&client, &url, Some(2)).await {
        Ok(resp) => {
            let body = resp
                .text()
                .await
                .map_err(|e| format!("读取响应失败: {}", e))?;
            parse_yahoo_response(symbol, &body)
        }
        Err(_) => {
            // 第二次尝试：先访问 fc.yahoo.com 获取 cookie，再请求数据
            let cookie_url = "https://fc.yahoo.com";
            let _ = client.get(cookie_url).send().await;

            let resp = get_with_retry(&client, &url, Some(2))
                .await
                .map_err(|e| format!("Yahoo Finance 请求失败({}): {}", symbol, e))?;
            let body = resp
                .text()
                .await
                .map_err(|e| format!("读取响应失败: {}", e))?;
            parse_yahoo_response(symbol, &body)
        }
    }
}
