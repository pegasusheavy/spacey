// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! spacey-node CLI - Node.js-compatible JavaScript runtime

use clap::Parser;
use owo_colors::OwoColorize;
use spacey_node::{NodeRuntime, VERSION};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "spacey-node",
    about = "Node.js-compatible JavaScript runtime powered by Spacey",
    version = VERSION,
    author = "Pegasus Heavy Industries"
)]
struct Cli {
    /// JavaScript file to execute
    script: Option<PathBuf>,

    /// Evaluate script from command line
    #[arg(short = 'e', long = "eval")]
    eval: Option<String>,

    /// Start interactive REPL
    #[arg(short = 'i', long = "interactive", alias = "repl")]
    interactive: bool,

    /// Print version and exit
    #[arg(short = 'v', long = "version")]
    show_version: bool,

    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,

    /// Arguments passed to the script
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("spacey_node=debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("spacey_node=warn")
            .init();
    }

    // Handle version flag
    if cli.show_version {
        println!("spacey-node v{}", VERSION);
        println!("Node.js API compatibility: v{}", spacey_node::NODE_API_VERSION);
        return Ok(());
    }

    // Create runtime
    let mut runtime = NodeRuntime::new(cli.args.clone());

    // Determine execution mode
    if let Some(code) = cli.eval {
        // Evaluate inline code
        match runtime.eval(&code).await {
            Ok(result) => {
                if !result.is_undefined() {
                    println!("{}", result);
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
        }
    } else if let Some(script_path) = cli.script {
        // Run script file
        match runtime.run_file(&script_path).await {
            Ok(exit_code) => {
                std::process::exit(exit_code);
            }
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
        }
    } else if cli.interactive || atty::is(atty::Stream::Stdin) {
        // Start REPL
        print_banner();
        runtime.run_repl().await?;
    } else {
        // Read from stdin
        let mut code = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut code)?;
        match runtime.eval(&code).await {
            Ok(result) => {
                if !result.is_undefined() {
                    println!("{}", result);
                }
            }
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

fn print_banner() {
    println!(
        "{} {} (Node.js API v{})",
        "spacey-node".cyan().bold(),
        VERSION.yellow(),
        spacey_node::NODE_API_VERSION.dimmed()
    );
    println!("Type {} for help, {} to exit", ".help".green(), ".exit".green());
    println!();
}

