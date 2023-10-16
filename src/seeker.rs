use std::fs::File;
use std::io::{Read, Seek};

use anyhow::Result;

pub fn seek_to_end(file: &mut File) -> Result<()> {
    file.seek(std::io::SeekFrom::End(0))?;
    Ok(())
}

pub fn read_to_end(file: &mut File, str: &mut String) -> Result<()> {
    file.read_to_string(str)?;
    Ok(())
}
