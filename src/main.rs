use cargo_metadata::{CargoOpt, MetadataCommand, TargetKind};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "rust-vscode-workspace-configurator")]
#[command(about = "Generate VS Code multi-root workspace configurations for all discovered Rust projects")]
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
    required_features: Vec<String>,
    project_path: PathBuf,
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

/// Generates VS Code multi-root workspace configurations with launch configurations for all discovered Rust projects.
///
/// This function parses command-line arguments, recursively discovers all Rust projects in the specified 
/// directory tree, and creates a comprehensive workspace.code-workspace file with launch configurations 
/// for all binaries and examples found across all projects.
///
/// # Usage
///
/// rust-vscode-workspace-configurator [--root <ROOT>]
///
/// - `--root`: Root directory to search for Rust projects recursively (defaults to current directory)
///
/// # Behavior
///
/// - If the root directory contains a Cargo.toml, processes that project directly
/// - Otherwise, recursively searches subdirectories for all Rust projects
/// - Creates a multi-root workspace with separate folders for each discovered project
/// - Generates namespaced launch configurations to avoid conflicts between projects
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
    
    let launch_config = generate_workspace_launch_config(&runnables, &root_dir);
    write_workspace_launch_config(&output_dir, &launch_config, &runnables, &root_dir)?;
    
    println!("Created workspace.code-workspace with launch configurations in {}", output_dir.display());
    
    Ok(())
}

fn discover_runnables(root_dir: &Path) -> Result<Vec<Runnable>, Box<dyn std::error::Error>> {
    let mut runnables = Vec::new();
    let mut found_projects = Vec::new();

    // First try to see if the root directory itself is a Rust project
    let manifest_path = root_dir.join("Cargo.toml");
    if manifest_path.exists() {
        found_projects.push(root_dir.to_path_buf());
    } else {
        // Search for Rust projects in subdirectories
        find_rust_projects_recursive(root_dir, &mut found_projects)?;
        
        if found_projects.is_empty() {
            return Err(format!("No Rust projects (Cargo.toml files) found in {}", root_dir.display()).into());
        }
    }

    println!("Found {} Rust project(s):", found_projects.len());
    for project_path in &found_projects {
        println!("  {}", project_path.display());
    }

    // Process each found project
    for project_path in found_projects {
        let manifest_path = project_path.join("Cargo.toml");
        
        // Get metadata for the workspace or single package
        let metadata = match MetadataCommand::new()
            .manifest_path(&manifest_path)
            .features(CargoOpt::AllFeatures)
            .exec() {
                Ok(metadata) => metadata,
                Err(e) => {
                    eprintln!("Warning: Failed to read metadata for {}: {}", manifest_path.display(), e);
                    continue;
                }
            };

        // Find the package corresponding to this manifest
        let root_package = match metadata.packages.iter().find(|p| p.manifest_path == manifest_path) {
            Some(package) => package,
            None => {
                eprintln!("Warning: Could not find package for manifest {}", manifest_path.display());
                continue;
            }
        };

        // Process targets for this package
        for target in &root_package.targets {
            if target.kind.contains(&TargetKind::Bin) {
                runnables.push(Runnable {
                    name: format!("{}::{}", root_package.name, target.name),
                    package: root_package.name.to_string(),
                    runnable_type: RunnableType::Binary,
                    required_features: target.required_features.clone(),
                    project_path: project_path.clone(),
                });
            }

            // Add example targets
            if target.kind.contains(&TargetKind::Example) {
                runnables.push(Runnable {
                    name: format!("{}::{} (example)", root_package.name, target.name),
                    package: root_package.name.to_string(),
                    runnable_type: RunnableType::Example,
                    required_features: target.required_features.clone(),
                    project_path: project_path.clone(),
                });
            }
        }
    }

    Ok(runnables)
}

fn find_rust_projects_recursive(dir: &Path, projects: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    if !dir.is_dir() {
        return Ok(());
    }

    // Check if this directory contains a Cargo.toml
    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        projects.push(dir.to_path_buf());
        // Don't recurse into subdirectories of a Rust project to avoid nested projects
        return Ok(());
    }

    // Recursively search subdirectories
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()), // Skip directories we can't read
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            // Skip common directories that are unlikely to contain Rust projects
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }
            }
            
            find_rust_projects_recursive(&path, projects)?;
        }
    }

    Ok(())
}

