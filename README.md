# Rust VS Code Workspace Configurator

A command-line tool that recursively discovers Rust projects (including Cargo workspaces) and creates VS Code multi-root workspace configurations with launch configurations for all discovered binaries and examples.

## Important notes

- The tool searches recursively for Rust projects (directories containing `Cargo.toml` files) starting from the provided `--root` directory (or the current working directory if `--root` is not supplied).
- **Supports both individual Rust packages and Cargo workspaces**: If the root directory contains a workspace `Cargo.toml`, it will process all workspace members. If it contains a package `Cargo.toml`, it processes that package directly.
- If the root directory doesn't contain a `Cargo.toml`, it scans subdirectories to find all Rust projects.
- Creates a multi-root VS Code workspace with separate folders for each discovered Rust project.
- **For Cargo workspaces**: Discovers and creates launch configurations for binaries and examples across all workspace members.
- The workspace filename is based on the root directory name (e.g., `my-project.code-workspace` for a directory named `my-project`).
- If a workspace file already exists at the output location, the tool makes a backup with the same base name (e.g., `my-project.code-workspace.backup`). If that name is already taken it will append `.1`, `.2`, etc. until an unused name is found.
- Generated launch configurations target the `lldb` debugger and assume you have an LLDB adapter in VS Code (for example, the CodeLLDB extension).
- The generated launch configurations set an environment variable `BEVY_ASSET_ROOT` to the appropriate project directory. This is included because some Rust projects (notably those using Bevy) expect an asset root; you can remove or modify this value in the resulting `workspace.code-workspace` file if it is not applicable to your projects.
- If a target (binary or example) declares required Cargo features, the tool appends a `--features=<comma-separated-features>` argument to the cargo invocation in the launch configuration.
- Launch configurations are namespaced with the project name to avoid conflicts when multiple projects have targets with the same name.

## Features

- Recursively discovers all Rust projects (directories containing `Cargo.toml` files) in the specified directory tree.
- **Full Cargo workspace support**: Automatically detects workspace manifests and processes all workspace members to discover their binaries and examples.
- Discovers `bin` targets and `example` targets for each found project/package using `cargo_metadata`.
- Generates LLDB-compatible debug configurations that run `cargo run` for each target with appropriate project-relative paths.
- Creates multi-root VS Code workspaces with proper folder structure for all discovered projects.
- Writes (or updates) a `workspace.code-workspace` file and preserves an existing file by creating a numerical backup as described above.
- Handles malformed existing workspace files gracefully by creating backups and starting fresh.

## Installation

Build and run from the repository or install with Cargo:

```bash
# build locally (debug build)
cargo build

# or build a release binary
cargo build --release
```

```bash
# execute the tool from the repository (no special flags required)
# use `--root <PATH>` or the short form `-r <PATH>` to point at a package folder
cargo run -- --root /path/to/rust/project

# run from inside a package directory (uses current directory)
cargo run --
```

```bash
# or install to your cargo bin
cargo install --path .
```

## Usage

Run the tool pointing at a directory containing Rust projects (defaults to current directory). You can use the long `--root <PATH>` or the short `-r <PATH>` flag.

```bash
# specifying a root path (long form) - will search for all Rust projects in this directory
rust-vscode-workspace-configurator --root /path/to/directory/containing/rust/projects

# specifying a root path (short form)
rust-vscode-workspace-configurator -r /path/to/directory/containing/rust/projects

# or from inside a directory (uses current directory)
rust-vscode-workspace-configurator
```

The tool will:

1. Check if the specified root directory contains a `Cargo.toml`.
   - **If it's a workspace manifest**: Processes all workspace members to discover their binaries and examples.
   - **If it's a package manifest**: Processes that package directly.
2. If no `Cargo.toml` is found in the root, it recursively searches subdirectories for Rust projects (directories containing `Cargo.toml` files).
3. Use `cargo metadata` (requesting all features) to discover `bin` and `example` targets for each found project/package.
4. Generate namespaced launch configurations compatible with VS Code that invoke `cargo run` with appropriate `--package`, `--bin` or `--example` arguments. If a target declares required features, the tool appends a `--features=<comma-separated-features>` argument.
5. Create a multi-root workspace configuration with separate folders for each discovered project.
6. Generate a workspace filename based on the root directory name (e.g., `my-projects.code-workspace`).
7. Write or update the workspace file in the specified root, creating a backup of any existing file with the same base name and adding numeric suffixes (`.1`, `.2`, ...) if needed.

## Example output

When run in a directory containing multiple Rust projects, you might see:

