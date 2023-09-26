use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::constants::LOCKFILE_PATH;

pub(crate) fn check_lockfile() -> Result<()> {
    if Path::new(LOCKFILE_PATH).exists() {
        Err(anyhow!(
            "The lock file ({}) exists. Remove it to proceed.",
            LOCKFILE_PATH
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn remove_lockfile() -> Result<()> {
    if Path::new(LOCKFILE_PATH).exists() {
        fs::remove_file(LOCKFILE_PATH).map_err(|x| anyhow!(x))
    } else {
        Ok(())
    }
}

pub(crate) fn write_to_fifo(path: &str, command: &str) -> Result<()> {
    let mut file = OpenOptions::new().write(true).open(path).unwrap();
    file.write_all(command.as_bytes()).unwrap();
    file.flush()?;
    Ok(())
}
