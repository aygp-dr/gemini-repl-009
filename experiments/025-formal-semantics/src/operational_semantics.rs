//! Operational Semantics for AI-Powered REPL
//! 
//! This module provides a formal operational semantics for REPL behavior,
//! defining precise transition rules and evaluation contexts.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Syntax of REPL expressions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// User input command
    Input(String),
    /// Parsed command
    Command(Command),
    /// Query to AI model
    Query(String, Context),
    /// Function call
    FunctionCall(String, Vec<Value>),
    /// Model response
    Response(String),
    /// Error
    Error(ErrorKind),
    /// Sequence of expressions
    Sequence(Box<Expression>, Box<Expression>),
    /// No operation
    Noop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    Help,
    Exit,
    Clear,
    ShowContext,
    SetModel(String),
    UserQuery(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    List(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    ParseError(String),
    FunctionNotFound(String),
    InvalidArguments(String),
    ModelError(String),
    Timeout,
}

/// Environment: variable bindings and function definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub model: String,
    pub functions: HashMap<String, FunctionDef>,
    pub variables: HashMap<String, Value>,
    pub api_key: Option<String>,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<String>,
    pub description: String,
    pub implementation: FunctionImpl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionImpl {
    Native(String), // Name of native function
    User(Expression), // User-defined function body
}

/// Context: conversation history
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Configuration for operational semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_context_length: usize,
    pub timeout_ms: u64,
    pub max_function_calls: usize,
}

/// Evaluation state
#[derive(Debug, Clone)]
pub struct EvalState {
    pub expression: Expression,
    pub environment: Environment,
    pub context: Context,
    pub output: Vec<String>,
}

/// Small-step operational semantics
pub struct OperationalSemantics {
    config: Config,
}

