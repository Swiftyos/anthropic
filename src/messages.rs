use derive_builder::Builder;
use futures_util::StreamExt;
use reqwest::Method;
use reqwest_eventsource::{CannotCloneRequestError, Event, EventSource};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use anyhow::Result;
use crate::{anthropic_post, anthropic_request_stream, ApiResponseOrError, Credentials, Usage};


/// Represents a full message response from the Anthropic API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MessageResponse {
    pub id: String,
    pub model: String,
    pub role: MessageRole,
    pub content: Vec<ResponseContentBlock>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    #[serde(rename = "type")]
    pub typ: String,
    pub usage: Usage,
}

/// Content block in a response, can be text or tool use.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum ResponseContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: Value },
}

/// Streaming events from the Anthropic API.
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: MessageStart },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: ContentBlockStart,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: u32, delta: ContentBlockDelta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: u32 },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDelta, usage: Usage },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
}

// Supporting structs for streaming events
#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MessageStart {
    pub id: String,
    pub model: String,
    pub role: MessageRole,
    pub content: Vec<ContentBlockStart>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum ContentBlockStart {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum ContentBlockDelta {
    Text { text: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct MessageDelta {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

/// Request to the Anthropic Messages API.
#[derive(Serialize, Builder, Debug, Clone)]
#[builder(derive(Clone, Debug, PartialEq))]
#[builder(pattern = "owned")]
#[builder(name = "MessagesBuilder")]
#[builder(setter(strip_option, into))]
pub struct MessagesRequest {
    /// The model to use (e.g., "claude-3--20240229").
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
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
}

/// Role of the message sender.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

/// Content of a message, either text or content blocks.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    ContentBlocks(Vec<RequestContentBlock>),
}

/// Content block in a request.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum RequestContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

/// Source of an image content block.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String, // e.g., "base64"
    pub media_type: String,  // e.g., "image/png"
    pub data: String,        // e.g., base64-encoded data
}

/// Tool definition.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Tool choice specification.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum ToolChoice {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "tool")]
    Tool { name: String },
    #[serde(rename = "none")]
    None,
}

/// Metadata for the request.
#[derive(Serialize, Debug, Clone, Eq, PartialEq)]
pub struct Metadata {
    pub user_id: Option<String>,
}

// Implementation for non-streaming response
impl MessageResponse {
    pub async fn create(request: MessagesRequest) -> ApiResponseOrError<Self> {
        let credentials_opt = request.credentials.clone();
        anthropic_post("messages", &request, credentials_opt).await
    }
}

// Implementation for streaming response
impl StreamEvent {
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

async fn forward_deserialized_anthropic_stream(
    mut stream: EventSource,
    tx: Sender<StreamEvent>,
) -> anyhow::Result<()> {
    while let Some(event) = stream.next().await {
        let event = event?;
        match event {
            Event::Message(event) => {
                let stream_event = serde_json::from_str::<StreamEvent>(&event.data)?;
                if matches!(stream_event, StreamEvent::Ping) {
                    continue; // Ignore ping events
                }
                tx.send(stream_event).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

// Builder convenience methods
impl MessagesBuilder {
    pub async fn create(self) -> ApiResponseOrError<MessageResponse> {
        let request = self.build().unwrap();
        MessageResponse::create(request).await
    }

    pub async fn create_stream(self) -> Result<Receiver<StreamEvent>, CannotCloneRequestError> {
        let mut request = self.build().expect("Failed to build MessagesRequest");
        request.stream = Some(true);
        StreamEvent::create_stream(request).await
    }
}

// Helper to create a builder with required fields
impl MessageResponse {
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

        let response = MessageResponse::builder(
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

        let mut stream = MessageResponse::builder(
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