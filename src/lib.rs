//! # Anthropic Rust SDK
//!
//! An unofficial Rust library for interacting with the [Anthropic API](https://www.anthropic.com/api).
//! This library provides an asynchronous interface to Anthropic's services, allowing Rust developers
//! to seamlessly integrate Anthropic's AI capabilities into their applications.
//!
//! ## Features
//!
//! - **Asynchronous API Requests**: Leverage Rust's async capabilities for efficient API interactions.
//! - **Message API**: Send and receive messages, similar to chat-based interactions.
//! - **Tool Use**: Integrate external tools that the AI can call during responses.
//! - **Streaming Responses**: Receive real-time streamed responses from the API.
//! - **Structured Logging & Tracing**: Built on top of the `tracing` crate to provide robust, context-rich logs.
//!
//! ## Basic Usage
//!
//! Before using the library, initialize your logging system (for example, using `tracing_subscriber`):
//!
//! ```no_run
//! use tracing_subscriber;
//!
//! // Set up logging using tracing_subscriber.
//! tracing_subscriber::fmt::init();
//! ```
//!
//! Then use the library as usual.

use reqwest::{header::CONTENT_TYPE, Client, Method, RequestBuilder, Response};
use reqwest_eventsource::{CannotCloneRequestError, EventSource, RequestBuilderExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::env;
use std::env::VarError;
use std::fmt::Debug;
use std::sync::{LazyLock, RwLock};
use tracing::{debug, error, info, instrument, trace, warn};

pub mod admin;
pub mod messages;
pub mod models;

/// Default base URL for the Anthropic API.
pub static DEFAULT_BASE_URL: LazyLock<String> =
    LazyLock::new(|| String::from("https://api.anthropic.com/v1/"));

/// Default credentials loaded from environment variables.
static DEFAULT_CREDENTIALS: LazyLock<RwLock<Credentials>> =
    LazyLock::new(|| RwLock::new(Credentials::from_env()));

/// Holds the API key and base URL for an Anthropic-compatible API.
///
/// This struct is used to authenticate requests to the Anthropic API.
/// It can be created from environment variables or explicitly with an API key and base URL.
#[derive(Clone, Eq, PartialEq)]
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
    #[instrument(skip(api_key, base_url))]
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        let base_url = if base_url.is_empty() {
            debug!("No base URL provided, using default");
            DEFAULT_BASE_URL.clone()
        } else {
            debug!("Using custom base URL");
            parse_base_url(base_url)
        };
        trace!("Credentials created with base URL: {}", base_url);
        Self {
            api_key: api_key.into(),
            base_url,
        }
    }

    /// Fetches the credentials from the environment variables `ANTHROPIC_API_KEY` and `ANTHROPIC_BASE_URL`.
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
    #[instrument]
    pub fn from_env() -> Credentials {
        debug!("Loading credentials from environment variables");
        let api_key = match env::var("ANTHROPIC_API_KEY") {
            Ok(key) => {
                debug!("Found ANTHROPIC_API_KEY in environment");
                key
            }
            Err(_) => {
                error!("ANTHROPIC_API_KEY not found in environment");
                panic!("ANTHROPIC_API_KEY environment variable is required");
            }
        };

        let base_url_unparsed = env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|e| match e {
            VarError::NotPresent => {
                debug!("ANTHROPIC_BASE_URL not found, using default");
                DEFAULT_BASE_URL.clone()
            }
            VarError::NotUnicode(v) => {
                error!("ANTHROPIC_BASE_URL is not valid unicode: {v:#?}");
                panic!("ANTHROPIC_BASE_URL is not unicode: {v:#?}");
            }
        });

        let base_url = parse_base_url(base_url_unparsed);
        debug!("Using base URL: {}", base_url);
        Credentials { api_key, base_url }
    }

    /// Returns the API key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Redact the API key for security.
        write!(
            f,
            "Credentials {{ api_key: [REDACTED], base_url: {} }}",
            self.base_url
        )
    }
}

/// Represents an error returned by the Anthropic API.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AnthropicError {
    /// The type of error.
    #[serde(rename = "type")]
    pub error_type: String,
    /// The error message.
    pub message: String,
}

/// Represents an error response from the Anthropic API.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct AnthropicErrorResponse {
    /// The type of response (always "error").
    #[serde(rename = "type")]
    pub response_type: String,
    /// The error details.
    pub error: AnthropicError,
}

impl AnthropicErrorResponse {
    /// Creates a new error response with the given message and error type.
    #[instrument]
    fn new(message: String, error_type: String) -> AnthropicErrorResponse {
        warn!(%message, %error_type, "Creating error response");
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

/// Represents a response from the Anthropic API, which can be either a success or an error.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    /// An error response.
    Err { error: AnthropicErrorResponse },
    /// A successful response.
    Ok(T),
}

/// Represents token usage statistics for a request and response.
#[derive(Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct Usage {
    /// Number of tokens in the input.
    pub input_tokens: u32,
    /// Number of tokens in the output.
    pub output_tokens: u32,
    /// Number of tokens used for cache creation, if applicable.
    pub cache_creation_input_tokens: Option<u32>,
    /// Number of tokens read from cache, if applicable.
    pub cache_read_input_tokens: Option<u32>,
}

/// Result type for Anthropic API responses.
pub type ApiResponseOrError<T> = Result<T, AnthropicErrorResponse>;

impl From<reqwest::Error> for AnthropicErrorResponse {
    fn from(value: reqwest::Error) -> Self {
        error!(error = %value, "Reqwest error occurred");
        AnthropicErrorResponse::new(value.to_string(), "reqwest".to_string())
    }
}

