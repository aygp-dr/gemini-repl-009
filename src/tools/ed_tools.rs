//! Ed-based file manipulation tools
//! 
//! This module implements file operations using ed(1) semantics,
//! providing a line-oriented approach to text manipulation.

use super::Tool;
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a line in the ed buffer
#[derive(Clone, Debug, PartialEq, Eq)]
struct Line {
    content: String,
}

impl Line {
    fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

/// Ed buffer implementation with line-oriented operations
pub struct EdBuffer {
    lines: Vec<Line>,
    current: usize,  // Current line number (1-indexed, 0 means before first line)
    modified: bool,
    marks: HashMap<char, usize>,
    filename: Option<String>,
}

/// Result of executing an ed command
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdResult {
    Success,
    Lines(Vec<String>),
    Written(usize),
    Read(usize),
    CurrentLine(usize),
    Error(String),
}

impl Default for EdBuffer {
    fn default() -> Self {
        Self {
            lines: Vec::new(),
            current: 0,
            modified: false,
            marks: HashMap::new(),
            filename: None,
        }
    }
}

impl EdBuffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create buffer from file contents
    pub fn from_string(content: &str) -> Self {
        let lines: Vec<Line> = content
            .lines()
            .map(Line::new)
            .collect();
        
        let line_count = lines.len();
        Self {
            lines,
            current: line_count,
            modified: false,
            marks: HashMap::new(),
            filename: None,
        }
    }

    /// Get current line number (1-indexed)
    pub fn current_line(&self) -> usize {
        self.current
    }

    /// Get total number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Parse line address from string
    fn parse_address(&self, addr: &str) -> Result<usize> {
        match addr {
            "." => Ok(self.current),
            "$" => Ok(self.lines.len()),
            "0" => Ok(0),
            _ => {
                if let Ok(n) = addr.parse::<usize>() {
                    if n <= self.lines.len() {
                        Ok(n)
                    } else {
                        bail!("Invalid address")
                    }
                } else if addr.starts_with('+') || addr.starts_with('-') {
                    let offset: i32 = addr.parse()?;
                    let new_addr = (self.current as i32) + offset;
                    if new_addr >= 0 && new_addr as usize <= self.lines.len() {
                        Ok(new_addr as usize)
                    } else {
                        bail!("Invalid address")
                    }
                } else {
                    bail!("Invalid address")
                }
            }
        }
    }

    /// Parse address range (e.g., "1,5" or "%" for all lines)
    fn parse_range(&self, range: &str) -> Result<(usize, usize)> {
        if range == "%" {
            return Ok((1, self.lines.len()));
        }

        let parts: Vec<&str> = range.split(',').collect();
        match parts.len() {
            1 => {
                let addr = self.parse_address(parts[0])?;
                Ok((addr, addr))
            }
            2 => {
                let start = if parts[0].is_empty() {
                    self.current
                } else {
                    self.parse_address(parts[0])?
                };
                let end = if parts[1].is_empty() {
                    self.lines.len()
                } else {
                    self.parse_address(parts[1])?
                };
                if start <= end && start > 0 {
                    Ok((start, end))
                } else {
                    bail!("Invalid range")
                }
            }
            _ => bail!("Invalid range"),
        }
    }

    /// Execute an ed command
    pub fn execute_command(&mut self, command: &str) -> Result<EdResult> {
        if command.is_empty() {
            return Ok(EdResult::Success);
        }

        // Parse address and command
        let trimmed = command.trim();
        
        // Handle simple address (just a number or special address)
        if let Ok(addr) = self.parse_address(trimmed) {
            if addr <= self.lines.len() {
                self.current = addr;
                return Ok(EdResult::CurrentLine(addr));
            }
        }

        // Extract command character and address
        let (addr_part, cmd_part) = self.split_command(trimmed)?;
        
        match cmd_part.chars().next() {
            Some('a') => self.cmd_append(addr_part),
            Some('i') => self.cmd_insert(addr_part),
            Some('d') => self.cmd_delete(addr_part),
            Some('c') => self.cmd_change(addr_part),
            Some('p') => self.cmd_print(addr_part),
            Some('n') => self.cmd_number(addr_part),
            Some('w') => self.cmd_write(&cmd_part[1..].trim()),
            Some('q') => self.cmd_quit(),
            Some('s') => self.cmd_substitute(addr_part, &cmd_part[1..]),
            Some('m') => self.cmd_move(addr_part, &cmd_part[1..]),
            Some('j') => self.cmd_join(addr_part),
            Some('=') => self.cmd_line_number(addr_part),
            _ => bail!("Unknown command"),
        }
    }

