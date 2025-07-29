//! Function Calling Experiment for Gemini API
//! 
//! Tests function calling capabilities with various example functions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, warn};

// Function registry type
type FunctionHandler = Box<dyn Fn(&Value) -> Result<Value> + Send + Sync>;

struct FunctionRegistry {
    functions: HashMap<String, FunctionHandler>,
    declarations: Vec<FunctionDeclaration>,
}

impl std::fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionRegistry")
            .field("declarations", &self.declarations)
            .field("functions_count", &self.functions.len())
            .finish()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: Option<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FunctionCall {
    name: String,
    args: Option<Value>,
}

impl FunctionRegistry {
    fn new() -> Self {
        Self {
            functions: HashMap::new(),
            declarations: Vec::new(),
        }
    }

    fn register<F>(&mut self, name: &str, description: &str, parameters: Option<Value>, handler: F)
    where
        F: Fn(&Value) -> Result<Value> + Send + Sync + 'static,
    {
        self.declarations.push(FunctionDeclaration {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
        });
        self.functions.insert(name.to_string(), Box::new(handler));
    }

    fn call(&self, name: &str, args: &Value) -> Result<Value> {
        match self.functions.get(name) {
            Some(handler) => handler(args),
            None => Err(anyhow::anyhow!("Function '{}' not found", name)),
        }
    }
}

// Example functions
fn calculator(args: &Value) -> Result<Value> {
    let a = args["a"].as_f64().ok_or(anyhow::anyhow!("Missing parameter 'a'"))?;
    let b = args["b"].as_f64().ok_or(anyhow::anyhow!("Missing parameter 'b'"))?;
    let operation = args["operation"].as_str().ok_or(anyhow::anyhow!("Missing parameter 'operation'"))?;
    
    let result = match operation {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Err(anyhow::anyhow!("Division by zero"));
            }
            a / b
        }
        _ => return Err(anyhow::anyhow!("Unknown operation: {}", operation)),
    };
    
    Ok(json!({ "result": result }))
}

fn get_weather(args: &Value) -> Result<Value> {
    let location = args["location"].as_str().ok_or(anyhow::anyhow!("Missing parameter 'location'"))?;
    
    // Mock weather data
    Ok(json!({
        "location": location,
        "temperature": 72,
        "conditions": "Partly cloudy",
        "humidity": 65,
        "wind_speed": 8
    }))
}

fn get_current_time(_args: &Value) -> Result<Value> {
    let now = chrono::Local::now();
    Ok(json!({
        "time": now.format("%H:%M:%S").to_string(),
        "date": now.format("%Y-%m-%d").to_string(),
        "timezone": "local"
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    info!("=== Function Calling Experiment ===");

    // Create function registry
    let mut registry = FunctionRegistry::new();
    
    // Register calculator function
    registry.register(
        "calculator",
        "Perform basic arithmetic operations",
        Some(json!({
            "type": "object",
            "properties": {
                "a": {"type": "number", "description": "First number"},
                "b": {"type": "number", "description": "Second number"},
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "Operation to perform"
                }
            },
            "required": ["a", "b", "operation"]
        })),
        calculator
    );

    // Register weather function
    registry.register(
        "get_weather",
        "Get current weather for a location",
        Some(json!({
            "type": "object",
            "properties": {
                "location": {"type": "string", "description": "City name or location"}
            },
            "required": ["location"]
        })),
        get_weather
    );

    // Register time function
    registry.register(
        "get_current_time",
        "Get the current time and date",
        None,
        get_current_time
    );

    info!("Registered {} functions:", registry.declarations.len());
    for decl in &registry.declarations {
        info!("  - {}: {}", decl.name, decl.description);
    }

    // Test function calls
    info!("\n--- Testing Function Calls ---");
    
    // Test calculator
    let calc_args = json!({"a": 10, "b": 5, "operation": "multiply"});
    match registry.call("calculator", &calc_args) {
        Ok(result) => info!("calculator({}) = {}", calc_args, result),
        Err(e) => warn!("calculator error: {}", e),
    }

    // Test weather
    let weather_args = json!({"location": "San Francisco"});
    match registry.call("get_weather", &weather_args) {
        Ok(result) => info!("get_weather({}) = {}", weather_args, result),
        Err(e) => warn!("get_weather error: {}", e),
    }

    // Test time
    match registry.call("get_current_time", &json!({})) {
        Ok(result) => info!("get_current_time() = {}", result),
        Err(e) => warn!("get_current_time error: {}", e),
    }

    // Test error handling
    info!("\n--- Testing Error Handling ---");
    
    // Unknown function
    match registry.call("unknown_function", &json!({})) {
        Ok(_) => warn!("Unexpected success"),
        Err(e) => info!("✓ Correctly caught error: {}", e),
    }

    // Missing parameters
    match registry.call("calculator", &json!({"a": 10})) {
        Ok(_) => warn!("Unexpected success"),
        Err(e) => info!("✓ Correctly caught error: {}", e),
    }

    // Division by zero
    let div_zero_args = json!({"a": 10, "b": 0, "operation": "divide"});
    match registry.call("calculator", &div_zero_args) {
        Ok(_) => warn!("Unexpected success"),
        Err(e) => info!("✓ Correctly caught error: {}", e),
    }

    info!("\n=== Function Calling Test Complete ===");
    
    // TODO: Integration with Gemini API
    // This would involve:
    // 1. Sending function declarations with the prompt
    // 2. Parsing function calls from the response
    // 3. Executing functions and sending results back
    // 4. Getting final response incorporating function results
    
    info!("\nNext steps:");
    info!("1. Integrate with Gemini API client");
    info!("2. Parse function_call from API responses");
    info!("3. Execute functions and return results");
    info!("4. Handle multi-turn function calling");

    Ok(())
}