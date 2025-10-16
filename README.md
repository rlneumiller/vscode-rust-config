# Rust VS Code Workspace Configurator

A command-line tool to generate VS Code launch configurations for Rust projects by embedding them directly into `workspace.code-workspace` files instead of separate `launch.json` files.

## Features

- Discovers binaries and examples in Rust projects using Cargo metadata
- Generates debug configurations for LLDB
- Updates existing `workspace.code-workspace` files or creates new ones
- Supports workspace-wide settings and tasks

## Installation

Clone the repository and build with Cargo:

```bash
git clone <repository-url>
cd rust-vscode-workspace-configurator
cargo build --release
```

## Usage

Run the tool in your Rust project directory:

```bash
cargo run -- --root /path/to/rust/project
```

If no `--root` is specified, it defaults to the current directory.

The tool will:
1. Search for Rust runnables (binaries and examples)
2. Generate launch configurations
3. Update or create the `workspace.code-workspace` file

## Example

```bash
# In a Rust project directory
cargo run

# Output:
# Searching for Rust projects in: /current/directory
# Found 2 runnables:
#   my_binary (Binary) in package my_project
#   example1 (Example) in package my_project
# Updated workspace.code-workspace with launch configurations in /current/directory
```

## Dependencies

- `serde` and `serde_json` for JSON handling
- `clap` for command-line argument parsing
- `cargo_metadata` for accessing Cargo project metadata

## License

[Specify your license here]
