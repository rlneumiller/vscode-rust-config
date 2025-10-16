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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
struct EnvVars {
    #[serde(rename = "BEVY_ASSET_ROOT")]
    bevy_asset_root: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct CargoConfig {
    args: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct WorkspaceLaunchConfig {
    version: String,
    configurations: Vec<Configuration>,
}

#[derive(Serialize, Deserialize)]
struct WorkspaceFile {
    folders: Vec<WorkspaceFolder>,
    settings: Option<serde_json::Value>,
    launch: Option<WorkspaceLaunchConfig>,
    tasks: Option<serde_json::Value>,
    extensions: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct WorkspaceFolder {
    path: String,
}

/// Generates VS Code launch configurations within workspace.code-workspace for Rust projects.
///
/// This function parses command-line arguments, discovers runnables in the specified root directory,
/// and updates the workspace.code-workspace file with launch configurations.
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
    
    let launch_config = generate_workspace_launch_config(&runnables);
    write_workspace_launch_config(&output_dir, &launch_config)?;
    
    println!("Created workspace.code-workspace with launch configurations in {}", output_dir.display());
    
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

fn generate_workspace_launch_config(runnables: &[Runnable]) -> WorkspaceLaunchConfig {
    let configurations = generate_launch_config(runnables).configurations;
    
    WorkspaceLaunchConfig {
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

fn write_workspace_launch_config(output_dir: &Path, launch_config: &WorkspaceLaunchConfig) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_path = output_dir.join("workspace.code-workspace");
    
    if workspace_path.exists() {
        let base_backup_name = "workspace.code-workspace.backup";
        let mut backup_path = output_dir.join(base_backup_name);
        
        if backup_path.exists() {
            let mut counter = 1;
            loop {
                backup_path = output_dir.join(format!("{}.{}", base_backup_name, counter));
                if !backup_path.exists() {
                    break;
                }
                counter += 1;
            }
        }
        
        fs::rename(&workspace_path, &backup_path)?;
        println!("Backed up existing workspace.code-workspace to {}", backup_path.display());
    }
    
    // Read the template workspace file
    let template_path = output_dir.join("workspace.code-workspace.template");
    let template_content = fs::read_to_string(&template_path)
        .map_err(|e| format!("Failed to read template file {}: {}", template_path.display(), e))?;
    let mut workspace_file: WorkspaceFile = serde_json::from_str(&template_content)
        .map_err(|e| format!("Failed to parse template file {}: {}", template_path.display(), e))?;
    
    // Update the launch section
    workspace_file.launch = Some((*launch_config).clone());
    
    // Write to file
    let json_content = serde_json::to_string_pretty(&workspace_file)?;
    fs::write(workspace_path, json_content)?;
    
    Ok(())
}
