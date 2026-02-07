use std::{
    error::Error,
    fmt::{Display, Formatter},
    sync::LazyLock,
};

use reqwest::{Client, Method, RequestBuilder};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::consts::{get_api_key, API_PATH, USER_AGENT};

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

pub enum BungieRequest<'a> {
    SearchDestinyPlayerByBungieName {
        display_name: &'a str,
        display_name_code: usize,
    },
    GetProfile {
        membership_type: usize,
        membership_id: &'a str,
        component: usize,
    },
    GetActivityHistory {
        membership_type: usize,
        membership_id: &'a str,
        character_id: &'a str,
        page: usize,
        mode: usize,
    },
    GetPostGameCarnageReport {
        activity_id: &'a str,
    },
    GetDestinyActivityDefinition {
        activity_hash: usize,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BungieResponseStatus {
    error_code: isize,
    message: String,
    throttle_seconds: isize,
    response: Option<Value>,
}

#[derive(Debug)]
pub enum BungieResponseError {
    DeserializeError {
        err: serde_json::Error,
        status_code: u16,
    },
    BungieError {
        message: String,
        error_code: isize,
        throttle_seconds: isize,
    },
    ResponseMissing,
    NetworkError(anyhow::Error),
}

impl Display for BungieResponseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BungieResponseError::DeserializeError { err, status_code } => {
                write!(f, "Failed to parse response (code {status_code}): {err}")
            }
            BungieResponseError::BungieError {
                message,
                error_code,
                throttle_seconds,
            } => {
                if *throttle_seconds > 0 {
                    write!(
                        f,
                        "{message} ({error_code}), throttled! ({throttle_seconds}s)"
                    )
                } else {
                    write!(f, "{message} ({error_code})")
                }
            }
            BungieResponseError::ResponseMissing => f.write_str("Response object missing"),
            BungieResponseError::NetworkError(e) => e.fmt(f),
        }
    }
}

impl Error for BungieResponseError {}

fn api_request(path: &str, method: Method) -> RequestBuilder {
    HTTP_CLIENT
        .request(method, format!("{API_PATH}{path}"))
        .header("User-Agent", USER_AGENT)
        .header("X-API-Key", get_api_key())
}

pub async fn make_request(req: BungieRequest<'_>) -> Result<Value, BungieResponseError> {
    make_request_with_retry(req, 3).await
}

async fn make_request_with_retry(req: BungieRequest<'_>, max_retries: u32) -> Result<Value, BungieResponseError> {
    let mut retry_count = 0;
    
    loop {
        let builder = match &req {
            BungieRequest::SearchDestinyPlayerByBungieName { display_name, display_name_code } => api_request(
                "/Destiny2/SearchDestinyPlayerByBungieName/All",
                Method::POST,
            ).body(json!({"displayName": display_name, "displayNameCode": display_name_code}).to_string()),
            BungieRequest::GetProfile { membership_type, membership_id, component } => {
                api_request(&format!("/Destiny2/{membership_type}/Profile/{membership_id}?components={component}"), Method::GET)
            }
            BungieRequest::GetActivityHistory { membership_type, membership_id, character_id, page, mode } => {
                api_request(&format!("/Destiny2/{membership_type}/Account/{membership_id}/Character/{character_id}/Stats/Activities?mode={mode}&count=25&page={page}"), Method::GET)
            }
            BungieRequest::GetPostGameCarnageReport { activity_id } => {
                api_request(&format!("/Destiny2/Stats/PostGameCarnageReport/{activity_id}"), Method::GET)
            }
            BungieRequest::GetDestinyActivityDefinition { activity_hash } => api_request(&format!("/Destiny2/Manifest/DestinyActivityDefinition/{activity_hash}"), Method::GET),
        };

        let resp = builder
            .send()
            .await
            .map_err(|e| BungieResponseError::NetworkError(e.into()))?;

        let status_code = resp.status().as_u16();
        
        // Handle 503 Service Unavailable with retry
        if status_code == 503 {
            if retry_count < max_retries {
                retry_count += 1;
                let wait_time = 2u64.pow(retry_count); // Exponential backoff: 2s, 4s, 8s
                tokio::time::sleep(tokio::time::Duration::from_secs(wait_time)).await;
                continue;
            } else {
                return Err(BungieResponseError::NetworkError(
                    anyhow::anyhow!("Bungie API unavailable (503) after {} retries", max_retries)
                ));
            }
        }

        let text = resp
            .text()
            .await
            .map_err(|e| BungieResponseError::NetworkError(e.into()))?;

        let status: BungieResponseStatus = match serde_json::from_str(&text) {
            Ok(s) => s,
            Err(e) => {
                return Err(BungieResponseError::DeserializeError {
                    err: e,
                    status_code,
                }
                .into())
            }
        };

        if status.error_code != 1 {
            return Err(BungieResponseError::BungieError {
                message: status.message,
                error_code: status.error_code,
                throttle_seconds: status.throttle_seconds,
            }
            .into());
        }

        return Ok(status
            .response
            .ok_or(BungieResponseError::ResponseMissing)?);
    }
}
