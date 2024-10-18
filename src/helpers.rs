use std::path::{Path, PathBuf};
use std::{env, fs};

pub fn get_install_path() -> anyhow::Result<std::path::PathBuf> {
    // on macos its /Applications/TIDAL.app/Contents/Resources
    #[cfg(target_os = "macos")]
    return Ok(std::path::PathBuf::from(
        "/Applications/TIDAL.app/Contents/Resources",
    ));
    // on windows, it's localappdata/TIDAL
    // TODO: Actually test on windows :)
    #[cfg(target_os = "windows")]
    return Ok({
        // From original neptune installer
        // https://github.com/uwu/neptune-installer/blob/61763c8143d7c00cc17f24e7e730b04ea679306a/src/neptune_installer.nim#L24-L37
        let mut current_app_dir = String::new();
        let mut current_parsed_version = 0;
        let tidal_directory = join_path(env::var("localappdata").unwrap(), "TIDAL");

        // Walk dir
        if let Ok(entries) = fs::read_dir(&tidal_directory) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    // Get latest parsed version (probably for checking eventual updates for compat?)
                    if name.starts_with("app-") {
                        let parsed_version = name[4..name.len() - 1]
                            .replace(".", "")
                            .parse::<i32>()
                            .unwrap_or(0);

                        if parsed_version > current_parsed_version {
                            current_parsed_version = parsed_version;
                            current_app_dir = name.to_string();
                        }
                    }
                }
            }
        }

        join_path(join_path(tidal_directory, &current_app_dir), "resources")
    });

    #[cfg(target_os = "linux")]
    todo!("Linux installation not implemented! If you need Linux support, please open an issue on GitHub! (sorry :* )");

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    todo!("OS not supported! Please open an issue on GitHub!");
}

pub fn join_path<P: AsRef<Path>>(base: P, component: &str) -> PathBuf {
    let mut path = base.as_ref().to_path_buf();
    path.push(component);
    path
}
