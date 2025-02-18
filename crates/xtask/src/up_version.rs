use std::{env::current_dir, fs};

use crate::{Module, UpVersionArgs};

pub fn exec(args: &UpVersionArgs) {
    println!(
        "Bump up [{:?}] version, is break change: {}",
        args.module, args.break_change
    );

    match args.module {
        Module::Sparreal => up_sparreal(break_change),
    }
}

fn up_sparreal(break_change: bool) {
    let mut workspace_root = current_dir().unwrap();
    let old = get_workspace_version(&workspace_root).unwrap();

    println!("Current version: {}", old);
}

fn get_workspace_version(workspace_root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;

    let cargo_toml: Value = toml::from_str(&cargo_toml_content)?;

    if let Some(workspace) = cargo_toml.get("workspace") {
        if let Some(package) = workspace.get("package") {
            if let Some(version) = package.get("version") {
                if let Some(version_str) = version.as_str() {
                    return Ok(version_str.to_string());
                }
            }
        }
    }

    Err("Workspace version not found in Cargo.toml".into())
}
