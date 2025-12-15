// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! Node.js module system implementation
//!
//! Implements CommonJS `require()` and module resolution.

mod cache;
mod loader;
mod require;
mod resolver;

pub use cache::ModuleCache;
pub use loader::ModuleLoader;
pub use require::require;
pub use resolver::{ModuleResolver, ResolveResult};

