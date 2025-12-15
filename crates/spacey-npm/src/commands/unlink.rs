//! Unlink command implementation.

use owo_colors::OwoColorize;
use crate::cli::{Cli, UnlinkArgs};
use crate::error::Result;

pub async fn run(_args: &UnlinkArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Unlink command not yet implemented".yellow());
    Ok(())
}