```text
Searching for Rust projects in: /path/to/projects
Found 3 Rust project(s):
  /path/to/projects/project1
  /path/to/projects/project2
  /path/to/projects/project3
Found 5 runnables:
  project1::my_binary (Binary) in package project1
  project1::example1 (example) (Example) in package project1
  project2::server (Binary) in package project2
  project3::cli_tool (Binary) in package project3
  project3::integration_test (example) (Example) in package project3
Backed up existing projects.code-workspace to projects.code-workspace.backup.1
Created projects.code-workspace with launch configurations in /path/to/projects
```

When run on a **Cargo workspace**, you might see output like this:

```text
Searching for Rust projects in: /path/to/my-workspace
Found 1 Rust project(s):
  /path/to/my-workspace
Found 18 runnables:
  bevy-text3d::basic (example) (Example) in package bevy-text3d
  bevy-text3d::custom_color (example) (Example) in package bevy-text3d
  bevy_renet_test::client (example) (Example) in package bevy_renet_test
  bevy_renet_test::server (example) (Example) in package bevy_renet_test
  fullscreen_toggle::fullscreen_toggle (Binary) in package fullscreen_toggle
  gabriels-horn::gabriels-horn (Binary) in package gabriels-horn
  render-debug::grid_lines (example) (Example) in package render-debug
  # ... more binaries and examples from workspace members
Created my-workspace.code-workspace with launch configurations in /path/to/my-workspace
```

And `projects.code-workspace` will contain a multi-root workspace configuration with a top-level `launch` object similar to the following.

Here is a short, pretty-printed example `projects.code-workspace` showing a multi-root workspace with three projects and their launch configurations. Note how each project gets its own folder and launch configurations are namespaced:

```json
{
  "folders": [
    { "path": "./project1" },
    { "path": "./project2" },
    { "path": "./project3" }
  ],
  "launch": {
    "version": "0.2.0",
    "configurations": [
      {
        "name": "Debug binary 'project1::my_binary'",
        "type": "lldb",
        "request": "launch",
        "cwd": "${workspaceFolder}/project1",
        "env": { "BEVY_ASSET_ROOT": "${workspaceFolder}/project1" },
        "cargo": {
          "args": [
            "run",
            "--bin=my_binary",
            "--package=project1",
            "--features=cool-feature"
          ]
        },
        "args": []
      },
      {
        "name": "Debug example 'project1::example1 (example)'",
        "type": "lldb",
        "request": "launch",
        "cwd": "${workspaceFolder}/project1",
        "env": { "BEVY_ASSET_ROOT": "${workspaceFolder}/project1" },
        "cargo": {
          "args": [
            "run",
            "--example=example1",
            "--package=project1"
          ]
        },
        "args": []
      },
      {
        "name": "Debug binary 'project2::server'",
        "type": "lldb",
        "request": "launch",
        "cwd": "${workspaceFolder}/project2",
        "env": { "BEVY_ASSET_ROOT": "${workspaceFolder}/project2" },
        "cargo": {
          "args": [
            "run",
            "--bin=server",
            "--package=project2"
          ]
        },
        "args": []
      }
    ]
  }
}
```

Notes:

- Generated configurations are named `Debug binary '<project>::<name>'` or `Debug example '<project>::<name> (example)'` to avoid naming conflicts between projects.
- Each configuration sets `type` to `lldb`, `request` to `launch`, and `cwd` to the appropriate project directory relative to the workspace folder.
- The `env.BEVY_ASSET_ROOT` is set to the project directory to ensure assets are loaded correctly for each project.
- The `cargo.args` array contains the `cargo run` subcommand and flags; `--features` is added when targets declare required features.
- Multi-root workspaces allow you to work with multiple Rust projects simultaneously while maintaining proper project isolation.

## VS Code Workspace Identification

The tool generates workspace files with descriptive names based on the root directory name (e.g., `my-projects.code-workspace`). This naming convention helps VS Code differentiate between workspaces in several ways:

- **Recent Workspaces List**: VS Code displays workspace files by their filename, making it easy to identify which workspace contains which projects.
- **Window Titles**: VS Code uses the workspace filename in window titles, helping you distinguish between multiple VS Code instances.
- **Workspace Settings**: Each workspace maintains its own settings, extensions, and state based on the workspace file path.
- **Global State**: VS Code tracks workspace-specific data (like recently opened files, search history, etc.) using the workspace file path as a unique identifier.

The generated workspace files also include a `name` property that provides a user-friendly name in VS Code's workspace switcher and other UI elements.

## Dependencies

- `serde` and `serde_json` for JSON handling
- `clap` for command-line parsing
- `cargo_metadata` for reading Cargo metadata
- `pathdiff` for calculating relative paths between directories

## License

MIT or Apache-2.0 (your choice)
