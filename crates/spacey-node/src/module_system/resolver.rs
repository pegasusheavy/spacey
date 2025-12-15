// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// Copyright (c) 2025 Pegasus Heavy Industries, LLC

//! Module path resolution (Node.js algorithm)

use crate::error::{NodeError, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Result of module resolution
#[derive(Debug, Clone)]
pub enum ResolveResult {
    /// Built-in module (fs, path, http, etc.)
    BuiltIn(String),
    /// File module (resolved path)
    File(PathBuf),
    /// JSON file
    Json(PathBuf),
    /// Native addon (.node file)
    Native(PathBuf),
}

/// Module resolver implementing Node.js resolution algorithm
pub struct ModuleResolver {
    /// Built-in module names
    builtins: Vec<String>,
    /// File extensions to try
    extensions: Vec<String>,
}

impl ModuleResolver {
    /// Create a new module resolver
    pub fn new() -> Self {
        Self {
            builtins: vec![
                "assert".to_string(),
                "buffer".to_string(),
                "child_process".to_string(),
                "cluster".to_string(),
                "console".to_string(),
                "constants".to_string(),
                "crypto".to_string(),
                "dgram".to_string(),
                "dns".to_string(),
                "domain".to_string(),
                "events".to_string(),
                "fs".to_string(),
                "http".to_string(),
                "https".to_string(),
                "module".to_string(),
                "net".to_string(),
                "os".to_string(),
                "path".to_string(),
                "perf_hooks".to_string(),
                "process".to_string(),
                "punycode".to_string(),
                "querystring".to_string(),
                "readline".to_string(),
                "repl".to_string(),
                "stream".to_string(),
                "string_decoder".to_string(),
                "sys".to_string(),
                "timers".to_string(),
                "tls".to_string(),
                "tty".to_string(),
                "url".to_string(),
                "util".to_string(),
                "v8".to_string(),
                "vm".to_string(),
                "worker_threads".to_string(),
                "zlib".to_string(),
            ],
            extensions: vec![
                ".js".to_string(),
                ".json".to_string(),
                ".node".to_string(),
            ],
        }
    }

    /// Check if a module is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        // Handle node: prefix
        let name = name.strip_prefix("node:").unwrap_or(name);
        self.builtins.contains(&name.to_string())
    }

    /// Resolve a module specifier
    pub fn resolve(&self, specifier: &str, parent_path: &Path) -> Result<ResolveResult> {
        // Handle node: prefix for built-ins
        let specifier = specifier.strip_prefix("node:").unwrap_or(specifier);

        // Check if built-in module
        if self.is_builtin(specifier) {
            return Ok(ResolveResult::BuiltIn(specifier.to_string()));
        }

        // Check if relative or absolute path
        if specifier.starts_with("./")
            || specifier.starts_with("../")
            || specifier.starts_with('/')
            || (cfg!(windows) && specifier.chars().nth(1) == Some(':'))
        {
            return self.resolve_file(specifier, parent_path);
        }

        // Otherwise, resolve as node_modules package
        self.resolve_node_modules(specifier, parent_path)
    }

    /// Resolve a file path
    fn resolve_file(&self, specifier: &str, parent_path: &Path) -> Result<ResolveResult> {
        let parent_dir = parent_path.parent().unwrap_or(Path::new("."));
        let path = parent_dir.join(specifier);

        // Try exact path first
        if path.is_file() {
            return self.categorize_file(&path);
        }

        // Try with extensions
        for ext in &self.extensions {
            let with_ext = path.with_extension(ext.trim_start_matches('.'));
            if with_ext.is_file() {
                return self.categorize_file(&with_ext);
            }

            // Also try appending extension to full path
            let mut full_path = path.clone();
            let mut filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            filename.push_str(ext);
            full_path.set_file_name(&filename);
            if full_path.is_file() {
                return self.categorize_file(&full_path);
            }
        }

        // Try as directory with index file
        if path.is_dir() {
            return self.resolve_directory(&path);
        }

        Err(NodeError::ModuleNotFound(specifier.to_string()))
    }

    /// Resolve a directory (look for package.json main or index.js)
    fn resolve_directory(&self, dir: &Path) -> Result<ResolveResult> {
        // Check for package.json
        let package_json_path = dir.join("package.json");
        if package_json_path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&package_json_path) {
                if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                    // Try "main" field
                    if let Some(main) = pkg.main {
                        let main_path = dir.join(&main);
                        if main_path.is_file() {
                            return self.categorize_file(&main_path);
                        }
                        // Try with extensions
                        for ext in &self.extensions {
                            let with_ext = main_path.with_extension(ext.trim_start_matches('.'));
                            if with_ext.is_file() {
                                return self.categorize_file(&with_ext);
                            }
                        }
                    }
                }
            }
        }

        // Try index files
        for ext in &self.extensions {
            let index = dir.join(format!("index{}", ext));
            if index.is_file() {
                return self.categorize_file(&index);
            }
        }

        Err(NodeError::ModuleNotFound(dir.display().to_string()))
    }

    /// Resolve a module from node_modules
    fn resolve_node_modules(&self, specifier: &str, parent_path: &Path) -> Result<ResolveResult> {
        // Split package name and subpath
        let (package_name, subpath) = self.parse_package_specifier(specifier);

        // Walk up directory tree looking for node_modules
        let mut current = parent_path.parent();
        while let Some(dir) = current {
            let node_modules = dir.join("node_modules").join(package_name);
            
            if node_modules.exists() {
                if let Some(sub) = subpath {
                    // Resolve subpath within package
                    let subpath_full = node_modules.join(sub);
                    if subpath_full.is_file() {
                        return self.categorize_file(&subpath_full);
                    }
                    for ext in &self.extensions {
                        let with_ext = subpath_full.with_extension(ext.trim_start_matches('.'));
                        if with_ext.is_file() {
                            return self.categorize_file(&with_ext);
                        }
                    }
                    if subpath_full.is_dir() {
                        return self.resolve_directory(&subpath_full);
                    }
                } else {
                    // Resolve package main
                    return self.resolve_directory(&node_modules);
                }
            }

            current = dir.parent();
        }

        Err(NodeError::ModuleNotFound(specifier.to_string()))
    }

    /// Parse a package specifier into name and optional subpath
    fn parse_package_specifier<'a>(&self, specifier: &'a str) -> (&'a str, Option<&'a str>) {
        if specifier.starts_with('@') {
            // Scoped package: @scope/name or @scope/name/subpath
            if let Some(slash_pos) = specifier[1..].find('/') {
                let after_scope = &specifier[slash_pos + 2..];
                if let Some(subpath_pos) = after_scope.find('/') {
                    let name_end = slash_pos + 2 + subpath_pos;
                    return (&specifier[..name_end], Some(&specifier[name_end + 1..]));
                }
            }
            (specifier, None)
        } else {
            // Regular package: name or name/subpath
            if let Some(slash_pos) = specifier.find('/') {
                (&specifier[..slash_pos], Some(&specifier[slash_pos + 1..]))
            } else {
                (specifier, None)
            }
        }
    }

    /// Categorize a file by extension
    fn categorize_file(&self, path: &Path) -> Result<ResolveResult> {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Ok(ResolveResult::Json(path)),
            Some("node") => Ok(ResolveResult::Native(path)),
            _ => Ok(ResolveResult::File(path)),
        }
    }
}

impl Default for ModuleResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Minimal package.json structure for resolution
#[derive(Debug, Deserialize)]
struct PackageJson {
    main: Option<String>,
    #[serde(default)]
    exports: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_builtin() {
        let resolver = ModuleResolver::new();
        assert!(resolver.is_builtin("fs"));
        assert!(resolver.is_builtin("path"));
        assert!(resolver.is_builtin("node:fs"));
        assert!(!resolver.is_builtin("lodash"));
    }

    #[test]
    fn test_parse_package_specifier() {
        let resolver = ModuleResolver::new();
        
        assert_eq!(resolver.parse_package_specifier("lodash"), ("lodash", None));
        assert_eq!(
            resolver.parse_package_specifier("lodash/get"),
            ("lodash", Some("get"))
        );
        assert_eq!(
            resolver.parse_package_specifier("@types/node"),
            ("@types/node", None)
        );
        assert_eq!(
            resolver.parse_package_specifier("@babel/core/lib/index"),
            ("@babel/core", Some("lib/index"))
        );
    }
}

