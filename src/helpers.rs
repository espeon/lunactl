pub fn get_install_path() -> anyhow::Result<std::path::PathBuf> {
    // on macos its /Applications/TIDAL.app/Contents/Resources
    #[cfg(target_os = "macos")]
    let path = std::path::PathBuf::from("/Applications/TIDAL.app/Contents/Resources");
    // on windows, it's localappdata/TIDAL
    // TODO: Actually test on windows :)
    #[cfg(target_os = "windows")]
    let path = {
        let mut current_app_dir = String::new();
        let mut current_parsed_version = 0;
        let tidal_directory = join_path(env::var("localappdata").unwrap(), "TIDAL");

        // Walk through the directory
        if let Ok(entries) = fs::read_dir(&tidal_directory) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
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

        join_path(tidal_directory, &current_app_dir, "resources")
    };

    #[cfg(target_os = "linux")]
    todo!("Linux installation not implented!");

    Ok(path)
}
