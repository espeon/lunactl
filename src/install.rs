use anyhow::Result;
use ripunzip::{UnzipEngine, UnzipOptions};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use tempfile::TempDir;

use crate::helpers::{get_install_path, join_path};
use crate::progress::ProgressDisplayer;
use crate::InstallOpts;

pub struct Installer {
    temp_dir: PathBuf,
    install_path: PathBuf,
    force: bool,
}

impl Drop for Installer {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            error!("error during cleanup: {}", e);
        }
    }
}

impl Installer {
    pub fn new(opts: InstallOpts) -> Result<Self> {
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
            force: opts.force.unwrap_or(false),
        })
    }

    fn cleanup(&mut self) -> Result<()> {
        info!("cleaning up...");
        Ok(())
    }

    pub fn init(&mut self) -> Result<()> {
        info!("downloading neptune");
        self.download_and_extract()?;
        info!("installing neptune");
        self.install()?;
        Ok(())
    }

    fn report_on_insufficient_readahead_size() {
        warn!("Warning: this operation required several HTTP(S) streams.\nThis can slow down decompression.");
    }

    fn download_and_extract(&self) -> Result<()> {
        debug!("Downloading to {}", self.temp_dir.display());

        let engine = UnzipEngine::for_uri(
            "https://github.com/uwu/neptune/archive/refs/heads/master.zip",
            None,
            Self::report_on_insufficient_readahead_size,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create UnzipEngine: {e}"))?;

        let opts: UnzipOptions = UnzipOptions {
            output_directory: Some(self.temp_dir.clone()),
            password: None,
            single_threaded: false,
            filename_filter: None,
            progress_reporter: Box::new(ProgressDisplayer::new()),
        };

        engine
            .unzip(opts)
            .map_err(|e| anyhow::anyhow!("failed to unzip: {e}"))?;

        Ok(())
    }

    fn install(&mut self) -> Result<()> {
        debug!("using install path: {}", self.install_path.display());

        let injector_path = join_path(&self.temp_dir, "neptune-master/injector");
        debug!("using injector path: {}", injector_path.display());

        let app_path = join_path(&self.install_path, "app");
        debug!("using app path: {}", app_path.display());

        let app_asar_path = join_path(&self.install_path, "app.asar");
        let original_asar_path = join_path(&self.install_path, "original.asar");

        if self.force {
            warn!("removing old neptune app directory {}!", app_path.display());
            std::fs::remove_dir_all(&app_path)?;
        } else {
            // check if app.asar is moved
            debug!("checking if app.asar is moved: {}", app_asar_path.display());
            if !original_asar_path.exists() {
                debug!(
                    "moving app.asar to original.asar: {}",
                    original_asar_path.display()
                );
                std::fs::rename(&app_asar_path, &original_asar_path)?;
            } else {
                debug!(
                    "app.asar already exists at {}",
                    original_asar_path.display()
                );
            }
            // Check if Neptune is already installed
            if app_path.exists() {
                anyhow::bail!("neptune is already installed. Use --force to override.");
            }
        }

        std::fs::rename(injector_path, app_path)
            .map_err(|e| anyhow::anyhow!("Failed to move injector: {}", e))?;

        // does original.asar already exist?
        if !original_asar_path.exists() {
            debug!(
                "moving app.asar to original.asar: {}",
                original_asar_path.display()
            );
            std::fs::rename(app_asar_path, original_asar_path)?;
        } else {
            debug!(
                "app.asar already exists at {}",
                original_asar_path.display()
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn test_installer_new() {
        let opts = InstallOpts {
            install_path: None,
            force: Some(true),
        };
        let installer = Installer::new(opts).unwrap();
        assert!(installer.force);
        assert!(installer.install_path.exists());
    }

    #[test]
    fn test_installer_new_with_invalid_path() {
        let opts = InstallOpts {
            install_path: Some(PathBuf::from("/nonexistent/path")),
            force: None,
        };
        assert!(Installer::new(opts).is_err());
    }

    #[test]
    fn test_download_and_extract() {
        let installer = Installer::new(InstallOpts {
            force: Some(false),
            install_path: Some(TempDir::new().unwrap().into_path()),
        })
        .unwrap();

        assert!(installer.download_and_extract().is_ok());
    }

    #[test]
    fn test_cleanup() {
        let mut installer = Installer::new(InstallOpts {
            force: Some(false),
            install_path: Some(TempDir::new().unwrap().into_path()),
        })
        .unwrap();

        assert!(installer.cleanup().is_ok());
        assert!(!installer.temp_dir.exists());
    }

    fn mock_neptune_dir(temp_dir: &PathBuf, install_path: &PathBuf) {
        // Create a mock Neptune directory structure
        let neptune_dir = temp_dir.join("neptune-master");
        fs::create_dir(&neptune_dir).unwrap();
        fs::create_dir(&neptune_dir.join("injector")).unwrap();

        // Create a mock app.asar file
        File::create(&install_path.join("app.asar")).unwrap();
    }

    #[test]
    fn test_install() {
        let mut installer = Installer::new(InstallOpts {
            force: Some(false),
            install_path: Some(TempDir::new().unwrap().into_path()),
        })
        .unwrap();

        mock_neptune_dir(&installer.temp_dir, &installer.install_path);

        assert!(installer.install().is_ok());

        // Check if the injector was moved correctly
        assert!(installer.install_path.join("app").exists());

        // Check if app.asar was renamed to original.asar
        assert!(installer.install_path.join("original.asar").exists());
        assert!(!installer.install_path.join("app.asar").exists());

        // Cleanup temp install_path
        assert!(fs::remove_dir_all(&installer.install_path).is_ok());
    }

    #[test]
    fn test_install_with_force() {
        let mut installer = Installer::new(InstallOpts {
            force: Some(true),
            install_path: Some(TempDir::new().unwrap().into_path()),
        })
        .unwrap();

        mock_neptune_dir(&installer.temp_dir, &installer.install_path);

        // Create a mock existing app directory
        fs::create_dir(&installer.install_path.join("app")).unwrap();

        assert!(&installer.install().is_ok());

        // Check if the injector was moved correctly
        assert!(&installer.install_path.join("app").exists());

        // Check if app.asar was renamed to original.asar
        assert!(&installer.install_path.join("original.asar").exists());
        assert!(!&installer.install_path.join("app.asar").exists());
    }
}
