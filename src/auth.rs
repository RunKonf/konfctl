use anyhow::{Context, Result, bail};
use tiny_http::Server;
use url::Url;
use uuid::Uuid;

#[derive(Debug)]
pub struct AuthResult {
    pub token: String,
    pub name: Option<String>,
    pub conference_id: Option<String>,
}

pub fn build_login_url(api_url: &str, port: u16, state: &str) -> String {
    format!("{api_url}/cli/login?port={port}&state={state}")
}

pub fn parse_callback(request_url: &str, expected_state: &str) -> Result<AuthResult> {
    let parsed = Url::parse(request_url).context("Invalid callback URL")?;
    let params: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();

    let received_state = params
        .get("state")
        .context("Missing state parameter in callback")?;
    if received_state != expected_state {
        bail!("State mismatch — possible CSRF attack");
    }

    let token = params
        .get("token")
        .context("Missing token in callback")?
        .clone();

    if token.is_empty() {
        bail!("Received empty token");
    }

    let name = params.get("name").cloned().filter(|n| !n.is_empty());
    let conference_id = params
        .get("conference_id")
        .cloned()
        .filter(|c| !c.is_empty());

    Ok(AuthResult {
        token,
        name,
        conference_id,
    })
}

pub fn browser_login(api_url: &str) -> Result<AuthResult> {
    let server = Server::http("127.0.0.1:0").map_err(|e| anyhow::anyhow!("Failed to bind: {e}"))?;

    let port = server.server_addr().to_ip().map_or(0, |a| a.port());
    if port == 0 {
        bail!("Failed to bind to a local port");
    }

    let state = Uuid::new_v4().to_string();
    let login_url = build_login_url(api_url, port, &state);

    println!("Opening browser for authentication...");
    open::that(&login_url).context("Failed to open browser")?;
    println!("Waiting for callback on localhost:{port}...");

    let deadline = std::time::Instant::now() + std::time::Duration::from_mins(2);

    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            bail!("Timeout waiting for authentication callback");
        }

        let request = match server.recv_timeout(remaining) {
            Ok(Some(req)) => req,
            Ok(None) => bail!("Timeout waiting for authentication callback"),
            Err(e) => bail!("Server error: {e}"),
        };

        let request_url = format!("http://localhost:{}{}", port, request.url());

        if let Ok(result) = parse_callback(&request_url, &state) {
            // Valid callback — send success page
            let response = tiny_http::Response::from_string(
                "<html><body><h1>Authenticated!</h1><p>You can close this tab.</p>\
                 <script>window.close()</script></body></html>",
            )
            .with_header(
                "Content-Type: text/html"
                    .parse::<tiny_http::Header>()
                    .unwrap(),
            );
            let _ = request.respond(response);
            return Ok(result);
        }

        // Not a valid callback (favicon, prefetch, etc.) — respond and retry
        let response = tiny_http::Response::from_string("").with_status_code(204);
        let _ = request.respond(response);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_login_url_format() {
        let url = build_login_url("https://example.com", 12345, "my-state");
        assert_eq!(
            url,
            "https://example.com/cli/login?port=12345&state=my-state"
        );
    }

    #[test]
    fn parse_callback_valid() {
        let url = "http://localhost:9999/callback?token=abc123&state=my-state&name=Alice";
        let result = parse_callback(url, "my-state").unwrap();
        assert_eq!(result.token, "abc123");
        assert_eq!(result.name.as_deref(), Some("Alice"));
    }

    #[test]
    fn parse_callback_without_name() {
        let url = "http://localhost:9999/callback?token=abc123&state=my-state";
        let result = parse_callback(url, "my-state").unwrap();
        assert_eq!(result.token, "abc123");
        assert!(result.name.is_none());
    }

    #[test]
    fn parse_callback_state_mismatch() {
        let url = "http://localhost:9999/callback?token=abc&state=wrong";
        let err = parse_callback(url, "expected").unwrap_err();
        assert!(err.to_string().contains("State mismatch"));
    }

    #[test]
    fn parse_callback_missing_state() {
        let url = "http://localhost:9999/callback?token=abc";
        let err = parse_callback(url, "any").unwrap_err();
        assert!(err.to_string().contains("Missing state"));
    }

    #[test]
    fn parse_callback_missing_token() {
        let url = "http://localhost:9999/callback?state=ok";
        let err = parse_callback(url, "ok").unwrap_err();
        assert!(err.to_string().contains("Missing token"));
    }

    #[test]
    fn parse_callback_empty_token() {
        let url = "http://localhost:9999/callback?token=&state=ok";
        let err = parse_callback(url, "ok").unwrap_err();
        assert!(err.to_string().contains("empty token"));
    }

    #[test]
    fn parse_callback_invalid_url() {
        let err = parse_callback("not a url", "any").unwrap_err();
        assert!(err.to_string().contains("Invalid callback URL"));
    }

    #[tokio::test]
    async fn callback_server_integration() {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let port = server.server_addr().to_ip().unwrap().port();

        let state = "test-state-123";
        let token = "jwt-token-xyz";

        let handle = tokio::task::spawn_blocking(move || {
            let req = server
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap()
                .unwrap();
            let url = format!("http://localhost:{}{}", port, req.url());
            let response = tiny_http::Response::from_string("ok");
            let _ = req.respond(response);
            url
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let callback_url = format!("http://127.0.0.1:{port}/callback?token={token}&state={state}");
        reqwest::get(&callback_url).await.unwrap();

        let received_url = handle.await.unwrap();
        let result = parse_callback(&received_url, state).unwrap();
        assert_eq!(result.token, token);
    }
}
