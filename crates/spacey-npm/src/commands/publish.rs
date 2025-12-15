//! Publish command implementation.

use owo_colors::OwoColorize;
use crate::cli::{Cli, PublishArgs};
use crate::error::Result;

pub async fn run(_args: &PublishArgs, cli: &Cli) -> Result<()> {
    println!("{}", "Publish command not yet implemented".yellow());
    Ok(())
}

