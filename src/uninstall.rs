use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

use crate::helpers::get_install_path;
use crate::UninstallOpts;

pub struct Uninstaller {
    install_path: PathBuf,
    force: bool,
}

impl Uninstaller {
    pub fn new(opts: UninstallOpts) -> Result<Self> {
        // check if paths exist
        let install_path = if let Some(install_path) = opts.install_path {
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
            install_path,
            force: opts.force.unwrap_or(false),
        })
    }

    pub fn init(&self) -> Result<()> {
        println!("Uninstalling neptune...");
        self.uninstall()?;
        Ok(())
    }

    fn join_path<P: AsRef<Path>>(&self, base: P, component: &str) -> PathBuf {
        let mut path = base.as_ref().to_path_buf();
        path.push(component);
        path
    }

    fn uninstall(&self) -> Result<()> {
        let app_path = self.join_path(&self.install_path, "app");
        let app_asar_path = self.join_path(&self.install_path, "app.asar");
        let original_asar_path = self.join_path(&self.install_path, "original.asar");

        // Check if Neptune is installed
        if !app_path.exists() || !original_asar_path.exists() {
            if self.force {
                println!(
                    "Neptune doesn't seem to be installed, but force flag is set. Continuing..."
                );
            } else {
                anyhow::bail!("Neptune doesn't seem to be installed. Use --force to override.");
            }
        }

        // Remove the injector directory
        println!("Removing Neptune app directory: {}", app_path.display());
        if app_path.exists() {
            std::fs::remove_dir_all(&app_path)?;
        }

        // Restore the original app.asar
        println!("Restoring original app.asar");
        if original_asar_path.exists() {
            std::fs::rename(&original_asar_path, &app_asar_path)?;
        } else {
            println!("Warning: original.asar not found. Unable to restore original app.asar!");
            bail!("You may need to reinstall Tidal!")
        }

        println!("Neptune has been uninstalled successfully.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_uninstaller_new() {
        let opts = UninstallOpts {
            install_path: None,
            force: Some(true),
        };
        let installer = Uninstaller::new(opts).unwrap();
        assert!(installer.force);
        assert!(installer.install_path.exists());
    }

    #[test]
    fn test_uninstaller_new_with_invalid_path() {
        let opts = UninstallOpts {
            install_path: Some(PathBuf::from("/nonexistent/path")),
            force: None,
        };
        assert!(Uninstaller::new(opts).is_err());
    }

    #[test]
    fn test_uninstall() {
        let install_dir = TempDir::new().unwrap();

        // Create a mock Neptune install dir structure
        let neptune_dir = install_dir.path().join("app");
        fs::create_dir(&neptune_dir).unwrap();

        // Create a mock app.asar file
        let app_asar = install_dir.path().join("original.asar");
        File::create(&app_asar).unwrap();

        let installer = Uninstaller {
            install_path: install_dir.path().to_path_buf(),
            force: false,
        };

        assert!(installer.uninstall().is_ok());

        // Check if the Neptune directory was removed
        assert!(!neptune_dir.exists());

        // Check if app.asar was restored
        assert!(!app_asar.exists());
        assert!(install_dir.path().join("app.asar").exists());
    }

    #[test]
    fn test_uninstall_with_force() {
        let install_dir = TempDir::new().unwrap();

        // Create a mock Neptune install dir structure
        let neptune_dir = install_dir.path().join("app");
        fs::create_dir(&neptune_dir).unwrap();

        // Skip creating the app.asar file

        let installer = Uninstaller {
            install_path: install_dir.path().to_path_buf(),
            force: true,
        };

        // Should fail with a message about unable to restore original app.asar
        assert!(installer.uninstall().is_err());

        // Check if the Neptune directory was removed
        assert!(!neptune_dir.exists());
    }
}
