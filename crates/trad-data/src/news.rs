/// 新闻数据获取模块
///
/// 支持多数据源新闻获取：
/// - Yahoo Finance 新闻（美股）
/// - Google News RSS（全球）
/// - 东方财富新闻（A股）
///
/// 对齐 Python fetch_news.py 的输出格式。
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::Client;
use serde_json::Value;

use crate::http::get_with_retry;
use crate::market::{detect_market, Market};

/// 摘要最大字符数
const MAX_SUMMARY_CHARS: usize = 200;

// ───────────────────────── 工具函数 ─────────────────────────

/// 截断文本到指定长度
fn truncate(text: &str, limit: usize) -> String {
    let text = text.trim();
    if text.chars().count() > limit {
        let truncated: String = text.chars().take(limit).collect();
        format!("{}...", truncated)
    } else {
        text.to_string()
    }
}

/// 格式化单条新闻为 markdown
fn format_news_item(title: &str, source: &str, summary: &str) -> String {
    let title = if title.is_empty() { "无标题" } else { title };
    let mut lines = vec![format!("- **{}**", title)];
    let mut parts = Vec::new();
    if !source.is_empty() {
        parts.push(format!("来源:{}", source));
    }
    if !summary.is_empty() {
        parts.push(truncate(summary, MAX_SUMMARY_CHARS));
    }
    if !parts.is_empty() {
        lines.push(format!("  - {}", parts.join(" | ")));
    }
    lines.join("\n")
}

/// 清除 HTML 标签（简单实现：移除 <...> 内容）
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result.trim().to_string()
}

// ───────────────────────── Yahoo Finance 新闻 ─────────────────────────

/// 通过 Yahoo Finance v10 API 获取个股新闻
async fn fetch_yfinance_news(client: &Client, symbol: &str, days: u32, limit: u32) -> String {
    let days = days.max(1);
    let url = format!(
        "https://query2.finance.yahoo.com/v10/finance/quoteSummary/{}?modules=news",
        symbol
    );

    let resp = match get_with_retry(client, &url, Some(2)).await {
        Ok(r) => r,
        Err(e) => return format!("错误: 获取 {} 新闻失败 - {}", symbol, e),
    };

    let body: Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => return format!("错误: 解析 {} 新闻响应失败 - {}", symbol, e),
    };

    // 提取新闻数组
    let news_arr = match body
        .get("quoteSummary")
        .and_then(|q| q.get("result"))
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .and_then(|r| r.get("news"))
        .and_then(|n| n.as_array())
    {
        Some(arr) if !arr.is_empty() => arr,
        _ => return format!("未找到 {} 的相关新闻。", symbol),
    };

    // 按时间过滤并收集结果
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let mut results = Vec::new();

    for item in news_arr {
        if results.len() >= limit as usize {
            break;
        }

        // 尝试从 content 嵌套结构或顶层提取字段
        let content_obj = item.get("content").and_then(|c| c.as_object());
        let title = content_obj
            .and_then(|c| c.get("title"))
            .and_then(|v| v.as_str())
            .or_else(|| item.get("title").and_then(|v| v.as_str()))
            .unwrap_or("无标题");

        // 提取来源
        let source = content_obj
            .and_then(|c| c.get("provider"))
            .and_then(|p| p.get("displayName"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 提取摘要
        let summary = content_obj
            .and_then(|c| c.get("summary").or_else(|| c.get("description")))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 时间过滤：尝试解析发布时间
        let content_val = content_obj
            .map(|m| Value::Object(m.clone()))
            .unwrap_or_else(|| item.clone());
        let pub_time = parse_news_time(&content_val);
        if let Some(pt) = pub_time {
            if pt < cutoff {
                continue;
            }
        }
        // 解析失败的条目保留（避免字段变更导致漏报）

        results.push(format_news_item(title, source, summary));
    }

    if results.is_empty() {
        return format!("未找到 {} 在最近 {} 天内的相关新闻。", symbol, days);
    }

    let header = format!(
        "## {} 相关新闻（最近 {} 天，共 {} 条）\n\n",
        symbol,
        days,
        results.len()
    );
    format!("{}{}", header, results.join("\n"))
}

/// 解析新闻发布时间
fn parse_news_time(item: &Value) -> Option<chrono::DateTime<chrono::Utc>> {
    // 尝试 ISO 8601 字符串
    for field in &["pubDate", "publishTime"] {
        if let Some(s) = item.get(field).and_then(|v| v.as_str()) {
            // 尝试 ISO 格式
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Some(dt.with_timezone(&chrono::Utc));
            }
            // 尝试常见格式
            if let Ok(dt) = chrono::DateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
                return Some(dt.with_timezone(&chrono::Utc));
            }
        }
    }
    // 尝试 Unix 时间戳
    for field in &["providerPublishTime", "pubDate", "publishTime"] {
        if let Some(n) = item.get(field).and_then(|v| v.as_f64()) {
            let ts = if n > 1e12 {
                (n / 1000.0) as i64
            } else {
                n as i64
            };
            if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
                return Some(dt);
            }
        }
    }
    None
}

