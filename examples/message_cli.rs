use anthropic::{messages::*, Credentials};
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
    let response = MessageResponse::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
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
        let response = MessageResponse::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
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
