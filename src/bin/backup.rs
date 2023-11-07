use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{command, Parser};
use mc_bedrock_tools::types::EngineMode;
use mc_bedrock_tools::{backup, utils};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Minecraft server input FIFO file path
    #[arg(short, long)]
    pipe_file: PathBuf,

    /// Log file path
    #[arg(short, long)]
    log_file: PathBuf,

    /// Output file path
    #[arg(short, long)]
    output_file: PathBuf,

    /// The worlds directory
    #[arg(short, long)]
    world_dir: Vec<PathBuf>,

    /// Engine mode
    #[arg(long)]
    mode: EngineMode,
}

#[cfg(target_os = "linux")]
fn main() -> Result<()> {
    pretty_env_logger::init();

    let args = Args::parse();

    let pipe_file = args.pipe_file;
    let log_file = args.log_file;
    let output_file = args.output_file;
    let worlds_dirs = args.world_dir;
    let mode = args.mode;

    if mode == EngineMode::Bedrock && worlds_dirs.len() > 1 {
        return Err(anyhow!(
            "Bedrock Minecraft should only have one world directory."
        ));
    }

    // Pre-check
    utils::check_lockfile()?;

    // let r =
    backup::backup(&pipe_file, &log_file, &output_file, &worlds_dirs, mode).unwrap();

    // Cleanup
    utils::remove_lockfile()?;

    // Result
    Ok(())
}
