use crate::market::OhlcvRow;
use crate::market::us;

/// 获取加密货币 OHLCV 数据
///
/// 加密货币复用 Yahoo Finance API（如 BTC-USD），
/// 与美股逻辑完全相同，直接委托给 us.rs。
pub async fn fetch_crypto_ohlcv(symbol: &str, start: &str, end: &str) -> Result<Vec<OhlcvRow>, String> {
    us::fetch_us_ohlcv(symbol, start, end).await
}
