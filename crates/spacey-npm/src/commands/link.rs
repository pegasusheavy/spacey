//! Link command implementation.

use owo_colors::OwoColorize;
use crate::cli::{Cli, LinkArgs};
use crate::error::Result;

pub async fn run(_args: &LinkArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Link command not yet implemented".yellow());
    Ok(())
}
