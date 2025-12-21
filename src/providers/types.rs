//! Common types for provider abstraction
//!
//! These types provide a provider-agnostic interface for messages,
//! responses, and tool definitions.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: Role,
    /// Content of the message
    pub content: MessageContent,
}

impl Message {
    /// Create a user message with text content
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Text(text.into()),
        }
    }

    /// Create an assistant message with text content
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::Text(text.into()),
        }
    }

    /// Create a system message
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: MessageContent::Text(text.into()),
        }
    }

    /// Create a function call message
    pub fn function_call(name: impl Into<String>, args: Value) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::FunctionCall(FunctionCall {
                name: name.into(),
                arguments: args,
            }),
        }
    }

    /// Create a function response message
    pub fn function_response(name: impl Into<String>, response: Value) -> Self {
        Self {
            role: Role::Function,
            content: MessageContent::FunctionResponse(FunctionResponse {
                name: name.into(),
                response,
            }),
        }
    }

    /// Get the text content if this is a text message
    pub fn text(&self) -> Option<&str> {
        match &self.content {
            MessageContent::Text(t) => Some(t),
            _ => None,
        }
    }
}

/// Role of the message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Function,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Function => write!(f, "function"),
        }
    }
}

/// Content of a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Plain text content
    Text(String),
    /// A function call from the assistant
    FunctionCall(FunctionCall),
    /// A function response
    FunctionResponse(FunctionResponse),
    /// Multiple parts (text + function call)
    Parts(Vec<MessagePart>),
}

/// A part of a multi-part message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessagePart {
    Text(String),
    FunctionCall(FunctionCall),
    FunctionResponse(FunctionResponse),
}

/// A function call request from the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Name of the function to call
    pub name: String,
    /// Arguments as JSON
    pub arguments: Value,
}

/// A function response to send back to the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResponse {
    /// Name of the function that was called
    pub name: String,
    /// Response data as JSON
    pub response: Value,
}

/// Response from a provider
#[derive(Debug, Clone)]
pub enum ProviderResponse {
    /// A text response
    Text(String),
    /// A function call request
    FunctionCall(FunctionCall),
    /// Both text and a function call
    TextWithFunctionCall {
        text: String,
        function_call: FunctionCall,
    },
}

impl ProviderResponse {
    /// Get the text content if present
    pub fn text(&self) -> Option<&str> {
        match self {
            ProviderResponse::Text(t) => Some(t),
            ProviderResponse::TextWithFunctionCall { text, .. } => Some(text),
            ProviderResponse::FunctionCall(_) => None,
        }
    }

    /// Get the function call if present
    pub fn function_call(&self) -> Option<&FunctionCall> {
        match self {
            ProviderResponse::FunctionCall(fc) => Some(fc),
            ProviderResponse::TextWithFunctionCall { function_call, .. } => Some(function_call),
            ProviderResponse::Text(_) => None,
        }
    }

    /// Convert to a Message for adding to conversation history
    pub fn to_message(&self) -> Message {
        match self {
            ProviderResponse::Text(text) => Message::assistant(text.clone()),
            ProviderResponse::FunctionCall(fc) => {
                Message::function_call(fc.name.clone(), fc.arguments.clone())
            }
            ProviderResponse::TextWithFunctionCall {
                text,
                function_call,
            } => Message {
                role: Role::Assistant,
                content: MessageContent::Parts(vec![
                    MessagePart::Text(text.clone()),
                    MessagePart::FunctionCall(function_call.clone()),
                ]),
            },
        }
    }
}

/// A chunk from a streaming response
#[derive(Debug, Clone)]
pub enum StreamChunk {
    /// A text chunk
    Text(String),
    /// Stream is complete
    Done,
}

/// Tool/function definition for the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Name of the tool
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// JSON schema for the parameters
    pub parameters: Value,
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_message_user() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.text(), Some("Hello"));
    }

    #[test]
    fn test_message_assistant() {
        let msg = Message::assistant("Hi there!");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.text(), Some("Hi there!"));
    }

    #[test]
    fn test_message_function_call() {
        let msg = Message::function_call("read_file", json!({"path": "test.txt"}));
        assert_eq!(msg.role, Role::Assistant);
        match &msg.content {
            MessageContent::FunctionCall(fc) => {
                assert_eq!(fc.name, "read_file");
            }
            _ => panic!("Expected FunctionCall content"),
        }
    }

    #[test]
    fn test_provider_response_text() {
        let resp = ProviderResponse::Text("Hello".to_string());
        assert_eq!(resp.text(), Some("Hello"));
        assert!(resp.function_call().is_none());
    }

    #[test]
    fn test_provider_response_function_call() {
        let resp = ProviderResponse::FunctionCall(FunctionCall {
            name: "test".to_string(),
            arguments: json!({}),
        });
        assert!(resp.text().is_none());
        assert!(resp.function_call().is_some());
    }

    #[test]
    fn test_provider_response_to_message() {
        let resp = ProviderResponse::Text("Hello".to_string());
        let msg = resp.to_message();
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.text(), Some("Hello"));
    }

    #[test]
    fn test_role_display() {
        assert_eq!(Role::User.to_string(), "user");
        assert_eq!(Role::Assistant.to_string(), "assistant");
        assert_eq!(Role::System.to_string(), "system");
        assert_eq!(Role::Function.to_string(), "function");
    }

    #[test]
    fn test_tool_definition() {
        let tool = ToolDefinition::new(
            "read_file",
            "Read a file",
            json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                }
            }),
        );
        assert_eq!(tool.name, "read_file");
        assert_eq!(tool.description, "Read a file");
    }
}
