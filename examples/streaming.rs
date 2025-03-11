//! # Streaming Example
//!
//! This example demonstrates how to use the streaming API with Claude.
//! It creates a conversation loop where the user can chat with the AI assistant,
//! and the assistant's responses are streamed in real-time.
//!
//! ## Features
//!
//! - Initializes with a system-like message to set the assistant's behavior
//! - Maintains conversation history for context
//! - Streams responses in real-time for a more interactive experience
//! - Simple command-line interface for user input
//!
//! ## Usage
//!
//! Run this example with:
//!
//! ```bash
//! cargo run --example streaming
//! ```
//!
//! Make sure you have set the `ANTHROPIC_API_KEY` environment variable.

use anthropic_llm::{messages::*, Credentials};
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() {
    let credentials = Credentials::from_env();

    let mut messages = vec![Message {
        role: MessageRole::User,
        content: MessageContent::Text(
            "You are a helpful AI assistant. Please introduce yourself briefly.".to_string(),
        ),
    }];

    // Create initial message request with streaming
    let mut stream = MessageResponse::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
        .credentials(credentials.clone())
        .create_stream()
        .await
        .unwrap();

    // Print assistant's streaming response
    print!("Assistant: ");
    stdout().flush().unwrap();
    while let Some(event) = stream.recv().await {
        match event {
            StreamEvent::ContentBlockDelta { delta, .. } => {
                if let ContentBlockDelta::Text { text } = delta {
                    print!("{}", text);
                    stdout().flush().unwrap();
                }
            }
            StreamEvent::MessageStop => {
                println!();
            }
            _ => {}
        }
    }

    // Start conversation loop
    loop {
        print!("\nUser: ");
        stdout().flush().unwrap();

        let mut user_input = String::new();
        stdin().read_line(&mut user_input).unwrap();

        // Add user message
        messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Text(user_input),
        });

        // Send message request with streaming
        let mut stream =
            MessageResponse::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
                .credentials(credentials.clone())
                .create_stream()
                .await
                .unwrap();

        // Print assistant's streaming response and store the text
        print!("\nAssistant: ");
        stdout().flush().unwrap();
        let mut full_response = String::new();
        while let Some(event) = stream.recv().await {
            match event {
                StreamEvent::ContentBlockDelta { delta, .. } => {
                    if let ContentBlockDelta::Text { text } = delta {
                        print!("{}", text);
                        stdout().flush().unwrap();
                        full_response.push_str(&text);
                    }
                }
                StreamEvent::MessageStop => {
                    println!();
                }
                _ => {}
            }
        }

        // Add assistant's complete response to messages
        messages.push(Message {
            role: MessageRole::Assistant,
            content: MessageContent::Text(full_response),
        });
    }
}
