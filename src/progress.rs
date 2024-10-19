// Taken from ripunzip source (Apache 2/MIT)
// https://github.com/google/ripunzip/blob/e715807bef798ed90f208049453e9b874bbba727/src/main.rs#L202-L226

use std::fmt::Write;

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
