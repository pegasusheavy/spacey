//! Package installation.

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

use crate::cache::PackageCache;
use crate::downloader::{DownloadTask, PackageDownloader, TarballExtractor};
use crate::error::{Result, SnpmError};
use crate::lockfile::{LockPackage, PackageLock};
use crate::package::PackageJson;
use crate::resolver::ResolvedPackage;

/// Package installer.
pub struct Installer {
    downloader: PackageDownloader,
    extractor: TarballExtractor,
    node_modules: PathBuf,
    run_scripts: bool,
}

impl Installer {
    /// Create a new installer.
    pub fn new(
        downloader: PackageDownloader,
        node_modules: PathBuf,
        run_scripts: bool,
        num_workers: usize,
    ) -> Self {
        Self {
            downloader,
            extractor: TarballExtractor::new(num_workers),
            node_modules,
            run_scripts,
        }
    }

    /// Install resolved packages.
    pub async fn install(&self, packages: &[ResolvedPackage]) -> Result<InstallResult> {
        if packages.is_empty() {
            return Ok(InstallResult::default());
        }

        info!("Installing {} packages to {}", packages.len(), self.node_modules.display());

        // Create node_modules directory
        std::fs::create_dir_all(&self.node_modules)?;

        // Prepare download tasks
        let tasks: Vec<DownloadTask> = packages
            .iter()
            .map(|pkg| DownloadTask {
                name: pkg.name.clone(),
                version: pkg.version.clone(),
                tarball_url: pkg.tarball_url.clone(),
                integrity: pkg.integrity.clone(),
                shasum: pkg.shasum.clone(),
                dest_path: self.package_path(&pkg.name),
            })
            .collect();

        // Download packages
        let download_results = self.downloader.download_packages(tasks).await?;

        // Prepare extraction tasks
        let extractions: Vec<(PathBuf, PathBuf)> = download_results
            .iter()
            .map(|r| {
                let dest = self.package_path(&r.name);
                (r.tarball_path.clone(), dest)
            })
            .collect();

        // Extract packages in parallel
        self.extractor.extract_all(extractions)?;

        // Create .bin symlinks
        self.create_bin_links(packages)?;

        // Run install scripts
        let mut scripts_run = 0;
        if self.run_scripts {
            for pkg in packages.iter().filter(|p| p.has_install_script) {
                if self.run_lifecycle_script(&pkg.name, "preinstall")? {
                    scripts_run += 1;
                }
                if self.run_lifecycle_script(&pkg.name, "install")? {
                    scripts_run += 1;
                }
                if self.run_lifecycle_script(&pkg.name, "postinstall")? {
                    scripts_run += 1;
                }
            }
        }

        let stats = self.downloader.stats();

        Ok(InstallResult {
            packages_installed: packages.len(),
            packages_from_cache: stats.from_cache.load(std::sync::atomic::Ordering::Relaxed),
            bytes_downloaded: stats.bytes_downloaded.load(std::sync::atomic::Ordering::Relaxed),
            scripts_run,
        })
    }

    /// Get the path to a package in node_modules.
    fn package_path(&self, name: &str) -> PathBuf {
        if name.starts_with('@') {
            // Scoped package: @scope/name -> node_modules/@scope/name
            self.node_modules.join(name)
        } else {
            self.node_modules.join(name)
        }
    }

