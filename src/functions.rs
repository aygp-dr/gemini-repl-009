//! Function definitions for tool calling

use crate::api::{FunctionDeclaration, FunctionParameters, ParameterProperty, Tool};
use std::collections::HashMap;

/// Get all available tools for function calling
pub fn get_available_tools() -> Vec<Tool> {
    vec![
        Tool {
            function_declarations: vec![
                FunctionDeclaration {
                    name: "read_file".to_string(),
                    description: "Read the contents of a file from the filesystem. Use this when the user asks to see, read, show, or check file contents.".to_string(),
                    parameters: FunctionParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = HashMap::new();
                            props.insert(
                                "file_path".to_string(),
                                ParameterProperty {
                                    prop_type: "string".to_string(),
                                    description: "Path to the file to read (e.g., 'Makefile', 'src/main.rs', 'Cargo.toml')".to_string(),
                                    items: None,
                                }
                            );
                            props
                        },
                        required: vec!["file_path".to_string()],
                    },
                },
                FunctionDeclaration {
                    name: "list_files".to_string(),
                    description: "List files in a directory. Use this when the user asks to list, show all, or find files.".to_string(),
                    parameters: FunctionParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = HashMap::new();
                            props.insert(
                                "directory".to_string(),
                                ParameterProperty {
                                    prop_type: "string".to_string(),
                                    description: "Directory path to list files from (default: '.')".to_string(),
                                    items: None,
                                }
                            );
                            props.insert(
                                "pattern".to_string(),
                                ParameterProperty {
                                    prop_type: "string".to_string(),
                                    description: "Optional glob pattern to filter files (e.g., '*.rs', '**/*.md')".to_string(),
                                    items: None,
                                }
                            );
                            props
                        },
                        required: vec![],
                    },
                },
                FunctionDeclaration {
                    name: "write_file".to_string(),
                    description: "Write content to a file. Use this when the user asks to create, write, save, or update a file.".to_string(),
                    parameters: FunctionParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = HashMap::new();
                            props.insert(
                                "file_path".to_string(),
                                ParameterProperty {
                                    prop_type: "string".to_string(),
                                    description: "Path to the file to write".to_string(),
                                    items: None,
                                }
                            );
                            props.insert(
                                "content".to_string(),
                                ParameterProperty {
                                    prop_type: "string".to_string(),
                                    description: "Content to write to the file".to_string(),
                                    items: None,
                                }
                            );
                            props
                        },
                        required: vec!["file_path".to_string(), "content".to_string()],
                    },
                },
                FunctionDeclaration {
                    name: "search_code".to_string(),
                    description: "Search for text patterns in code files. Use this when the user asks to search, find occurrences, or look for specific text.".to_string(),
                    parameters: FunctionParameters {
                        param_type: "object".to_string(),
                        properties: {
                            let mut props = HashMap::new();
                            props.insert(
                                "pattern".to_string(),
                                ParameterProperty {
                                    prop_type: "string".to_string(),
                                    description: "Text or regex pattern to search for".to_string(),
                                    items: None,
                                }
                            );
                            props.insert(
                                "file_pattern".to_string(),
                                ParameterProperty {
                                    prop_type: "string".to_string(),
                                    description: "Optional file pattern to limit search (e.g., '*.rs')".to_string(),
                                    items: None,
                                }
                            );
                            props
                        },
                        required: vec!["pattern".to_string()],
                    },
                },
            ],
        },
    ]
}