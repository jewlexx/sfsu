#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

// TODO: Replace regex with glob

mod commands;
mod opt;

use clap::Parser;

use commands::Commands;
use sfsu::get_scoop_path;

#[macro_use]
extern crate log;

/// Scoop utilities that can replace the slowest parts of Scoop, and run anywhere from 30-100 times faster
#[derive(Debug, Parser)]
#[clap(about, long_about, author, version)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[clap(long, global = true, help = "Disable terminal formatting")]
    no_color: bool,
}

fn main() -> anyhow::Result<()> {
    logger::handle_panics();
    let logging_path = logger::initialize_logging(log::LevelFilter::Trace)?;

    let args = Args::parse();
    if args.no_color {
        info!("Colour disabled");
        colored::control::set_override(false);
    }

    args.command.run()?;

    Ok(())
}
