use reqwest::{header::CONTENT_TYPE, Client, Method, RequestBuilder, Response};
use reqwest_eventsource::{CannotCloneRequestError, EventSource, RequestBuilderExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::env;
use std::env::VarError;
use std::sync::{LazyLock, RwLock};

pub mod messages;

pub static DEFAULT_BASE_URL: LazyLock<String> =
    LazyLock::new(|| String::from("https://api.anthropic.com/v1/"));
static DEFAULT_CREDENTIALS: LazyLock<RwLock<Credentials>> =
    LazyLock::new(|| RwLock::new(Credentials::from_env()));

/// Holds the API key and base URL for an Anthropic-compatible API.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Credentials {
    api_key: String,
    base_url: String,
}

impl Credentials {
    /// Creates credentials with the given API key and base URL.
    ///
    /// If the base URL is empty, it will use the default.
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let base_url = if base_url.is_empty() {
            DEFAULT_BASE_URL.clone()
        } else {
            parse_base_url(base_url)
        };
        Self {
            api_key: api_key.into(),
            base_url,
        }
    }

    /// Fetches the credentials from the ENV variables
    /// ANTHROPIC_KEY and ANTHROPIC_BASE_URL.
    /// # Panics
    /// This function will panic if the key variable is missing from the env.
    /// If only the base URL variable is missing, it will use the default.
    pub fn from_env() -> Credentials {
        let api_key = env::var("ANTHROPIC_API_KEY").unwrap();
        let base_url_unparsed = env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|e| match e {
            VarError::NotPresent => DEFAULT_BASE_URL.clone(),
            VarError::NotUnicode(v) => panic!("ANTHROPIC_BASE_URL is not unicode: {v:#?}"),
        });
        let base_url = parse_base_url(base_url_unparsed);
        Credentials { api_key, base_url }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AnthropicError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AnthropicErrorResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub error: AnthropicError,
}
impl AnthropicErrorResponse {
    fn new(message: String, error_type: String) -> AnthropicErrorResponse {
        AnthropicErrorResponse {
            response_type: "error".to_string(),
            error: AnthropicError {
                message,
                error_type,
            },
        }
    }
}

impl std::fmt::Display for AnthropicErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.error.message)
    }
}

impl std::error::Error for AnthropicErrorResponse {}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Err { error: AnthropicErrorResponse },
    Ok(T),
}

#[derive(Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}

pub type ApiResponseOrError<T> = Result<T, AnthropicErrorResponse>;

impl From<reqwest::Error> for AnthropicErrorResponse {
    fn from(value: reqwest::Error) -> Self {
        AnthropicErrorResponse::new(value.to_string(), "reqwest".to_string())
    }
}

impl From<std::io::Error> for AnthropicErrorResponse {
    fn from(value: std::io::Error) -> Self {
        AnthropicErrorResponse::new(value.to_string(), "io".to_string())
    }
}

async fn anthropic_request_json<F, T>(
    method: Method,
    route: &str,
    builder: F,
    credentials_opt: Option<Credentials>,
) -> ApiResponseOrError<T>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
    T: DeserializeOwned,
{
    let api_response = anthropic_request(method, route, builder, credentials_opt)
        .await?
        .json()
        .await?;

    match api_response {
        ApiResponse::Ok(t) => Ok(t),
        ApiResponse::Err { error } => Err(error),
    }
}

async fn anthropic_request<F>(
    method: Method,
    route: &str,
    builder: F,
    credentials_opt: Option<Credentials>,
) -> ApiResponseOrError<Response>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    let client = Client::new();
    let credentials =
        credentials_opt.unwrap_or_else(|| DEFAULT_CREDENTIALS.read().unwrap().clone());
    let mut request = client.request(method, format!("{}{route}", credentials.base_url));

    request = builder(request);

    let response = request
        .header("x-api-key", credentials.api_key)
        .header("anthropic-version", "2023-06-01")
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    Ok(response)
}

async fn anthropic_request_stream<F>(
    method: Method,
    route: &str,
    builder: F,
    credentials_opt: Option<Credentials>,
) -> Result<EventSource, CannotCloneRequestError>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    let client = Client::new();
    let credentials =
        credentials_opt.unwrap_or_else(|| DEFAULT_CREDENTIALS.read().unwrap().clone());
    let mut request = client.request(method, format!("{}{route}", credentials.base_url));
    request = builder(request);
    let stream = request
        .header("x-api-key", credentials.api_key)
        .header("anthropic-version", "2023-06-01")
        .header(CONTENT_TYPE, "application/json")
        .eventsource()?;
    Ok(stream)
}


async fn anthropic_post<J, T>(
    route: &str,
    json: &J,
    credentials_opt: Option<Credentials>,
) -> ApiResponseOrError<T>
where
    J: Serialize + ?Sized,
    T: DeserializeOwned,
{
    anthropic_request_json(
        Method::POST,
        route,
        |request| request.json(json),
        credentials_opt,
    )
    .await
}

fn parse_base_url(mut value: String) -> String {
    if !value.ends_with('/') {
        value += "/";
    }
    value
}

#[cfg(test)]
pub mod tests {
    pub const DEFAULT_LEGACY_MODEL: &str = "claude-3-5-sonnet-20240620";
}