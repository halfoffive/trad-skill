/// 基本面数据获取模块
///
/// 支持美股（Yahoo Finance v10 API）和A股（东方财富 API）的基本面数据获取。
/// 对齐 Python fetch_fundamentals.py 的输出格式。
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

use crate::http::get_with_retry;
use crate::market::{detect_market, Market};
use crate::yahoo::yahoo_get_body;

// ───────────────────────── 工具函数 ─────────────────────────

/// 格式化数值：缺失返回 "N/A"，否则 round 到 2 位。
///
/// Yahoo quoteSummary 的多数指标是 `{"fmt":"466.82B","raw":466822987776}` 对象，
/// 优先取 `fmt`（已格式化的字符串），否则回退到 `raw` 数值。
fn fmt_num(v: Option<&Value>) -> String {
    match v {
        None => "N/A".to_string(),
        Some(Value::Null) => "N/A".to_string(),
        Some(Value::Object(_)) => {
            // Yahoo {fmt, raw} 对象：优先 fmt，其次 raw
            if let Some(fmt) = v.and_then(|o| o.get("fmt")).and_then(|f| f.as_str()) {
                if !fmt.is_empty() {
                    return fmt.to_string();
                }
            }
            match v.and_then(|o| o.get("raw")) {
                Some(raw) => fmt_num(Some(raw)),
                None => "N/A".to_string(),
            }
        }
        Some(Value::Number(n)) => {
            if let Some(f) = n.as_f64() {
                // 大数值（如市值）直接显示原始值，小数保留 2 位
                if f == f.round() && f.abs() > 1e6 {
                    format!("{}", n)
                } else {
                    format!("{:.2}", f)
                }
            } else {
                n.to_string()
            }
        }
        Some(Value::String(s)) => {
            if s.is_empty() || s == "N/A" {
                "N/A".to_string()
            } else {
                s.clone()
            }
        }
        Some(v) => v.to_string(),
    }
}

/// A股财务指标格式类型
enum CnFmt {
    /// 金额（元）：按亿/万换算，便于阅读
    Amount,
    /// 普通数值（每股指标、比率、同比等）：保留 2 位小数
    Num,
}

/// A股金额格式化：≥1亿用「X.XX亿」，≥1万用「X.XX万」，否则保留 2 位小数。
/// 缺失返回 "N/A"。
fn fmt_cn_amount(v: Option<&Value>) -> String {
    match v {
        Some(Value::Number(n)) => match n.as_f64() {
            Some(f) => {
                let a = f.abs();
                if a >= 1e8 {
                    format!("{:.2}亿", f / 1e8)
                } else if a >= 1e4 {
                    format!("{:.2}万", f / 1e4)
                } else {
                    format!("{:.2}", f)
                }
            }
            None => fmt_num(v),
        },
        _ => fmt_num(v),
    }
}

/// 从嵌套 JSON 中按路径取值，如 get_nested(val, &["result", "0", "assetProfile", "sector"])
fn get_nested<'a>(val: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cur = val;
    for &key in path {
        cur = if let Ok(idx) = key.parse::<usize>() {
            cur.get(idx)?
        } else {
            cur.get(key)?
        };
    }
    Some(cur)
}

/// 计算同比变化百分比
fn calc_yoy(cur: Option<f64>, prev: Option<f64>) -> String {
    match (cur, prev) {
        (Some(c), Some(p)) if p != 0.0 => format!("{:.2}%", (c / p - 1.0) * 100.0),
        _ => "N/A".to_string(),
    }
}

// ───────────────────────── 美股基本面 ─────────────────────────

