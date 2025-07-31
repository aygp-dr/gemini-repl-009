//! Comprehensive test scenarios for function calling evaluation
//! 
//! Categories:
//! 1. No-tool scenarios (should NOT trigger function calls)
//! 2. Single-tool scenarios (should trigger exactly one function)
//! 3. Multi-tool scenarios (should trigger multiple functions in sequence)
//! 4. Edge cases and adversarial inputs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    pub id: String,
    pub category: TestCategory,
    pub prompt: String,
    pub expected_behavior: ExpectedBehavior,
    pub rationale: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestCategory {
    NoTool,
    SingleTool,
    MultiTool,
    EdgeCase,
    Adversarial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpectedBehavior {
    NoFunctionCall,
    SingleFunction {
        name: String,
        args_pattern: HashMap<String, String>,
    },
    MultipleFunctions {
        sequence: Vec<(String, HashMap<String, String>)>,
        allow_reordering: bool,
    },
}

pub struct ScenarioGenerator;

impl ScenarioGenerator {
    /// Generate scenarios that should NOT trigger function calls
    pub fn no_tool_scenarios() -> Vec<TestScenario> {
        vec![
            // Conceptual questions
            TestScenario {
                id: "no_tool_001".to_string(),
                category: TestCategory::NoTool,
                prompt: "What is the purpose of a README file?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Conceptual question about general knowledge".to_string(),
                tags: vec!["conceptual".to_string(), "documentation".to_string()],
            },
            TestScenario {
                id: "no_tool_002".to_string(),
                category: TestCategory::NoTool,
                prompt: "Explain the difference between git pull and git fetch".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Explanation request, not action request".to_string(),
                tags: vec!["git".to_string(), "explanation".to_string()],
            },
            TestScenario {
                id: "no_tool_003".to_string(),
                category: TestCategory::NoTool,
                prompt: "How do I write better unit tests?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "General advice request".to_string(),
                tags: vec!["testing".to_string(), "advice".to_string()],
            },
            
            // Mathematical/logical questions
            TestScenario {
                id: "no_tool_004".to_string(),
                category: TestCategory::NoTool,
                prompt: "What is 42 multiplied by 17?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Simple calculation, no file operation needed".to_string(),
                tags: vec!["math".to_string(), "calculation".to_string()],
            },
            TestScenario {
                id: "no_tool_005".to_string(),
                category: TestCategory::NoTool,
                prompt: "Is 97 a prime number?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Mathematical question, no file operation".to_string(),
                tags: vec!["math".to_string(), "prime".to_string()],
            },
            
            // Programming concepts
            TestScenario {
                id: "no_tool_006".to_string(),
                category: TestCategory::NoTool,
                prompt: "What are the SOLID principles in object-oriented design?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Conceptual explanation of design principles".to_string(),
                tags: vec!["oop".to_string(), "design".to_string()],
            },
            TestScenario {
                id: "no_tool_007".to_string(),
                category: TestCategory::NoTool,
                prompt: "Give me an example of a recursive function in Python".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Code generation request, not file operation".to_string(),
                tags: vec!["python".to_string(), "example".to_string()],
            },
            
            // Opinions and recommendations
            TestScenario {
                id: "no_tool_008".to_string(),
                category: TestCategory::NoTool,
                prompt: "What's the best programming language for beginners?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Opinion/recommendation request".to_string(),
                tags: vec!["opinion".to_string(), "recommendation".to_string()],
            },
            TestScenario {
                id: "no_tool_009".to_string(),
                category: TestCategory::NoTool,
                prompt: "Should I use tabs or spaces for indentation?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Opinion question about coding style".to_string(),
                tags: vec!["style".to_string(), "opinion".to_string()],
            },
            
            // Conversational
            TestScenario {
                id: "no_tool_010".to_string(),
                category: TestCategory::NoTool,
                prompt: "Hello! How are you doing today?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Greeting/conversational prompt".to_string(),
                tags: vec!["greeting".to_string(), "conversational".to_string()],
            },
            TestScenario {
                id: "no_tool_011".to_string(),
                category: TestCategory::NoTool,
                prompt: "Tell me a programming joke".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Entertainment request".to_string(),
                tags: vec!["joke".to_string(), "entertainment".to_string()],
            },
            
            // Historical questions
            TestScenario {
                id: "no_tool_012".to_string(),
                category: TestCategory::NoTool,
                prompt: "Who created the Rust programming language?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Historical fact question".to_string(),
                tags: vec!["history".to_string(), "rust".to_string()],
            },
            TestScenario {
                id: "no_tool_013".to_string(),
                category: TestCategory::NoTool,
                prompt: "When was Python first released?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Historical date question".to_string(),
                tags: vec!["history".to_string(), "python".to_string()],
            },
            
            // Algorithm explanations
            TestScenario {
                id: "no_tool_014".to_string(),
                category: TestCategory::NoTool,
                prompt: "Explain how quicksort works".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Algorithm explanation request".to_string(),
                tags: vec!["algorithm".to_string(), "explanation".to_string()],
            },
            TestScenario {
                id: "no_tool_015".to_string(),
                category: TestCategory::NoTool,
                prompt: "What's the time complexity of binary search?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Theoretical computer science question".to_string(),
                tags: vec!["complexity".to_string(), "algorithm".to_string()],
            },
            
            // Best practices
            TestScenario {
                id: "no_tool_016".to_string(),
                category: TestCategory::NoTool,
                prompt: "What are some best practices for API design?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "General best practices question".to_string(),
                tags: vec!["api".to_string(), "best_practices".to_string()],
            },
            TestScenario {
                id: "no_tool_017".to_string(),
                category: TestCategory::NoTool,
                prompt: "How should I structure my Git commits?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Process/methodology question".to_string(),
                tags: vec!["git".to_string(), "process".to_string()],
            },
            
            // Debugging concepts
            TestScenario {
                id: "no_tool_018".to_string(),
                category: TestCategory::NoTool,
                prompt: "What are common causes of memory leaks?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "General debugging knowledge".to_string(),
                tags: vec!["debugging".to_string(), "memory".to_string()],
            },
            TestScenario {
                id: "no_tool_019".to_string(),
                category: TestCategory::NoTool,
                prompt: "How do I debug a segmentation fault?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Debugging methodology question".to_string(),
                tags: vec!["debugging".to_string(), "segfault".to_string()],
            },
            
            // Language comparisons
            TestScenario {
                id: "no_tool_020".to_string(),
                category: TestCategory::NoTool,
                prompt: "Compare Python and JavaScript for web development".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Comparison/analysis request".to_string(),
                tags: vec!["comparison".to_string(), "languages".to_string()],
            },
        ]
    }

    /// Generate scenarios for first-round tool recommendations
    pub fn single_tool_scenarios() -> Vec<TestScenario> {
        vec![
            // read_file scenarios
            TestScenario {
                id: "single_tool_001".to_string(),
                category: TestCategory::SingleTool,
                prompt: "Show me the contents of README.md".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "read_file".to_string(),
                    args_pattern: HashMap::from([
                        ("file_path".to_string(), "README.md".to_string())
                    ]),
                },
                rationale: "Direct request to read a specific file".to_string(),
                tags: vec!["read".to_string(), "direct".to_string()],
            },
            TestScenario {
                id: "single_tool_002".to_string(),
                category: TestCategory::SingleTool,
                prompt: "What's in the Cargo.toml file?".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "read_file".to_string(),
                    args_pattern: HashMap::from([
                        ("file_path".to_string(), "Cargo.toml".to_string())
                    ]),
                },
                rationale: "Indirect request to read a file".to_string(),
                tags: vec!["read".to_string(), "indirect".to_string()],
            },
            TestScenario {
                id: "single_tool_003".to_string(),
                category: TestCategory::SingleTool,
                prompt: "Can you read the file at src/main.rs?".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "read_file".to_string(),
                    args_pattern: HashMap::from([
                        ("file_path".to_string(), "src/main.rs".to_string())
                    ]),
                },
                rationale: "Polite request with path".to_string(),
                tags: vec!["read".to_string(), "polite".to_string()],
            },
            
            // write_file scenarios
            TestScenario {
                id: "single_tool_004".to_string(),
                category: TestCategory::SingleTool,
                prompt: "Create a file called test.txt with the content 'Hello World'".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "write_file".to_string(),
                    args_pattern: HashMap::from([
                        ("file_path".to_string(), "test.txt".to_string()),
                        ("content".to_string(), "Hello World".to_string()),
                    ]),
                },
                rationale: "Explicit file creation request".to_string(),
                tags: vec!["write".to_string(), "create".to_string()],
            },
            TestScenario {
                id: "single_tool_005".to_string(),
                category: TestCategory::SingleTool,
                prompt: "Write a Python hello world program to hello.py".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "write_file".to_string(),
                    args_pattern: HashMap::from([
                        ("file_path".to_string(), "hello.py".to_string()),
                        ("content".to_string(), "print('Hello, World!')".to_string()),
                    ]),
                },
                rationale: "Code generation with file write".to_string(),
                tags: vec!["write".to_string(), "code".to_string()],
            },
            
            // list_files scenarios
            TestScenario {
                id: "single_tool_006".to_string(),
                category: TestCategory::SingleTool,
                prompt: "List all Python files in the current directory".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "list_files".to_string(),
                    args_pattern: HashMap::from([
                        ("pattern".to_string(), "*.py".to_string())
                    ]),
                },
                rationale: "File listing with pattern".to_string(),
                tags: vec!["list".to_string(), "pattern".to_string()],
            },
            TestScenario {
                id: "single_tool_007".to_string(),
                category: TestCategory::SingleTool,
                prompt: "Show me all Rust files in the src directory".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "list_files".to_string(),
                    args_pattern: HashMap::from([
                        ("pattern".to_string(), "src/*.rs".to_string())
                    ]),
                },
                rationale: "Directory-specific listing".to_string(),
                tags: vec!["list".to_string(), "directory".to_string()],
            },
            
            // search_code scenarios
            TestScenario {
                id: "single_tool_008".to_string(),
                category: TestCategory::SingleTool,
                prompt: "Search for 'TODO' comments in the codebase".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "search_code".to_string(),
                    args_pattern: HashMap::from([
                        ("pattern".to_string(), "TODO".to_string())
                    ]),
                },
                rationale: "Code search request".to_string(),
                tags: vec!["search".to_string(), "todo".to_string()],
            },
            TestScenario {
                id: "single_tool_009".to_string(),
                category: TestCategory::SingleTool,
                prompt: "Find all occurrences of 'function_call' in Rust files".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "search_code".to_string(),
                    args_pattern: HashMap::from([
                        ("pattern".to_string(), "function_call".to_string()),
                        ("file_pattern".to_string(), "*.rs".to_string()),
                    ]),
                },
                rationale: "Pattern search with file filter".to_string(),
                tags: vec!["search".to_string(), "filtered".to_string()],
            },
            
            // More nuanced read scenarios
            TestScenario {
                id: "single_tool_010".to_string(),
                category: TestCategory::SingleTool,
                prompt: "I need to see what's in the configuration file at config/settings.json".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "read_file".to_string(),
                    args_pattern: HashMap::from([
                        ("file_path".to_string(), "config/settings.json".to_string())
                    ]),
                },
                rationale: "Nested path with context".to_string(),
                tags: vec!["read".to_string(), "config".to_string()],
            },
        ]
    }

    /// Generate edge cases and adversarial inputs
    pub fn edge_cases() -> Vec<TestScenario> {
        vec![
            // Ambiguous file references
            TestScenario {
                id: "edge_001".to_string(),
                category: TestCategory::EdgeCase,
                prompt: "Tell me about the main file".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Ambiguous - could mean explain or read".to_string(),
                tags: vec!["ambiguous".to_string(), "file".to_string()],
            },
            
            // File paths in questions
            TestScenario {
                id: "edge_002".to_string(),
                category: TestCategory::EdgeCase,
                prompt: "Why would someone name a file README.md?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Contains file name but asking why, not to read".to_string(),
                tags: vec!["misleading".to_string(), "filename".to_string()],
            },
            
            // Code patterns in explanations
            TestScenario {
                id: "edge_003".to_string(),
                category: TestCategory::EdgeCase,
                prompt: "Explain what 'async fn' means in Rust".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Contains search-like pattern but is explanation".to_string(),
                tags: vec!["pattern".to_string(), "explanation".to_string()],
            },
            
            // Mixed intent
            TestScenario {
                id: "edge_004".to_string(),
                category: TestCategory::EdgeCase,
                prompt: "Can you explain what's in a typical Cargo.toml file?".to_string(),
                expected_behavior: ExpectedBehavior::NoFunctionCall,
                rationale: "Asks for explanation, not specific file content".to_string(),
                tags: vec!["mixed".to_string(), "explanation".to_string()],
            },
            
            // Quoted patterns
            TestScenario {
                id: "edge_005".to_string(),
                category: TestCategory::EdgeCase,
                prompt: "Search for the string 'search for' in files".to_string(),
                expected_behavior: ExpectedBehavior::SingleFunction {
                    name: "search_code".to_string(),
                    args_pattern: HashMap::from([
                        ("pattern".to_string(), "search for".to_string())
                    ]),
                },
                rationale: "Meta-search with misleading keywords".to_string(),
                tags: vec!["meta".to_string(), "search".to_string()],
            },
        ]
    }

    /// Generate all test scenarios
    pub fn all_scenarios() -> Vec<TestScenario> {
        let mut scenarios = Vec::new();
        scenarios.extend(Self::no_tool_scenarios());
        scenarios.extend(Self::single_tool_scenarios());
        scenarios.extend(Self::edge_cases());
        
        // Add more categories as implemented
        scenarios
    }

    /// Get scenarios by category
    pub fn by_category(category: TestCategory) -> Vec<TestScenario> {
        Self::all_scenarios()
            .into_iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Get scenarios by tag
    pub fn by_tag(tag: &str) -> Vec<TestScenario> {
        Self::all_scenarios()
            .into_iter()
            .filter(|s| s.tags.contains(&tag.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_generation() {
        let no_tool = ScenarioGenerator::no_tool_scenarios();
        assert!(no_tool.len() >= 20);
        
        let single_tool = ScenarioGenerator::single_tool_scenarios();
        assert!(single_tool.len() >= 10);
        
        let edge_cases = ScenarioGenerator::edge_cases();
        assert!(edge_cases.len() >= 5);
    }

    #[test]
    fn test_category_filtering() {
        let no_tool_scenarios = ScenarioGenerator::by_category(TestCategory::NoTool);
        assert!(no_tool_scenarios.iter().all(|s| s.category == TestCategory::NoTool));
    }

    #[test]
    fn test_tag_filtering() {
        let search_scenarios = ScenarioGenerator::by_tag("search");
        assert!(search_scenarios.iter().all(|s| s.tags.contains(&"search".to_string())));
    }
}