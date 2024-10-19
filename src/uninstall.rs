use anyhow::{bail, Result};
use tracing::{debug, error, info, warn};

use crate::base::NeptuneInstall;

pub fn uninstall(neptune: &NeptuneInstall, force: bool) -> Result<()> {
    info!("uninstalling Neptune...");

    let app_exists = neptune.app_path.exists();
    let original_asar_exists = neptune.orig_asar_path.exists();

    // Check if Neptune is installed
    if force {
        if !app_exists {
            warn!(
                "Neptune app path {:?} doesnt exist! Flag --force is set continuing...",
                neptune.app_path.display()
            );
        }
        if !original_asar_exists {
            warn!(
                "Original Neptune app.asar file {:?} doesnt exist! --force is set continuing...",
                neptune.orig_asar_path.display()
            );
        }
    } else {
        if !app_exists {
            anyhow::bail!(
                "Neptune app path {:?} doesnt exist! Use --force to override.",
                neptune.app_path.display()
            );
        }
        if !original_asar_exists {
            anyhow::bail!(
                "Original Neptune app.asar file {:?} doesnt exist! Use --force to override. !!WARNING!! this may require you to reinstall Tidal.", 
                neptune.orig_asar_path.display()
            );
        }
    }

    // Remove the injector directory
    if neptune.app_path.exists() {
        debug!(
            "Removing Neptune app directory: {}",
            neptune.app_path.display()
        );
        std::fs::remove_dir_all(&neptune.app_path)?;
    }

    // Restore the original app.asar
    debug!("Restoring original app.asar");
    if neptune.orig_asar_path.exists() {
        std::fs::rename(&neptune.orig_asar_path, &neptune.app_asar_path)?;
    } else {
        error!("original.asar not found. unable to restore original app.asar!");
        bail!("Could not restore original app.asar, you may need to reinstall Tidal...");
    }

    info!("Neptune has been uninstalled successfully.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};

    #[test]
    fn test_uninstall() -> Result<()> {
        let neptune = NeptuneInstall::mock()?;

        fs::create_dir(&neptune.app_path)?;
        File::create(&neptune.orig_asar_path)?;

        assert!(uninstall(&neptune, false).is_ok());

        // Check if the Neptune directory was removed
        assert!(!neptune.app_path.exists());

        // Check if app.asar was restored
        assert!(!neptune.orig_asar_path.exists());
        assert!(neptune.app_asar_path.exists());

        Ok(())
    }

    #[test]
    fn test_uninstall_no_installation() -> Result<()> {
        let neptune = NeptuneInstall::mock()?;

        fs::create_dir(&neptune.app_path)?;

        // Should fail with a message about unable to restore original app.asar
        assert!(uninstall(&neptune, true).is_err());

        // Check if the Neptune app path was removed
        assert!(!neptune.app_path.exists());

        Ok(())
    }
}
