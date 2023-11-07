use std::fs::File;
use std::path::{Path, PathBuf};
use std::{thread, time};

use anyhow::{anyhow, Result};
use log::{debug, info};
use tar::Builder;

use crate::constants::{SAVE_COMPLETE_TEXT_BEDROCK, SAVE_COMPLETE_TEXT_JAVA};
use crate::types::EngineMode;
use crate::{seeker, utils};

struct SaveFile {
    path: String,
    size: u64,
}

pub fn backup(
    pipe_file_path: &Path,
    log_file_path: &PathBuf,
    output_file_path: &PathBuf,
    worlds_dir_paths: &[PathBuf],
    engine_mode: EngineMode,
) -> Result<()> {
    let mut log_file = File::open(log_file_path)?;
    let pipe_file_path_str = pipe_file_path.to_str().unwrap();

    let mut text = String::new();

    seeker::seek_to_end(&mut log_file)?;

    // Stage 1: Run "/save hold"
    info!("Starting saving the world.");
    utils::write_to_fifo(
        pipe_file_path_str,
        match engine_mode {
            EngineMode::Bedrock => "save hold\n",
            EngineMode::Java => "save-off\nsave-all\n",
        },
    )?;

    // Stage 2: Run "/save query" to check
    info!("Checking the progress.");
    loop {
        thread::sleep(time::Duration::from_secs(2));
        text.clear();
        if engine_mode == EngineMode::Bedrock {
            utils::write_to_fifo(pipe_file_path_str, "save query\n")?;
            thread::sleep(time::Duration::from_secs(1));
        }
        seeker::read_to_end(&mut log_file, &mut text)?;
        if text.contains(match engine_mode {
            EngineMode::Bedrock => SAVE_COMPLETE_TEXT_BEDROCK,
            EngineMode::Java => SAVE_COMPLETE_TEXT_JAVA,
        }) {
            break;
        }
        info!("Backup not finished - wait for 3 secs");
    }

    info!("Backup finished! Export the files.");
    match engine_mode {
        EngineMode::Bedrock => {
            export_files_bedrock(text, worlds_dir_paths.first().unwrap(), output_file_path)?;
        },
        EngineMode::Java => {
            export_files_java(worlds_dir_paths, output_file_path)?;
        },
    }

    // Stage 3: Run "/save resume" to finalize
    info!("Backup created, restoring.");
    utils::write_to_fifo(
        pipe_file_path_str,
        match engine_mode {
            EngineMode::Bedrock => "save resume\n",
            EngineMode::Java => "save-on\n",
        },
    )?;

    Ok(())
}

fn export_files_java(worlds_dir_paths: &[PathBuf], output_file_path: &PathBuf) -> Result<()> {
    let tar_file = File::create(output_file_path)?;
    let mut ar = Builder::new(tar_file);
    for src_path in worlds_dir_paths {
        let path = Path::new(src_path);
        debug!("Saving path {:?}", src_path);
        ar.append_dir_all(
            path.file_name()
                .ok_or(anyhow!("Invalid path name: {:?}", src_path))?,
            src_path,
        )?;
    }
    ar.finish()?;
    Ok(())
}

fn export_files_bedrock(
    text: String,
    worlds_dir_path: &Path,
    output_file_path: &Path,
) -> Result<()> {
    let target_line = text
        .split('\n')
        .skip_while(|x| !x.contains(SAVE_COMPLETE_TEXT_BEDROCK))
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
