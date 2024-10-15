use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

mod helpers;
mod install;
mod uninstall;

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
    Install(InstallOpts),
    #[clap(about = "Uninstall Neptune")]
    Uninstall(UninstallOpts),
}

#[derive(Parser, Debug, Clone)]
struct InstallOpts {
    #[clap(
        long,
        default_value = "false",
        help = "Force overwrite existing Neptune installation"
    )]
    force: Option<bool>,

    #[clap(
        long,
        default_value = None,
        help = "The installation directory where app.asar or original.asar is found."
    )]
    install_path: Option<PathBuf>,
}

#[derive(Parser, Debug, Clone)]
struct UninstallOpts {
    #[clap(
        long,
        default_value = "false",
        help = "Force uninstall Neptune even if it is not installed"
    )]
    force: Option<bool>,
    #[clap(
        long,
        default_value = None,
        help = "The installation directory where app.asar or original.asar is found."
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
        info!("After scanning finishes, you can force install by running `neptune install --force`");
    }


    match args.command {
        Commands::Install(opts) => install::Installer::new(opts)?.init(),
        Commands::Uninstall(opts) => uninstall::Uninstaller::new(opts)?.init(),
        _ => {
            println!("You may want to specify a command.");
            Ok(())
        },
    }?;

    Ok(())
}
