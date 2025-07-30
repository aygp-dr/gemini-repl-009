//! Unit tests for function calling functionality

use gemini_repl::api::{Content, FunctionCall, FunctionResponse, Part};
use gemini_repl::functions::get_available_tools;

#[test]
fn test_get_available_tools() {
    let tools = get_available_tools();
    assert_eq!(tools.len(), 1);

    let tool = &tools[0];
    assert_eq!(tool.function_declarations.len(), 4);

    // Check function names
    let names: Vec<&str> = tool
        .function_declarations
        .iter()
        .map(|f| f.name.as_str())
        .collect();

    assert!(names.contains(&"read_file"));
    assert!(names.contains(&"list_files"));
    assert!(names.contains(&"write_file"));
    assert!(names.contains(&"search_code"));
}

#[test]
fn test_part_serialization_with_text() {
    let part = Part {
        text: Some("Hello, world!".to_string()),
        function_call: None,
        function_response: None,
    };

    let json = serde_json::to_string(&part).unwrap();
    assert!(json.contains("\"text\":\"Hello, world!\""));
    assert!(!json.contains("functionCall"));
    assert!(!json.contains("functionResponse"));
}

#[test]
fn test_part_serialization_with_function_call() {
    let part = Part {
        text: None,
        function_call: Some(FunctionCall {
            name: "read_file".to_string(),
            args: Some(serde_json::json!({
                "file_path": "Makefile"
            })),
        }),
        function_response: None,
    };

    let json = serde_json::to_string(&part).unwrap();
    assert!(json.contains("\"functionCall\""));
    assert!(json.contains("\"name\":\"read_file\""));
    assert!(json.contains("\"file_path\":\"Makefile\""));
    assert!(!json.contains("\"text\""));
}

#[test]
fn test_content_with_function_call() {
    let content = Content {
        role: "user".to_string(),
        parts: vec![Part {
            text: None,
            function_call: Some(FunctionCall {
                name: "list_files".to_string(),
                args: Some(serde_json::json!({
                    "directory": "src"
                })),
            }),
            function_response: None,
        }],
    };

    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains("\"role\":\"user\""));
    assert!(json.contains("\"functionCall\""));
    assert!(json.contains("\"name\":\"list_files\""));
}

#[test]
fn test_function_response_serialization() {
    let response = FunctionResponse {
        name: "read_file".to_string(),
        response: serde_json::json!({
            "content": "# Makefile content here"
        }),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"name\":\"read_file\""));
    assert!(json.contains("\"content\":\"# Makefile content here\""));
}