impl OperationalSemantics {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Single evaluation step
    /// Returns None if evaluation is complete
    pub fn step(&self, state: EvalState) -> Option<EvalState> {
        match &state.expression {
            Expression::Input(input) => {
                // Rule: INPUT
                // ⟨Input(s), σ, h⟩ → ⟨Command(parse(s)), σ, h⟩
                let cmd = self.parse_input(input);
                Some(EvalState {
                    expression: Expression::Command(cmd),
                    ..state
                })
            }

            Expression::Command(cmd) => {
                // Rule: COMMAND
                match cmd {
                    Command::Help => {
                        // ⟨Help, σ, h⟩ → ⟨Noop, σ, h⟩ with output
                        let mut new_state = state;
                        new_state.output.push(self.help_text());
                        new_state.expression = Expression::Noop;
                        Some(new_state)
                    }
                    
                    Command::Exit => {
                        // ⟨Exit, σ, h⟩ → terminal
                        None
                    }
                    
                    Command::Clear => {
                        // ⟨Clear, σ, h⟩ → ⟨Noop, σ, []⟩
                        Some(EvalState {
                            expression: Expression::Noop,
                            context: Context { messages: vec![] },
                            ..state
                        })
                    }
                    
                    Command::ShowContext => {
                        // ⟨ShowContext, σ, h⟩ → ⟨Noop, σ, h⟩ with output
                        let mut new_state = state;
                        new_state.output.push(format!("{:?}", new_state.context));
                        new_state.expression = Expression::Noop;
                        Some(new_state)
                    }
                    
                    Command::SetModel(model) => {
                        // ⟨SetModel(m), σ, h⟩ → ⟨Noop, σ[model ↦ m], h⟩
                        let mut new_env = state.environment;
                        new_env.model = model.clone();
                        Some(EvalState {
                            expression: Expression::Noop,
                            environment: new_env,
                            ..state
                        })
                    }
                    
                    Command::UserQuery(query) => {
                        // ⟨UserQuery(q), σ, h⟩ → ⟨Query(q, h), σ, h + UserMsg(q)⟩
                        let mut new_context = state.context;
                        new_context.messages.push(Message {
                            role: "user".to_string(),
                            content: query.clone(),
                        });
                        Some(EvalState {
                            expression: Expression::Query(query.clone(), new_context.clone()),
                            context: new_context,
                            ..state
                        })
                    }
                }
            }

            Expression::Query(query, context) => {
                // Rule: QUERY-FUNCTION
                // If query requires function call:
                // ⟨Query(q, h), σ, h⟩ → ⟨FunctionCall(f, args), σ, h⟩
                if let Some((func, args)) = self.analyze_for_function_call(query) {
                    Some(EvalState {
                        expression: Expression::FunctionCall(func, args),
                        ..state
                    })
                } else {
                    // Rule: QUERY-MODEL
                    // ⟨Query(q, h), σ, h⟩ → ⟨Response(model(q, h)), σ, h⟩
                    let response = self.mock_model_response(query, context);
                    Some(EvalState {
                        expression: Expression::Response(response),
                        ..state
                    })
                }
            }

            Expression::FunctionCall(name, args) => {
                // Rule: FUNCTION-CALL
                if let Some(func_def) = state.environment.functions.get(name) {
                    match &func_def.implementation {
                        FunctionImpl::Native(native_name) => {
                            // ⟨FunctionCall(f, args), σ, h⟩ → ⟨Response(eval(f, args)), σ, h⟩
                            let result = self.eval_native_function(native_name, args);
                            Some(EvalState {
                                expression: Expression::Response(result),
                                ..state
                            })
                        }
                        FunctionImpl::User(body) => {
                            // User-defined function evaluation
                            Some(EvalState {
                                expression: body.clone(),
                                ..state
                            })
                        }
                    }
                } else {
                    // Function not found
                    Some(EvalState {
                        expression: Expression::Error(ErrorKind::FunctionNotFound(name.clone())),
                        ..state
                    })
                }
            }

            Expression::Response(response) => {
                // Rule: RESPONSE
                // ⟨Response(r), σ, h⟩ → ⟨Noop, σ, h + ModelMsg(r)⟩ with output
                let mut new_state = state;
                new_state.output.push(response.clone());
                new_state.context.messages.push(Message {
                    role: "assistant".to_string(),
                    content: response.clone(),
                });
                new_state.expression = Expression::Noop;
                Some(new_state)
            }

            Expression::Error(err) => {
                // Rule: ERROR
                // ⟨Error(e), σ, h⟩ → ⟨Noop, σ, h⟩ with error output
                let mut new_state = state;
                new_state.output.push(format!("Error: {:?}", err));
                new_state.expression = Expression::Noop;
                Some(new_state)
            }

            Expression::Sequence(e1, e2) => {
                // Rule: SEQ-LEFT
                // ⟨e1, σ, h⟩ → ⟨e1', σ', h'⟩
                // ────────────────────────────────
                // ⟨e1; e2, σ, h⟩ → ⟨e1'; e2, σ', h'⟩
                if let Expression::Noop = **e1 {
                    // Rule: SEQ-NOOP
                    // ⟨Noop; e2, σ, h⟩ → ⟨e2, σ, h⟩
                    Some(EvalState {
                        expression: (**e2).clone(),
                        ..state
                    })
                } else {
                    let mut new_state = state;
                    new_state.expression = (**e1).clone();
                    if let Some(stepped) = self.step(new_state) {
                        Some(EvalState {
                            expression: Expression::Sequence(
                                Box::new(stepped.expression),
                                e2.clone()
                            ),
                            environment: stepped.environment,
                            context: stepped.context,
                            output: stepped.output,
                        })
                    } else {
                        None
                    }
                }
            }

            Expression::Noop => {
                // Terminal state for successful evaluation
                None
            }
        }
    }

    /// Big-step evaluation (evaluates to completion)
    pub fn eval(&self, initial_state: EvalState) -> Result<EvalState, String> {
        let mut state = initial_state;
        let mut steps = 0;
        const MAX_STEPS: usize = 1000;

        while steps < MAX_STEPS {
            match self.step(state.clone()) {
                Some(new_state) => state = new_state,
                None => return Ok(state),
            }
            steps += 1;
        }

        Err("Evaluation did not terminate within step limit".to_string())
    }

    // Helper methods

    fn parse_input(&self, input: &str) -> Command {
        let trimmed = input.trim();
        if trimmed.starts_with('/') {
            match trimmed {
                "/help" => Command::Help,
                "/exit" | "/quit" => Command::Exit,
                "/clear" => Command::Clear,
                "/context" => Command::ShowContext,
                _ if trimmed.starts_with("/model ") => {
                    let model = trimmed.trim_start_matches("/model ").to_string();
                    Command::SetModel(model)
                }
                _ => Command::UserQuery(input.to_string()),
            }
        } else {
            Command::UserQuery(input.to_string())
        }
    }

    fn analyze_for_function_call(&self, query: &str) -> Option<(String, Vec<Value>)> {
        // Simplified analysis - in practice would use NLP/AI
        let lower = query.to_lowercase();
        
        if lower.contains("read") && lower.contains("file") {
            if let Some(path) = self.extract_file_path(query) {
                return Some(("read_file".to_string(), vec![Value::String(path)]));
            }
        }
        
        if lower.contains("write") || lower.contains("create") {
            if let Some((path, content)) = self.extract_write_params(query) {
                return Some(("write_file".to_string(), vec![
                    Value::String(path),
                    Value::String(content),
                ]));
            }
        }
        
        None
    }

