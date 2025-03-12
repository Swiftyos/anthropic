//! # Anthropic Rust SDK
//!
//! An unofficial Rust library for interacting with the [Anthropic API](https://www.anthropic.com/api).
//! This library provides an asynchronous interface to Anthropic's services, allowing Rust developers
//! to seamlessly integrate Anthropic's AI capabilities into their applications.
//!
//! ## Features
//!
//! - **Asynchronous API Requests**: Leverage Rust's async capabilities for efficient API interactions
//! - **Message API**: Send and receive messages, similar to chat-based interactions
//! - **Tool Use**: Integrate external tools that the AI can call during responses
//! - **Streaming Responses**: Receive real-time streamed responses from the API
//!
//! ## Basic Usage
//!
//! ```no_run
//! use anthropic_api::{messages::*, Credentials};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Load credentials from the environment
//!     let credentials = Credentials::from_env();
//!
//!     // Create a message
//!     let messages = vec![Message {
//!         role: MessageRole::User,
//!         content: MessageContent::Text("Hello, Claude!".to_string()),
//!     }];
//!
//!     // Send the message to the Anthropic API
//!     let response = MessagesAPI::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
//!         .credentials(credentials.clone())
//!         .create()
//!         .await
//!         .unwrap();
//!
//!     // Print the assistant's response
//!     if let Some(ResponseContentBlock::Text { text }) = response.content.first() {
//!         println!("Assistant: {}", text.trim());
//!     }
//! }
//! ```

use reqwest::{header::CONTENT_TYPE, Client, Method, RequestBuilder, Response};
use reqwest_eventsource::{CannotCloneRequestError, EventSource, RequestBuilderExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::env;
use std::env::VarError;
use std::sync::{LazyLock, RwLock};

pub mod admin;
pub mod messages;
pub mod models;
/// Default base URL for the Anthropic API
pub static DEFAULT_BASE_URL: LazyLock<String> =
    LazyLock::new(|| String::from("https://api.anthropic.com/v1/"));
/// Default credentials loaded from environment variables
static DEFAULT_CREDENTIALS: LazyLock<RwLock<Credentials>> =
    LazyLock::new(|| RwLock::new(Credentials::from_env()));

/// Holds the API key and base URL for an Anthropic-compatible API.
///
/// This struct is used to authenticate requests to the Anthropic API.
/// It can be created from environment variables or explicitly with an API key and base URL.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Credentials {
    api_key: String,
    base_url: String,
}

impl Credentials {
    /// Creates credentials with the given API key and base URL.
    ///
    /// If the base URL is empty, it will use the default Anthropic API URL.
    ///
    /// # Examples
    ///
    /// ```
    /// use anthropic_api::Credentials;
    ///
    /// let credentials = Credentials::new("your-api-key", "");
    /// ```
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

    /// Fetches the credentials from the environment variables
    /// `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL`.
    ///
    /// # Panics
    ///
    /// This function will panic if the `ANTHROPIC_API_KEY` variable is missing from the environment.
    /// If only the `ANTHROPIC_BASE_URL` variable is missing, it will use the default URL.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use anthropic_api::Credentials;
    ///
    /// // Assumes ANTHROPIC_API_KEY is set in the environment
    /// let credentials = Credentials::from_env();
    /// ```
    pub fn from_env() -> Credentials {
        let api_key = env::var("ANTHROPIC_API_KEY").unwrap();
        let base_url_unparsed = env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|e| match e {
            VarError::NotPresent => DEFAULT_BASE_URL.clone(),
            VarError::NotUnicode(v) => panic!("ANTHROPIC_BASE_URL is not unicode: {v:#?}"),
        });
        let base_url = parse_base_url(base_url_unparsed);
        Credentials { api_key, base_url }
    }

    /// Returns the API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Represents an error returned by the Anthropic API
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AnthropicError {
    /// The type of error
    #[serde(rename = "type")]
    pub error_type: String,
    /// The error message
    pub message: String,
}

/// Represents an error response from the Anthropic API
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AnthropicErrorResponse {
    /// The type of response (always "error")
    #[serde(rename = "type")]
    pub response_type: String,
    /// The error details
    pub error: AnthropicError,
}

impl AnthropicErrorResponse {
    /// Creates a new error response with the given message and error type
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

/// Represents a response from the Anthropic API, which can be either a success or an error
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    /// An error response
    Err { error: AnthropicErrorResponse },
    /// A successful response
    Ok(T),
}

/// Represents token usage statistics for a request and response
#[derive(Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage {
    /// Number of tokens in the input
    pub input_tokens: u32,
    /// Number of tokens in the output
    pub output_tokens: u32,
    /// Number of tokens used for cache creation, if applicable
    pub cache_creation_input_tokens: Option<u32>,
    /// Number of tokens read from cache, if applicable
    pub cache_read_input_tokens: Option<u32>,
}

/// Result type for Anthropic API responses
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

/// Makes a request to the Anthropic API and deserializes the JSON response
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

/// Makes a request to the Anthropic API
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

/// Creates an event source for streaming responses from the Anthropic API
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

/// Makes a POST request to the Anthropic API with the given JSON payload
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

/// Ensures the base URL ends with a trailing slash
fn parse_base_url(mut value: String) -> String {
    if !value.ends_with('/') {
        value += "/";
    }
    value
}

/// Test utilities
#[cfg(test)]
pub mod tests {
    /// Default model to use in tests
    pub const DEFAULT_LEGACY_MODEL: &str = "claude-3-5-sonnet-20240620";
}
