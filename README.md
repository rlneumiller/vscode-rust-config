# Rust VS Code Workspace Configurator

A command-line tool to generate VS Code launch configurations for Rust projects by embedding them directly into `workspace.code-workspace` files instead of separate `.vscode/launch.json` file.

## WARNING

- Be aware the this is a tool that I use to configure my workspaces to suit my personal preferences.
- It should work fine as is but you may want to edit to suit your specific desires.
- If you have a pre- existing `.code-workspace` file this tool will rename it by adding a `.backup` extension to it. You can then diff it with the new `workspace.code-workspace` file that this tool creates and transfer over your customizations as desired.



## Features

- Discovers binaries and examples in Rust projects using Cargo metadata
- Generates debug configurations for LLDB (assumes you have installed the CodeLLDB extension)
- Create `workspace.code-workspace` file and renames existing `.code-workspace` files to `<workspace_name>.code-workspace.backup`

## Installation

I prefer not to offer precompiled releases and to encourage you to review the code first and then build it yourself. [see](https://www.cve.org/)

Clone the repository and build with Cargo:

```bash
git clone https://github.com/rlneumiller/rust-vscode-workspace-configurator.git
cd rust-vscode-workspace-configurator
cargo build --release
```

Or if you are feeling brave:

```bash
git clone https://github.com/rlneumiller/rust-vscode-workspace-configurator.git
cd rust-vscode-workspace-configurator
cargo install --path .
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
3. Create the `workspace.code-workspace` file

## Example

```bash
# In a Rust project directory
cargo run

# Output:
# Searching for Rust projects in: /current/directory
# Found 2 runnables:
#   my_binary (Binary) in package my_project
#   example1 (Example) in package my_project
# Created workspace.code-workspace with launch configurations in /current/directory
```

## Dependencies

- `serde` and `serde_json` for JSON handling
- `clap` for command-line argument parsing
- `cargo_metadata` for accessing Cargo project metadata

## License

MIT or Apache 2.0 - your choice
