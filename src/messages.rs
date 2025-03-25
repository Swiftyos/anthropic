//! # Messages API
//!
//! This module provides a Rust interface to Anthropic's Messages API, which allows you to interact
//! with Claude models in a conversational manner.
//!
//! ## Key Features
//!
//! - Send messages to Claude models and receive responses
//! - Support for streaming responses
//! - Tool usage capabilities
//! - Image input support
//!
//! ## Basic Usage
//!
//! ```no_run
//! use anthropic_api::{messages::*, Credentials};
//!
//! #[tokio::main]
//! async fn main() {
//!     let credentials = Credentials::from_env();
//!
//!     let response =MessagesBuilder::builder(
//!         "claude-3-7-sonnet-20250219",
//!         vec![Message {
//!             role: MessageRole::User,
//!             content: MessageContent::Text("Hello, Claude!".to_string()),
//!         }],
//!         1024,
//!     )
//!     .credentials(credentials)
//!     .create()
//!     .await
//!     .unwrap();
//!
//!     println!("Claude says: {:?}", response.content);
//! }
//! ```

use crate::{anthropic_post, anthropic_request_stream, ApiResponseOrError, Credentials, Usage};
use anyhow::Result;
use derive_builder::Builder;
use futures_util::StreamExt;
use reqwest::Method;
use reqwest_eventsource::{CannotCloneRequestError, Event, EventSource};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// Represents a full message response from the Anthropic API.
///
/// This struct contains the complete response from a message request, including
/// the model's generated content and usage statistics.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MessagesResponse {
    /// Unique identifier for this message
    pub id: String,
    /// The model that generated the response
    pub model: String,
    /// The role of the message sender (always Assistant for responses)
    pub role: MessageRole,
    /// The content blocks in the response (text, tool use, thinking, redacted thinking)
    pub content: Vec<ResponseContentBlock>,
    /// Reason why the model stopped generating, if applicable
    pub stop_reason: Option<String>,
    /// The specific sequence that caused generation to stop, if applicable
    pub stop_sequence: Option<String>,
    /// The type of the response (always "message")
    #[serde(rename = "type")]
    pub typ: String,
    /// Token usage statistics for the request and response
    pub usage: Usage,
}

/// Content block in a response, can be text or tool use.
///
/// Claude's responses can contain different types of content blocks.
/// Currently, this can be either text, a tool use request, a thinking block, or a redacted thinking block.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum ResponseContentBlock {
    /// A text content block containing natural language
    #[serde(rename = "text")]
    Text { text: String },
    /// A tool use request from the model
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    /// A thinking block from the model
    #[serde(rename = "thinking")]
    Thinking { signature: String, thinking: String },
    /// A redacted thinking block from the model
    #[serde(rename = "redacted_thinking")]
    RedactedThinking { data: String },
}

/// Streaming events from the Anthropic API.
///
/// When using streaming mode, the API returns a series of events that
/// incrementally build up the complete response.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum StreamEvent {
    /// Indicates the start of a message
    #[serde(rename = "message_start")]
    MessageStart { message: MessageStart },
    /// Indicates the start of a content block
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: ContentBlockStart,
    },
    /// Contains a delta (incremental update) to a content block
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u32,
        delta: ContentBlockDelta,
    },
    /// Indicates the end of a content block
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },
    /// Contains final message information like stop reason
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDelta, usage: Usage },
    /// Indicates the end of the message
    #[serde(rename = "message_stop")]
    MessageStop,
    /// A keepalive event that can be ignored
    #[serde(rename = "ping")]
    Ping,
}

/// Initial message information in a streaming response.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MessageStart {
    /// Unique identifier for this message
    pub id: String,
    /// The model generating the response
    pub model: String,
    /// The role of the message sender (always Assistant for responses)
    pub role: MessageRole,
    /// Initial content blocks in the response
    pub content: Vec<ContentBlockStart>,
}

/// Initial content block in a streaming response.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum ContentBlockStart {
    /// A text content block
    Text { text: String },
    /// A tool use request
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

/// Incremental update to a content block in a streaming response.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum ContentBlockDelta {
    /// Text delta for a text content block
    Text { text: String },
    /// JSON delta for a tool use input
    InputJsonDelta { partial_json: String },
}

/// Final message information in a streaming response.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MessageDelta {
    /// Reason why the model stopped generating, if applicable
    pub stop_reason: Option<String>,
    /// The specific sequence that caused generation to stop, if applicable
    pub stop_sequence: Option<String>,
}

