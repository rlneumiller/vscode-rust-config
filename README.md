# Rust VS Code Workspace Configurator

A small command-line tool that discovers Rust binaries and examples (for the package whose `Cargo.toml` you point it at) and embeds VS Code launch configurations into a `workspace.code-workspace` file under a `"launch"` section.

## Important notes

- The tool examines the package located at the provided `--root` manifest path (or the current working directory if `--root` is not supplied). It does not scan sibling packages in a multi-package workspace — it only processes the package whose `Cargo.toml` you pass in.
- If a `workspace.code-workspace` file already exists at the output location, the tool makes a backup named `workspace.code-workspace.backup`. If that name is already taken it will append `.1`, `.2`, etc. until an unused name is found (e.g. `workspace.code-workspace.backup.1`).
- Generated launch configurations target the `lldb` debugger and assume you have an LLDB adapter in VS Code (for example, the CodeLLDB extension).
- The generated launch configurations set an environment variable `BEVY_ASSET_ROOT` to `${workspaceFolder}`. This is included because some Rust projects (notably those using Bevy) expect an asset root; you can remove or modify this value in the resulting `workspace.code-workspace` file if it is not applicable to your project.
- If a target (binary or example) declares required Cargo features, the tool appends a `--features=<comma-separated-features>` argument to the cargo invocation in the launch configuration.

## Features

- Discovers `bin` targets and `example` targets for the package at the provided `Cargo.toml` using `cargo_metadata`.
- Generates LLDB-compatible debug configurations that run `cargo run` for the selected target and package.
- Writes (or updates) a `workspace.code-workspace` file and preserves an existing file by creating a numerical backup as described above.

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

Run the tool pointing at the package root (defaults to current directory). You can use the long `--root <PATH>` or the short `-r <PATH>` flag.

```bash
# specifying a root path (long form)
rust-vscode-workspace-configurator --root /path/to/rust/project

# specifying a root path (short form)
rust-vscode-workspace-configurator -r /path/to/rust/project

# or from inside a project directory (uses current directory)
rust-vscode-workspace-configurator
```

The tool will:

1. Require that a `Cargo.toml` exists at the specified root. It returns an error if no manifest (Cargo.toml) is found there.
2. Use `cargo metadata` (requesting all features) to discover `bin` and `example` targets for the single package whose `Cargo.toml` you provided. It does not scan sibling packages in a workspace.
3. Generate a `launch` section compatible with VS Code that invokes `cargo run` with appropriate `--package`, `--bin` or `--example` arguments. If a target declares required features, the tool appends a `--features=<comma-separated-features>` argument.
4. Write or update `workspace.code-workspace` in the specified root, creating a backup of any existing file named `workspace.code-workspace.backup` and adding numeric suffixes (`.1`, `.2`, ...) if needed.

## Example output

When run in a project containing a binary `my_binary` and an example `example1` you might see:

```text
Searching for Rust projects in: /path/to/project
Found 2 runnables:
  my_binary (Binary) in package my_project
  example1 (Example) in package my_project
Backed up existing workspace.code-workspace to workspace.code-workspace.backup
Created workspace.code-workspace with launch configurations in /path/to/project
```

And `workspace.code-workspace` will contain a top-level `launch` object similar to the following.

Here is a short, pretty-printed example `workspace.code-workspace` showing two configurations: one for a binary named `my_binary` and one for an example named `example1`. The binary declares a required feature `cool-feature`, which is included in `--features`.

```json
{
  "folders": [ { "path": "." } ],
  "launch": {
    "version": "0.2.0",
    "configurations": [
      {
        "name": "Debug binary 'my_binary'",
        "type": "lldb",
        "request": "launch",
        "cwd": "${workspaceFolder}",
        "env": { "BEVY_ASSET_ROOT": "${workspaceFolder}" },
        "cargo": {
          "args": [
            "run",
            "--bin=my_binary",
            "--package=my_project",
            "--features=cool-feature"
          ]
        },
        "args": []
      },
      {
        "name": "Debug example 'example1'",
        "type": "lldb",
        "request": "launch",
        "cwd": "${workspaceFolder}",
        "env": { "BEVY_ASSET_ROOT": "${workspaceFolder}" },
        "cargo": {
          "args": [
            "run",
            "--example=example1",
            "--package=my_project"
          ]
        },
        "args": []
      }
    ]
  }
}
```

Notes:

- Generated configurations are named `Debug binary '<name>'` or `Debug example '<name>'`.
- Each configuration sets `type` to `lldb`, `request` to `launch`, `cwd` to `${workspaceFolder}`, and includes `env.BEVY_ASSET_ROOT` = `${workspaceFolder}` by default.
- The `cargo.args` array contains the `cargo run` subcommand and flags; `--features` is added when targets declare required features.

## Dependencies

- `serde` and `serde_json` for JSON handling
- `clap` for command-line parsing
- `cargo_metadata` for reading Cargo metadata

## License

MIT or Apache-2.0 (your choice)
