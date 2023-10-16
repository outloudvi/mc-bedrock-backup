use std::fs::File;
use std::path::PathBuf;
use std::{thread, time};

use anyhow::Result;
use log::{debug, info};
use tar::{Builder, Header};

use crate::constants::SAVE_COMPLETE_TEXT;
use crate::{seeker, utils};

struct SaveFile {
    path: String,
    size: u64,
}

pub fn backup(
    pipe_file_path: &PathBuf,
    log_file_path: &PathBuf,
    output_file_path: &PathBuf,
    worlds_dir_path: &PathBuf,
) -> Result<()> {
    let mut log_file = File::open(log_file_path)?;
    let pipe_file_path_str = pipe_file_path.to_str().unwrap();

    let mut text = String::new();

    seeker::seek_to_end(&mut log_file)?;

    // Stage 1: Run "/save hold"
    info!("Starting saving the world.");
    utils::write_to_fifo(pipe_file_path_str, "save hold\n")?;

    // Stage 2: Run "/save query" to check
    info!("Checking the progress.");
    loop {
        thread::sleep(time::Duration::from_secs(2));
        text.clear();
        utils::write_to_fifo(pipe_file_path_str, "save query\n")?;
        thread::sleep(time::Duration::from_secs(1));
        seeker::read_to_end(&mut log_file, &mut text)?;
        if text.contains(SAVE_COMPLETE_TEXT) {
            break;
        }
        info!("Backup not finished - wait for 3 secs");
    }

    info!("Backup finished! Export the files.");
    export_files(text, worlds_dir_path, output_file_path)?;

    // Stage 3: Run "/save resume" to finalize
    info!("Backup created, restoring.");
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
                        debug!("Parsing error: {}", e);
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
        debug!("Saving file {} (size: {})", file.path, file.size);
        let base_file_path = worlds_dir_path.join(&file.path);
        let mut base_file = File::open(base_file_path)?;
        ar.append_file(&file.path, &mut base_file)?;
    }

    Ok(())
}
