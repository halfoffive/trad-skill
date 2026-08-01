use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

/// 默认请求超时（秒）
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// 默认重试次数
const DEFAULT_RETRIES: u32 = 3;

/// 默认重试间隔（秒）
const DEFAULT_RETRY_DELAY_SECS: u64 = 1;

/// 构建带超时配置的 reqwest HTTP 客户端
///
/// 启用 cookie store：Yahoo Finance 的 crumb 握手需要先种 cookie 再取 crumb，
/// 后续 chart 请求必须带回同一 cookie，否则 crumb 校验失败。
pub fn build_client() -> Result<Client> {
    let client = Client::builder()
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .connect_timeout(Duration::from_secs(10))
        .cookie_store(true)
        .user_agent("trad-skill/1.8.0")
        .build()?;
    Ok(client)
}

/// 带重试的 GET 请求
///
/// 如果请求失败，会按指数退避策略重试，最多重试 `retries` 次。
pub async fn get_with_retry(
    client: &Client,
    url: &str,
    retries: Option<u32>,
) -> Result<reqwest::Response> {
    get_with_retry_headers(client, url, &[], retries).await
}

/// 带重试的 GET 请求，每次尝试附加自定义请求头
///
/// 用于需要覆盖默认 User-Agent 的场景（如 Yahoo Finance 需要浏览器 UA）。
pub async fn get_with_retry_headers(
    client: &Client,
    url: &str,
    headers: &[(&str, &str)],
    retries: Option<u32>,
) -> Result<reqwest::Response> {
    let max_retries = retries.unwrap_or(DEFAULT_RETRIES);
    let mut last_err = None;

    for attempt in 0..=max_retries {
        let mut req = client.get(url);
        for (key, value) in headers {
            req = req.header(*key, *value);
        }
        match req.send().await {
            Ok(resp) if resp.status().is_success() => return Ok(resp),
            Ok(resp) => {
                let status = resp.status();
                last_err = Some(anyhow::anyhow!("HTTP 请求失败: {}", status));
                if attempt < max_retries {
                    let delay = Duration::from_secs(DEFAULT_RETRY_DELAY_SECS * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
            }
            Err(e) => {
                last_err = Some(anyhow::anyhow!("HTTP 请求异常: {}", e));
                if attempt < max_retries {
                    let delay = Duration::from_secs(DEFAULT_RETRY_DELAY_SECS * 2u64.pow(attempt));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("未知请求错误")))
}
