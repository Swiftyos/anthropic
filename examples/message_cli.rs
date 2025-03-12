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

#[tokio::main]
async fn main() {
    // Load .env file containing ANTHROPIC_API_KEY
    let credentials = Credentials::from_env();

    let mut messages = vec![Message {
        role: MessageRole::User,
        content: MessageContent::Text(
            "You are a helpful AI assistant. Please introduce yourself briefly.".to_string(),
        ),
    }];

    // Create initial message request
    let response = MessagesAPI::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
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
        let response = MessagesAPI::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
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
