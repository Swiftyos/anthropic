//! # Message CLI Example
//!
//! This example demonstrates a simple command-line interface for interacting with Claude.
//! It creates a conversation loop where the user can chat with the AI assistant.
//!
//! ## Features
//!
//! - Initializes with a system-like message to set the assistant's behavior
//! - Maintains conversation history for context
//! - Simple command-line interface for user input
//! - Logging with tracing subscriber
//!
//! ## Usage
//!
//! Run this example with:
//!
//! ```bash
//! cargo run --example message_cli
//! ```
//!
//! Make sure you have set the `ANTHROPIC_API_KEY` environment variable.

use anthropic_api::{messages::*, Credentials};
use std::io::{stdin, stdout, Write};
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for console logging
    tracing_subscriber::fmt()
        // Enable DEBUG or Tracing to see the raw request and response bodies
        // .with_max_level(tracing::Level::TRACE)
        .init();

    // Load .env file containing ANTHROPIC_API_KEY
    let credentials = Credentials::from_env();

    let mut messages = vec![Message {
        role: MessageRole::User,
        content: MessageContent::Text(
            "You are a helpful AI assistant. Please introduce yourself briefly.".to_string(),
        ),
    }];

    // Create initial message request
    let response = MessagesBuilder::builder("claude-3-7-sonnet-20250219", messages.clone(), 2048)
        .credentials(credentials.clone())
        // Uncomment this to enable thinking
        // .thinking(Thinking {
        //     thinking_type: ThinkingType::Enabled,
        //     budget_tokens: 1024,
        // })
        .create()
        .await
        .unwrap();

    // Print assistant's response
    // Iterate through all content blocks in the response
    let mut assistant_response = String::new();
    for content in &response.content {
        match content {
            ResponseContentBlock::Text { text } => {
                println!("Assistant: {}", text.trim());
                assistant_response.push_str(text);
            }
            ResponseContentBlock::Thinking {
                signature,
                thinking,
            } => {
                println!("Assistant: [Thinking content] {}", thinking);
            }
            _ => {
                println!("Assistant: [Unsupported content type]");
            }
        }
    }

    // Add the assistant's response to the message history
    if !assistant_response.is_empty() {
        messages.push(Message {
            role: MessageRole::Assistant,
            content: MessageContent::Text(assistant_response),
        });
    }

    // Start conversation loop
    loop {
        print!("User: ");
        stdout().flush().unwrap();

        let mut user_input = String::new();
        stdin().read_line(&mut user_input).unwrap();

        // Add user message
        messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Text(user_input),
        });

        // Send message request
        let response =
            MessagesResponse::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
                .credentials(credentials.clone())
                .create()
                .await
                .unwrap();

        // Print assistant's response
        if let Some(content) = response.content.first() {
            match content {
                ResponseContentBlock::Text { text } => {
                    println!("Assistant: {}", text.trim());
                    messages.push(Message {
                        role: MessageRole::Assistant,
                        content: MessageContent::Text(text.clone()),
                    });
                }
                _ => {}
            }
        }
    }
}
