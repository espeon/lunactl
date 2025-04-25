use std::io::{self, Write};
use std::path::PathBuf;

use base::LunaInstall;
use clap::{CommandFactory, Parser};
#[cfg(target_os = "windows")]
use tracing::info;
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

use anyhow::Result;

mod progress;

mod base;
mod install;
mod uninstall;

use crate::install::install;
use crate::uninstall::uninstall;

/// A CLI tool to manage Luna on your system
#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// subcommand for install/uninstall
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[clap(about = "Install Luna from `master` branch")]
    Install(MainOpts),
    #[clap(about = "Uninstall Luna")]
    Uninstall(MainOpts),
}

#[derive(clap::Args)]
struct MainOpts {
    #[clap(
        long,
        action = clap::ArgAction::SetTrue, // Sets `force` to Some(true) when provided
        help = "Force regardless of if Luna is installed/uninstalled"
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

fn anykey() -> Result<()> {
    io::stdout().flush().expect("Failed to flush stdout");

    // Wait for the user to press Enter
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    #[cfg(target_os = "windows")]
    {
        info!("Fresh TIDAL installs may need to wait for Defender to finish scanning.");
    }

    match &cli.command {
        Some(Commands::Install(opts)) => {
            install(&LunaInstall::new(opts.install_path.clone())?, opts.force)
        }
        Some(Commands::Uninstall(opts)) => {
            uninstall(&LunaInstall::new(opts.install_path.clone())?, opts.force)
        }
        None => {
            Cli::command().print_help()?;
            println!("\nNo commands specified! Using defaults...");

            let luna = LunaInstall::new(None)?;
            let installed = luna.installed();
            let action_text = if installed { "uninstall" } else { "install" };
            println!(
                "Press Enter to {} TidaLuna. Press Ctrl+C to exit.",
                action_text
            );
            anykey()?;

            if installed {
                uninstall(&luna, false)?
            } else {
                install(&luna, false)?
            }

            println!(
                "\nTidaLuna {}ed successfully! Press Enter to exit.",
                action_text
            );
            anykey()?;

            Ok(())
        }
    }?;

    Ok(())
}