fn generate_launch_config(runnables: &[Runnable], root_dir: &Path) -> LaunchConfig {
    let mut configurations = Vec::new();
    
    for runnable in runnables {
        // Calculate relative path from root to project
        let relative_path = match pathdiff::diff_paths(&runnable.project_path, root_dir) {
            Some(path) => path,
            None => runnable.project_path.clone(),
        };
        
        let cwd = if relative_path == Path::new("") || relative_path == Path::new(".") {
            "${workspaceFolder}".to_string()
        } else {
            format!("${{workspaceFolder}}/{}", relative_path.display())
        };
        
        let config = match runnable.runnable_type {
            RunnableType::Binary => {
                // Extract the actual binary name from the prefixed name
                let binary_name = runnable.name.split("::").last().unwrap_or(&runnable.name);
                Configuration {
                    name: format!("Debug binary '{}'", runnable.name),
                    config_type: "lldb".to_string(),
                    request: "launch".to_string(),
                    cwd: cwd.clone(),
                    env: EnvVars {
                        bevy_asset_root: cwd.clone(),
                    },
                    cargo: CargoConfig {
                        args: {
                            let mut args = if binary_name == "main" || binary_name == runnable.package {
                                vec!["run".to_string(), format!("--package={}", runnable.package)]
                            } else {
                                vec![
                                    "run".to_string(),
                                    format!("--bin={}", binary_name),
                                    format!("--package={}", runnable.package),
                                ]
                            };

                            if !runnable.required_features.is_empty() {
                                let feats = runnable.required_features.join(",");
                                args.push(format!("--features={}", feats));
                            }

                            args
                        },
                    },
                    args: vec![],
                }
            },
            RunnableType::Example => {
                // Extract the actual example name from the prefixed name
                let example_name = runnable.name.split("::").nth(1)
                    .and_then(|s| s.strip_suffix(" (example)"))
                    .unwrap_or(&runnable.name);
                Configuration {
                    name: format!("Debug example '{}'", runnable.name),
                    config_type: "lldb".to_string(),
                    request: "launch".to_string(),
                    cwd: cwd.clone(),
                    env: EnvVars {
                        bevy_asset_root: cwd.clone(),
                    },
                    cargo: CargoConfig {
                        args: {
                            let mut args = vec![
                                "run".to_string(),
                                format!("--example={}", example_name),
                                format!("--package={}", runnable.package),
                            ];

                            if !runnable.required_features.is_empty() {
                                let feats = runnable.required_features.join(",");
                                args.push(format!("--features={}", feats));
                            }

                            args
                        },
                    },
                    args: vec![],
                }
            },
        };
        
        configurations.push(config);
    }
    
    LaunchConfig {
        version: "0.2.0".to_string(),
        configurations,
    }
}

fn generate_workspace_launch_config(runnables: &[Runnable], root_dir: &Path) -> WorkspaceLaunchConfig {
    let configurations = generate_launch_config(runnables, root_dir).configurations;
    
    WorkspaceLaunchConfig {
        version: "0.2.0".to_string(),
        configurations,
    }
}

fn write_workspace_launch_config(output_dir: &Path, launch_config: &WorkspaceLaunchConfig, runnables: &[Runnable], root_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let workspace_path = output_dir.join("workspace.code-workspace");
    
    let mut workspace_file = if workspace_path.exists() {
        // Create backup of existing workspace file
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
        
        fs::copy(&workspace_path, &backup_path)?;
        println!("Backed up existing workspace.code-workspace to {}", backup_path.display());
        
        // Read existing workspace file
        let content = fs::read_to_string(&workspace_path)?;
        match serde_json::from_str(&content) {
            Ok(workspace) => workspace,
            Err(e) => {
                eprintln!("Warning: Failed to parse existing workspace.code-workspace: {}", e);
                eprintln!("Creating a new workspace file instead.");
                // Create new workspace file with basic structure
                WorkspaceFile {
                    folders: vec![],
                    settings: None,
                    launch: None,
                    tasks: None,
                    extensions: Some(serde_json::Value::Object(serde_json::Map::new())),
                }
            }
        }
    } else {
        // Create new workspace file with basic structure
        WorkspaceFile {
            folders: vec![],
            settings: None,
            launch: None,
            tasks: None,
            extensions: Some(serde_json::Value::Object(serde_json::Map::new())),
        }
    };
    
    // Collect unique project paths
    let mut project_paths: Vec<PathBuf> = runnables.iter()
        .map(|r| r.project_path.clone())
        .collect();
    project_paths.sort();
    project_paths.dedup();
    
    // Create folders for all discovered projects
    let mut folders = Vec::new();
    for project_path in project_paths {
        let relative_path = match pathdiff::diff_paths(&project_path, root_dir) {
            Some(path) if path != Path::new("") && path != Path::new(".") => format!("./{}", path.display()),
            _ => ".".to_string(),
        };
        
        folders.push(WorkspaceFolder {
            path: relative_path,
        });
    }
    
    // If no projects found or only root project, add current directory
    if folders.is_empty() {
        folders.push(WorkspaceFolder {
            path: ".".to_string(),
        });
    }
    
    workspace_file.folders = folders;
    
    // Update the launch section
    workspace_file.launch = Some((*launch_config).clone());
    
    // Write back to file
    let json_content = serde_json::to_string_pretty(&workspace_file)?;
    fs::write(workspace_path, json_content)?;
    
    Ok(())
}