    fn extract_file_path(&self, query: &str) -> Option<String> {
        // Mock implementation
        if query.contains("README.md") {
            Some("README.md".to_string())
        } else if query.contains("Cargo.toml") {
            Some("Cargo.toml".to_string())
        } else {
            None
        }
    }

    fn extract_write_params(&self, query: &str) -> Option<(String, String)> {
        // Mock implementation
        if query.contains("test.txt") {
            Some(("test.txt".to_string(), "Hello World".to_string()))
        } else {
            None
        }
    }

    fn mock_model_response(&self, query: &str, _context: &Context) -> String {
        format!("Mock response to: {}", query)
    }

    fn eval_native_function(&self, name: &str, args: &[Value]) -> String {
        match name {
            "read_file" => {
                if let Some(Value::String(path)) = args.first() {
                    format!("Contents of {}: [file data]", path)
                } else {
                    "Error: Invalid arguments".to_string()
                }
            }
            "write_file" => {
                if args.len() >= 2 {
                    "File written successfully".to_string()
                } else {
                    "Error: Invalid arguments".to_string()
                }
            }
            _ => format!("Unknown function: {}", name),
        }
    }

    fn help_text(&self) -> String {
        "Available commands:\n\
         /help - Show this help\n\
         /exit - Exit the REPL\n\
         /clear - Clear context\n\
         /context - Show conversation history\n\
         /model <name> - Set model".to_string()
    }
}

/// Formal typing rules
pub mod typing {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Type {
        String,
        Number,
        Boolean,
        List(Box<Type>),
        Object(HashMap<String, Type>),
        Function(Vec<Type>, Box<Type>),
        Any,
    }

    pub struct TypeChecker;

    impl TypeChecker {
        pub fn type_of(expr: &Expression, env: &Environment) -> Result<Type, String> {
            match expr {
                Expression::Input(_) => Ok(Type::String),
                Expression::Response(_) => Ok(Type::String),
                Expression::Error(_) => Ok(Type::Any),
                Expression::Noop => Ok(Type::Any),
                _ => Ok(Type::Any), // Simplified
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_environment() -> Environment {
        let mut functions = HashMap::new();
        
        functions.insert("read_file".to_string(), FunctionDef {
            name: "read_file".to_string(),
            parameters: vec!["path".to_string()],
            description: "Read a file".to_string(),
            implementation: FunctionImpl::Native("read_file".to_string()),
        });
        
        Environment {
            model: "gemini-2.0-flash-lite".to_string(),
            functions,
            variables: HashMap::new(),
            api_key: Some("test-key".to_string()),
        }
    }

    #[test]
    fn test_input_parsing() {
        let config = Config {
            max_context_length: 100,
            timeout_ms: 5000,
            max_function_calls: 10,
        };
        let semantics = OperationalSemantics::new(config);
        
        let state = EvalState {
            expression: Expression::Input("/help".to_string()),
            environment: default_environment(),
            context: Context { messages: vec![] },
            output: vec![],
        };
        
        let next = semantics.step(state).unwrap();
        assert!(matches!(next.expression, Expression::Command(Command::Help)));
    }

    #[test]
    fn test_help_command() {
        let config = Config {
            max_context_length: 100,
            timeout_ms: 5000,
            max_function_calls: 10,
        };
        let semantics = OperationalSemantics::new(config);
        
        let state = EvalState {
            expression: Expression::Command(Command::Help),
            environment: default_environment(),
            context: Context { messages: vec![] },
            output: vec![],
        };
        
        let next = semantics.step(state).unwrap();
        assert!(matches!(next.expression, Expression::Noop));
        assert!(!next.output.is_empty());
    }

    #[test]
    fn test_function_call_analysis() {
        let config = Config {
            max_context_length: 100,
            timeout_ms: 5000,
            max_function_calls: 10,
        };
        let semantics = OperationalSemantics::new(config);
        
        let state = EvalState {
            expression: Expression::Query(
                "Read the README.md file".to_string(),
                Context { messages: vec![] }
            ),
            environment: default_environment(),
            context: Context { messages: vec![] },
            output: vec![],
        };
        
        let next = semantics.step(state).unwrap();
        assert!(matches!(
            next.expression,
            Expression::FunctionCall(name, _) if name == "read_file"
        ));
    }
}