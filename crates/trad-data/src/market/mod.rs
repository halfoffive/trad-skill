pub mod cn;
pub mod crypto;
pub mod hk;
pub mod us;

/// 市场类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Market {
    US,
    CNStock,
    HKStock,
    Crypto,
}

/// OHLCV 数据行（各市场共享）
pub struct OhlcvRow {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// 根据 symbol 格式自动检测市场类型
///
/// 规则：
/// - `-USD` 后缀 → 加密货币
/// - `.HK` 后缀 → 港股
/// - 6位纯数字 → A股
/// - 5位纯数字 → 港股
/// - 其他 → 美股
pub fn detect_market(symbol: &str) -> Market {
    let s = symbol.trim();
    // -USD 后缀 → 加密货币
    if s.to_uppercase().ends_with("-USD") {
        return Market::Crypto;
    }
    // .HK 后缀 → 港股
    if s.to_uppercase().ends_with(".HK") {
        return Market::HKStock;
    }
    // 6位纯数字 → A股
    if s.len() == 6 && s.chars().all(|c| c.is_ascii_digit()) {
        return Market::CNStock;
    }
    // 5位纯数字 → 港股
    if s.len() == 5 && s.chars().all(|c| c.is_ascii_digit()) {
        return Market::HKStock;
    }
    // 其他 → 美股
    Market::US
}

/// 解析东方财富 klines 字符串数组为 OhlcvRow
///
/// klines 字段顺序: 日期,开盘,收盘,最高,最低,成交量,成交额,振幅,涨跌幅,涨跌额,换手率[,股票代码]
/// 注意：收盘在开盘后面（与 Yahoo 不同）
///
/// 可选日期过滤：传入 `date_range` 时仅保留 `[start, end] 范围内的行（YYYYMMDD 格式比较）
pub fn parse_eastmoney_klines(
    klines: &[serde_json::Value],
    date_range: Option<(&str, &str)>,
) -> Vec<OhlcvRow> {
    let mut rows = Vec::new();
    for line in klines {
        let s = match line.as_str() {
            Some(s) => s,
            None => continue,
        };
        let fields: Vec<&str> = s.split(',').collect();
        // 至少需要: 日期(0),开盘(1),收盘(2),最高(3),最低(4),成交量(5)
        if fields.len() < 6 {
            continue;
        }

        let date = fields[0].to_string();

        // 日期过滤（港股 API 返回全量数据时需要）
        // klines 日期格式为 YYYY-MM-DD，date_range 为 YYYYMMDD，需去掉短横线再比较
        if let Some((start, end)) = date_range {
            let date_compact = date.replace('-', "");
            if date_compact.as_str() < start || date_compact.as_str() > end {
                continue;
            }
        }

        let open: f64 = fields[1].parse().unwrap_or(0.0);
        let close: f64 = fields[2].parse().unwrap_or(0.0);
        let high: f64 = fields[3].parse().unwrap_or(0.0);
        let low: f64 = fields[4].parse().unwrap_or(0.0);
        let volume: f64 = fields[5].parse().unwrap_or(0.0);

        rows.push(OhlcvRow {
            date,
            open,
            high,
            low,
            close,
            volume,
        });
    }
    rows
}

/// 统一 OHLCV 数据获取入口，自动检测市场
pub async fn fetch_ohlcv(
    client: &reqwest::Client,
    symbol: &str,
    start: &str,
    end: &str,
) -> Result<Vec<OhlcvRow>, String> {
    match detect_market(symbol) {
        Market::US => us::fetch_us_ohlcv(client, symbol, start, end).await,
        Market::Crypto => crypto::fetch_crypto_ohlcv(client, symbol, start, end).await,
        Market::CNStock => cn::fetch_cn_ohlcv(client, symbol, start, end).await,
        Market::HKStock => hk::fetch_hk_ohlcv(client, symbol, start, end).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_us_stock() {
        assert_eq!(detect_market("AAPL"), Market::US);
        assert_eq!(detect_market("MSFT"), Market::US);
    }

    #[test]
    fn test_detect_cn_stock() {
        assert_eq!(detect_market("600519"), Market::CNStock);
        assert_eq!(detect_market("000001"), Market::CNStock);
    }

    #[test]
    fn test_detect_hk_stock() {
        assert_eq!(detect_market("00700.HK"), Market::HKStock);
        assert_eq!(detect_market("09988"), Market::HKStock);
    }

    #[test]
    fn test_detect_crypto() {
        assert_eq!(detect_market("BTC-USD"), Market::Crypto);
        assert_eq!(detect_market("ETH-USD"), Market::Crypto);
    }

    #[test]
    fn test_parse_eastmoney_klines_basic() {
        let klines = vec![
            serde_json::json!("2024-01-02,100.0,101.0,102.0,99.0,5000,12345,1.5,0.5,0.5,0.3"),
            serde_json::json!("2024-01-03,101.0,103.0,104.0,100.0,6000,23456,2.0,1.0,1.0,0.4"),
        ];
        let rows = parse_eastmoney_klines(&klines, None);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].date, "2024-01-02");
        assert!((rows[0].open - 100.0).abs() < 1e-10);
        assert!((rows[0].close - 101.0).abs() < 1e-10); // 收盘在开盘后面
        assert!((rows[0].high - 102.0).abs() < 1e-10);
        assert!((rows[0].low - 99.0).abs() < 1e-10);
        assert!((rows[0].volume - 5000.0).abs() < 1e-10);
    }

    #[test]
    fn test_parse_eastmoney_klines_date_filter() {
        let klines = vec![
            serde_json::json!("2024-01-01,10,11,12,9,100"),
            serde_json::json!("2024-01-15,20,21,22,19,200"),
            serde_json::json!("2024-02-01,30,31,32,29,300"),
        ];
        let rows = parse_eastmoney_klines(&klines, Some(("20240110", "20240120")));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].date, "2024-01-15");
    }

    #[test]
    fn test_parse_eastmoney_klines_skip_short() {
        let klines = vec![
            serde_json::json!("2024-01-01,10,11"), // 太少字段
            serde_json::json!("2024-01-02,10,11,12,9,100"),
        ];
        let rows = parse_eastmoney_klines(&klines, None);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn test_parse_eastmoney_klines_empty() {
        let rows = parse_eastmoney_klines(&[], None);
        assert!(rows.is_empty());
    }
}
