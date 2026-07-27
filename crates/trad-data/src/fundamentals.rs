/// 基本面数据获取模块
///
/// 支持美股（Yahoo Finance v10 API）和A股（东方财富 API）的基本面数据获取。
/// 对齐 Python fetch_fundamentals.py 的输出格式。
use serde_json::Value;

use crate::http::{build_client, get_with_retry};
use crate::market::{detect_market, Market};

// ───────────────────────── 工具函数 ─────────────────────────

/// 格式化数值：缺失返回 "N/A"，否则 round 到 2 位
fn fmt_num(v: Option<&Value>) -> String {
    match v {
        None => "N/A".to_string(),
        Some(Value::Null) => "N/A".to_string(),
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

/// 从报表数组中按行标签提取数值序列（按年份倒序）
fn extract_row_values(statements: &[Value], row_label: &str) -> Vec<Option<f64>> {
    statements
        .iter()
        .map(|stmt| {
            stmt.get(row_label)
                .and_then(|v| v.get("raw"))
                .and_then(|r| r.as_f64())
        })
        .collect()
}

// ───────────────────────── 美股基本面 ─────────────────────────

/// 获取美股基本面数据（Yahoo Finance v10 API）
async fn fetch_us_fundamentals(symbol: &str) -> String {
    let mut sections = Vec::new();
    sections.push(format!("# {} 基本面（精简）\n", symbol));

    // 构建 quoteSummary 请求 URL
    let modules = "assetProfile,financialData,defaultKeyStats,incomeStatementHistory,balanceSheetHistory,cashflowStatementHistory";
    let url = format!(
        "https://query2.finance.yahoo.com/v10/finance/quoteSummary/{}?modules={}",
        symbol, modules
    );

    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return format!("错误: 创建 HTTP 客户端失败 - {}", e),
    };

    let resp = match get_with_retry(&client, &url, Some(2)).await {
        Ok(r) => r,
        Err(e) => return format!("错误: 获取 {} 基本面数据失败 - {}", symbol, e),
    };

    let body: Value = match resp.json().await {
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
    let key_stats = result.get("defaultKeyStats").unwrap_or(&Value::Null);

    let fields = [
        ("公司名称", fmt_num(profile.get("longName"))),
        (
            "行业",
            format!(
                "{} / {}",
                fmt_num(profile.get("sector")),
                fmt_num(profile.get("industry"))
            ),
        ),
        ("市值", fmt_num(key_stats.get("marketCap"))),
        ("市盈率(PE)", fmt_num(key_stats.get("trailingPE"))),
        ("市净率(PB)", fmt_num(key_stats.get("priceToBook"))),
        ("ROE", fmt_num(fin_data.get("returnOnEquity"))),
        ("总营收", fmt_num(fin_data.get("totalRevenue"))),
        ("利润率", fmt_num(fin_data.get("profitMargins"))),
    ];
    for (k, v) in &fields {
        sections.push(format!("- **{}**: {}", k, v));
    }
    sections.push(String::new());

    // ── 关键财务指标表 ──
    let income_stmts = result
        .get("incomeStatementHistory")
        .and_then(|v| v.get("incomeStatementHistory"))
        .and_then(|v| v.as_array());
    let balance_stmts = result
        .get("balanceSheetHistory")
        .and_then(|v| v.get("balanceSheetStatements"))
        .and_then(|v| v.as_array());
    let cashflow_stmts = result
        .get("cashflowStatementHistory")
        .and_then(|v| v.get("cashflowStatements"))
        .and_then(|v| v.as_array());

    let has_data = income_stmts.is_some_and(|a| !a.is_empty())
        || balance_stmts.is_some_and(|a| !a.is_empty())
        || cashflow_stmts.is_some_and(|a| !a.is_empty());

    if has_data {
        sections.push("## 关键财务指标（最近年度，脚本抽取）\n".to_string());
        sections.push(build_us_metric_table(
            income_stmts,
            balance_stmts,
            cashflow_stmts,
        ));
    } else {
        sections.push("## 关键财务指标\n\n> 无数据\n".to_string());
    }

    sections.join("\n")
}

/// 构建美股关键财务指标 markdown 表格
fn build_us_metric_table(
    income: Option<&Vec<Value>>,
    balance: Option<&Vec<Value>>,
    cashflow: Option<&Vec<Value>>,
) -> String {
    // 行项映射：(显示名, 数据源类型, 行标签)
    struct RowDef<'a> {
        display: &'a str,
        source: &'a str, // "income" / "balance" / "cashflow"
        label: &'a str,
    }
    let rows = [
        RowDef {
            display: "营收",
            source: "income",
            label: "totalRevenue",
        },
        RowDef {
            display: "净利润",
            source: "income",
            label: "netIncome",
        },
        RowDef {
            display: "摊薄EPS",
            source: "income",
            label: "dilutedEPS",
        },
        RowDef {
            display: "毛利",
            source: "income",
            label: "grossProfit",
        },
        RowDef {
            display: "总资产",
            source: "balance",
            label: "totalAssets",
        },
        RowDef {
            display: "总负债",
            source: "balance",
            label: "totalDebt",
        },
        RowDef {
            display: "股东权益",
            source: "balance",
            label: "totalStockholderEquity",
        },
        RowDef {
            display: "经营现金流",
            source: "cashflow",
            label: "operatingCashFlow",
        },
        RowDef {
            display: "自由现金流",
            source: "cashflow",
            label: "freeCashFlow",
        },
    ];

    // 收集年份标签（取最近 4 年）
    let mut year_labels = Vec::new();
    for stmts in [income, balance, cashflow].iter().flatten() {
        for stmt in *stmts {
            if let Some(end_date) = stmt
                .get("endDate")
                .and_then(|v| v.get("fmt"))
                .and_then(|v| v.as_str())
            {
                let label = end_date.get(..4).unwrap_or(end_date).to_string();
                if !year_labels.contains(&label) {
                    year_labels.push(label);
                }
            }
            if year_labels.len() >= 4 {
                break;
            }
        }
        if year_labels.len() >= 4 {
            break;
        }
    }
    year_labels.truncate(4);

    if year_labels.is_empty() {
        return "> 无可用年度数据\n".to_string();
    }

    let ncols = year_labels.len();

    // 表头
    let mut lines = Vec::new();
    let header = format!("| 指标 | {} | YoY(营收/净利) |", year_labels.join(" | "));
    lines.push(header);
    lines.push(format!("|{}|", "---|".repeat(ncols + 2)));

    // 提取各行的数值
    for row_def in &rows {
        let stmts = match row_def.source {
            "income" => income,
            "balance" => balance,
            "cashflow" => cashflow,
            _ => None,
        };
        let vals = if let Some(stmts) = stmts {
            extract_row_values(stmts, row_def.label)
        } else {
            vec![None; ncols]
        };

        let mut cells = vec![row_def.display.to_string()];
        for i in 0..ncols {
            let cell = vals.get(i).copied().flatten();
            cells.push(match cell {
                Some(v) => format!("{:.2}", v),
                None => "N/A".to_string(),
            });
        }

        // YoY 仅对营收和净利润计算
        let yoy = if row_def.display == "营收" || row_def.display == "净利润" {
            let cur = vals.first().copied().flatten();
            let prev = vals.get(1).copied().flatten();
            calc_yoy(cur, prev)
        } else {
            String::new()
        };
        cells.push(yoy);

        lines.push(format!("| {} |", cells.join(" | ")));
    }

    lines.join("\n") + "\n"
}

// ───────────────────────── A股基本面 ─────────────────────────

/// 获取A股基本面数据（东方财富 API）
async fn fetch_cn_fundamentals(symbol: &str) -> String {
    let mut sections = Vec::new();
    sections.push(format!("# {} A股基本面（精简）\n", symbol));

    let client = match build_client() {
        Ok(c) => c,
        Err(e) => return format!("错误: 创建 HTTP 客户端失败 - {}", e),
    };

    // ── 个股基本信息（东方财富 push2 API）──
    let market_id = if symbol.starts_with('6') { "1" } else { "0" };
    let secid = format!("{}.{}", market_id, symbol);
    let info_url = format!(
        "https://push2.eastmoney.com/api/qt/stock/get?fltt=2&invt=2&fields=f57,f58,f84,f85,f127,f116,f117,f189,f43&secid={}",
        secid
    );

    let mut has_info = false;
    if let Ok(resp) = get_with_retry(&client, &info_url, Some(2)).await {
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
        symbol
    );

    match get_with_retry(&client, &fin_url, Some(2)).await {
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

    // 要展示的指标行：(显示名, 字段名)
    let indicators = [
        ("每股收益", "BASIC_EPS"),
        ("每股净资产", "BASIC_BPS"),
        ("净资产收益率(%)", "WEIGHTAVG_ROE"),
        ("营业总收入", "TOTAL_OPERATE_INCOME"),
        ("净利润", "PARENT_NETPROFIT"),
        ("毛利率(%)", "XSJLL"),
        ("净利率(%)", "XSJLR"),
        ("经营现金流", "OPERATE_CASHFLOW"),
    ];

    let mut lines = Vec::new();
    let header = format!("| 指标 | {} |", periods.join(" | "));
    lines.push(header);
    lines.push(format!("|{}|", "---|".repeat(ncols + 1)));

    for (display, field) in &indicators {
        let mut cells = vec![display.to_string()];
        for row in data {
            let val = row.get(field).unwrap_or(&Value::Null);
            cells.push(fmt_num(Some(val)));
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
pub async fn fetch_fundamentals(symbol: &str) -> String {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return "错误: 股票代码不能为空".to_string();
    }

    let market = detect_market(symbol);
    match market {
        Market::CNStock => fetch_cn_fundamentals(symbol).await,
        _ => fetch_us_fundamentals(symbol).await,
    }
}