/// Request to the Anthropic Messages API.
///
/// This struct represents a complete request to the Messages API,
/// including all parameters that control generation behavior.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "MessagesBuilder")]
#[builder(setter(strip_option, into))]
pub struct MessagesRequest {
    /// The model to use (e.g., "claude-3-7-sonnet-20250219").
    pub model: String,
    /// The conversation messages.
    pub messages: Vec<Message>,
    /// Maximum number of tokens to generate.
    pub max_tokens: u64,
    /// Optional metadata.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// Sequences where generation should stop.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Whether to stream the response.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// System prompt to guide the assistant's behavior.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Sampling temperature (0.0 to 1.0).
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<Thinking>,
    /// Tool choice specification.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Tools the assistant can use.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Top-k sampling parameter.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// Top-p (nucleus) sampling parameter.
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Credentials for authentication (not serialized).
    #[serde(skip_serializing)]
    #[builder(default)]
    pub credentials: Option<Credentials>,
}

/// Message in the conversation.
///
/// Represents a single message in the conversation history,
/// with a role (user or assistant) and content.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Message {
    /// The role of the message sender (user or assistant)
    pub role: MessageRole,
    /// The content of the message (text or content blocks)
    pub content: MessageContent,
}

/// Role of the message sender.
///
/// In the Messages API, messages can be from either the user or the assistant.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from the user
    User,
    /// Message from the assistant (Claude)
    Assistant,
}

/// Content of a message, either text or content blocks.
///
/// Messages can contain either simple text or structured content blocks
/// that can include text and images.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text(String),
    /// Structured content blocks (text and images)
    ContentBlocks(Vec<RequestContentBlock>),
}

/// Content block in a request.
///
/// Request content blocks can be either text or images.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum RequestContentBlock {
    /// A text content block
    #[serde(rename = "text")]
    Text { text: String },
    /// An image content block
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

/// Source of an image content block.
///
/// Currently, images must be provided as base64-encoded data.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ImageSource {
    /// The type of image source (currently only "base64" is supported)
    #[serde(rename = "type")]
    pub source_type: String,
    /// The MIME type of the image (e.g., "image/png", "image/jpeg")
    pub media_type: String,
    /// The base64-encoded image data
    pub data: String,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub enum ThinkingType {
    /// Whether Claude is to use thinking
    #[serde(rename = "enabled")]
    Enabled,
    /// Whether Claude is not to use thinking
    #[serde(rename = "disabled")]
    Disabled,
}

#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Thinking {
    #[serde(rename = "type")]
    pub thinking_type: ThinkingType,
    /// The budget for the thinking in tokens must
    /// be at least 1024 and less than max_tokens
    #[serde(rename = "budget_tokens")]
    pub budget_tokens: u64,
}

/// Tool definition.
///
/// Tools allow Claude to perform actions outside its context,
/// such as calculations or API calls.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Tool {
    /// The name of the tool
    pub name: String,
    /// A description of what the tool does
    pub description: String,
    /// JSON Schema defining the input format for the tool
    pub input_schema: Value,
}

/// Tool choice specification.
///
/// Controls how Claude decides whether to use tools.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum ToolChoice {
    /// Claude decides whether to use tools
    #[serde(rename = "auto")]
    Auto,
    /// Claude can use any available tool
    #[serde(rename = "any")]
    Any,
    /// Claude must use the specified tool
    #[serde(rename = "tool")]
    Tool { name: String },
    /// Claude must not use any tools
    #[serde(rename = "none")]
    None,
}

/// Metadata for the request.
///
/// Additional information about the request that isn't
/// directly related to generation behavior.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Metadata {
    /// Optional user identifier for tracking purposes
    pub user_id: Option<String>,
}

// Implementation for non-streaming response
impl MessagesResponse {
    /// Creates a new message request and returns the response.
    ///
    /// This method sends a request to the Messages API and returns
    /// the complete response.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{messages::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let request = MessagesRequest {
    ///     model: "claude-3-7-sonnet-20250219".to_string(),
    ///     messages: vec![Message {
    ///         role: MessageRole::User,
    ///         content: MessageContent::Text("Hello!".to_string()),
    ///     }],
    ///     max_tokens: 100,
    ///     credentials: Some(credentials),
    ///     metadata: None,
    ///     stop_sequences: None,
    ///     stream: None,
    ///     system: None,
    ///     temperature: None,
    ///     thinking: None,
    ///     tool_choice: None,
    ///     tools: None,
    ///     top_k: None,
    ///     top_p: None,
    /// };
    ///
    /// let response = MessagesResponse::create(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(request: MessagesRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        anthropic_post("messages", &request, credentials_opt).await
    }
}

