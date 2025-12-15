//! Version command implementation.

use owo_colors::OwoColorize;
use crate::cli::{Cli, VersionArgs};
use crate::error::Result;

pub async fn run(_args: &VersionArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Version command not yet implemented".yellow());
    Ok(())
}
