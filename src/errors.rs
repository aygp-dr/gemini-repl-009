//! Custom error types for the Gemini REPL

use std::fmt;

/// Main error type for the Gemini REPL
#[derive(Debug)]
pub enum GeminiError {
    /// Tool-related errors
    Tool(ToolError),
    /// API-related errors
    Api(ApiError),
    /// Security-related errors
    Security(SecurityError),
    /// IO-related errors
    Io(std::io::Error),
    /// Serialization errors
    Serialization(serde_json::Error),
    /// Configuration errors
    Config(String),
}

/// Tool-specific errors
#[derive(Debug)]
pub enum ToolError {
    /// Tool not found
    NotFound(String),
    /// Invalid parameters
    InvalidParameters(String),
    /// Execution failed
    ExecutionFailed(String),
    /// Code analysis failed
    AnalysisFailed(String),
    /// Build failed
    BuildFailed(String),
}

/// API-related errors
#[derive(Debug)]
pub enum ApiError {
    /// Authentication failed
    Authentication(String),
    /// Rate limit exceeded
    RateLimit,
    /// Network error
    Network(String),
    /// Invalid response
    InvalidResponse(String),
}

/// Security-related errors
#[derive(Debug)]
pub enum SecurityError {
    /// Path traversal attempt
    PathTraversal(String),
    /// Access denied
    AccessDenied(String),
    /// Unsafe operation
    UnsafeOperation(String),
}

impl fmt::Display for GeminiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeminiError::Tool(e) => write!(f, "Tool error: {}", e),
            GeminiError::Api(e) => write!(f, "API error: {}", e),
            GeminiError::Security(e) => write!(f, "Security error: {}", e),
            GeminiError::Io(e) => write!(f, "IO error: {}", e),
            GeminiError::Serialization(e) => write!(f, "Serialization error: {}", e),
            GeminiError::Config(e) => write!(f, "Configuration error: {}", e),
        }
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::NotFound(name) => write!(f, "Tool '{}' not found", name),
            ToolError::InvalidParameters(msg) => write!(f, "Invalid parameters: {}", msg),
            ToolError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            ToolError::AnalysisFailed(msg) => write!(f, "Code analysis failed: {}", msg),
            ToolError::BuildFailed(msg) => write!(f, "Build failed: {}", msg),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Authentication(msg) => write!(f, "Authentication failed: {}", msg),
            ApiError::RateLimit => write!(f, "Rate limit exceeded"),
            ApiError::Network(msg) => write!(f, "Network error: {}", msg),
            ApiError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
        }
    }
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::PathTraversal(path) => write!(f, "Path traversal attempt: {}", path),
            SecurityError::AccessDenied(resource) => write!(f, "Access denied to: {}", resource),
            SecurityError::UnsafeOperation(op) => write!(f, "Unsafe operation: {}", op),
        }
    }
}

impl std::error::Error for GeminiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GeminiError::Io(e) => Some(e),
            GeminiError::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl std::error::Error for ToolError {}
impl std::error::Error for ApiError {}
impl std::error::Error for SecurityError {}

// Conversion implementations
impl From<std::io::Error> for GeminiError {
    fn from(err: std::io::Error) -> Self {
        GeminiError::Io(err)
    }
}

impl From<serde_json::Error> for GeminiError {
    fn from(err: serde_json::Error) -> Self {
        GeminiError::Serialization(err)
    }
}

impl From<ToolError> for GeminiError {
    fn from(err: ToolError) -> Self {
        GeminiError::Tool(err)
    }
}

impl From<ApiError> for GeminiError {
    fn from(err: ApiError) -> Self {
        GeminiError::Api(err)
    }
}

impl From<SecurityError> for GeminiError {
    fn from(err: SecurityError) -> Self {
        GeminiError::Security(err)
    }
}

// Convenience constructors
impl ToolError {
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound(name.into())
    }
    
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParameters(msg.into())
    }
    
    pub fn execution_failed(msg: impl Into<String>) -> Self {
        Self::ExecutionFailed(msg.into())
    }
    
    pub fn analysis_failed(msg: impl Into<String>) -> Self {
        Self::AnalysisFailed(msg.into())
    }
    
    pub fn build_failed(msg: impl Into<String>) -> Self {
        Self::BuildFailed(msg.into())
    }
}

impl SecurityError {
    pub fn path_traversal(path: impl Into<String>) -> Self {
        Self::PathTraversal(path.into())
    }
    
    pub fn access_denied(resource: impl Into<String>) -> Self {
        Self::AccessDenied(resource.into())
    }
    
    pub fn unsafe_operation(op: impl Into<String>) -> Self {
        Self::UnsafeOperation(op.into())
    }
}

impl ApiError {
    pub fn authentication(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }
    
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }
    
    pub fn invalid_response(msg: impl Into<String>) -> Self {
        Self::InvalidResponse(msg.into())
    }
}

/// Type alias for Results using our custom error type
pub type Result<T> = std::result::Result<T, GeminiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let tool_error = ToolError::not_found("test_tool");
        let gemini_error = GeminiError::from(tool_error);
        assert!(gemini_error.to_string().contains("Tool 'test_tool' not found"));
    }

    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let gemini_error = GeminiError::from(io_error);
        assert!(matches!(gemini_error, GeminiError::Io(_)));
    }
}