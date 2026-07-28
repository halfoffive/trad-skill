use crate::http::{build_client, get_with_retry};
use crate::market::{parse_eastmoney_klines, OhlcvRow};

/// 获取港股 OHLCV 数据（东方财富 API）
///
/// 直接调用东方财富 push2his 港股接口：
/// - secid 固定前缀 116
/// - API 返回全量数据，客户端按日期过滤
/// - klines 字段顺序同A股但无最后的股票代码列
pub async fn fetch_hk_ohlcv(symbol: &str, start: &str, end: &str) -> Result<Vec<OhlcvRow>, String> {
    // 日期转为 YYYYMMDD 格式
    let beg = start.replace('-', "");
    let end_fmt = end.replace('-', "");

    let url = format!(
        "https://33.push2his.eastmoney.com/api/qt/stock/kline/get?\
         secid=116.{}\
         &fields1=f1,f2,f3,f4,f5,f6\
         &fields2=f51,f52,f53,f54,f55,f56,f57,f58,f59,f60,f61\
         &klt=101&fqt=1\
         &end=20500000&lmt=1000000",
        symbol
    );

    let client = build_client().map_err(|e| format!("构建 HTTP 客户端失败: {}", e))?;
    let resp = get_with_retry(&client, &url, Some(3))
        .await
        .map_err(|e| format!("东方财富港股 API 请求失败({}): {}", symbol, e))?;
    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let root: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("JSON 解析失败: {}", e))?;

    let data = root
        .get("data")
        .ok_or_else(|| format!("东方财富港股返回无 data 字段: {}", symbol))?;
    let klines = data
        .get("klines")
        .and_then(|k| k.as_array())
        .ok_or_else(|| format!("东方财富港股返回无 klines 数据: {}", symbol))?;

    // 港股 API 返回全量数据，需按日期过滤
    Ok(parse_eastmoney_klines(klines, Some((&beg, &end_fmt))))
}
