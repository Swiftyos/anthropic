//! # Tool Use Example
//!
//! This example demonstrates how to use Claude's tool use capabilities.
//! It creates a calculator tool that Claude can use to perform basic arithmetic operations.
//!
//! ## Features
//!
//! - Defines a calculator tool with a JSON schema
//! - Allows Claude to use the tool when appropriate
//! - Maintains conversation history for context
//! - Simple command-line interface for user input
//!
//! ## Usage
//!
//! Run this example with:
//!
//! ```bash
//! cargo run --example tool_use
//! ```
//!
//! Make sure you have set the `ANTHROPIC_API_KEY` environment variable.

use anthropic::{messages::*, Credentials};
use serde_json::json;
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() {

    let credentials = Credentials::from_env();

    // Define a calculator tool
    let calculator_tool = Tool {
        name: "calculator".to_string(),
        description: "A calculator that can perform basic arithmetic operations".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"]
                },
                "operands": {
                    "type": "array",
                    "items": {"type": "number"},
                    "minItems": 2,
                    "maxItems": 2
                }
            },
            "required": ["operation", "operands"]
        }),
    };

    let mut messages = vec![Message {
        role: MessageRole::User,
        content: MessageContent::Text(
            "You are a helpful AI assistant. Please calculate 15 + 27 using the calculator tool.".to_string(),
        ),
    }];

    // Create message request with tool
    let response = MessageResponse::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
        .credentials(credentials.clone())
        .tools(vec![calculator_tool.clone()])
        .tool_choice(ToolChoice::Any)
        .create()
        .await
        .unwrap();

    // Print assistant's response and tool usage
    for content in response.content {
        match content {
            ResponseContentBlock::Text { text } => {
                println!("Assistant: {}", text.trim());
                messages.push(Message {
                    role: MessageRole::Assistant,
                    content: MessageContent::Text(text),
                });
            }
            ResponseContentBlock::ToolUse { name, input, .. } => {
                println!("Tool use - {}: {}", name, input);
            }
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

        // Send message request with tool
        let response = MessageResponse::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
            .credentials(credentials.clone())
            .tools(vec![calculator_tool.clone()])
            .tool_choice(ToolChoice::Any)
            .create()
            .await
            .unwrap();

        // Print assistant's response and tool usage
        for content in response.content {
            match content {
                ResponseContentBlock::Text { text } => {
                    println!("Assistant: {}", text.trim());
                    messages.push(Message {
                        role: MessageRole::Assistant,
                        content: MessageContent::Text(text),
                    });
                }
                ResponseContentBlock::ToolUse { name, input, .. } => {
                    println!("Tool use - {}: {}", name, input);
                }
            }
        }
    }
}