/// 获取美股基本面数据（Yahoo Finance v10 API）
async fn fetch_us_fundamentals(client: &Client, symbol: &str) -> String {
    let mut sections = Vec::new();
    sections.push(format!("# {} 基本面（精简）\n", symbol));

    // 构建 quoteSummary 请求 URL（公司概况所需模块；财务报表改用 time-series 接口）
    let modules = "assetProfile,financialData,defaultKeyStats,price,summaryDetail";
    let url = format!(
        "https://query2.finance.yahoo.com/v10/finance/quoteSummary/{}?modules={}",
        symbol, modules
    );

    let resp_text = match yahoo_get_body(client, &url).await {
        Ok(b) => b,
        Err(e) => {
            return format!(
                "错误: 获取 {} 基本面数据失败 - {}（Yahoo 在部分地区不可达；A股请用 6 位代码，自动走东方财富）",
                symbol, e
            )
        }
    };

    let body: Value = match serde_json::from_str(&resp_text) {
        Ok(v) => v,
        Err(e) => return format!("错误: 解析响应失败 - {}", e),
    };

    // 检查是否有结果
    let result = match get_nested(&body, &["quoteSummary", "result"]) {
        Some(Value::Array(arr)) if !arr.is_empty() => &arr[0],
        _ => {
            let err_msg = body
                .get("quoteSummary")
                .and_then(|q| q.get("error"))
                .and_then(|e| e.get("description"))
                .and_then(|d| d.as_str())
                .unwrap_or("未知错误");
            return format!("错误: {} 基本面数据不可用 - {}", symbol, err_msg);
        }
    };

    // ── 公司概况 ──
    sections.push("## 公司概况\n".to_string());
    let profile = result.get("assetProfile").unwrap_or(&Value::Null);
    let fin_data = result.get("financialData").unwrap_or(&Value::Null);

    // 公司名称：longName → price.longName → shortName → symbol 兜底
    let company_name = {
        let n = first_available(
            result,
            &[
                ("assetProfile", "longName"),
                ("price", "longName"),
                ("assetProfile", "shortName"),
            ],
        );
        if n == "N/A" {
            symbol.to_string()
        } else {
            n
        }
    };

    let fields = [
        ("公司名称", company_name),
        (
            "行业",
            format!(
                "{} / {}",
                fmt_num(profile.get("sector")),
                fmt_num(profile.get("industry"))
            ),
        ),
        (
            "市值",
            first_available(
                result,
                &[("price", "marketCap"), ("defaultKeyStats", "marketCap")],
            ),
        ),
        (
            "市盈率(PE)",
            first_available(
                result,
                &[
                    ("summaryDetail", "trailingPE"),
                    ("defaultKeyStats", "trailingPE"),
                ],
            ),
        ),
        (
            "市净率(PB)",
            first_available(
                result,
                &[
                    ("defaultKeyStats", "priceToBook"),
                    ("summaryDetail", "priceToBook"),
                ],
            ),
        ),
        ("ROE", fmt_num(fin_data.get("returnOnEquity"))),
        ("总营收", fmt_num(fin_data.get("totalRevenue"))),
        ("利润率", fmt_num(fin_data.get("profitMargins"))),
    ];
    for (k, v) in &fields {
        sections.push(format!("- **{}**: {}", k, v));
    }
    sections.push(String::new());

    // ── 关键财务指标表（Yahoo fundamentals-timeseries API）──
    // quoteSummary 的报表模块（incomeStatementHistory 等）常返回空，改用 time-series 接口。
    sections.push("## 关键财务指标（最近年度）\n".to_string());
    sections.push(fetch_us_financial_table(client, symbol).await);

    sections.join("\n")
}

/// 依次尝试多个 (section, key)，返回第一个非空且非 "N/A" 的格式化值。
fn first_available(result: &Value, paths: &[(&str, &str)]) -> String {
    for (section, key) in paths {
        if let Some(val) = result.get(section).and_then(|s| s.get(key)) {
            if !val.is_null() {
                let formatted = fmt_num(Some(val));
                if formatted != "N/A" {
                    return formatted;
                }
            }
        }
    }
    "N/A".to_string()
}

