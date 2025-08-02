//! Code analysis tools for understanding Rust code structure

use super::Tool;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{parse_file, Item, ItemFn, ItemStruct, ItemEnum, ItemImpl, ItemTrait, Visibility};

/// Tool for analyzing Rust code
pub struct AnalyzeRustCodeTool;

impl Default for AnalyzeRustCodeTool {
    fn default() -> Self {
        Self
    }
}

impl AnalyzeRustCodeTool {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Tool for AnalyzeRustCodeTool {
    fn name(&self) -> &str {
        "analyze_rust_code"
    }
    
    fn description(&self) -> &str {
        "Analyze Rust code to understand its structure (functions, structs, traits, etc.)"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "Rust code to analyze"
                }
            },
            "required": ["code"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            code: String,
        }
        
        let params: Params = serde_json::from_value(params)?;
        
        // Parse the Rust code
        let syntax_tree = parse_file(&params.code).map_err(|e| {
            anyhow::anyhow!("Failed to parse Rust code: {}", e)
        })?;
        
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut impls = Vec::new();
        
        // Analyze items
        for item in syntax_tree.items {
            match item {
                Item::Fn(item_fn) => {
                    functions.push(analyze_function(&item_fn));
                }
                Item::Struct(item_struct) => {
                    structs.push(json!({
                        "name": item_struct.ident.to_string(),
                        "visibility": visibility_to_string(&item_struct.vis),
                        "generics": item_struct.generics.params.len(),
                        "fields": match &item_struct.fields {
                            syn::Fields::Named(fields) => fields.named.len(),
                            syn::Fields::Unnamed(fields) => fields.unnamed.len(),
                            syn::Fields::Unit => 0,
                        }
                    }));
                }
                Item::Enum(item_enum) => {
                    enums.push(json!({
                        "name": item_enum.ident.to_string(),
                        "visibility": visibility_to_string(&item_enum.vis),
                        "variants": item_enum.variants.len(),
                    }));
                }
                Item::Trait(item_trait) => {
                    traits.push(json!({
                        "name": item_trait.ident.to_string(),
                        "visibility": visibility_to_string(&item_trait.vis),
                        "methods": item_trait.items.len(),
                    }));
                }
                Item::Impl(item_impl) => {
                    let type_name = if let Some((_, path, _)) = &item_impl.trait_ {
                        format!("{} for {}", quote::quote!(#path), quote::quote!(#item_impl.self_ty))
                    } else {
                        format!("{}", quote::quote!(#item_impl.self_ty))
                    };
                    
                    impls.push(json!({
                        "type": type_name,
                        "methods": item_impl.items.len(),
                    }));
                }
                _ => {}
            }
        }
        
        Ok(json!({
            "success": true,
            "analysis": {
                "functions": functions,
                "structs": structs,
                "enums": enums,
                "traits": traits,
                "impls": impls,
                "summary": {
                    "total_functions": functions.len(),
                    "total_structs": structs.len(),
                    "total_enums": enums.len(),
                    "total_traits": traits.len(),
                    "total_impls": impls.len(),
                }
            }
        }))
    }
}

fn analyze_function(item_fn: &ItemFn) -> Value {
    let mut params = Vec::new();
    for input in &item_fn.sig.inputs {
        match input {
            syn::FnArg::Receiver(_) => params.push("self".to_string()),
            syn::FnArg::Typed(pat_type) => {
                params.push(quote::quote!(#pat_type.pat).to_string());
            }
        }
    }
    
    json!({
        "name": item_fn.sig.ident.to_string(),
        "visibility": visibility_to_string(&item_fn.vis),
        "async": item_fn.sig.asyncness.is_some(),
        "params": params,
        "return_type": match &item_fn.sig.output {
            syn::ReturnType::Default => "()".to_string(),
            syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
        }
    })
}

/// Tool for finding functions in Rust code
pub struct FindFunctionTool {
    workspace: PathBuf,
}

impl FindFunctionTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for FindFunctionTool {
    fn name(&self) -> &str {
        "find_function"
    }
    
    fn description(&self) -> &str {
        "Find function definitions in Rust files"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Function name to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Optional path to search in (defaults to src/)"
                }
            },
            "required": ["name"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            name: String,
            path: Option<String>,
        }
        
        let params: Params = serde_json::from_value(params)?;
        let search_path = params.path.unwrap_or_else(|| "src".to_string());
        let full_path = self.workspace.join(&search_path);
        
        let mut results = Vec::new();
        search_rust_files(&full_path, |file_path, content| {
            if let Ok(syntax_tree) = parse_file(&content) {
                for item in syntax_tree.items {
                    if let Item::Fn(item_fn) = item {
                        if item_fn.sig.ident == params.name {
                            results.push(json!({
                                "file": file_path.strip_prefix(&self.workspace)
                                    .unwrap_or(file_path)
                                    .to_string_lossy(),
                                "function": analyze_function(&item_fn),
                                "line": 0, // TODO: Add line number tracking
                            }));
                        }
                    }
                }
            }
        })?;
        
        Ok(json!({
            "success": true,
            "query": params.name,
            "results": results,
            "count": results.len(),
        }))
    }
}

/// Tool for finding structs in Rust code
pub struct FindStructTool {
    workspace: PathBuf,
}

impl FindStructTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for FindStructTool {
    fn name(&self) -> &str {
        "find_struct"
    }
    
    fn description(&self) -> &str {
        "Find struct definitions in Rust files"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Struct name to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Optional path to search in (defaults to src/)"
                }
            },
            "required": ["name"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            name: String,
            path: Option<String>,
        }
        
        let params: Params = serde_json::from_value(params)?;
        let search_path = params.path.unwrap_or_else(|| "src".to_string());
        let full_path = self.workspace.join(&search_path);
        
        let mut results = Vec::new();
        search_rust_files(&full_path, |file_path, content| {
            if let Ok(syntax_tree) = parse_file(&content) {
                for item in syntax_tree.items {
                    if let Item::Struct(item_struct) = item {
                        if item_struct.ident == params.name {
                            let fields = match &item_struct.fields {
                                syn::Fields::Named(fields) => {
                                    fields.named.iter()
                                        .map(|f| json!({
                                            "name": f.ident.as_ref().map(|i| i.to_string()),
                                            "type": quote::quote!(#f.ty).to_string(),
                                            "visibility": visibility_to_string(&f.vis),
                                        }))
                                        .collect::<Vec<_>>()
                                }
                                syn::Fields::Unnamed(fields) => {
                                    fields.unnamed.iter()
                                        .enumerate()
                                        .map(|(i, f)| json!({
                                            "index": i,
                                            "type": quote::quote!(#f.ty).to_string(),
                                            "visibility": visibility_to_string(&f.vis),
                                        }))
                                        .collect::<Vec<_>>()
                                }
                                syn::Fields::Unit => vec![],
                            };
                            
                            results.push(json!({
                                "file": file_path.strip_prefix(&self.workspace)
                                    .unwrap_or(file_path)
                                    .to_string_lossy(),
                                "struct": {
                                    "name": item_struct.ident.to_string(),
                                    "visibility": visibility_to_string(&item_struct.vis),
                                    "generics": item_struct.generics.params.len(),
                                    "fields": fields,
                                },
                                "line": 0, // TODO: Add line number tracking
                            }));
                        }
                    }
                }
            }
        })?;
        
        Ok(json!({
            "success": true,
            "query": params.name,
            "results": results,
            "count": results.len(),
        }))
    }
}

fn search_rust_files<F>(path: &Path, mut callback: F) -> Result<()>
where
    F: FnMut(&Path, &str),
{
    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
        let content = fs::read_to_string(path)?;
        callback(path, &content);
    } else if path.is_dir() {
        for entry in walkdir::WalkDir::new(path)
            .follow_links(false) // Don't follow symlinks for security
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(path) {
                    callback(path, &content);
                }
            }
        }
    }
    Ok(())
}

/// Convert a Visibility to a string representation
fn visibility_to_string(vis: &Visibility) -> String {
    match vis {
        Visibility::Public(_) => "pub".to_string(),
        Visibility::Restricted(restricted) => {
            format!("pub({})", quote::quote!(#restricted.path))
        }
        Visibility::Inherited => "private".to_string(),
    }
}