use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

/// Builds an http client that is a bit more resilient to be able to connect through VPNs.
///
/// # Panics
/// If the client can't be crated.
#[must_use]
pub fn build_robust_client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .redirect(Policy::limited(100))
        // TIMEOUTS:
        // 1. Connect: Give it time to handshake with the VPN endpoint
        .connect_timeout(Duration::from_secs(10))
        // 2. Total: If the whole request takes > 30s, give up (prevents hanging forever)
        .timeout(Duration::from_secs(30))
        // KEEP-ALIVE:
        // Helps keep the VPN tunnel active during pauses in data
        .tcp_keepalive(Duration::from_secs(60))
        // POOLING:
        // Reuse connections to avoid the expensive handshake overhead
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .expect("Failed to build client")
}

/// Tries to get the content with retries and trying different domains.
/// # Errors
///   - If the request is not successful despite our best efforts.
pub async fn get_with_retries(
    client: &Client,
    original_url: &str,
    domain_keyword: &str,
    extensions: &[String],
) -> Result<Response, Box<dyn std::error::Error>> {
    let parsed_url = Url::parse(original_url)?;
    let host_str = parsed_url.host_str().ok_or("No host in URL")?;

    if !host_str.contains(domain_keyword) {
        // Not the target domain, just request it directly
        let res = client.get(original_url).send().await?;
        if res.status().is_success() {
            return Ok(res);
        }
        return Err(format!("Request failed with status: {}", res.status()).into());
    }

    let mut last_error = None;

    for ext in extensions {
        let new_host = format!("{domain_keyword}.{ext}");
        // We need to replace the host in the URL.
        let mut new_url = parsed_url.clone();
        if new_url.set_host(Some(&new_host)).is_err() {
            warn!("Failed to set host to {}", new_host);
            continue;
        }

        let url_str = new_url.to_string();
        debug!("Trying URL: {}", url_str);

        match client.get(&url_str).send().await {
            Ok(res) => {
                if res.status().is_success() || res.status().is_client_error() {
                    return Ok(res);
                }
                warn!(
                    "Request to {} failed with status: {}",
                    url_str,
                    res.status()
                );
                last_error = Some(format!("Status: {}", res.status()));
            }
            Err(e) => {
                warn!("Request to {} failed with error: {}", url_str, e);
                last_error = Some(e.to_string());
            }
        }
    }

    Err(format!("All mirrors failed. Last error: {last_error:?}").into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_with_retries_non_target_domain() {
        let mock_server = MockServer::start().await;
        let client = build_robust_client();

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        // Use a keyword that is NOT in the localhost URL (e.g. "audiobookbay")
        let domain_keyword = "audiobookbay";
        let extensions = vec!["is".to_string(), "lu".to_string()];

        let result = get_with_retries(&client, &url, domain_keyword, &extensions).await;
        assert!(result.is_ok());
        assert!(result.unwrap().status().is_success());
    }

    #[tokio::test]
    async fn test_get_with_retries_fail_retry_logic() {
        let mock_server = MockServer::start().await;
        let client = build_robust_client();

        // Even if the server is up, we expect the function to try "127.0.0.1.is" etc. and fail.
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", mock_server.uri());
        // Extract the host from the mock server URI (usually 127.0.0.1)
        let parsed_url = Url::parse(&url).unwrap();
        let domain_keyword = parsed_url.host_str().unwrap();

        let extensions = vec!["is".to_string(), "lu".to_string()];

        let result = get_with_retries(&client, &url, domain_keyword, &extensions).await;
        assert!(result.is_err());
    }
}