// Implementation for streaming response
impl StreamEvent {
    /// Creates a new streaming message request and returns a channel of events.
    ///
    /// This method sends a request to the Messages API in streaming mode
    /// and returns a channel that will receive the streaming events.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{messages::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    /// let mut request = MessagesRequest {
    ///     model: "claude-3-7-sonnet-20250219".to_string(),
    ///     messages: vec![Message {
    ///         role: MessageRole::User,
    ///         content: MessageContent::Text("Hello!".to_string()),
    ///     }],
    ///     max_tokens: 100,
    ///     credentials: Some(credentials),
    ///     metadata: None,
    ///     stop_sequences: None,
    ///     stream: Some(true),
    ///     system: None,
    ///     temperature: None,
    ///     thinking: None,
    ///     tool_choice: None,
    ///     tools: None,
    ///     top_k: None,
    ///     top_p: None,
    /// };
    ///
    /// let mut stream = StreamEvent::create_stream(request).await?;
    ///
    /// while let Some(event) = stream.recv().await {
    ///     // Process streaming events
    ///     println!("{:?}", event);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_stream(
        request: MessagesRequest,
    ) -> Result<Receiver<Self>, CannotCloneRequestError> {
        let credentials_opt = request.credentials.clone();
        let stream = anthropic_request_stream(
            Method::POST,
            "messages",
            |r| r.json(&request),
            credentials_opt,
        )
        .await?;
        let (tx, rx) = channel::<Self>(32);
        tokio::spawn(forward_deserialized_anthropic_stream(stream, tx));
        Ok(rx)
    }
}

/// Processes the event stream and forwards events to the channel.
///
/// This internal function handles the raw event stream from the API
/// and deserializes events into the `StreamEvent` enum.
async fn forward_deserialized_anthropic_stream(
    mut stream: EventSource,
    tx: Sender<StreamEvent>,
) -> anyhow::Result<()> {
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Event::Message(event) = event {
            let stream_event = serde_json::from_str::<StreamEvent>(&event.data)?;
            if matches!(stream_event, StreamEvent::Ping) {
                continue; // Ignore ping events
            }
            tx.send(stream_event).await?;
        }
    }
    Ok(())
}

// Builder convenience methods
impl MessagesBuilder {
    pub fn builder(model: &str, messages: impl Into<Vec<Message>>, max_tokens: u64) -> Self {
        Self::create_empty()
            .model(model)
            .messages(messages)
            .max_tokens(max_tokens)
    }

    /// Creates a new message request and returns the response.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Messages API.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{messages::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let response =MessagesBuilder::builder("claude-3-7-sonnet-20250219",[], 1024)
    ///     .credentials(credentials.clone())
    ///     .create()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(self) -> ApiResponseOrError<MessagesResponse> {
        let request = self.build().unwrap();
        MessagesResponse::create(request).await
    }

    /// Creates a new streaming message request and returns a channel of events.
    ///
    /// This is a convenience method that builds the request from the builder
    /// and sends it to the Messages API in streaming mode.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{messages::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let mut stream =MessagesBuilder::builder("claude-3-7-sonnet-20250219", [], 1024)
    ///     .credentials(credentials)
    ///     .create_stream()
    ///     .await?;
    ///
    /// while let Some(event) = stream.recv().await {
    ///     // Process streaming events
    ///     println!("{:?}", event);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_stream(self) -> Result<Receiver<StreamEvent>, CannotCloneRequestError> {
        let mut request = self.build().expect("Failed to build MessagesRequest");
        request.stream = Some(true);
        StreamEvent::create_stream(request).await
    }
}

// Helper to create a builder with required fields
impl MessagesResponse {
    /// Creates a new builder with the required fields.
    ///
    /// This is a convenience method to create a builder with the
    /// minimum required fields for a message request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use anthropic_api::{messages::*, Credentials};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let credentials = Credentials::from_env();
    ///
    /// let response =MessagesBuilder::builder(
    ///     "claude-3-7-sonnet-20250219",
    ///     vec![Message {
    ///         role: MessageRole::User,
    ///         content: MessageContent::Text("Hello!".to_string()),
    ///     }],
    ///     100,
    /// )
    /// .credentials(credentials)
    /// .create()
    /// .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder(
        model: &str,
        messages: impl Into<Vec<Message>>,
        max_tokens: u64,
    ) -> MessagesBuilder {
        MessagesBuilder::create_empty()
            .model(model)
            .messages(messages)
            .max_tokens(max_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_message() {
        let credentials = Credentials::from_env();

        let response = MessagesResponse::builder(
            "claude-3-7-sonnet-20250219",
            vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text("Hello!".to_string()),
            }],
            100,
        )
        .credentials(credentials)
        .create()
        .await
        .unwrap();

        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_streaming_message() {
        let credentials = Credentials::from_env();

        let mut stream = MessagesResponse::builder(
            "claude-3-7-sonnet-20250219",
            vec![Message {
                role: MessageRole::User,
                content: MessageContent::Text("Hello!".to_string()),
            }],
            100,
        )
        .credentials(credentials)
        .create_stream()
        .await
        .unwrap();

        while let Some(event) = stream.recv().await {
            match event {
                StreamEvent::ContentBlockDelta { delta, .. } => {
                    if let ContentBlockDelta::Text { text } = delta {
                        print!("{}", text);
                    }
                }
                _ => {}
            }
        }
    }
}
