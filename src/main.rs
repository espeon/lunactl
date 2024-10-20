use std::path::PathBuf;

use base::NeptuneInstall;
use clap::{Parser, Subcommand};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

mod progress;

mod base;
mod install;
mod uninstall;

use crate::install::install;
use crate::uninstall::uninstall;

/// A CLI tool to manage Neptune on your system
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// subcommand for install/uninstall
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    #[clap(about = "Install Neptune from `master` branch")]
    Install(MainOpts),
    #[clap(about = "Uninstall Neptune")]
    Uninstall(MainOpts),
}

#[derive(Parser, Debug, Clone)]
struct MainOpts {
    #[clap(
        long,
        action = clap::ArgAction::SetTrue, // Sets `force` to Some(true) when provided
        help = "Force regardless of if Neptune is installed/uninstalled"
    )]
    force: bool,

    #[clap(
        long,
        default_value = None,
        help = "The directory where app.asar or original.asar is found. Typically found in TIDAL\\app-x.xx.x\\resources"
    )]
    install_path: Option<PathBuf>,
}

fn main() {
    // Set up logs
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .event_format(tracing_subscriber::fmt::format().without_time())
        .with_env_filter(filter)
        .init();

    if let Err(e) = run() {
        error!("{e}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    #[cfg(target_os = "windows")]
    {
        info!("If you have a fresh install of TIDAL, you may need to wait for Defender to finish scanning the app files.");
    }

    match args.command {
        Commands::Install(opts) => install(&NeptuneInstall::new(opts.install_path)?, opts.force),
        Commands::Uninstall(opts) => {
            uninstall(&NeptuneInstall::new(opts.install_path)?, opts.force)
        }
    }?;

    Ok(())
}
