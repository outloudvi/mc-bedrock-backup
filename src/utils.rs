use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

use crate::constants::LOCKFILE_PATH;

lazy_static! {
    static ref USER_STATUS_REGEX: Regex = RegexBuilder::new(
        r"^\[\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}:\d{3} INFO\] Player (dis|)connected: ([^,]+),",
    )
    .multi_line(true)
    .build()
    .unwrap();
}

pub fn check_lockfile() -> Result<()> {
    if Path::new(LOCKFILE_PATH).exists() {
        Err(anyhow!(
            "The lock file ({}) exists. Remove it to proceed.",
            LOCKFILE_PATH
        ))
    } else {
        Ok(())
    }
}

pub fn remove_lockfile() -> Result<()> {
    if Path::new(LOCKFILE_PATH).exists() {
        fs::remove_file(LOCKFILE_PATH).map_err(|x| anyhow!(x))
    } else {
        Ok(())
    }
}

pub fn write_to_fifo(path: &str, command: &str) -> Result<()> {
    let mut file = OpenOptions::new().write(true).open(path).unwrap();
    file.write_all(command.as_bytes()).unwrap();
    file.flush()?;
    Ok(())
}

#[derive(PartialEq, Debug)]
pub enum StateChange {
    Connected,
    Disconnected,
}

#[derive(PartialEq, Debug)]
pub struct UserStateChange {
    pub username: String,
    pub state: StateChange,
}

pub fn find_user_state_change(lines: &str) -> Vec<UserStateChange> {
    let mut ret = Vec::new();
    for (_, [state, username]) in USER_STATUS_REGEX.captures_iter(lines).map(|c| c.extract()) {
        ret.push(UserStateChange {
            username: username.to_string(),
            state: if state.is_empty() {
                StateChange::Connected
            } else {
                StateChange::Disconnected
            },
        });
    }
    ret
}

#[cfg(test)]
mod test {
    use super::{find_user_state_change as f, StateChange, UserStateChange};

    #[test]
    fn test_one() {
        let a = f(
            "[2023-10-16 21:23:07:321 INFO] Player connected: Test1234, xuid: abc123, pfid: 12abc",
        );
        assert_eq!(a.len(), 1);
        assert_eq!(
            a[0],
            UserStateChange {
                username: "Test1234".to_string(),
                state: StateChange::Connected
            }
        );

        let a = f(
            "[2023-10-16 21:23:07:321 INFO] Player disconnected: Test1234, xuid: abc123, pfid: 12abc",
        );
        assert_eq!(a.len(), 1);
        assert_eq!(
            a[0],
            UserStateChange {
                username: "Test1234".to_string(),
                state: StateChange::Disconnected
            }
        );
    }

    #[test]
    fn test_utf8_name() {
        let a = f(
            "[2023-10-16 21:23:07:321 INFO] Player connected: 钟岚珠, xuid: abc123, pfid: 12abc",
        );
        assert_eq!(a.len(), 1);
        assert_eq!(
            a[0],
            UserStateChange {
                username: "钟岚珠".to_string(),
                state: StateChange::Connected
            }
        );
    }

    #[test]
    fn test_multi() {
        let a = f(
            "[2023-10-16 21:23:07:321 INFO] Player connected: Test1234, xuid: 2535454618798386
[2023-10-16 22:02:35:102 INFO] Player disconnected: Test1234, xuid: 1234abcd, pfid: abcd1234",
        );
        assert_eq!(a.len(), 2);
        assert_eq!(
            a[0],
            UserStateChange {
                username: "Test1234".to_string(),
                state: StateChange::Connected
            }
        );
    }
}
