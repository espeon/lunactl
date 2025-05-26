use anyhow::{Context, Result};
use ripunzip::{UnzipEngine, UnzipOptions};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};

use crate::base::LunaInstall;
use crate::progress::ProgressDisplayer;

fn report_on_insufficient_readahead_size() {
    warn!("Warning: this operation required several HTTP(S) streams.\nThis can slow down decompression.");
}

#[derive(Serialize, Deserialize)]
struct GithubRelease {
    tag_name: String,
    prerelease: bool,
    draft: bool,
    assets: Vec<GithubAsset>,
}
#[derive(Serialize, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

// Returns 'version, browser_download_url'
fn get_latest_release(get_prerelease: bool) -> Result<(String, String)> {
    let client = reqwest::blocking::Client::new();
    info!("Fetching release metadata");
    let response = client
        .get("https://api.github.com/repos/Inrixia/TidaLuna/releases")
        .header("user-agent", "neptunectl-beta")
        .send()?;
    info!("Fetched release metadata, decoding response body");
    let release = response.json::<Vec<GithubRelease>>()?;
    info!("Parsing release metadata");
    for release in release {
        if release.prerelease == get_prerelease {
            for asset in release.assets {
                if asset.name.contains("luna.zip") {
                    return Ok((release.tag_name, asset.browser_download_url));
                }
            }
        }
    }
    Err(anyhow::anyhow!(
        "Failed to find luna.zip in latest releases"
    ))
}

fn download_and_extract(output_directory: &Path) -> Result<()> {
    // get latest release
    let (version, url) = get_latest_release(false)?;
    let engine = UnzipEngine::for_uri(&url, None, report_on_insufficient_readahead_size)
        .map_err(|e| anyhow::anyhow!("Failed to create UnzipEngine: {e}"))?;

    let opts: UnzipOptions = UnzipOptions {
        output_directory: Some(output_directory.to_path_buf()),
        password: None,
        single_threaded: false,
        filename_filter: None,
        progress_reporter: Box::new(ProgressDisplayer::new()),
    };

    info!("Downloading luna version {} ({})", version, url);

    engine
        .unzip(opts)
        .map_err(|e| anyhow::anyhow!("Failed to unzip: {e}"))?;

    Ok(())
}

pub fn install(install: &LunaInstall, force: bool) -> Result<()> {
    debug!("Using install path: {}", install.install_path.display());

    // Check if luna is already installed
    if install.app_path.exists() {
        if force {
            info!(
                "Removing old luna app folder {}",
                install.app_path.display()
            );
            std::fs::remove_dir_all(&install.app_path)?;
        } else {
            anyhow::bail!("luna app folder already exists. Use --force to override.");
        }
    }

    // check if original app.asar moved
    if !install.orig_asar_path.exists() {
        info!(
            "Backing up {} to {}",
            install.orig_asar_path.display(),
            install.app_asar_path.display()
        );
        std::fs::rename(&install.app_asar_path, &install.orig_asar_path)?;
    }

    info!(
        "Downloading & extracting luna to {}",
        install.temp_path.display()
    );
    download_and_extract(&install.temp_path)?;
    if !install.temp_path.exists() {
        anyhow::bail!(
            "luna injector failed to extract to {}",
            install.temp_path.display()
        );
    }

    info!("Installing luna to {}", install.app_path.display());
    std::fs::rename(&install.temp_path, &install.app_path).with_context(|| {
        format!(
            "Failed to move injector from {} to {}",
            install.temp_path.display(),
            install.app_path.display()
        )
    })?;

    info!("luna has been installed successfully.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    fn mock_install_fs(install: &LunaInstall) -> Result<()> {
        // Create a mock luna directory structure
        let luna_download_dir = install.temp_path.join("app");
        fs::create_dir(&luna_download_dir)?;

        // Create a mock app.asar file
        File::create(install.install_path.join("app.asar"))?;

        Ok(())
    }

    fn assert_install_success(luna: &LunaInstall) -> Result<()> {
        // Check if the injector was moved correctly
        assert!(luna.install_path.join("app").exists());

        // Check if app.asar was renamed to original.asar
        assert!(luna.install_path.join("original.asar").exists());
        assert!(!luna.install_path.join("app.asar").exists());

        Ok(())
    }

    #[test]
    fn test_install() -> Result<()> {
        let luna = LunaInstall::mock()?;

        mock_install_fs(&luna)?;

        assert!(install(&luna, false).is_ok());

        assert_install_success(&luna)?;

        Ok(())
    }

    #[test]
    fn test_install_with_force() -> Result<()> {
        let luna = LunaInstall::mock()?;

        mock_install_fs(&luna)?;

        // Create a mock existing app directory
        fs::create_dir(&luna.app_path)?;

        assert!(install(&luna, true).is_ok());

        assert_install_success(&luna)?;

        Ok(())
    }
}
