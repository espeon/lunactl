use anyhow::{bail, Result};
use std::{env, fs, path::PathBuf};
use tracing::warn;

use crate::MainOpts;
use tempfile::TempDir;

pub struct NeptuneInstall {
    pub temp_dir: PathBuf,
    pub install_path: PathBuf,
    pub app_path: PathBuf,
}

impl NeptuneInstall {
    pub fn new(opts: MainOpts) -> Result<Self> {
        let install_path = if let Some(install_path) = opts.install_path {
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
        Ok(Self {
            temp_dir: TempDir::new().unwrap().into_path(),
            install_path,
            app_path: install_path.join("app"),
        })
    }
}

fn find_latest_version(tidal_directory: &PathBuf) -> Result<Option<PathBuf>> {
    let mut current_parsed_version = 0;
    let mut current_app_dir: Option<PathBuf> = None;

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

fn get_install_path() -> anyhow::Result<std::path::PathBuf> {
    // on macos its /Applications/TIDAL.app/Contents/Resources
    #[cfg(target_os = "macos")]
    return Ok(std::path::PathBuf::from(
        "/Applications/TIDAL.app/Contents/Resources",
    ));

    // on windows, it's localappdata/TIDAL
    #[cfg(target_os = "windows")]
    return Ok({
        // From original neptune installer
        // https://github.com/uwu/neptune-installer/blob/61763c8143d7c00cc17f24e7e730b04ea679306a/src/neptune_installer.nim#L24-L37
        let mut current_app_dir = String::new();
        let mut current_parsed_version = 0;
        let tidal_directory = {
            match env::var("localappdata") {
                Ok(localappdata) => PathBuf::from(localappdata).join("TIDAL"),
                Err(e) => {
                    bail!("Cannot find Tidal directory: {}", e);
                }
            }
        };

        let latest_app_dir = match find_latest_version(&tidal_directory) {
            Ok(Some(app_dir)) => app_dir,
            Ok(None) => bail!("Cannot find app directory in {}", tidal_directory.display()),
            Err(e) => {
                bail!(
                    "Error finding latest app directory in {}: {}",
                    tidal_directory.display(),
                    e
                )
            }
        };

        tidal_directory.join(latest_app_dir).join("resources")
    });

    #[cfg(target_os = "linux")]
    todo!("Linux installation not implemented! If you need Linux support, please open an issue on GitHub! (sorry :* )");

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    todo!("OS not supported! Please open an issue on GitHub!");
}
