use anyhow::Result;
use ripunzip::{NullProgressReporter, UnzipEngine, UnzipOptions, UnzipProgressReporter};
use std::iter::repeat_with;
use std::path::{Path, PathBuf};

use crate::helpers::get_install_path;
use crate::InstallOpts;

pub struct Installer {
    temp_dir: PathBuf,
    install_path: PathBuf,
    force: bool,
}

impl Installer {
    pub fn new(opts: InstallOpts) -> Result<Self> {
        if let Some(install_path) = opts.install_path {
            if !install_path.exists() {
                anyhow::bail!("Install path does not exist");
            }
            if !install_path.is_dir() {
                anyhow::bail!("Install path is not a directory");
            }
        }
        Ok(Self {
            temp_dir: std::env::temp_dir(),
            install_path: get_install_path()?,
            force: opts.force.unwrap_or(false),
        })
    }

    pub fn init(&self) -> Result<()> {
        println!("Downloading neptune...");
        let path = self.download_and_extract()?;
        println!("Installing neptune...");
        self.install(&path)?;
        Ok(())
    }

    fn report_on_insufficient_readahead_size() {
        eprintln!("Warning: this operation required several HTTP(S) streams.\nThis can slow down decompression.");
    }

    fn download_and_extract(&self) -> Result<PathBuf> {
        let random_string: String = repeat_with(fastrand::alphanumeric).take(10).collect();
        let file_name = format!("neptune-master-temp_{random_string}");
        let path = self.temp_dir.join(file_name);

        println!("Downloading to {}", path.display());

        let engine = UnzipEngine::for_uri(
            "https://github.com/uwu/neptune/archive/refs/heads/master.zip",
            None,
            Self::report_on_insufficient_readahead_size,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create UnzipEngine: {e}"))?;

        // TODO: use a progress bar
        let progress_reporter: Box<dyn UnzipProgressReporter + Sync> =
            Box::new(NullProgressReporter);

        let opts: UnzipOptions = UnzipOptions {
            output_directory: Some(path.clone()),
            password: None,
            single_threaded: false,
            filename_filter: None,
            progress_reporter,
        };

        engine
            .unzip(opts)
            .map_err(|e| anyhow::anyhow!("Failed to unzip: {e}"))?;

        Ok(path)
    }

    fn cleanup(&mut self) -> Result<()> {
        println!("Cleaning up...");
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }

    fn join_path<P: AsRef<Path>>(&self, base: P, component: &str) -> PathBuf {
        let mut path = base.as_ref().to_path_buf();
        path.push(component);
        path
    }

    fn install(&self, tempdir: &Path) -> Result<()> {
        let injector_path = self.join_path(tempdir, "neptune-master/injector");
        println!("Got install path: {}", self.install_path.display());
        let app_path = self.join_path(&self.install_path, "app");
        println!("Moving injector to install path: {}", app_path.display());
        if self.force {
            println!("Removing old neptune app directory");
            std::fs::remove_dir_all(&app_path)?;
        }
        std::fs::rename(injector_path, app_path)
            .map_err(|e| anyhow::anyhow!("Failed to move injector: {}", e))?;

        let app_asar_path = self.join_path(&self.install_path, "app.asar");
        let original_asar_path = self.join_path(&self.install_path, "original.asar");
        println!(
            "Moving app.asar to original.asar: {}",
            original_asar_path.display()
        );
        std::fs::rename(app_asar_path, original_asar_path)?;

        Ok(())
    }
}

impl Drop for Installer {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup() {
            eprintln!("Error during cleanup: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

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
        let temp_dir = TempDir::new().unwrap();
        let installer = Installer {
            temp_dir: temp_dir.path().to_path_buf(),
            install_path: PathBuf::new(),
            force: false,
        };

        let result = installer.download_and_extract();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.starts_with(temp_dir.path()));
        assert!(path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("neptune-master-temp_"));
    }

    #[test]
    fn test_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test_file.txt");
        File::create(&test_file).unwrap();

        let mut installer = Installer {
            temp_dir: temp_dir.path().to_path_buf(),
            install_path: PathBuf::new(),
            force: false,
        };

        assert!(installer.cleanup().is_ok());
        assert!(!temp_dir.path().exists());
    }

    #[test]
    fn test_install() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = TempDir::new().unwrap();

        // Create a mock Neptune directory structure
        let neptune_dir = temp_dir.path().join("neptune-master");
        fs::create_dir(&neptune_dir).unwrap();
        let injector_dir = neptune_dir.join("injector");
        fs::create_dir(&injector_dir).unwrap();

        // Create a mock app.asar file
        let app_asar = install_dir.path().join("app.asar");
        File::create(&app_asar).unwrap();

        let installer = Installer {
            temp_dir: temp_dir.path().to_path_buf(),
            install_path: install_dir.path().to_path_buf(),
            force: false,
        };

        assert!(installer.install(temp_dir.path()).is_ok());

        // Check if the injector was moved correctly
        assert!(install_dir.path().join("app").exists());

        // Check if app.asar was renamed to original.asar
        assert!(install_dir.path().join("original.asar").exists());
        assert!(!install_dir.path().join("app.asar").exists());
    }

    #[test]
    fn test_install_with_force() {
        let temp_dir = TempDir::new().unwrap();
        let install_dir = TempDir::new().unwrap();

        // Create a mock Neptune directory structure
        let neptune_dir = temp_dir.path().join("neptune-master");
        fs::create_dir(&neptune_dir).unwrap();
        let injector_dir = neptune_dir.join("injector");
        fs::create_dir(&injector_dir).unwrap();

        // Create a mock existing app directory
        let existing_app_dir = install_dir.path().join("app");
        fs::create_dir(&existing_app_dir).unwrap();

        // Create a mock app.asar file
        let app_asar = install_dir.path().join("app.asar");
        File::create(&app_asar).unwrap();

        let installer = Installer {
            temp_dir: temp_dir.path().to_path_buf(),
            install_path: install_dir.path().to_path_buf(),
            force: true,
        };

        assert!(installer.install(temp_dir.path()).is_ok());

        // Check if the injector was moved correctly
        assert!(install_dir.path().join("app").exists());

        // Check if app.asar was renamed to original.asar
        assert!(install_dir.path().join("original.asar").exists());
        assert!(!install_dir.path().join("app.asar").exists());
    }
}
