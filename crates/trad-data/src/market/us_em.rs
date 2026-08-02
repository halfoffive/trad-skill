use crate::http::get_with_retry;
use crate::market::{parse_eastmoney_klines, OhlcvRow};
use reqwest::Client;

/// 获取美股 OHLCV 数据（东方财富 API，Yahoo 区域封锁时的备用渠道）
///
/// 东方财富美股接口与 A股/港股同源（push2his），secid 前缀：
/// 105=NASDAQ, 106=NYSE, 107=AMEX。无交易所映射时依次尝试三个前缀，
/// 取第一个返回非空 klines 的结果。
/// 响应自带 market 字段（请求的交易所）时须与请求前缀一致；字段缺失时信任数据
/// （保持旧行为）。防止同名代码在多个交易所是不同的公司时静默取错公司。
fn market_matches(resp_market: Option<i64>, expected: Option<i64>) -> bool {
    match (resp_market, expected) {
        (Some(rm), Some(em)) => rm == em,
        _ => true,
    }
}

pub async fn fetch_us_ohlcv_eastmoney(
    client: &Client,
    symbol: &str,
    start: &str,
    end: &str,
) -> Result<Vec<OhlcvRow>, String> {
    let beg = start.replace('-', "");
    let end_fmt = end.replace('-', "");

    let mut last_err = String::new();
    for secid in super::us_eastmoney_secids(symbol) {
        let expected_market = secid.split('.').next().and_then(|p| p.parse::<i64>().ok());
        match fetch_one(client, &secid, &beg, &end_fmt).await {
            // 响应市场与请求前缀不符 → 命中的是另一交易所的同名代码，继续尝试下一个
            Ok((rows, m)) if !rows.is_empty() && market_matches(m, expected_market) => {
                return Ok(rows);
            }
            Ok(_) => last_err = format!("东方财富美股返回空数据({})", symbol),
            Err(e) => last_err = e,
        }
    }
    Err(format!(
        "东方财富美股 API 请求失败({}): {}",
        symbol, last_err
    ))
}

/// 单个 secid 的抓取。secid 无效时东方财富返回 `data:null`，
/// 此时返回空 Vec（而非 Err），让上层尝试下一个交易所前缀。
/// 同时返回响应的 `market` 字段（所属交易所），供上层做同名代码的交易所一致性校验。
async fn fetch_one(
    client: &Client,
    secid: &str,
    beg: &str,
    end_fmt: &str,
) -> Result<(Vec<OhlcvRow>, Option<i64>), String> {
    let url = format!(
        "https://push2his.eastmoney.com/api/qt/stock/kline/get?\
         fields1=f1,f2,f3,f4,f5,f6\
         &fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61,f116\
         &ut=7eea3edcaed734bea9cbfc24409ed989\
         &klt=101&fqt=1\
         &secid={}\
         &beg={}&end={}",
        secid, beg, end_fmt
    );

    let resp = get_with_retry(client, &url, Some(2))
        .await
        .map_err(|e| format!("东方财富美股 API 请求失败: {}", e))?;
    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let root: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("JSON 解析失败: {}", e))?;

    let data = root.get("data");
    let market = data.and_then(|d| d.get("market")).and_then(|m| m.as_i64());
    let klines = data
        .and_then(|d| d.get("klines"))
        .and_then(|k| k.as_array());

    match klines {
        Some(k) => Ok((parse_eastmoney_klines(k, None), market)),
        None => Ok((Vec::new(), market)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 真实网络集成测试：默认不进 CI，手动 `cargo test -- --ignored` 运行。
    #[tokio::test]
    #[ignore = "hits the live Eastmoney API"]
    async fn test_live_eastmoney_aapl() {
        let client = crate::http::build_client().unwrap();
        let rows = fetch_us_ohlcv_eastmoney(&client, "AAPL", "2024-01-01", "2024-06-30")
            .await
            .expect("东方财富 AAPL 应返回数据");
        assert!(!rows.is_empty(), "东方财富 AAPL 数据不应为空");
    }
}