/// 美股年度财务指标表（Yahoo fundamentals-timeseries API）。
///
/// quoteSummary 的报表模块（incomeStatementHistory / balanceSheetHistory /
/// cashflowStatementHistory）现已常返回空数组，改用 yfinance 同款的
/// fundamentals-timeseries 接口按年度取数。同样需要 crumb 握手。
async fn fetch_us_financial_table(client: &Client, symbol: &str) -> String {
    // 指标：(显示名, time-series 年度类型)
    let metrics: [(&str, &str); 9] = [
        ("营收", "annualTotalRevenue"),
        ("净利润", "annualNetIncome"),
        ("摊薄EPS", "annualDilutedEPS"),
        ("毛利", "annualGrossProfit"),
        ("总资产", "annualTotalAssets"),
        ("总负债", "annualTotalDebt"),
        ("股东权益", "annualStockholdersEquity"),
        ("经营现金流", "annualOperatingCashFlow"),
        ("自由现金流", "annualFreeCashFlow"),
    ];
    let types: Vec<&str> = metrics.iter().map(|(_, t)| *t).collect();

    let now = chrono::Utc::now();
    let period2 = now.timestamp();
    let period1 = (now - chrono::Duration::days(365 * 5)).timestamp();
    let url = format!(
        "https://query2.finance.yahoo.com/ws/fundamentals-timeseries/v1/finance/timeseries/{}?symbol={}&type={}&period1={}&period2={}",
        symbol,
        symbol,
        types.join(","),
        period1,
        period2
    );

    let resp_text = match yahoo_get_body(client, &url).await {
        Ok(b) => b,
        Err(e) => return format!("> 财务指标获取失败: {}\n", e),
    };
    let body: Value = match serde_json::from_str(&resp_text) {
        Ok(v) => v,
        Err(e) => return format!("> 财务指标解析失败: {}\n", e),
    };

    build_us_timeseries_table(&body, &metrics)
}

