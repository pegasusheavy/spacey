// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! Spacey - A JavaScript engine inspired by SpiderMonkey, written in Rust
//!
//! This is the main entry point for the spacey CLI/REPL.

use spacey_spidermonkey::Engine;

fn main() {
    println!("Spacey JavaScript Engine v{}", env!("CARGO_PKG_VERSION"));
    println!("Type JavaScript code to evaluate, or 'exit' to quit.\n");

    let mut engine = Engine::new();

    // Simple REPL loop (placeholder)
    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush().unwrap();

        input.clear();
        if stdin.read_line(&mut input).is_err() {
            break;
        }

        let trimmed = input.trim();
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        if trimmed.is_empty() {
            continue;
        }

        match engine.eval(trimmed) {
            Ok(value) => println!("{}", value),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    println!("Goodbye!");
}
