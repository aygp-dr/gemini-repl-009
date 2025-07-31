use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct GenerateRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<Content>,
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub args: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct GenerateResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub error: Option<ApiError>,
}

#[derive(Deserialize)]
pub struct Candidate {
    pub content: ContentResponse,
}

#[derive(Deserialize)]
pub struct ContentResponse {
    pub parts: Vec<PartResponse>,
}

#[derive(Deserialize)]
pub struct PartResponse {
    pub text: Option<String>,
    #[serde(rename = "functionCall")]
    pub function_call: Option<FunctionCall>,
}

#[derive(Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

#[derive(Serialize)]
pub struct Tool {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Serialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: FunctionParameters,
}

#[derive(Serialize)]
pub struct FunctionParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, ParameterProperty>,
    pub required: Vec<String>,
}

#[derive(Serialize)]
pub struct ParameterProperty {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        
        Ok(Self {
            client,
            api_key,
            model,
        })
    }

    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(anyhow!("API request failed with status {}: {}", status, text));
        }

        serde_json::from_str(&text)
            .map_err(|e| anyhow!("Failed to parse response: {} - Body: {}", e, text))
    }
}

// Tool definitions for our evaluation
pub fn create_evaluation_tools() -> Vec<Tool> {
    vec![
        Tool {
            function_declarations: vec![
                FunctionDeclaration {
                    name: "list_files".to_string(),
                    description: "List files in a directory".to_string(),
                    parameters: FunctionParameters {
                        param_type: "object".to_string(),
                        properties: HashMap::from([
                            ("path".to_string(), ParameterProperty {
                                prop_type: "string".to_string(),
                                description: "Directory path to list".to_string(),
                            }),
                        ]),
                        required: vec!["path".to_string()],
                    },
                },
                FunctionDeclaration {
                    name: "read_file".to_string(),
                    description: "Read contents of a file".to_string(),
                    parameters: FunctionParameters {
                        param_type: "object".to_string(),
                        properties: HashMap::from([
                            ("path".to_string(), ParameterProperty {
                                prop_type: "string".to_string(),
                                description: "File path to read".to_string(),
                            }),
                        ]),
                        required: vec!["path".to_string()],
                    },
                },
                FunctionDeclaration {
                    name: "search_code".to_string(),
                    description: "Search for patterns in code".to_string(),
                    parameters: FunctionParameters {
                        param_type: "object".to_string(),
                        properties: HashMap::from([
                            ("pattern".to_string(), ParameterProperty {
                                prop_type: "string".to_string(),
                                description: "Search pattern".to_string(),
                            }),
                            ("path".to_string(), ParameterProperty {
                                prop_type: "string".to_string(),
                                description: "Path to search in".to_string(),
                            }),
                        ]),
                        required: vec!["pattern".to_string()],
                    },
                },
                FunctionDeclaration {
                    name: "write_file".to_string(),
                    description: "Write content to a file".to_string(),
                    parameters: FunctionParameters {
                        param_type: "object".to_string(),
                        properties: HashMap::from([
                            ("path".to_string(), ParameterProperty {
                                prop_type: "string".to_string(),
                                description: "File path to write".to_string(),
                            }),
                            ("content".to_string(), ParameterProperty {
                                prop_type: "string".to_string(),
                                description: "Content to write".to_string(),
                            }),
                        ]),
                        required: vec!["path".to_string(), "content".to_string()],
                    },
                },
            ],
        },
    ]
}