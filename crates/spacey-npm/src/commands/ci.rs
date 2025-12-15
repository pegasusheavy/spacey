//! CI command implementation.

use std::path::PathBuf;
use owo_colors::OwoColorize;
use crate::cli::{Cli, CiArgs, InstallArgs};
use crate::error::{Result, SnpmError};

pub async fn run(args: &CiArgs, cli: &Cli) -> Result<()> {
    // Check for package-lock.json
    if !PathBuf::from("package-lock.json").exists() {
        return Err(SnpmError::InvalidLockfile(
            "package-lock.json not found. CI requires a lockfile.".into()
        ));
    }

    // Remove existing node_modules
    let node_modules = PathBuf::from("node_modules");
    if node_modules.exists() {
        if !cli.quiet {
            println!("{}", "Removing existing node_modules...".dimmed());
        }
        std::fs::remove_dir_all(&node_modules)?;
    }

    // Run install with strict lockfile
    let install_args = InstallArgs {
        ignore_scripts: args.ignore_scripts,
        production: args.production,
        no_package_lock: true, // Don't update lockfile
        ..Default::default()
    };

    super::install::run(&install_args, cli).await
}