    /// Create .bin symlinks for package binaries.
    fn create_bin_links(&self, packages: &[ResolvedPackage]) -> Result<()> {
        let bin_dir = self.node_modules.join(".bin");
        std::fs::create_dir_all(&bin_dir)?;

        for pkg in packages {
            let pkg_dir = self.package_path(&pkg.name);
            let pkg_json_path = pkg_dir.join("package.json");

            if !pkg_json_path.exists() {
                continue;
            }

            // Read package.json to get bin field
            let pkg_json: PackageJson = match std::fs::read_to_string(&pkg_json_path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(p) => p,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };

            // Create bin links
            if let Some(bin) = &pkg_json.bin {
                match bin {
                    crate::package::BinField::Single(script_path) => {
                        let bin_name = pkg.name.split('/').last().unwrap_or(&pkg.name);
                        self.create_bin_link(&bin_dir, bin_name, &pkg_dir.join(script_path))?;
                    }
                    crate::package::BinField::Multiple(bins) => {
                        for (bin_name, script_path) in bins {
                            self.create_bin_link(&bin_dir, bin_name, &pkg_dir.join(script_path))?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Create a single bin link.
    fn create_bin_link(&self, bin_dir: &Path, name: &str, target: &Path) -> Result<()> {
        let link_path = bin_dir.join(name);

        // Remove existing link
        if link_path.exists() {
            std::fs::remove_file(&link_path)?;
        }

        // Create symlink
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            if let Err(e) = symlink(target, &link_path) {
                warn!("Failed to create bin link {}: {}", name, e);
            } else {
                // Make executable
                use std::os::unix::fs::PermissionsExt;
                if target.exists() {
                    let mut perms = std::fs::metadata(target)?.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(target, perms)?;
                }
                debug!("Created bin link: {} -> {}", link_path.display(), target.display());
            }
        }

        #[cfg(windows)]
        {
            // On Windows, create a cmd script
            let cmd_content = format!("@echo off\nnode \"{}\" %*", target.display());
            std::fs::write(link_path.with_extension("cmd"), cmd_content)?;
        }

        Ok(())
    }

    /// Run a lifecycle script for a package.
    fn run_lifecycle_script(&self, name: &str, script_name: &str) -> Result<bool> {
        let pkg_dir = self.package_path(name);
        let pkg_json_path = pkg_dir.join("package.json");

        if !pkg_json_path.exists() {
            return Ok(false);
        }

        let pkg_json: PackageJson = serde_json::from_str(&std::fs::read_to_string(&pkg_json_path)?)?;

        if let Some(script) = pkg_json.scripts.get(script_name) {
            info!("Running {} script for {}", script_name, name);

            let status = Command::new(if cfg!(windows) { "cmd" } else { "sh" })
                .args(if cfg!(windows) {
                    vec!["/C", script]
                } else {
                    vec!["-c", script]
                })
                .current_dir(&pkg_dir)
                .env("PATH", self.get_path_env())
                .status()?;

            if !status.success() {
                return Err(SnpmError::ScriptFailed {
                    script: format!("{}:{}", name, script_name),
                    code: status.code().unwrap_or(-1),
                });
            }

            return Ok(true);
        }

        Ok(false)
    }

    /// Get PATH environment variable with node_modules/.bin prepended.
    fn get_path_env(&self) -> String {
        let bin_dir = self.node_modules.join(".bin");
        let current_path = std::env::var("PATH").unwrap_or_default();

        if cfg!(windows) {
            format!("{};{}", bin_dir.display(), current_path)
        } else {
            format!("{}:{}", bin_dir.display(), current_path)
        }
    }
}

/// Result of an install operation.
#[derive(Debug, Default)]
pub struct InstallResult {
    /// Number of packages installed
    pub packages_installed: usize,
    /// Number of packages from cache
    pub packages_from_cache: usize,
    /// Total bytes downloaded
    pub bytes_downloaded: u64,
    /// Number of scripts run
    pub scripts_run: usize,
}

impl InstallResult {
    /// Format the result as a summary string.
    pub fn summary(&self) -> String {
        format!(
            "Added {} packages ({} from cache), downloaded {}",
            self.packages_installed,
            self.packages_from_cache,
            format_bytes(self.bytes_downloaded)
        )
    }
}

/// Format bytes as a human-readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Update the lockfile with installed packages.
pub fn update_lockfile(
    lockfile: &mut PackageLock,
    packages: &[ResolvedPackage],
) {
    for pkg in packages {
        let path = format!("node_modules/{}", pkg.name);

        let lock_pkg = LockPackage {
            version: pkg.version.clone(),
            resolved: Some(pkg.tarball_url.clone()),
            integrity: pkg.integrity.clone(),
            dependencies: pkg.dependencies.clone(),
            peer_dependencies: pkg.peer_dependencies.clone(),
            optional_dependencies: pkg.optional_dependencies.clone(),
            dev: matches!(pkg.dep_type, crate::package::DependencyType::Development),
            optional: pkg.optional,
            has_install_script: pkg.has_install_script,
            ..Default::default()
        };

        lockfile.add_package(&path, lock_pkg);
    }
}

