//! Example Rust code for proxy configuration

use reqwest::{Client, Proxy};
use std::time::Duration;

/// Create HTTP client with proxy support
pub fn create_proxied_client() -> Result<Client, Box<dyn std::error::Error>> {
    let client = Client::builder()
        // Configure HTTP proxy
        .proxy(Proxy::http("http://localhost:3129")?)
        // Configure HTTPS proxy (same proxy for both)
        .proxy(Proxy::https("http://localhost:3129")?)
        // Set reasonable timeout
        .timeout(Duration::from_secs(30))
        // Build the client
        .build()?;
    
    Ok(client)
}

/// Example usage with Gemini API
pub async fn test_gemini_through_proxy() -> Result<(), Box<dyn std::error::Error>> {
    let client = create_proxied_client()?;
    
    // Test request to Gemini API
    let response = client
        .get("https://generativelanguage.googleapis.com/v1beta/models")
        .send()
        .await?;
    
    println!("Status: {}", response.status());
    println!("Headers: {:#?}", response.headers());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_proxy_client_creation() {
        let client = create_proxied_client();
        assert!(client.is_ok());
    }
}