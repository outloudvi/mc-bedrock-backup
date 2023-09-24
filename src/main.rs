mod backup;
mod constants;
mod seeker;
mod utils;

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{arg, command, value_parser};

#[cfg(target_os = "linux")]
fn main() -> Result<()> {
    let matches = command!() // requires `cargo` feature
        .arg(
            arg!(
                -p --pipefile <FILE> "Sets the Minecraft server input FIFO file path"
            )
            .required(true)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                -l --logfile <FILE> "Sets the log file path"
            )
            .required(true)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                -o --outputfile <FILE> "Set the output file path"
            )
            .default_value("level.tar")
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(
                -w --worlds <DIR> "Set the worlds directory"
            )
            .required(true)
            .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    let pipe_file = matches
        .get_one::<PathBuf>("pipefile")
        .ok_or(anyhow!("Invalid pipe file path"))?;
    let log_file = matches
        .get_one::<PathBuf>("logfile")
        .ok_or(anyhow!("Invalid log file path"))?;
    let output_file = matches
        .get_one::<PathBuf>("outputfile")
        .ok_or(anyhow!("Invalid output file path"))?;
    let worlds_dir = matches
        .get_one::<PathBuf>("worlds")
        .ok_or(anyhow!("Invalid worlds dir path"))?;

    // Pre-check
    utils::check_lockfile()?;

    // let r =
    backup::backup(pipe_file, log_file, output_file, worlds_dir).unwrap();

    // Cleanup
    utils::remove_lockfile()?;

    // Result
    Ok(())
}
