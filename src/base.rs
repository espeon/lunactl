use anyhow::{bail, Result};
use std::{env, fs, path::PathBuf};
use tracing::{debug, info, warn};

use tempfile::TempDir;
use which::which;

pub struct NeptuneInstall {
    pub temp_path: PathBuf,
    pub install_path: PathBuf,
    pub app_path: PathBuf,
    pub app_asar_path: PathBuf,
    pub orig_asar_path: PathBuf,
    is_mock: bool,
}

impl Drop for NeptuneInstall {
    fn drop(&mut self) {
        debug!("Cleanup removing temp_path: {}", self.temp_path.display());
        fs::remove_dir_all(&self.temp_path).unwrap();
        if self.is_mock {
            debug!(
                "Cleanup mock removing install_path: {}",
                self.install_path.display()
            );
            fs::remove_dir_all(&self.install_path).unwrap();
        }
    }
}

impl NeptuneInstall {
    pub fn new(install_path: Option<PathBuf>) -> Result<Self> {
        let install_path = if let Some(install_path) = install_path {
            dbg!(&install_path);
            if !install_path.exists() {
                anyhow::bail!("Install path does not exist");
            }
            if !install_path.is_dir() {
                anyhow::bail!("Install path is not a directory");
            }
            install_path
        } else {
            get_install_path()?
        };
        info!("Using install path: {}", install_path.display());
        Ok(Self {
            temp_path: TempDir::new()?.into_path(),
            install_path: install_path.clone(),
            app_path: install_path.join("app"),
            app_asar_path: install_path.join("app.asar"),
            orig_asar_path: install_path.join("original.asar"),
            is_mock: false,
        })
    }

    #[cfg(test)]
    pub fn mock() -> Result<Self> {
        let install_path = TempDir::new()?.into_path();
        Ok(Self {
            temp_path: TempDir::new()?.into_path(),
            install_path: install_path.clone(),
            app_path: install_path.join("app"),
            app_asar_path: install_path.join("app.asar"),
            orig_asar_path: install_path.join("original.asar"),
            is_mock: true,
        })
    }
}

fn find_latest_version(tidal_directory: &PathBuf) -> Result<Option<PathBuf>> {
    let mut current_parsed_version = 0;
    let mut current_app_dir: Option<PathBuf> = None;

    // From original neptune installer
    // https://github.com/uwu/neptune-installer/blob/61763c8143d7c00cc17f24e7e730b04ea679306a/src/neptune_installer.nim#L24-L37
    if let Ok(entries) = fs::read_dir(tidal_directory) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                // Only process entries that start with "app-"
                if name.starts_with("app-") {
                    // Safely parse the version part
                    let parsed_version = name[4..name.len()]
                        .trim_end_matches('/')
                        .replace(".", "")
                        .parse::<i32>()
                        .unwrap_or_else(|_| {
                            warn!("Failed to parse version from {}", name);
                            0
                        });

                    // Update if we find a newer version
                    if parsed_version > current_parsed_version {
                        current_parsed_version = parsed_version;
                        current_app_dir = Some(path.clone());
                    }
                }
            }
        }
    }

    Ok(current_app_dir)
}

fn get_install_path() -> Result<PathBuf> {
    let tidal_directory: Option<PathBuf> = match which("tidal") {
        Ok(path) => {
            info!("Found Tidal binary at: {:?}", path);
            path.parent().map(|p| p.to_path_buf())
        }
        Err(e) => {
            warn!(
                "Tidal binary not found in PATH, attempting to fallback on hardcoded paths! {}",
                e
            );
            None
        }
    };

    #[cfg(target_os = "macos")]
    return {
        if tidal_directory.is_none() {
            Ok(PathBuf::from("/Applications/TIDAL.app/Contents/Resources"));
        } else {
            Ok(PathBuf::from(format!(
                "{}/Contents/Resources",
                tidal_directory.display()
            )));
        }
    };

    #[cfg(target_os = "windows")]
    return {
        let install_dir = match tidal_directory {
            Some(tidal_directory) => tidal_directory,
            None => match env::var("localappdata") {
                Ok(localappdata) => PathBuf::from(localappdata).join("TIDAL"),
                Err(e) => {
                    bail!("Cannot find Tidal directory: {}", e);
                }
            },
        };
        let latest_app_dir = match find_latest_version(&install_dir) {
            Ok(Some(app_dir)) => app_dir,
            Ok(None) => bail!("Cannot find app directory in {}", install_dir.display()),
            Err(e) => {
                bail!(
                    "Error finding latest app directory in {}: {}",
                    install_dir.display(),
                    e
                )
            }
        };

        Ok(install_dir.join(latest_app_dir).join("resources"))
    };

    #[cfg(target_os = "linux")]
    bail!("Linux not supported! Please specify your Tidal installation path (location of app.asar) and consider opening a issue on GitHub.");

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    bail!("OS not supported! Please specify your Tidal installation path (location of app.asar) and consider opening a issue on GitHub.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neptune_install_new() -> Result<()> {
        let neptune = NeptuneInstall::mock()?;
        assert!(neptune.install_path.exists());
        assert!(neptune.temp_path.exists());
        Ok(())
    }

    #[test]
    fn test_neptune_install_invalid_path() {
        assert!(NeptuneInstall::new(Some(PathBuf::from("/nonexistent/path"))).is_err());
    }

    #[test]
    fn test_neptune_install_cleanup() -> Result<()> {
        let neptune = NeptuneInstall::mock()?;

        // Make a copy of the temp_dir path
        let temp_dir = neptune.temp_path.clone();
        let install_path = neptune.install_path.clone();

        // Explicitly drop the temp paths to ensure cleanup is tested
        drop(neptune);

        // Check if the temp dirs are cleaned up (mock uses TempDir for install_path)
        assert!(!temp_dir.exists());
        assert!(!install_path.exists());

        Ok(())
    }
}
