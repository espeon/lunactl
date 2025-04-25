use anyhow::Result;
use tracing::{info, warn};

use crate::base::LunaInstall;

pub fn uninstall(luna: &LunaInstall, force: bool) -> Result<()> {
    let app_exists = luna.app_path.exists();
    let original_asar_exists = luna.orig_asar_path.exists();

    // Check if luna is installed
    if force {
        if !app_exists {
            warn!("luna app path {:?} doesn't exist!", luna.app_path.display());
        }
        if !original_asar_exists {
            warn!(
                "Original luna app.asar file {:?} doesn't exist!",
                luna.orig_asar_path.display()
            );
        }
    } else {
        if !app_exists {
            anyhow::bail!(
                "luna app path {:?} doesn't exist! Use --force to override.",
                luna.app_path.display()
            );
        }
        if !original_asar_exists {
            anyhow::bail!(
                "Original luna app.asar file {:?} doesn't exist! Use --force to override. !!WARNING!! This may require you to reinstall Tidal.",
                luna.orig_asar_path.display()
            );
        }
    }

    // Remove the injector directory
    if luna.app_path.exists() {
        info!("Removing luna app directory: {}", luna.app_path.display());
        std::fs::remove_dir_all(&luna.app_path)?;
    }

    // Restore the original app.asar
    info!("Restoring original app.asar");
    if luna.orig_asar_path.exists() {
        std::fs::rename(&luna.orig_asar_path, &luna.app_asar_path)?;
    } else {
        anyhow::bail!(
            "Unable to restore original app.asar: {} not found! You may need to reinstall Tidal!",
            luna.orig_asar_path.display()
        );
    }

    info!("luna has been uninstalled successfully.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn test_uninstall() -> Result<()> {
        let luna = LunaInstall::mock()?;

        fs::create_dir(&luna.app_path)?;
        File::create(&luna.orig_asar_path)?;

        assert!(uninstall(&luna, false).is_ok());

        // Check if the luna directory was removed
        assert!(!luna.app_path.exists());

        // Check if app.asar was restored
        assert!(!luna.orig_asar_path.exists());
        assert!(luna.app_asar_path.exists());

        Ok(())
    }

    #[test]
    fn test_uninstall_no_installation() -> Result<()> {
        let luna = LunaInstall::mock()?;

        fs::create_dir(&luna.app_path)?;

        // Should fail with a message about unable to restore original app.asar
        assert!(uninstall(&luna, true).is_err());

        // Check if the luna app path was removed
        assert!(!luna.app_path.exists());

        Ok(())
    }
}
