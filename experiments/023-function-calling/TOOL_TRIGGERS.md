# Tool Calling Trigger Prompts

## read_file
Triggers when user wants to see file contents:
- "Read the README.md file"
- "What's in the Cargo.toml file?"
- "Show me the contents of src/main.rs"
- "Can you check what's in the .env.example file?"
- "I need to see the configuration in config/settings.json"
- "What are the target dependencies of Makefile?"
- "Show me the targets in the Makefile"

## write_file
Triggers when user wants to create/update files:
- "Create a new file called test.txt with the content 'Hello World'"
- "Write a Python script hello.py that prints 'Hello, World!'"
- "Save the following JSON to data.json: {\"name\": \"test\", \"value\": 42}"
- "Update the README.md file to include '# My Project'"
- "Create a .gitignore file with node_modules and .env"

## list_files
Triggers when user wants to see what files exist:
- "List all files in the current directory"
- "Show me all Python files"
- "What Rust files are in the src directory?"
- "Find all markdown files recursively"
- "Show me all test files in the project"

## search_code
Triggers when user wants to find text/patterns in code:
- "Search for 'TODO' in the codebase"
- "Find all occurrences of 'function_call' in Rust files"
- "Look for any async functions in the code"
- "Find where 'ApiLogger' is used"
- "Search for 'Result<' in the source files"

## Key Patterns
The Gemini API should detect these patterns and return a function call instead of trying to answer directly:
- File operations: read, show, display, check, create, write, save, update
- Listing: list, find, show all, what files
- Searching: search, find, look for, occurrences, where is