impl From<std::io::Error> for AnthropicErrorResponse {
    fn from(value: std::io::Error) -> Self {
        error!(error = %value, "IO error occurred");
        AnthropicErrorResponse::new(value.to_string(), "io".to_string())
    }
}

/// Makes a request to the Anthropic API and deserializes the JSON response.
///
/// This function logs the raw API response for debugging while ensuring sensitive data remains redacted.
#[instrument(skip(builder, credentials_opt), fields(route = %route))]
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
    debug!(?method, "Making JSON request to Anthropic API");
    let response = anthropic_request(method, route, builder, credentials_opt).await?;

    // Log the raw response body for debugging.
    let response_text = response.text().await?;
    debug!(response_body = %response_text, "Raw API response");

    // Parse the response text back to JSON.
    let api_response: ApiResponse<T> = match serde_json::from_str(&response_text) {
        Ok(parsed) => parsed,
        Err(e) => {
            error!(error = %e, response_text = %response_text, "Failed to parse API response");
            return Err(AnthropicErrorResponse::new(
                format!("Failed to parse API response: {}", e),
                "json_parse_error".to_string(),
            ));
        }
    };

    match api_response {
        ApiResponse::Ok(t) => {
            info!("Successfully received and parsed JSON response");
            Ok(t)
        }
        ApiResponse::Err { error } => {
            warn!(error_type = %error.error.error_type, message = %error.error.message, "Received error response from API");
            Err(error)
        }
    }
}

/// Makes a request to the Anthropic API.
///
/// This function logs only non-sensitive details (method and URL) to avoid exposing confidential data.
#[instrument(skip(builder, credentials_opt), fields(route = %route))]
async fn anthropic_request<F>(
    method: Method,
    route: &str,
    builder: F,
    credentials_opt: Option<Credentials>,
) -> ApiResponseOrError<Response>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    debug!(?method, "Making request to Anthropic API");
    let client = Client::new();
    let credentials =
        credentials_opt.unwrap_or_else(|| DEFAULT_CREDENTIALS.read().unwrap().clone());
    let base_url = credentials.base_url();
    let url = format!("{}{route}", base_url);
    trace!(url = %url, "Constructed full URL");

    let mut request = client.request(method.clone(), url.clone());
    request = builder(request);

    // Log safe request details.
    debug!(method = ?method, url = %url, "Request details");

    trace!("Sending request with headers");
    let response = request
        .header("x-api-key", credentials.api_key)
        .header("anthropic-version", "2023-06-01")
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await?;

    let status = response.status();
    debug!(status = %status, headers = ?response.headers(), "Response headers");

    if status.is_success() {
        info!(status = %status, "Request successful");
    } else {
        warn!(status = %status, "Request returned non-success status code");
    }

    Ok(response)
}

/// Creates an event source for streaming responses from the Anthropic API.
///
/// This function ensures that only safe-to-log information (method and URL) is included.
#[instrument(skip(builder, credentials_opt), fields(route = %route))]
async fn anthropic_request_stream<F>(
    method: Method,
    route: &str,
    builder: F,
    credentials_opt: Option<Credentials>,
) -> Result<EventSource, CannotCloneRequestError>
where
    F: FnOnce(RequestBuilder) -> RequestBuilder,
{
    debug!(
        ?method,
        "Creating event source for streaming from Anthropic API"
    );
    let client = Client::new();
    let credentials =
        credentials_opt.unwrap_or_else(|| DEFAULT_CREDENTIALS.read().unwrap().clone());
    let base_url = credentials.base_url();
    let url = format!("{}{route}", base_url);
    trace!(url = %url, "Constructed full URL for streaming");

    let mut request = client.request(method.clone(), url.clone());
    request = builder(request);

    // Log safe details for the streaming request.
    debug!(method = ?method, url = %url, "Streaming request details");

    trace!("Creating event source");
    let stream = request
        .header("x-api-key", credentials.api_key)
        .header("anthropic-version", "2023-06-01")
        .header(CONTENT_TYPE, "application/json")
        .eventsource()?;

    info!("Successfully created event source for streaming");
    Ok(stream)
}

/// Makes a POST request to the Anthropic API with the given JSON payload.
///
/// This function logs the payload after redacting sensitive data.
#[instrument(skip(json, credentials_opt), fields(route = %route))]
async fn anthropic_post<J, T>(
    route: &str,
    json: &J,
    credentials_opt: Option<Credentials>,
) -> ApiResponseOrError<T>
where
    J: Serialize + ?Sized,
    T: DeserializeOwned,
{
    debug!("Making POST request to Anthropic API");
    // Log the payload with sensitive data redacted.
    if let Ok(json_str) = serde_json::to_string(json) {
        let default_creds = DEFAULT_CREDENTIALS.read().unwrap();
        let credentials = credentials_opt.as_ref().unwrap_or(&default_creds);
        let redacted_json = json_str.replace(credentials.api_key(), "[REDACTED_API_KEY]");
        debug!(payload = %redacted_json, "POST request payload");
    }

    anthropic_request_json(
        Method::POST,
        route,
        |request| request.json(json),
        credentials_opt,
    )
    .await
}

/// Ensures the base URL ends with a trailing slash.
///
/// This function adds a trailing slash if not already present to avoid URL construction errors.
#[instrument]
fn parse_base_url(mut value: String) -> String {
    trace!(original_url = %value, "Parsing base URL");
    if !value.ends_with('/') {
        debug!("Adding trailing slash to base URL");
        value.push('/');
    }
    trace!(parsed_url = %value, "Parsed base URL");
    value
}

/// Test utilities.
#[cfg(test)]
pub mod tests {
    /// Default model to use in tests.
    pub const DEFAULT_LEGACY_MODEL: &str = "claude-3-5-sonnet-20240620";
}
