//! Yahoo Finance 共享反爬握手（cookie + crumb + 浏览器 UA）。
//!
//! Yahoo 的 chart (v8) 与 quoteSummary (v10) 等端点在数据中心 / 海外 IP 上
//! 需要 crumb token，否则返回 401 / 403，或 HTTP 200 但空数据。本模块把
//! 握手逻辑抽成共享代码，供 `market/us.rs`（行情）、`fundamentals.rs`、
//! `news.rs`（quoteSummary）复用。
use crate::http::get_with_retry_headers;
use reqwest::Client;

/// 浏览器 User-Agent：Yahoo 会封锁非浏览器 UA（尤其数据中心 IP），
/// 所有 Yahoo 请求必须携带真实浏览器 UA。
pub const BROWSER_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// 简单的 URL 编码（用于 crumb 参数，镜像 news.rs 的同名实现）
pub fn url_encode(s: &str) -> String {
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

/// 获取 Yahoo Finance crumb token（yfinance 底层反爬握手）
///
/// 流程：先访问 fc.yahoo.com 种 cookie（cookie store 自动保存），
/// 再用同一 cookie 请求 v1/test/getcrumb 取得 crumb 字符串。
/// 任一步失败返回 None（调用方仍可尝试无 crumb 请求）。
pub async fn get_crumb(client: &Client) -> Option<String> {
    // 种 cookie（响应状态码无关紧要，忽略结果）
    let _ = client
        .get("https://fc.yahoo.com")
        .header("User-Agent", BROWSER_UA)
        .send()
        .await;

    let resp = client
        .get("https://query2.finance.yahoo.com/v1/test/getcrumb")
        .header("User-Agent", BROWSER_UA)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let crumb = resp.text().await.ok()?;
    let crumb = crumb.trim();
    if crumb.is_empty() {
        None
    } else {
        Some(crumb.to_string())
    }
}

/// 给 URL 追加 crumb 查询参数（自动判断分隔符 `?` / `&`）。
pub fn append_crumb(url: &str, crumb: &str) -> String {
    let sep = if url.contains('?') { "&" } else { "?" };
    format!("{}{}crumb={}", url, sep, url_encode(crumb))
}

/// 带 crumb 握手的 Yahoo GET，返回响应 body 文本。
///
/// 先直连（浏览器 UA）；若 HTTP 层失败（如 401 / 403），取 crumb 后重试。
/// 适用于 quoteSummary 等 "HTTP 层失败" 的端点。chart 端点因存在
/// "HTTP 200 但空数据" 的封锁形态，由 `market/us.rs` 自行处理重试。
pub async fn yahoo_get_body(client: &Client, url: &str) -> Result<String, String> {
    // 第一步：直连（无 crumb）。
    if let Ok(resp) =
        get_with_retry_headers(client, url, &[("User-Agent", BROWSER_UA)], Some(2)).await
    {
        if let Ok(body) = resp.text().await {
            return Ok(body);
        }
    }

    // 第二步：cookie + crumb 握手后重试。
    let crumb = get_crumb(client).await;
    let url2 = match &crumb {
        Some(c) => append_crumb(url, c),
        None => url.to_string(),
    };
    let resp = get_with_retry_headers(client, &url2, &[("User-Agent", BROWSER_UA)], Some(2))
        .await
        .map_err(|e| format!("Yahoo Finance 请求失败: {}", e))?;
    resp.text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("AAPL"), "AAPL");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(url_encode("a+b/c"), "a%2Bb%2Fc");
    }

    #[test]
    fn test_append_crumb() {
        // 已有查询串 → 用 &
        assert_eq!(
            append_crumb("https://x/y?a=1", "abc"),
            "https://x/y?a=1&crumb=abc"
        );
        // 无查询串 → 用 ?
        assert_eq!(append_crumb("https://x/y", "abc"), "https://x/y?crumb=abc");
        // crumb 含特殊字符需编码
        assert_eq!(
            append_crumb("https://x/y?a=1", "a b&c"),
            "https://x/y?a=1&crumb=a%20b%26c"
        );
    }
}
