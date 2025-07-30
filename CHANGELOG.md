# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-07-30

### Added
- Full function calling support with gemini-2.0-flash-exp model
- Four core tools: read_file, list_files, write_file, search_code
- System instructions for improved tool usage
- Comprehensive test suite (10 tests, 100% passing)
- Function call detection and formatting

### Changed
- Default model switched from gemini-2.0-flash-lite to gemini-2.0-flash-exp
- Enhanced Part struct to support function calls and responses
- Improved response parsing to detect function calls

### Fixed
- Function calling now works reliably with direct commands
- Proper JSON serialization of function call structures

### Tested
- ✅ Direct file reading: "read the Makefile" → successful function call
- ✅ File listing: "list files in src" → successful function call
- ✅ Code search: "search for TODO" → successful function call
- ✅ Alternative phrasings: "show me README.md" → successful function call
- ✅ Explicit tool usage: "use read_file tool" → successful function call

## [0.1.1] - 2025-07-29

### Added
- Initial function calling infrastructure
- API client improvements
- Basic REPL functionality

## [0.1.0] - 2025-07-28

### Added
- Initial release
- Basic Gemini API integration
- REPL interface
- Proxy support
- Debug logging