    /// Split command into address and command parts
    fn split_command<'a>(&self, cmd: &'a str) -> Result<(&'a str, &'a str)> {
        // Find where the command letter starts
        for (i, ch) in cmd.char_indices() {
            if ch.is_alphabetic() || ch == '=' {
                return Ok((&cmd[..i], &cmd[i..]));
            }
        }
        Ok((cmd, ""))
    }

    /// Append lines after the given address
    fn cmd_append(&mut self, addr: &str) -> Result<EdResult> {
        let line_num = if addr.is_empty() {
            self.current
        } else {
            self.parse_address(addr)?
        };
        
        // In a real implementation, this would read from input
        // For now, we'll just mark as ready for append
        self.current = line_num;
        Ok(EdResult::Success)
    }

    /// Insert lines before the given address
    fn cmd_insert(&mut self, addr: &str) -> Result<EdResult> {
        let line_num = if addr.is_empty() {
            self.current
        } else {
            self.parse_address(addr)?
        };
        
        self.current = if line_num > 0 { line_num - 1 } else { 0 };
        Ok(EdResult::Success)
    }

    /// Delete lines in range
    fn cmd_delete(&mut self, addr: &str) -> Result<EdResult> {
        let (start, end) = if addr.is_empty() {
            (self.current, self.current)
        } else {
            self.parse_range(addr)?
        };

        if start == 0 || start > self.lines.len() {
            bail!("Invalid address");
        }

        // Remove lines (convert to 0-indexed)
        for _ in start..=end {
            if start - 1 < self.lines.len() {
                self.lines.remove(start - 1);
            }
        }

        self.modified = true;
        self.current = if start > 1 { start - 1 } else { 0 };
        
        Ok(EdResult::Success)
    }

    /// Change (replace) lines
    fn cmd_change(&mut self, addr: &str) -> Result<EdResult> {
        self.cmd_delete(addr)?;
        Ok(EdResult::Success)
    }

    /// Print lines
    fn cmd_print(&self, addr: &str) -> Result<EdResult> {
        let (start, end) = if addr.is_empty() {
            (self.current, self.current)
        } else {
            self.parse_range(addr)?
        };

        if start == 0 || start > self.lines.len() {
            bail!("Invalid address");
        }

        let mut output = Vec::new();
        for i in start..=end {
            if i <= self.lines.len() {
                output.push(self.lines[i - 1].content.clone());
            }
        }

        Ok(EdResult::Lines(output))
    }

    /// Print lines with line numbers
    fn cmd_number(&self, addr: &str) -> Result<EdResult> {
        let (start, end) = if addr.is_empty() {
            (self.current, self.current)
        } else {
            self.parse_range(addr)?
        };

        if start == 0 || start > self.lines.len() {
            bail!("Invalid address");
        }

        let mut output = Vec::new();
        for i in start..=end {
            if i <= self.lines.len() {
                output.push(format!("{}\t{}", i, self.lines[i - 1].content));
            }
        }

        Ok(EdResult::Lines(output))
    }

    /// Write buffer to file
    fn cmd_write(&mut self, filename: &str) -> Result<EdResult> {
        let fname = if filename.is_empty() {
            self.filename.as_ref().ok_or_else(|| anyhow::anyhow!("No filename"))?
        } else {
            filename
        };

        let content: Vec<String> = self.lines.iter()
            .map(|l| l.content.clone())
            .collect();
        let text = content.join("\n");
        let bytes = text.len();

        // In real implementation, would write to file
        self.filename = Some(fname.to_string());
        self.modified = false;

        Ok(EdResult::Written(bytes))
    }

    /// Quit editor
    fn cmd_quit(&self) -> Result<EdResult> {
        if self.modified {
            bail!("Warning: buffer modified");
        }
        Ok(EdResult::Success)
    }

    /// Substitute text
    fn cmd_substitute(&mut self, addr: &str, args: &str) -> Result<EdResult> {
        let (start, end) = if addr.is_empty() {
            (self.current, self.current)
        } else {
            self.parse_range(addr)?
        };

        // Parse s/pattern/replacement/flags
        if args.is_empty() || !args.starts_with('/') {
            bail!("Invalid substitute command");
        }

        let parts: Vec<&str> = args[1..].split('/').collect();
        if parts.len() < 2 {
            bail!("Invalid substitute command");
        }

        let pattern = parts[0];
        let replacement = parts[1];
        let global = parts.get(2).map(|f| f.contains('g')).unwrap_or(false);

        let mut changed = false;
        for i in start..=end {
            if i <= self.lines.len() {
                let new_content = if global {
                    self.lines[i - 1].content.replace(pattern, replacement)
                } else {
                    self.lines[i - 1].content.replacen(pattern, replacement, 1)
                };
                
                if new_content != self.lines[i - 1].content {
                    self.lines[i - 1].content = new_content;
                    changed = true;
                }
            }
        }

        if changed {
            self.modified = true;
        }

        Ok(EdResult::Success)
    }

    /// Move lines to destination
    fn cmd_move(&mut self, addr: &str, dest: &str) -> Result<EdResult> {
        let (start, end) = self.parse_range(addr)?;
        let dest_addr = self.parse_address(dest.trim())?;

        if dest_addr >= start && dest_addr <= end {
            bail!("Invalid destination");
        }

        // Extract lines to move
        let mut to_move = Vec::new();
        for _ in start..=end {
            if start - 1 < self.lines.len() {
                to_move.push(self.lines.remove(start - 1));
            }
        }

        // Insert at destination
        let insert_pos = if dest_addr < start {
            dest_addr
        } else {
            dest_addr - (end - start + 1)
        };

        for (i, line) in to_move.into_iter().enumerate() {
            self.lines.insert(insert_pos + i, line);
        }

        self.modified = true;
        self.current = insert_pos + (end - start);

        Ok(EdResult::Success)
    }

    /// Join lines
    fn cmd_join(&mut self, addr: &str) -> Result<EdResult> {
        let (start, end) = if addr.is_empty() {
            if self.current < self.lines.len() {
                (self.current, self.current + 1)
            } else {
                bail!("Invalid address");
            }
        } else {
            self.parse_range(addr)?
        };

        if start >= end || end > self.lines.len() {
            return Ok(EdResult::Success);
        }

        // Join lines
        let mut joined = self.lines[start - 1].content.clone();
        for i in start..end {
            joined.push_str(&self.lines[i].content);
        }

        // Remove joined lines and replace with combined line
        for _ in start..end {
            self.lines.remove(start);
        }
        self.lines[start - 1].content = joined;

        self.modified = true;
        self.current = start;

        Ok(EdResult::Success)
    }

    /// Print line number
    fn cmd_line_number(&self, addr: &str) -> Result<EdResult> {
        let line_num = if addr.is_empty() {
            self.lines.len()
        } else if addr == "." {
            self.current
        } else {
            self.parse_address(addr)?
        };

        Ok(EdResult::Lines(vec![line_num.to_string()]))
    }

    /// Append a line to the buffer (used after 'a' or 'i' command)
    pub fn append_line(&mut self, line: &str) -> Result<()> {
        self.lines.insert(self.current, Line::new(line));
        self.current += 1;
        self.modified = true;
        Ok(())
    }

    /// Get buffer contents as string
    pub fn to_string(&self) -> String {
        if self.lines.is_empty() {
            String::new()
        } else {
            let capacity = self.lines.iter()
                .map(|l| l.content.len() + 1) // +1 for newline
                .sum::<usize>()
                .saturating_sub(1); // Remove last newline
            
            let mut result = String::with_capacity(capacity);
            for (i, line) in self.lines.iter().enumerate() {
                if i > 0 {
                    result.push('\n');
                }
                result.push_str(&line.content);
            }
            result
        }
    }
}

