use std::path::PathBuf;

use anyhow::Result;
use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    log_file: PathBuf,

    #[arg(short, long)]
    chat_id: String,

    #[arg(short = 'b', long)]
    bot_token: String,
}

#[cfg(target_os = "linux")]
fn main() -> Result<()> {
    use std::fs::File;
    use std::thread;
    use std::time::Duration;

    use log::error;
    use mc_bedrock_tools::telegram::{self, send_message};
    use mc_bedrock_tools::{seeker, utils};

    pretty_env_logger::init();

    let args = Args::parse();
    let log_file_path = args.log_file;
    let chat_id = args.chat_id;
    let bot_token = args.bot_token;

    let mut log_file = File::open(log_file_path)?;
    seeker::seek_to_end(&mut log_file)?;

    let mut str = String::new();
    loop {
        str.clear();
        seeker::read_to_end(&mut log_file, &mut str)?;
        let results = utils::find_user_state_change(&str);
        for i in results {
            let _ = telegram::send_message(
                &bot_token,
                &chat_id,
                &format!(
                    "{} {} the game.",
                    i.username,
                    match i.state {
                        utils::StateChange::Connected => "joined",
                        utils::StateChange::Disconnected => "left",
                    }
                ),
            )
            .map_err(|x| {
                error!("{}", x);
            });
        }
        thread::sleep(Duration::from_secs(4));
    }

    // Result
    Ok(())
}
