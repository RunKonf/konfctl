use std::time::Duration;

use anyhow::{Context, Result, bail};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::config::Config;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
struct TrpcResponse<T> {
    result: TrpcResult<T>,
}

#[derive(Deserialize)]
struct TrpcResult<T> {
    data: T,
}

#[derive(Deserialize)]
struct TrpcErrorResponse {
    error: TrpcErrorDetail,
}

#[derive(Deserialize)]
struct TrpcErrorDetail {
    message: String,
}

pub struct TrpcClient {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl TrpcClient {
    fn build_http() -> reqwest::Client {
        reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("failed to build HTTP client")
    }

    pub fn from_config(config: &Config) -> Self {
        Self {
            http: Self::build_http(),
            base_url: config.api_url.clone(),
            token: config.token.clone(),
        }
    }

    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            http: Self::build_http(),
            base_url: base_url.to_string(),
            token: token.to_string(),
        }
    }

    pub async fn query<T: DeserializeOwned>(
        &self,
        procedure: &str,
        input: Option<&serde_json::Value>,
    ) -> Result<T> {
        let mut url = format!("{}/api/trpc/{}", self.base_url, procedure);

        if let Some(input) = input {
            let encoded = serde_json::to_string(input)?;
            url = format!("{}?input={}", url, urlencoding::encode(&encoded));
        }

        let resp = self
            .http
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .send()
            .await
            .context("Failed to connect to API")?;

        self.handle_response(resp, procedure).await
    }

    pub async fn mutate<T: DeserializeOwned>(
        &self,
        procedure: &str,
        input: &serde_json::Value,
    ) -> Result<T> {
        let url = format!("{}/api/trpc/{}", self.base_url, procedure);

        let resp = self
            .http
            .post(&url)
            .header(AUTHORIZATION, format!("Bearer {}", self.token))
            .header(CONTENT_TYPE, "application/json")
            .json(input)
            .send()
            .await
            .context("Failed to connect to API")?;

        self.handle_response(resp, procedure).await
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        resp: reqwest::Response,
        procedure: &str,
    ) -> Result<T> {
        let status = resp.status();
        let body = resp.text().await.context("Failed to read response body")?;

        if !status.is_success() {
            if let Ok(err) = serde_json::from_str::<TrpcErrorResponse>(&body) {
                bail!("{}: {}", procedure, err.error.message);
            }
            bail!("{procedure}: HTTP {status} — {body}");
        }

        let parsed: TrpcResponse<T> = serde_json::from_str(&body).with_context(|| {
            let preview = truncate_body(&body, 512);
            format!("Failed to parse response from {procedure}\n\nResponse body (first 512 bytes):\n{preview}")
        })?;

        Ok(parsed.result.data)
    }
}

fn truncate_body(body: &str, max: usize) -> String {
    if body.chars().count() <= max {
        body.to_string()
    } else {
        let truncated: String = body.chars().take(max).collect();
        format!("{truncated}…")
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method, path, query_param_contains};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn query_without_input() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/trpc/test.procedure"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"result": {"data": "hello"}})),
            )
            .mount(&server)
            .await;

        let client = TrpcClient::new(&server.uri(), "test-token");
        let result: String = client.query("test.procedure", None).await.unwrap();
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn query_with_input() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/trpc/proposal.admin.list"))
            .and(query_param_contains("input", "conferenceId"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": {"data": [{"id": "1", "title": "Talk 1"}]}
            })))
            .mount(&server)
            .await;

        let client = TrpcClient::new(&server.uri(), "tok");
        let input = serde_json::json!({"conferenceId": "conf-123"});

        #[derive(Debug, Deserialize, PartialEq)]
        struct Item {
            id: String,
            title: String,
        }

        let result: Vec<Item> = client
            .query("proposal.admin.list", Some(&input))
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Talk 1");
    }

    #[tokio::test]
    async fn mutate_sends_post_with_json_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/trpc/test.mutate"))
            .and(header("Authorization", "Bearer my-token"))
            .and(header("Content-Type", "application/json"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"result": {"data": true}})),
            )
            .mount(&server)
            .await;

        let client = TrpcClient::new(&server.uri(), "my-token");
        let result: bool = client
            .mutate("test.mutate", &serde_json::json!({"key": "value"}))
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn trpc_error_response_extracts_message() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/trpc/bad.proc"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "error": {"message": "UNAUTHORIZED"}
            })))
            .mount(&server)
            .await;

        let client = TrpcClient::new(&server.uri(), "bad-token");
        let err = client.query::<String>("bad.proc", None).await.unwrap_err();
        assert!(err.to_string().contains("UNAUTHORIZED"));
        assert!(err.to_string().contains("bad.proc"));
    }

    #[tokio::test]
    async fn non_trpc_error_includes_status_and_body() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/trpc/fail"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;

        let client = TrpcClient::new(&server.uri(), "tok");
        let err = client.query::<String>("fail", None).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("fail"));
        assert!(msg.contains("500"));
    }

    #[tokio::test]
    async fn invalid_json_response_errors() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/trpc/bad.json"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json at all"))
            .mount(&server)
            .await;

        let client = TrpcClient::new(&server.uri(), "tok");
        let err = client.query::<String>("bad.json", None).await.unwrap_err();
        assert!(err.to_string().contains("Failed to parse response"));
    }

    #[tokio::test]
    async fn connection_refused_errors() {
        let client = TrpcClient::new("http://127.0.0.1:1", "tok");
        let err = client.query::<String>("any.proc", None).await.unwrap_err();
        assert!(err.to_string().contains("Failed to connect to API"));
    }
}
