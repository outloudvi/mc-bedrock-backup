use std::fs::File;
use std::os::unix::prelude::FileExt;
use std::path::PathBuf;
use std::{thread, time};

use anyhow::Result;
use tar::{Builder, Header};

use crate::constants::SAVE_COMPLETE_TEXT;
use crate::{seeker, utils};

struct SaveFile {
    path: String,
    size: u64,
}

pub(crate) fn backup(
    pipe_file_path: &PathBuf,
    log_file_path: &PathBuf,
    output_file_path: &PathBuf,
    worlds_dir_path: &PathBuf,
) -> Result<()> {
    let mut log_file = File::open(log_file_path)?;
    let pipe_file_path_str = pipe_file_path.to_str().unwrap();

    let mut text = String::new();

    println!("Running /save hold 00");

    seeker::seek_to_end(&mut log_file)?;

    println!("Running /save hold");
    // Stage 1: Run "/save hold"
    utils::write_to_fifo(pipe_file_path_str, "save hold\n")?;
    // let _ = pipe_file.sync_all();

    println!("Looking for the progress");
    // Stage 2: Run "/save query" to check
    loop {
        thread::sleep(time::Duration::from_secs(2));
        text.clear();
        utils::write_to_fifo(pipe_file_path_str, "save query\n")?;
        thread::sleep(time::Duration::from_secs(1));
        seeker::read_to_end(&mut log_file, &mut text)?;
        if text.contains(SAVE_COMPLETE_TEXT) {
            break;
        }
        println!("Backup not finished - wait for 3 secs");
    }

    println!("Backup finished! Exporting files");
    export_files(text, worlds_dir_path, output_file_path)?;

    // Stage 3: Run "/save resume" to finalize
    utils::write_to_fifo(pipe_file_path_str, "save resume\n")?;

    Ok(())
}

fn export_files(text: String, worlds_dir_path: &PathBuf, output_file_path: &PathBuf) -> Result<()> {
    let target_line = text
        .split('\n')
        .skip_while(|x| !x.contains(SAVE_COMPLETE_TEXT))
        .skip(1)
        .take(1)
        .collect::<String>();
    let target_files = target_line
        .split(", ")
        .map(|x| {
            let sp = x.split(':').collect::<Vec<_>>();
            SaveFile {
                path: sp[0].to_string(),
                size: match sp[1].parse::<u64>() {
                    Ok(x) => x,
                    Err(e) => {
                        println!("Parsing error: {}", e);
                        0
                    },
                },
            }
        })
        .filter(|x| x.size != 0)
        .collect::<Vec<_>>();
    let tar_file = File::create(output_file_path)?;
    let mut ar = Builder::new(tar_file);
    for file in target_files {
        let base_file_path = worlds_dir_path.join(&file.path);
        let mut base_file = File::open(base_file_path)?;
        ar.append_file(&file.path, &mut base_file)?;
    }

    Ok(())
}
