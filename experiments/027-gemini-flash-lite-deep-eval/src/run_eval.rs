use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run full evaluation (all 1000 questions)
    #[arg(long)]
    full: bool,

    /// Batch size for partial runs
    #[arg(long, default_value = "25")]
    batch_size: usize,

    /// Delay between API calls in seconds
    #[arg(long, default_value = "2")]
    delay: u64,

    /// Model to use
    #[arg(long, default_value = "gemini-2.0-flash-lite")]
    model: String,

    /// Starting batch number (1-40)
    #[arg(long, default_value = "1")]
    start_batch: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Question {
    id: String,
    question: String,
    expected_tool_calls: Vec<String>,
    category: String,
    context: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Batch {
    batch_id: String,
    created_at: String,
    model_target: String,
    questions: Vec<Question>,
}

#[derive(Debug, Serialize)]
struct EvalResult {
    question_id: String,
    question: String,
    expected_tools: Vec<String>,
    actual_tools: Vec<String>,
    success: bool,
    response_time_ms: u64,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct BatchResult {
    batch_id: String,
    model: String,
    total_questions: usize,
    successful: usize,
    failed: usize,
    success_rate: f64,
    results: Vec<EvalResult>,
    started_at: String,
    completed_at: String,
}

async fn evaluate_question(question: &Question, _model: &str) -> EvalResult {
    let start = std::time::Instant::now();
    
    // TODO: Implement actual API call to Gemini
    // For now, simulate with mock response
    let mock_success = question.id.ends_with('1') || question.id.ends_with('5');
    let actual_tools = if mock_success {
        question.expected_tool_calls.clone()
    } else {
        vec![]
    };

    EvalResult {
        question_id: question.id.clone(),
        question: question.question.clone(),
        expected_tools: question.expected_tool_calls.clone(),
        actual_tools,
        success: mock_success,
        response_time_ms: start.elapsed().as_millis() as u64,
        error: if !mock_success {
            Some("Mock failure".to_string())
        } else {
            None
        },
    }
}

async fn process_batch(batch_path: &Path, model: &str, delay_secs: u64) -> Result<BatchResult> {
    let content = fs::read_to_string(batch_path)?;
    let batch: Batch = serde_json::from_str(&content)?;
    
    let started_at = Utc::now().to_rfc3339();
    let mut results = Vec::new();
    
    println!("Processing batch: {}", batch.batch_id);
    
    for (i, question) in batch.questions.iter().enumerate() {
        if i > 0 {
            sleep(Duration::from_secs(delay_secs)).await;
        }
        
        print!("  Question {}/{}: ", i + 1, batch.questions.len());
        let result = evaluate_question(question, model).await;
        
        if result.success {
            println!("✓");
        } else {
            println!("✗");
        }
        
        results.push(result);
    }
    
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;
    let success_rate = successful as f64 / results.len() as f64;
    
    Ok(BatchResult {
        batch_id: batch.batch_id,
        model: model.to_string(),
        total_questions: results.len(),
        successful,
        failed,
        success_rate,
        results,
        started_at,
        completed_at: Utc::now().to_rfc3339(),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("Gemini Flash Lite Deep Evaluation Runner");
    println!("Model: {}", args.model);
    println!("Rate limit delay: {}s", args.delay);
    
    let data_dir = Path::new("data");
    let results_dir = Path::new("results");
    fs::create_dir_all(results_dir)?;
    
    if args.full {
        println!("Running full evaluation (1000 questions)...");
        
        for batch_num in 1..=40 {
            let batch_file = data_dir.join(format!("eval_questions_batch_{:03}.json", batch_num));
            
            if batch_file.exists() {
                let result = process_batch(&batch_file, &args.model, args.delay).await?;
                
                let result_file = results_dir.join(format!(
                    "results_batch_{:03}_{}.json",
                    batch_num,
                    Utc::now().timestamp()
                ));
                
                fs::write(&result_file, serde_json::to_string_pretty(&result)?)?;
                
                println!(
                    "Batch {} complete: {}/{} successful ({:.1}%)",
                    batch_num,
                    result.successful,
                    result.total_questions,
                    result.success_rate * 100.0
                );
            }
        }
    } else {
        println!("Running batch evaluation (batch {}, {} questions)...", 
                 args.start_batch, args.batch_size);
        
        let batch_file = data_dir.join(format!("eval_questions_batch_{:03}.json", args.start_batch));
        
        if batch_file.exists() {
            let result = process_batch(&batch_file, &args.model, args.delay).await?;
            
            let result_file = results_dir.join(format!(
                "results_batch_{:03}_{}.json",
                args.start_batch,
                Utc::now().timestamp()
            ));
            
            fs::write(&result_file, serde_json::to_string_pretty(&result)?)?;
            
            println!(
                "Evaluation complete: {}/{} successful ({:.1}%)",
                result.successful,
                result.total_questions,
                result.success_rate * 100.0
            );
        } else {
            eprintln!("Batch file not found: {:?}", batch_file);
        }
    }
    
    Ok(())
}