use anyhow::Result;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    println!("Evaluation Generation Complete!");
    println!();
    
    let data_dir = Path::new("data");
    let mut total_questions = 0;
    let mut total_batches = 0;
    
    for entry in fs::read_dir(data_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if filename.starts_with("eval_questions_batch_") {
                    total_batches += 1;
                    
                    // Count questions in this batch
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(questions) = json["questions"].as_array() {
                                total_questions += questions.len();
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("Summary:");
    println!("  Total batches: {}", total_batches);
    println!("  Total questions: {}", total_questions);
    println!("  Average questions per batch: {:.1}", 
             total_questions as f64 / total_batches as f64);
    println!();
    println!("All evaluation questions have been generated!");
    println!("Run 'make test-batch' to test a single batch");
    println!("Run 'make test-full' to run the complete evaluation");
    
    Ok(())
}