// ───────────────────────── Google News RSS ─────────────────────────

/// 通过 Google News RSS 获取新闻
async fn fetch_google_news(
    client: &Client,
    query: &str,
    days: u32,
    limit: u32,
    lang: &str,
) -> String {
    let days = days.max(1);
    let encoded_query = urlencoding_encode(query);
    let (hl, gl, ceid) = match lang {
        "zh" => ("zh-CN", "CN", "CN:zh-Hans"),
        _ => ("en", "US", "US:en"),
    };
    let url = format!(
        "https://news.google.com/rss/search?q={}+when:{}d&hl={}&gl={}&ceid={}",
        encoded_query, days, hl, gl, ceid
    );

    let resp = match get_with_retry(client, &url, Some(2)).await {
        Ok(r) => r,
        Err(e) => return format!("错误: Google News 请求失败 - {}", e),
    };

    let xml_text = match resp.text().await {
        Ok(t) => t,
        Err(e) => return format!("错误: 读取 Google News 响应失败 - {}", e),
    };

    // 解析 RSS XML
    parse_google_news_rss(&xml_text, query, days, limit)
}

/// 简单的 URL 编码
fn urlencoding_encode(s: &str) -> String {
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

/// 解析 Google News RSS XML
fn parse_google_news_rss(xml: &str, query: &str, days: u32, limit: u32) -> String {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut in_item = false;
    let mut current_title = String::new();
    let mut current_source = String::new();
    let mut current_desc = String::new();
    let mut current_tag = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match tag_name.as_str() {
                    "item" => {
                        in_item = true;
                        current_title.clear();
                        current_source.clear();
                        current_desc.clear();
                    }
                    "title" if in_item => current_tag = "title".to_string(),
                    "source" if in_item => current_tag = "source".to_string(),
                    "description" if in_item => current_tag = "description".to_string(),
                    _ => current_tag.clear(),
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                match current_tag.as_str() {
                    "title" => current_title.push_str(&text),
                    "source" => current_source.push_str(&text),
                    "description" => current_desc.push_str(&text),
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "item" {
                    in_item = false;
                    if !current_title.is_empty() {
                        items.push(format_news_item(
                            &current_title,
                            &current_source,
                            &strip_html(&current_desc),
                        ));
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return format!("错误: 解析 Google News RSS 响应失败 - {}", e),
            _ => {}
        }
    }

    if items.is_empty() {
        return format!("Google News 未找到与 \"{}\" 相关的新闻。", query);
    }

    let display_items: Vec<_> = items.into_iter().take(limit as usize).collect();
    let header = format!(
        "## Google News: \"{}\"（最近 {} 天，共 {} 条）\n\n",
        query,
        days,
        display_items.len()
    );
    format!("{}{}", header, display_items.join("\n"))
}

// ───────────────────────── A股新闻（东方财富）─────────────────────────

/// 通过东方财富搜索 API 获取A股新闻
async fn fetch_cn_news(client: &Client, symbol: &str, days: u32, limit: u32) -> String {
    // 东方财富搜索 API（JSONP 格式）
    let param = serde_json::json!({
        "uid": "",
        "keyword": symbol,
        "type": ["cmsArticleWebOld"],
        "client": "web",
        "clientType": "web",
        "clientVersion": "curr",
        "param": {
            "cmsArticleWebOld": {
                "searchScope": "default",
                "sort": "default",
                "pageIndex": 1,
                "pageSize": limit,
                "preTag": "",
                "postTag": ""
            }
        }
    });

    let url = format!(
        "https://search-api-web.eastmoney.com/search/jsonp?cb=jQuery&param={}",
        urlencoding_encode(&param.to_string())
    );

    match get_with_retry(client, &url, Some(2)).await {
        Ok(resp) => {
            let text = match resp.text().await {
                Ok(t) => t,
                Err(_e) => {
                    // 降级到 Google News 中文
                    return fetch_google_news(
                        client,
                        &format!("{} A股", symbol),
                        days,
                        limit,
                        "zh",
                    )
                    .await;
                }
            };

            // 剥离 JSONP 包装: jQuery(...)
            let json_str = strip_jsonp(&text);
            let body: Value = match serde_json::from_str(&json_str) {
                Ok(v) => v,
                Err(_) => {
                    return fetch_google_news(
                        client,
                        &format!("{} A股", symbol),
                        days,
                        limit,
                        "zh",
                    )
                    .await;
                }
            };

            // 提取新闻数组
            let articles = body
                .get("result")
                .and_then(|r| r.get("cmsArticleWebOld"))
                .and_then(|a| a.as_array());

            if let Some(arr) = articles {
                if !arr.is_empty() {
                    let mut results = Vec::new();
                    for article in arr {
                        let title = strip_html(
                            article
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("无标题"),
                        );
                        let source = article
                            .get("mediaName")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let content = strip_html(
                            article
                                .get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or(""),
                        );
                        results.push(format_news_item(&title, source, &content));
                    }

                    if !results.is_empty() {
                        let header =
                            format!("## A股 {} 相关新闻（共 {} 条）\n\n", symbol, results.len());
                        return format!("{}{}", header, results.join("\n"));
                    }
                }
            }

            // 东方财富无结果，降级到 Google News 中文
            fetch_google_news(client, &format!("{} A股", symbol), days, limit, "zh").await
        }
        Err(_) => {
            // 降级到 Google News 中文
            fetch_google_news(client, &format!("{} A股", symbol), days, limit, "zh").await
        }
    }
}

/// 剥离 JSONP 包装：jQuery...({...}) → {...}
fn strip_jsonp(text: &str) -> String {
    let trimmed = text.trim();
    // 找到第一个 '(' 和最后一个 ')'
    if let (Some(start), Some(end)) = (trimmed.find('('), trimmed.rfind(')')) {
        if start < end {
            return trimmed[start + 1..end].to_string();
        }
    }
    trimmed.to_string()
}

// ───────────────────────── 统一入口 ─────────────────────────

/// 统一新闻获取入口
///
/// 自动检测市场类型：
/// - A股（6位纯数字）→ 东方财富新闻，失败降级 Google News 中文
/// - 其他 → Yahoo Finance 新闻 + Google News，并行获取后合并输出
///
/// 契约：永不 panic，错误以字符串形式返回
pub async fn fetch_news(client: &Client, symbol: &str, days: u32, limit: u32) -> String {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        return "错误: 股票代码不能为空".to_string();
    }

    let market = detect_market(symbol);
    match market {
        Market::CNStock => fetch_cn_news(client, symbol, days, limit).await,
        _ => {
            // 非A股：并行获取 Yahoo Finance + Google News
            let (yf_news, google_news) = tokio::join!(
                fetch_yfinance_news(client, symbol, days, limit),
                fetch_google_news(client, symbol, days, limit, "en")
            );

            let mut sections = Vec::new();
            if !yf_news.starts_with("错误") {
                sections.push(yf_news.clone());
            }
            if !google_news.starts_with("错误") {
                sections.push(google_news);
            }

            if sections.is_empty() {
                yf_news
            } else {
                sections.join("\n---\n\n")
            }
        }
    }
}