/// Ed-style line editor tool
pub struct EdTool {
    workspace: PathBuf,
}

impl EdTool {
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }
}

#[async_trait]
impl Tool for EdTool {
    fn name(&self) -> &str {
        "ed_editor"
    }
    
    fn description(&self) -> &str {
        "Ed-style line editor for precise text manipulation"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file": {
                    "type": "string",
                    "description": "File to edit (relative to workspace)"
                },
                "commands": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Ed commands to execute"
                },
                "content": {
                    "type": "string",
                    "description": "Initial content for new file"
                }
            },
            "required": ["commands"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        #[derive(Deserialize)]
        struct Params {
            file: Option<String>,
            commands: Vec<String>,
            content: Option<String>,
        }
        
        let params: Params = serde_json::from_value(params)?;
        
        // Create buffer
        let mut buffer = if let Some(content) = params.content {
            EdBuffer::from_string(&content)
        } else if let Some(file) = &params.file {
            let file_path = self.workspace.join(file);
            if file_path.exists() {
                let content = std::fs::read_to_string(&file_path)?;
                EdBuffer::from_string(&content)
            } else {
                EdBuffer::new()
            }
        } else {
            EdBuffer::new()
        };
        
        // Execute commands
        let mut results = Vec::new();
        for command in params.commands {
            match buffer.execute_command(&command) {
                Ok(result) => {
                    match result {
                        EdResult::Lines(lines) => {
                            results.push(json!({
                                "command": command,
                                "output": lines
                            }));
                        }
                        EdResult::Success => {
                            results.push(json!({
                                "command": command,
                                "status": "success"
                            }));
                        }
                        EdResult::Written(bytes) => {
                            results.push(json!({
                                "command": command,
                                "status": "written",
                                "bytes": bytes
                            }));
                        }
                        EdResult::Read(bytes) => {
                            results.push(json!({
                                "command": command,
                                "status": "read",
                                "bytes": bytes
                            }));
                        }
                        EdResult::CurrentLine(line) => {
                            results.push(json!({
                                "command": command,
                                "current_line": line
                            }));
                        }
                        EdResult::Error(msg) => {
                            results.push(json!({
                                "command": command,
                                "error": msg
                            }));
                        }
                    }
                }
                Err(e) => {
                    results.push(json!({
                        "command": command,
                        "error": e.to_string()
                    }));
                }
            }
        }
        
        // Save to file if specified
        let file_ref = if let Some(ref file) = params.file {
            let file_path = self.workspace.join(file);
            std::fs::write(&file_path, buffer.to_string())?;
            Some(file.clone())
        } else {
            None
        };
        
        Ok(json!({
            "success": true,
            "file": file_ref,
            "results": results,
            "final_content": buffer.to_string(),
            "line_count": buffer.line_count(),
            "current_line": buffer.current_line()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut ed = EdBuffer::from_string("line1\nline2\nline3");
        
        // Test print
        match ed.execute_command("2p").unwrap() {
            EdResult::Lines(lines) => assert_eq!(lines, vec!["line2"]),
            _ => panic!("Expected Lines result"),
        }

        // Test delete
        ed.execute_command("2d").unwrap();
        assert_eq!(ed.line_count(), 2);

        // Test substitute
        ed.execute_command("1s/line/LINE/").unwrap();
        match ed.execute_command("1p").unwrap() {
            EdResult::Lines(lines) => assert_eq!(lines, vec!["LINE1"]),
            _ => panic!("Expected Lines result"),
        }
    }

    #[test]
    fn test_range_operations() {
        let mut ed = EdBuffer::from_string("a\nb\nc\nd\ne");
        
        // Test range print
        match ed.execute_command("2,4p").unwrap() {
            EdResult::Lines(lines) => assert_eq!(lines, vec!["b", "c", "d"]),
            _ => panic!("Expected Lines result"),
        }

        // Test % (all lines)
        match ed.execute_command("%p").unwrap() {
            EdResult::Lines(lines) => assert_eq!(lines.len(), 5),
            _ => panic!("Expected Lines result"),
        }
    }
}