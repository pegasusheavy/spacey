//! Audit command implementation.

use owo_colors::OwoColorize;
use crate::cli::{Cli, AuditArgs};
use crate::error::Result;

pub async fn run(_args: &AuditArgs, _cli: &Cli) -> Result<()> {
    println!("{}", "Audit command not yet implemented".yellow());
    Ok(())
}
