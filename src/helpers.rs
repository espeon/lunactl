use std::fmt::Write;
use std::{env, fs};
use std::path::{Path, PathBuf};

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

// Taken from ripunzip source (Apache 2/MIT)
// https://github.com/google/ripunzip/blob/e715807bef798ed90f208049453e9b874bbba727/src/main.rs#L202-L226

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use ripunzip::UnzipProgressReporter;


pub struct ProgressDisplayer(ProgressBar);

impl ProgressDisplayer {
    pub fn new() -> Self {
        Self(ProgressBar::new(0))
    }
}

impl UnzipProgressReporter for ProgressDisplayer {
    fn extraction_starting(&self, display_name: &str) {
        self.0.set_message(format!("Extracting {display_name}"))
    }

    fn total_bytes_expected(&self, expected: u64) {
        self.0.set_length(expected);
        self.0.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})\n{msg}")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#-"));
    }

    fn bytes_extracted(&self, count: u64) {
        self.0.inc(count)
    }
}

