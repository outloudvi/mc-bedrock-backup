use anyhow::Result;
use log::{debug, error};
use serde::Serialize;
use ureq::{self, Error};

#[derive(Serialize)]
struct SendMessageBody {
    chat_id: String,
    text: String,
}

pub fn send_message(bot_token: &str, chat_id: &str, text: &str) -> Result<()> {
    let req = SendMessageBody {
        chat_id: chat_id.to_owned(),
        text: text.to_owned(),
    };
    debug!("Sending: {}", serde_json::to_string(&req)?);
    let resp = ureq::post(&format!(
        "https://api.telegram.org/bot{}/sendMessage",
        bot_token
    ))
    .set("Content-Type", "application/json")
    .send_json(req);
    if let Err(Error::Status(code, response)) = resp {
        match response.into_string() {
            Ok(x) => error!("Request error [HTTP {}]: {}", code, x),
            Err(e) => error!("Request error [HTTP {}]: Unparsable - {}", code, e),
        };
    }
    Ok(())
}
