//! Main entry point for the test runner

use anyhow::Result;
use function_calling::test_runner::{run_test_suite, print_summary};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    let test_file = "tests/test_cases.json";
    let results = run_test_suite(test_file)?;
    print_summary(&results);
    
    Ok(())
}