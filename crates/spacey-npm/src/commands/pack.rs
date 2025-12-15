//! Pack command implementation.

use owo_colors::OwoColorize;
use crate::cli::{Cli, PackArgs};
use crate::error::Result;

pub async fn run(_args: &PackArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Pack command not yet implemented".yellow());
    Ok(())
}