/// 解析 fundamentals-timeseries 响应为按年表格。
///
/// 响应结构：`timeseries.result[]` 每项含 `meta.type[0]`、`timestamp[]`，
/// 以及以类型名为键的数组（元素为 `{reportedValue:{raw,fmt}}` 或 null）。
fn build_us_timeseries_table(body: &Value, metrics: &[(&str, &str)]) -> String {
    let results = body
        .get("timeseries")
        .and_then(|t| t.get("result"))
        .and_then(|r| r.as_array());
    let results = match results {
        Some(r) if !r.is_empty() => r,
        _ => return "> 无年度财务数据\n".to_string(),
    };

    // 类型名 → { 年份 → (raw, fmt) }
    let mut by_type: HashMap<String, HashMap<i64, (Option<f64>, String)>> = HashMap::new();
    let mut years: Vec<i64> = Vec::new();

    for entry in results {
        let type_name = entry
            .get("meta")
            .and_then(|m| m.get("type"))
            .and_then(|t| t.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if type_name.is_empty() {
            continue;
        }
        let (Some(ts_arr), Some(val_arr)) = (
            entry.get("timestamp").and_then(|t| t.as_array()),
            entry.get(&type_name).and_then(|v| v.as_array()),
        ) else {
            continue;
        };

        let mut year_map: HashMap<i64, (Option<f64>, String)> = HashMap::new();
        for (ts, val) in ts_arr.iter().zip(val_arr.iter()) {
            let Some(sec) = ts.as_i64() else { continue };
            let Some(dt) = chrono::DateTime::from_timestamp(sec, 0) else {
                continue;
            };
            let Ok(y) = dt.format("%Y").to_string().parse::<i64>() else {
                continue;
            };
            let reported = val.get("reportedValue");
            let raw = reported.and_then(|r| r.get("raw")).and_then(|r| r.as_f64());
            let fmt = reported
                .and_then(|r| r.get("fmt"))
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            if !years.contains(&y) {
                years.push(y);
            }
            year_map.insert(y, (raw, fmt));
        }
        by_type.insert(type_name, year_map);
    }

    years.sort_unstable();
    years.reverse();
    years.truncate(4);
    if years.is_empty() {
        return "> 无年度财务数据\n".to_string();
    }

    let ncols = years.len();
    let year_labels: Vec<String> = years.iter().map(|y| y.to_string()).collect();

    let mut lines = Vec::new();
    lines.push(format!(
        "| 指标 | {} | YoY(营收/净利) |",
        year_labels.join(" | ")
    ));
    lines.push(format!("|{}|", "---|".repeat(ncols + 2)));

    for (display, type_name) in metrics {
        let year_map = by_type.get(*type_name);
        let mut cells = vec![display.to_string()];
        let mut raws: Vec<Option<f64>> = Vec::new();
        for y in &years {
            let (raw, fmt) = year_map
                .and_then(|m| m.get(y))
                .map(|(r, f)| (*r, f.clone()))
                .unwrap_or((None, String::new()));
            raws.push(raw);
            let cell = if !fmt.is_empty() {
                fmt
            } else if let Some(r) = raw {
                format!("{:.2}", r)
            } else {
                "N/A".to_string()
            };
            cells.push(cell);
        }
        // YoY 仅对营收和净利润
        let yoy = if *display == "营收" || *display == "净利润" {
            let cur = raws.first().copied().flatten();
            let prev = raws.get(1).copied().flatten();
            calc_yoy(cur, prev)
        } else {
            String::new()
        };
        cells.push(yoy);
        lines.push(format!("| {} |", cells.join(" | ")));
    }

    lines.join("\n") + "\n"
}

// ───────────────────────── 东方财富基本面（A股/港股共享） ─────────────────────────

/// 东方财富基本面请求参数（A股/港股共享）。
struct EastmoneyParams {
    /// 标题中展示的代码（用户原始输入，如 600519 / 0700.HK）
    display_symbol: String,
    /// push2 API secid（如 1.600519 / 116.00700）
    secid: String,
    /// datacenter SECURITY_CODE 过滤值（A股=代码，港股=去 .HK 后缀）
    datacenter_code: String,
    /// 标题市场标签（"A股基本面" / "港股基本面"）
    title: String,
}

/// 东方财富基本面（A股/港股共享）：push2 个股基本信息 + datacenter 财务指标。
///
/// datacenter 若无对应市场数据，优雅降级为「财务指标数据暂不可用」。
async fn fetch_eastmoney_fundamentals(client: &Client, p: &EastmoneyParams) -> String {
    let mut sections = Vec::new();
    sections.push(format!("# {} {}（精简）\n", p.display_symbol, p.title));

    // ── 个股基本信息（东方财富 push2 API）──
    let info_url = format!(
        "https://push2.eastmoney.com/api/qt/stock/get?fltt=2&invt=2&fields=f57,f58,f84,f85,f127,f116,f117,f189,f43&secid={}",
        p.secid
    );

    let mut has_info = false;
    if let Ok(resp) = get_with_retry(client, &info_url, Some(2)).await {
        if let Ok(body) = resp.json::<Value>().await {
            if let Some(data) = body.get("data") {
                sections.push("## 个股基本信息\n".to_string());
                let field_map = [
                    ("代码", "f57"),
                    ("简称", "f58"),
                    ("总股本", "f84"),
                    ("流通股", "f85"),
                    ("行业", "f127"),
                    ("总市值", "f116"),
                    ("流通市值", "f117"),
                    ("上市时间", "f189"),
                    ("最新价", "f43"),
                ];
                for (display, key) in &field_map {
                    let val = data.get(key).unwrap_or(&Value::Null);
                    sections.push(format!("- **{}**: {}", display, fmt_num(Some(val))));
                }
                sections.push(String::new());
                has_info = true;
            }
        }
    }

    if !has_info {
        sections.push("## 个股基本信息\n\n> 获取失败\n".to_string());
    }

    // ── 财务分析指标（东方财富 datacenter API）──
    let fin_url = format!(
        "https://datacenter-web.eastmoney.com/api/data/v1/get?reportName=RPT_F10_FINANCE_MAINFINADATA&filter=(SECURITY_CODE=%22{}%22)&columns=ALL&pageSize=4&sortColumns=REPORT_DATE&sortTypes=-1",
        p.datacenter_code
    );

    match get_with_retry(client, &fin_url, Some(2)).await {
        Ok(resp) => {
            if let Ok(body) = resp.json::<Value>().await {
                let data_arr = body
                    .get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.as_array());

                if let Some(arr) = data_arr {
                    if !arr.is_empty() {
                        sections.push("## 关键财务指标（最近 4 期）\n".to_string());
                        sections.push(build_cn_financial_table(arr));
                    } else {
                        sections.push("## 关键财务指标\n\n> 财务指标数据暂不可用\n".to_string());
                    }
                } else {
                    sections.push("## 关键财务指标\n\n> 财务指标数据暂不可用\n".to_string());
                }
            } else {
                sections.push("## 关键财务指标\n\n> 财务指标数据暂不可用\n".to_string());
            }
        }
        Err(_) => {
            sections.push("## 关键财务指标\n\n> 财务指标数据暂不可用\n".to_string());
        }
    }

    sections.join("\n")
}

