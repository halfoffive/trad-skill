use crate::market::{parse_eastmoney_klines, OhlcvRow};
use reqwest::Client;

/// 获取A股 OHLCV 数据（东方财富 API）
///
/// 直接调用东方财富 push2his 接口（akshare 底层协议）：
/// - market: 6开头=1(沪), 其他=0(深)
/// - klines 字段顺序: 日期,开盘,收盘,最高,最低,成交量,成交额,振幅,涨跌幅,涨跌额,换手率,股票代码
/// - 注意：收盘在开盘后面（与 Yahoo 不同）
pub async fn fetch_cn_ohlcv(
    client: &Client,
    symbol: &str,
    start: &str,
    end: &str,
) -> Result<Vec<OhlcvRow>, String> {
    // 判断市场：6xx/9xx（含 900xxx 沪B）→ 沪市(1)，其余（深市 0x/2x/3x、北交所 8xx/92x）→ 0
    let market = crate::market::cn_market_id(symbol);
    // 日期转为 YYYYMMDD 格式（main 已统一校验 YYYY-MM-DD）
    let beg = start.replace('-', "");
    let end_fmt = end.replace('-', "");

    let secid = format!("{market}.{symbol}");
    let Some((klines, _)) =
        crate::market::fetch_eastmoney_kline(client, &secid, &beg, &end_fmt, None, 3)
            .await
            .map_err(|e| format!("东方财富 A股 API 请求失败({}): {}", symbol, e))?
    else {
        return Err(format!(
            "东方财富返回无 klines 数据: {} (可能代码错误或无交易记录)",
            symbol
        ));
    };

    // A股成交量单位是手（1 手 = 100 股），统一换算为股，与 Yahoo 渠道（股）对齐，
    // 否则默认渠道（东方财富）的成交量比 Yahoo 渠道系统性小 100 倍。
    let rows = parse_eastmoney_klines(&klines, None);
    Ok(rows
        .into_iter()
        .map(|mut r| {
            r.volume *= 100.0;
            r
        })
        .collect())
}
