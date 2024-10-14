use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Install(opts) => install::Installer::new(opts)?.init(),
        Commands::Uninstall(opts) => uninstall::Uninstaller::new(opts)?.init(),
    }?;

    Ok(())
}