/// 获取A股基本面数据（东方财富 API）
async fn fetch_cn_fundamentals(client: &Client, symbol: &str) -> String {
    let market_id = if symbol.starts_with('6') { "1" } else { "0" };
    fetch_eastmoney_fundamentals(
        client,
        &EastmoneyParams {
            display_symbol: symbol.to_string(),
            secid: format!("{}.{}", market_id, symbol),
            datacenter_code: symbol.to_string(),
            title: "A股基本面".to_string(),
        },
    )
    .await
}

/// 获取港股基本面数据（东方财富 API，secid 前缀 116）
async fn fetch_hk_fundamentals(client: &Client, symbol: &str) -> String {
    // 港股 secid / datacenter 代码必须 5 位零填充（0700.HK -> 00700）
    let code = crate::market::hk_eastmoney_code(symbol);
    fetch_eastmoney_fundamentals(
        client,
        &EastmoneyParams {
            display_symbol: symbol.to_string(),
            secid: format!("116.{}", code),
            datacenter_code: code,
            title: "港股基本面".to_string(),
        },
    )
    .await
}

/// 构建A股关键财务指标 markdown 表格
fn build_cn_financial_table(data: &[Value]) -> String {
    // 提取报告期标签
    let periods: Vec<String> = data
        .iter()
        .map(|row| {
            row.get("REPORT_DATE")
                .and_then(|v| v.as_str())
                .map(|s| s.get(..10).unwrap_or(s).to_string())
                .unwrap_or_else(|| "N/A".to_string())
        })
        .collect();

    let ncols = periods.len();
    if ncols == 0 {
        return "> 无可用财务数据\n".to_string();
    }

    // 要展示的指标行：(显示名, 东方财富字段名, 格式类型)
    // 字段名对应 datacenter RPT_F10_FINANCE_MAINFINADATA 的实际返回字段。
    let indicators: [(&str, &str, CnFmt); 14] = [
        ("每股收益", "EPSJB", CnFmt::Num),
        ("每股净资产", "BPS", CnFmt::Num),
        ("每股经营现金流", "MGJYXJJE", CnFmt::Num),
        ("净资产收益率(%)", "ROEJQ", CnFmt::Num),
        ("ROIC(%)", "ROIC", CnFmt::Num),
        ("营业总收入", "TOTALOPERATEREVE", CnFmt::Amount),
        ("毛利", "MLR", CnFmt::Amount),
        ("净利润(归母)", "PARENTNETPROFIT", CnFmt::Amount),
        ("扣非净利润", "KCFJCXSYJLR", CnFmt::Amount),
        ("毛利率(%)", "XSMLL", CnFmt::Num),
        ("净利率(%)", "XSJLL", CnFmt::Num),
        ("营收同比(%)", "TOTALOPERATEREVETZ", CnFmt::Num),
        ("净利同比(%)", "PARENTNETPROFITTZ", CnFmt::Num),
        ("资产负债率(%)", "ZCFZL", CnFmt::Num),
    ];

    let mut lines = Vec::new();
    let header = format!("| 指标 | {} |", periods.join(" | "));
    lines.push(header);
    lines.push(format!("|{}|", "---|".repeat(ncols + 1)));

    for (display, field, kind) in &indicators {
        let mut cells = vec![display.to_string()];
        for row in data {
            let val = row.get(*field).unwrap_or(&Value::Null);
            let cell = match kind {
                CnFmt::Amount => fmt_cn_amount(Some(val)),
                CnFmt::Num => fmt_num(Some(val)),
            };
            cells.push(cell);
        }
        lines.push(format!("| {} |", cells.join(" | ")));
    }

    lines.join("\n") + "\n"
}

