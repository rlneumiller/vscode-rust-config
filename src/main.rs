use cargo_metadata::{CargoOpt, MetadataCommand};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "vscode-rust-config")]
#[command(about = "Generate VS Code launch.json and workspace.code-workspace for Rust projects")]
struct Args {
    /// Root directory to search for Rust projects (defaults to current directory)
    #[arg(short, long)]
    root: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct Runnable {
    name: String,
    package: String,
    runnable_type: RunnableType,
}

#[derive(Debug, Clone)]
enum RunnableType {
    Binary,
    Example,
}

#[derive(Serialize, Deserialize)]
struct LaunchConfig {
    version: String,
    configurations: Vec<Configuration>,
}

#[derive(Serialize, Deserialize)]
struct Configuration {
    name: String,
    #[serde(rename = "type")]
    config_type: String,
    request: String,
    cwd: String,
    env: EnvVars,
    cargo: CargoConfig,
    args: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct EnvVars {
    #[serde(rename = "BEVY_ASSET_ROOT")]
    bevy_asset_root: String,
}

#[derive(Serialize, Deserialize)]
struct CargoConfig {
    args: Vec<String>,
}

/// Generates VS Code launch.json configurations and workspace.code-workspace for Rust projects.
///
/// This function parses command-line arguments, discovers runnables in the specified root directory,
/// generates a launch.json file in the root directory, and copies a workspace.code-workspace file.
///
/// # Usage
///
/// vscode-rust-config [--root <ROOT>]
///
/// - `--root`: Root directory to search for Rust projects (defaults to current directory)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let root_dir = args.root.unwrap_or_else(|| std::env::current_dir().unwrap());
    let output_dir = root_dir.clone();
    
    println!("Searching for Rust projects in: {}", root_dir.display());
    
    let runnables = discover_runnables(&root_dir)?;
    
    if runnables.is_empty() {
        println!("No runnables found in {}", root_dir.display());
        return Ok(());
    }
    
    println!("Found {} runnables:", runnables.len());
    for runnable in &runnables {
        println!("  {} ({:?}) in package {}", runnable.name, runnable.runnable_type, runnable.package);
    }
    
    let launch_config = generate_launch_config(&runnables);
    write_launch_json(&output_dir, &launch_config)?;
    
    println!("Generated .vscode/launch.json in {}", output_dir.display());
    
    // Copy workspace.code-workspace
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap().parent().unwrap().parent().unwrap();
    let workspace_src = exe_dir.join("workspace.code-workspace");
    if workspace_src.exists() {
        let workspace_dst = output_dir.join("workspace.code-workspace");
        fs::copy(&workspace_src, &workspace_dst)?;
        println!("Copied workspace.code-workspace to {}", output_dir.display());
    } else {
        eprintln!("Warning: Source workspace.code-workspace not found at {}", workspace_src.display());
    }
    
    Ok(())
}

fn discover_runnables(root_dir: &Path) -> Result<Vec<Runnable>, Box<dyn std::error::Error>> {
    let manifest_path = root_dir.join("Cargo.toml");
    if !manifest_path.exists() {
        return Err(format!("No Cargo.toml found in {}", root_dir.display()).into());
    }

    let mut runnables = Vec::new();

    // Get metadata for the workspace or single package
    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .features(CargoOpt::AllFeatures)
        .exec()?;

    // Find the package corresponding to the root manifest
    let root_package = metadata.packages.iter().find(|p| p.manifest_path == manifest_path).ok_or_else(|| {
        format!("Could not find package for manifest {}", manifest_path.display())
    })?;

    // Process only the root package
    for target in &root_package.targets {
        if target.kind.contains(&"bin".to_string()) {
            runnables.push(Runnable {
                name: target.name.clone(),
                package: root_package.name.clone(),
                runnable_type: RunnableType::Binary,
            });
        }

        // Add example targets
        if target.kind.contains(&"example".to_string()) {
            runnables.push(Runnable {
                name: target.name.clone(),
                package: root_package.name.clone(),
                runnable_type: RunnableType::Example,
            });
        }
    }

    Ok(runnables)
}

fn generate_launch_config(runnables: &[Runnable]) -> LaunchConfig {
    let mut configurations = Vec::new();
    
    for runnable in runnables {
        let config = match runnable.runnable_type {
            RunnableType::Binary => Configuration {
                name: format!("Debug binary '{}'", runnable.name),
                config_type: "lldb".to_string(),
                request: "launch".to_string(),
                cwd: "${workspaceFolder}".to_string(),
                env: EnvVars {
                    bevy_asset_root: "${workspaceFolder}".to_string(),
                },
                cargo: CargoConfig {
                    args: if runnable.name == "main" || runnable.name == runnable.package {
                        vec!["run".to_string(), format!("--package={}", runnable.package)]
                    } else {
                        vec![
                            "run".to_string(),
                            format!("--bin={}", runnable.name),
                            format!("--package={}", runnable.package),
                        ]
                    },
                },
                args: vec![],
            },
            RunnableType::Example => Configuration {
                name: format!("Debug example '{}'", runnable.name),
                config_type: "lldb".to_string(),
                request: "launch".to_string(),
                cwd: "${workspaceFolder}".to_string(),
                env: EnvVars {
                    bevy_asset_root: "${workspaceFolder}".to_string(),
                },
                cargo: CargoConfig {
                    args: vec![
                        "run".to_string(),
                        format!("--example={}", runnable.name),
                        format!("--package={}", runnable.package),
                    ],
                },
                args: vec![],
            },
        };
        
        configurations.push(config);
    }
    
    LaunchConfig {
        version: "0.2.0".to_string(),
        configurations,
    }
}

fn write_launch_json(output_dir: &Path, launch_config: &LaunchConfig) -> Result<(), Box<dyn std::error::Error>> {
    let vscode_dir = output_dir.join(".vscode");
    fs::create_dir_all(&vscode_dir)?;
    
    let launch_json_path = vscode_dir.join("launch.json");
    let json_content = serde_json::to_string_pretty(launch_config)?;
    
    fs::write(launch_json_path, json_content)?;
    
    Ok(())
}
