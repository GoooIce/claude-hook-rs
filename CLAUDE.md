# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Claude Hook Advisor** is a Rust CLI tool (v0.2.0) that provides intelligent command suggestions and semantic directory aliasing for Claude Code through a triple-hook architecture. The tool integrates seamlessly with Claude Code to enhance AI-assisted development workflows.

## Development Commands

### Building and Installation
```bash
# Build in debug mode
make build
# or: cargo build

# Build in release mode
make release
# or: cargo build --release

# Install globally via cargo
make install
# or: cargo install --path .

# Install to local bin directory
make install-local

# System-wide installation (requires sudo)
make install-system
```

### Testing and Code Quality
```bash
# Run all tests (single-threaded to avoid race conditions)
make test
# or: cargo test -- --test-threads=1

# Run linting (zero warnings required)
make lint
# or: cargo clippy -- -D warnings

# Format code
make fmt
# or: cargo fmt

# Check code without building
make check
# or: cargo check

# Test with example JSON input
make run-example
```

### Cleaning
```bash
# Clean build artifacts
make clean
# or: cargo clean
```

## Architecture Overview

### Core Module Structure
- **`src/main.rs`**: Program entry point (233 lines)
- **`src/lib.rs`**: Library interface with public API exports
- **`src/cli.rs`**: CLI interface and argument parsing (24KB+)
- **`src/types.rs`**: Core data structures (Config, HookInput, HookOutput, DirectoryResolution)
- **`src/config.rs`**: TOML configuration file management (1.6KB)
- **`src/hooks.rs`**: Triple-hook system implementation (11KB+)
- **`src/directory.rs`**: Semantic directory alias resolution (6.5KB)
- **`src/installer.rs`**: Claude Code hook installation system (22KB+)

### Triple-Hook Architecture
The tool implements three Claude Code hooks:
1. **PreToolUse**: Command interception and intelligent suggestions
2. **UserPromptSubmit**: Semantic directory alias resolution
3. **PostToolUse**: Execution tracking and analysis

### Configuration System
- **Primary config**: `~/.claude-hook-advisor.toml`
- **Project-specific**: `.claude-hook-advisor.toml` in project directories
- **Example config**: `example.claude-hook-advisor.toml` (comprehensive examples)

### Key Dependencies
- `serde` + `serde_json`: JSON serialization for hook communication
- `toml`: Configuration file parsing with order preservation
- `clap`: Command-line argument parsing with derive macros
- `regex`: Pattern matching for command mapping (cached with `once_cell`)
- `anyhow`: Consistent error handling across the codebase
- `which`: Command availability checking

## Development Workflow

### Code Quality Standards
- **Zero linting warnings**: All clippy warnings must be addressed
- **Comprehensive testing**: 24+ tests covering all major functionality
- **Error handling**: Consistent use of `anyhow::Result` throughout
- **Path safety**: All paths use `fs::canonicalize()` to prevent traversal attacks
- **Performance**: Regex patterns cached with `once_cell::Lazy` for ~1ms response times

### Testing Approach
- **Unit tests**: Embedded in each module using `#[cfg(test)]`
- **Integration tests**: JSON input/output validation
- **File system tests**: Use `tempfile` for temporary file testing
- **Single-threaded**: Tests run with `--test-threads=1` to avoid temp directory conflicts

### Adding New Features
1. **Start with tests**: Write failing tests first (TDD approach)
2. **Update types**: Modify `src/types.rs` for new data structures
3. **Implement logic**: Add functionality to appropriate modules
4. **Update CLI**: Modify `src/cli.rs` for new commands/options
5. **Document**: Update README.md and add examples to config file

### Configuration Management
- Command mappings support regex patterns for flexible matching
- Semantic directories use space-separated, quoted aliases for natural language
- Per-project configuration overrides global settings
- Automatic tilde expansion and path canonicalization

## Key Implementation Details

### Hook Communication
All hooks communicate via JSON on stdin/stdout following Claude Code's hook protocol. The `HookInput` and `HookOutput` types handle serialization/deserialization.

### Directory Resolution
The `resolve_directory()` function handles:
- Alias lookup in configuration
- Path tilde expansion (`~` → home directory)
- Canonicalization for security
- Graceful fallback when aliases aren't found

### Command Mapping
Commands are mapped using:
- Exact string matching for simple commands
- Regex patterns for complex command transformations
- Per-project configuration inheritance
- Automatic suggestion of preferred alternatives

### Installation System
The installer handles:
- Automatic detection of Claude Code configuration
- Safe hook installation with backup creation
- Configuration file generation from templates
- Graceful uninstallation with backup restoration