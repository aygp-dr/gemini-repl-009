# RFC: Ed-Based Tool System for File Manipulation

## Abstract

This RFC proposes an experimental file manipulation tool system based entirely on the semantics of the `ed` line editor. By leveraging ed's minimalist, line-oriented approach, we can create a robust and predictable file editing interface that operates through well-defined, atomic operations.

## Background

Ed is a line-oriented text editor that has been part of Unix systems since 1969. Its design philosophy emphasizes:
- Line-based operations
- Minimal interface with maximum power
- Stateful editing with an implicit current line
- Pattern-based addressing
- Atomic, composable commands

## Analysis of Ed's Core Semantics

Based on analysis of FreeBSD's ed implementation (`/usr/src/bin/ed/`), the core components are:

### 1. Line Buffer Structure
```c
typedef struct line {
    struct line *q_forw;  // Forward pointer in circular queue
    struct line *q_back;  // Backward pointer
    off_t seek;          // Address in scratch buffer
    int len;             // Length of line
} line_t;
```

### 2. Addressing Modes
- Absolute: `1`, `5`, `$` (last line)
- Relative: `+3`, `-2`, `^` (previous line)
- Current: `.` (current line)
- Pattern: `/regex/`, `?regex?`
- Range: `1,5`, `%` (all lines), `,` (current to last)

### 3. Core Commands
- **a** (append): Insert text after specified line
- **i** (insert): Insert text before specified line
- **c** (change): Replace lines with new text
- **d** (delete): Remove lines
- **s** (substitute): Replace pattern within lines
- **m** (move): Move lines to new location
- **j** (join): Join lines together
- **w** (write): Save to file
- **r** (read): Read from file

### 4. Command Structure
Commands follow the pattern: `[address[,address]]command[parameters]`

## Proposed Design

### EdTool Trait
```rust
pub trait EdTool {
    /// Execute an ed command and return the result
    fn execute_ed_command(&mut self, command: &str) -> Result<EdResult>;
    
    /// Get current line number
    fn current_line(&self) -> usize;
    
    /// Get total number of lines
    fn line_count(&self) -> usize;
}
```

### EdResult Type
```rust
pub enum EdResult {
    Success,
    Lines(Vec<String>),  // For print commands
    Written(usize),      // Bytes written
    Error(String),
}
```

### Core Operations as Ed Commands

#### 1. Append Operation
```rust
pub fn append_lines(ed: &mut impl EdTool, after_line: usize, lines: &[String]) -> Result<()> {
    // Move to target line
    ed.execute_ed_command(&format!("{}", after_line))?;
    
    // Enter append mode
    ed.execute_ed_command("a")?;
    
    // Add lines
    for line in lines {
        ed.execute_ed_command(line)?;
    }
    
    // Exit append mode
    ed.execute_ed_command(".")?;
    
    Ok(())
}
```

#### 2. Replace Operation
```rust
pub fn replace_text(ed: &mut impl EdTool, pattern: &str, replacement: &str) -> Result<()> {
    // Global substitution
    let cmd = format!("%s/{}/{}/g", pattern, replacement);
    ed.execute_ed_command(&cmd)?;
    Ok(())
}
```

#### 3. Delete Operation
```rust
pub fn delete_lines(ed: &mut impl EdTool, start: usize, end: usize) -> Result<()> {
    let cmd = format!("{},{}d", start, end);
    ed.execute_ed_command(&cmd)?;
    Ok(())
}
```

## Implementation Approach

### 1. Pure Ed Backend
Use actual ed process for maximum compatibility:

```rust
pub struct EdProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl EdProcess {
    pub fn new() -> Result<Self> {
        let mut child = Command::new("/bin/ed")
            .arg("-s")  // Suppress diagnostics
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
            
        // ... initialization ...
    }
}
```

### 2. In-Memory Ed Emulation
Implement ed semantics in pure Rust:

```rust
pub struct EdBuffer {
    lines: Vec<String>,
    current: usize,
    modified: bool,
    marks: HashMap<char, usize>,
}
```

## Benefits

1. **Atomicity**: Each ed command is atomic and well-defined
2. **Simplicity**: No complex state management beyond current line
3. **Robustness**: Ed has decades of battle-testing
4. **Portability**: Ed is available on all Unix systems
5. **Scriptability**: Commands can be easily batched and replayed
6. **Undo/Redo**: Ed's undo stack provides natural versioning

## Limitations

1. **Line-oriented**: Not suitable for byte-level operations
2. **Performance**: Process overhead for pure ed backend
3. **Binary files**: Ed is designed for text files
4. **Large files**: Ed loads entire file into memory
5. **Concurrent access**: No built-in support for multiple editors

## Example Usage

### Creating a New File
```rust
let mut ed = EdBuffer::new();
ed.execute_ed_command("a")?;
ed.execute_ed_command("Hello, World!")?;
ed.execute_ed_command("This is line 2")?;
ed.execute_ed_command(".")?;
ed.execute_ed_command("w hello.txt")?;
```

### Modifying an Existing File
```rust
let mut ed = EdBuffer::from_file("config.toml")?;
ed.execute_ed_command("/version = /")?;  // Find version line
ed.execute_ed_command("s/\".*\"/\"2.0.0\"/")?;  // Update version
ed.execute_ed_command("w")?;  // Save changes
```

### Complex Multi-line Operation
```rust
// Insert header at beginning of file
ed.execute_ed_command("0a")?;
ed.execute_ed_command("# Generated by Gemini REPL")?;
ed.execute_ed_command("# Date: 2025-08-01")?;
ed.execute_ed_command("")?;  // Empty line
ed.execute_ed_command(".")?;

// Append footer
ed.execute_ed_command("$a")?;
ed.execute_ed_command("")?;
ed.execute_ed_command("# End of file")?;
ed.execute_ed_command(".")?;
```

## Integration with Existing Tools

The ed-based tools can coexist with current file tools:

```rust
impl Tool for EdFileTool {
    fn name(&self) -> &str {
        "ed_file"
    }
    
    fn description(&self) -> &str {
        "Edit files using ed semantics"
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let cmd = params["command"].as_str().unwrap();
        let result = self.ed_buffer.execute_ed_command(cmd)?;
        
        Ok(json!({
            "success": true,
            "result": result,
            "current_line": self.ed_buffer.current_line(),
            "total_lines": self.ed_buffer.line_count(),
        }))
    }
}
```

## Security Considerations

1. **Command injection**: Must sanitize patterns and filenames
2. **Path traversal**: Validate all file paths
3. **Resource limits**: Set maximum file size and line count
4. **Shell escapes**: Ed's `!` command must be disabled or sandboxed

## Future Enhancements

1. **Batch operations**: Execute multiple commands atomically
2. **Transaction support**: Rollback on error
3. **Streaming mode**: Handle large files without full load
4. **Collaborative editing**: Multiple ed instances with conflict resolution
5. **Ed script generation**: Convert high-level operations to ed scripts

## Conclusion

By embracing ed's minimalist philosophy, we can create a file manipulation system that is both powerful and predictable. The line-oriented approach, while limiting in some ways, provides a solid foundation for reliable text file operations. This experimental approach could offer insights into alternative ways of thinking about file manipulation in modern systems.

The ed-based tool system would serve as a research platform for exploring:
- Minimalist interface design
- Line-oriented vs. character-oriented editing
- The relationship between constraints and robustness
- Alternative approaches to file manipulation APIs

This RFC proposes implementing this system as an experimental feature alongside existing tools, allowing for comparison and evaluation of both approaches.