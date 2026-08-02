use crate::market::{hk_eastmoney_code, parse_eastmoney_klines, OhlcvRow};
use reqwest::Client;

/// 获取港股 OHLCV 数据（东方财富 API）
///
/// 直接调用东方财富 push2his 港股接口：
/// - secid 固定前缀 116，代码为 5 位零填充（`0700.HK` -> `116.00700`）
/// - API 返回全量数据，客户端按日期过滤
/// - klines 字段顺序同A股但无最后的股票代码列
pub async fn fetch_hk_ohlcv(
    client: &Client,
    symbol: &str,
    start: &str,
    end: &str,
) -> Result<Vec<OhlcvRow>, String> {
    // 日期转为 YYYYMMDD 格式
    let beg = start.replace('-', "");
    let end_fmt = end.replace('-', "");
    // 港股 secid 必须 5 位零填充，否则东方财富返回 data:null
    let code = hk_eastmoney_code(symbol);
    let secid = format!("116.{code}");

    let Some((klines, _)) = crate::market::fetch_eastmoney_kline(
        client,
        &secid,
        &beg,
        &end_fmt,
        Some("lmt=1000000"),
        3,
    )
    .await
    .map_err(|e| format!("东方财富港股 API 请求失败({}): {}", symbol, e))?
    else {
        return Err(format!("东方财富港股返回无 klines 数据: {}", symbol));
    };

    // 港股 API 返回全量数据，需按日期过滤（兜底：服务端已传 beg/end，但仍客户端校验）
    Ok(parse_eastmoney_klines(&klines, Some((&beg, &end_fmt))))
}

#[cfg(test)]
mod tests {
    use super::*;

    // 验证 `.HK` 后缀 + 4 位代码（文档主用例 0700.HK）能取到港股行情。
    // 修复前 secid=116.0700.HK -> data:null；修复后 secid=116.00700。
    #[tokio::test]
    #[ignore = "hits the live Eastmoney API"]
    async fn test_live_hk_ohlcv_0700hk() {
        let client = crate::http::build_client().unwrap();
        let rows = fetch_hk_ohlcv(&client, "0700.HK", "2026-06-01", "2026-07-31")
            .await
            .expect("0700.HK 港股行情应返回数据");
        assert!(!rows.is_empty(), "0700.HK 行情不应为空");
    }

    // 5 位无后缀代码同样应工作（09988 -> secid 116.09988）。
    #[tokio::test]
    #[ignore = "hits the live Eastmoney API"]
    async fn test_live_hk_ohlcv_09988() {
        let client = crate::http::build_client().unwrap();
        let rows = fetch_hk_ohlcv(&client, "09988", "2026-06-01", "2026-07-31")
            .await
            .expect("09988 港股行情应返回数据");
        assert!(!rows.is_empty(), "09988 行情不应为空");
    }
}
