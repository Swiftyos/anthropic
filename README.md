# Anthropic Unofficial Rust SDK


[![crates.io](https://img.shields.io/crates/v/anthropic-api.svg)](https://crates.io/crates/anthropic-api/)
[![Tests](https://github.com/Swiftyos/anthropic/actions/workflows/test.yml/badge.svg)](https://github.com/Swiftyos/anthropic/actions/workflows/test.yml)

An unofficial Rust library for interacting with the [Anthropic API](https://www.anthropic.com/api). This library provides an asynchronous interface to Anthropic's services, allowing Rust developers to seamlessly integrate Anthropic's AI capabilities—such as message-based interactions and tool use—into their applications.

> **Note**: This is an unofficial library and is not maintained by Anthropic.

---

## Getting Started

Follow these steps to start using the Anthropic Rust SDK in your project:

### 1. Installation

Add the library to your project using Cargo:

```sh
cargo add anthropic-api
```

### 2. Set Up Your API Key

To use this library, you need an Anthropic API key. Set the `ANTHROPIC_API_KEY` environment variable with your key:

```sh
export ANTHROPIC_API_KEY="your-api-key-here"
```

The library will automatically load this key when you create a `Credentials` object with `Credentials::from_env()`.

### 3. Basic Usage

Here’s a simple example of sending a message to the Anthropic API and receiving a response:

```rust
use anthropic_api::{messages::*, Credentials};

#[tokio::main]
async fn main() {
    // Load credentials from the environment
    let credentials = Credentials::from_env();

    // Create a message
    let messages = vec![Message {
        role: MessageRole::User,
        content: MessageContent::Text("Hello, Claude!".to_string()),
    }];

    // Send the message to the Anthropic API
    let response = MessageResponse::builder("claude-3-sonnet-20240229", messages, 1024)
        .credentials(credentials)
        .create()
        .await
        .unwrap();

    // Print the assistant's response
    if let Some(ResponseContentBlock::Text { text }) = response.content.first() {
        println!("Assistant: {}", text.trim());
    }
}
```

This example uses the `claude-3-sonnet-20240229` model. Replace it with the desired Anthropic model name as per their official documentation.

---

## Features

The Anthropic Rust SDK currently supports the following features:

- **Asynchronous API Requests**: Leverage Rust’s async capabilities for efficient API interactions.
- **Message API**: Send and receive messages, similar to chat-based interactions.
- **Tool Use**: Integrate external tools (e.g., a calculator) that the AI can call during responses.
- **Streaming Responses**: Receive real-time streamed responses from the API.

More features are planned for future releases—stay tuned!

---

## Examples

Detailed examples can be found in the [`examples` directory](https://github.com/swiftyos/anthropic/tree/main/examples). Below are two key examples to get you started:

### Messages Example

This example demonstrates a basic conversation loop with the Anthropic API:

```rust
use anthropic_api::{messages::*, Credentials};
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() {
    let credentials = Credentials::from_env();
    let mut messages = vec![Message {
        role: MessageRole::User,
        content: MessageContent::Text("You are a helpful AI assistant. Please introduce yourself briefly.".to_string()),
    }];

    // Initial message
    let response = MessagesAPI::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
        .credentials(credentials.clone())
        .create()
        .await
        .unwrap();

    if let Some(ResponseContentBlock::Text { text }) = response.content.first() {
        println!("Assistant: {}", text.trim());
        messages.push(Message {
            role: MessageRole::Assistant,
            content: MessageContent::Text(text.clone()),
        });
    }

    // Conversation loop
    loop {
        print!("User: ");
        stdout().flush().unwrap();
        let mut user_input = String::new();
        stdin().read_line(&mut user_input).unwrap();

        messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::Text(user_input),
        });

        let response = MessagesAPI::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
            .credentials(credentials.clone())
            .create()
            .await
            .unwrap();

        if let Some(ResponseContentBlock::Text { text }) = response.content.first() {
            println!("Assistant: {}", text.trim());
            messages.push(Message {
                role: MessageRole::Assistant,
                content: MessageContent::Text(text.clone()),
            });
        }
    }
}
```

### Tool Use Example

This example shows how to use a calculator tool with the API:

```rust
use anthropic_api::{messages::*, Credentials};
use serde_json::json;

#[tokio::main]
async fn main() {
    let credentials = Credentials::from_env();

    // Define a calculator tool
    let calculator_tool = Tool {
        name: "calculator".to_string(),
        description: "A calculator for basic arithmetic operations".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string", "enum": ["add", "subtract", "multiply", "divide"]},
                "operands": {"type": "array", "items": {"type": "number"}, "minItems": 2, "maxItems": 2}
            },
            "required": ["operation", "operands"]
        }),
    };

    let mut messages = vec![Message {
        role: MessageRole::User,
        content: MessageContent::Text("Calculate 15 + 27 using the calculator tool.".to_string()),
    }];

    // Send message with tool
    let response = MessagesAPI::builder("claude-3-7-sonnet-20250219", messages.clone(), 1024)
        .credentials(credentials)
        .tools(vec![calculator_tool])
        .tool_choice(ToolChoice::Any)
        .create()
        .await
        .unwrap();

    // Process response
    for content in response.content {
        match content {
            ResponseContentBlock::Text { text } => println!("Assistant: {}", text.trim()),
            ResponseContentBlock::ToolUse { name, input, .. } => println!("Tool use - {}: {}", name, input),
        }
    }
}
```

For more advanced examples, including streaming, check the [`examples` directory](https://github.com/swiftyos/anthropic/tree/main/examples).

---

## Contributing

Contributions are warmly welcomed! Whether you spot a bug, have a feature suggestion, or want to improve the examples, please feel free to:

1. Open an [issue](https://github.com/swiftyos/anthropic/issues) to report problems or suggest enhancements.
2. Submit a [pull request](https://github.com/swiftyos/anthropic/pulls) with your changes.

To ensure high-quality contributions:

- **Write Unit Tests**: Tests are strongly encouraged to maintain reliability.
- **Format Your Code**: Run `cargo fmt` to keep the codebase consistent.
- **Check for Warnings**: Use `cargo clippy` to catch potential issues.

Run the test suite with:

```sh
cargo test
```


## Implementation Progress

`██████████` Messages

`░░░░░░░░░░` Batch Messages

`██████████` Members

`██████████` Invites

`██████████` Workspaces

`██████████` Workspace Members

`██████████` API Keys

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

> **Notice**
>
> This library was heavily inspired by and based on the [OpenAI Rust SDK](https://github.com/rellfy/openai) by Lorenzo Fontoura. A huge thanks to their foundational work!