// ───────────────────────── 统一入口 ─────────────────────────

/// 统一基本面数据获取入口
///
/// 自动检测市场类型：
/// - 6位纯数字 → A股
/// - 其他 → 美股
///
/// 契约：永不 panic，错误以字符串形式返回
pub async fn fetch_fundamentals(client: &Client, symbol: &str) -> String {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return "错误: 股票代码不能为空".to_string();
    }

    let market = detect_market(symbol);
    match market {
        Market::CNStock => fetch_cn_fundamentals(client, symbol).await,
        Market::HKStock => fetch_hk_fundamentals(client, symbol).await,
        _ => fetch_us_fundamentals(client, symbol).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 真实网络集成测试：默认不进 CI，手动 `cargo test -- --ignored` 运行。
    // 验证 Yahoo quoteSummary 的 crumb 握手（修复 401 Unauthorized）。
    #[tokio::test]
    #[ignore = "hits the live Yahoo Finance API"]
    async fn test_live_fundamentals_aapl() {
        let client = crate::http::build_client().unwrap();
        let out = fetch_fundamentals(&client, "AAPL").await;
        assert!(!out.starts_with("错误"), "AAPL 基本面不应报错: {}", out);
        assert!(out.contains("公司概况"), "应包含公司概况段落");
    }

    // A股基本面走东方财富，国内网络即可达。
    #[tokio::test]
    #[ignore = "hits the live Eastmoney API"]
    async fn test_live_fundamentals_cn() {
        let client = crate::http::build_client().unwrap();
        let out = fetch_fundamentals(&client, "600519").await;
        assert!(!out.starts_with("错误"), "600519 基本面不应报错: {}", out);
        assert!(out.contains("个股基本信息"), "应包含个股基本信息段落");
    }

    #[test]
    fn test_fmt_cn_amount() {
        use serde_json::json;
        assert_eq!(fmt_cn_amount(Some(&json!(54702912385.23))), "547.03亿");
        assert_eq!(fmt_cn_amount(Some(&json!(12345.0))), "1.23万");
        assert_eq!(fmt_cn_amount(Some(&json!(99.0))), "99.00");
        assert_eq!(fmt_cn_amount(Some(&Value::Null)), "N/A");
        assert_eq!(fmt_cn_amount(None), "N/A");
    }

    // 验证 A股财务表使用正确的东方财富字段名（修复 7/8 指标 N/A 的 bug）。
    #[test]
    fn test_build_cn_financial_table_fields() {
        use serde_json::json;
        let data = vec![json!({
            "REPORT_DATE": "2026-03-31 00:00:00",
            "EPSJB": 21.76,
            "BPS": 216.32,
            "MGJYXJJE": 21.49,
            "ROEJQ": 10.57,
            "ROIC": 9.83,
            "TOTALOPERATEREVE": 54702912385.23,
            "MLR": 48388523020.19,
            "PARENTNETPROFIT": 27242512886.45,
            "KCFJCXSYJLR": 27239985194.41,
            "XSMLL": 89.76,
            "XSJLL": 52.22,
            "TOTALOPERATEREVETZ": 6.34,
            "PARENTNETPROFITTZ": 1.47,
            "ZCFZL": 12.12
        })];
        let table = build_cn_financial_table(&data);
        // 报告期
        assert!(table.contains("2026-03-31"));
        // 每股 / 比率指标（正确字段）
        assert!(table.contains("21.76"), "EPSJB 每股收益");
        assert!(table.contains("89.76"), "XSMLL 毛利率");
        assert!(table.contains("52.22"), "XSJLL 净利率");
        assert!(table.contains("10.57"), "ROEJQ 净资产收益率");
        // 金额指标按亿换算
        assert!(table.contains("547.03亿"), "TOTALOPERATEREVE 营业总收入");
        assert!(table.contains("272.43亿"), "PARENTNETPROFIT 净利润");
        // 不应再出现整行 N/A（旧错误字段名导致）
        assert!(!table.contains("| 每股收益 | N/A |"));
    }

    // fmt_num 需解包 Yahoo 的 {fmt, raw} 对象（修复打印原始 JSON 的 bug）。
    #[test]
    fn test_fmt_num_object() {
        use serde_json::json;
        // 优先取 fmt
        assert_eq!(
            fmt_num(Some(&json!({"fmt":"466.82B","raw":466822987776.0}))),
            "466.82B"
        );
        assert_eq!(
            fmt_num(Some(&json!({"fmt":"148.75%","raw":1.4875}))),
            "148.75%"
        );
        // 仅 raw → 格式化数值
        assert_eq!(fmt_num(Some(&json!({"raw":1234.5}))), "1234.50");
        // 空对象 → N/A
        assert_eq!(fmt_num(Some(&json!({}))), "N/A");
        // 普通数值/字符串不受影响
        assert_eq!(fmt_num(Some(&json!(1.23456))), "1.23");
        assert_eq!(fmt_num(Some(&json!("Technology"))), "Technology");
        assert_eq!(fmt_num(Some(&Value::Null)), "N/A");
    }

    // 验证 fundamentals-timeseries 响应解析为按年表格（替换空的 quoteSummary 报表模块）。
    #[test]
    fn test_build_us_timeseries_table() {
        use serde_json::json;
        let body = json!({
            "timeseries": {
                "result": [
                    {
                        "meta": {"symbol":["AAPL"],"type":["annualTotalRevenue"]},
                        "timestamp": [1664496000, 1696032000],
                        "annualTotalRevenue": [
                            {"reportedValue":{"raw":394328000000.0,"fmt":"394.33B"}},
                            {"reportedValue":{"raw":383285000000.0,"fmt":"383.29B"}}
                        ]
                    },
                    {
                        "meta": {"symbol":["AAPL"],"type":["annualNetIncome"]},
                        "timestamp": [1664496000, 1696032000],
                        "annualNetIncome": [
                            {"reportedValue":{"raw":99803000000.0,"fmt":"99.80B"}},
                            {"reportedValue":{"raw":96995000000.0,"fmt":"97.00B"}}
                        ]
                    }
                ],
                "error": null
            }
        });
        let metrics = [
            ("营收", "annualTotalRevenue"),
            ("净利润", "annualNetIncome"),
        ];
        let table = build_us_timeseries_table(&body, &metrics);
        // 年份降序
        assert!(table.contains("2023"), "应含 2023 列");
        assert!(table.contains("2022"), "应含 2022 列");
        // fmt 值
        assert!(table.contains("383.29B"), "营收 2023");
        assert!(table.contains("394.33B"), "营收 2022");
        assert!(table.contains("97.00B"), "净利润 2023");
        // 营收 YoY
        assert!(table.contains("-2.80%"), "营收 YoY");
        // 无 N/A
        assert!(!table.contains("N/A"), "不应有 N/A: {}", table);
    }

    // 港股基本面走东方财富，国内网络即可达。
    #[tokio::test]
    #[ignore = "hits the live Eastmoney API"]
    async fn test_live_fundamentals_hk() {
        let client = crate::http::build_client().unwrap();
        let out = fetch_fundamentals(&client, "0700.HK").await;
        assert!(!out.starts_with("错误"), "0700.HK 基本面不应报错: {}", out);
        assert!(out.contains("个股基本信息"), "应包含个股基本信息段落");
        assert!(out.contains("港股基本面"), "应走港股东方财富通道而非 Yahoo");
    }
}
