use crate::client::Minecraft;
use jvm::warning;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::PathBuf;

const PIN_FILE: &str = "auth_servers.txt";

fn pinPath() -> PathBuf { Minecraft::getMinecraftDir().join(PIN_FILE) }

fn hex(key: &[u8]) -> String { key.iter().map(|byte| format!("{byte:02x}")).collect() }

// trust on first use like ssh known hosts, server keys are public so the file needs no protection
pub fn check(server: &str, key: &[u8]) -> Result<(), String>
{
    let path: PathBuf = pinPath();
    let wanted: String = hex(key);

    let pins: String = match fs::read_to_string(&path)
    {
        Ok(pins) => pins,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("Cannot read {}: {error}", path.display())),
    };

    let pinned: Option<&str> =
        pins.lines().filter_map(|line| line.split_once('\t')).find(|(host, _)| *host == server).map(|(_, key)| key);

    match pinned
    {
        Some(pinned) if pinned == wanted => Ok(()),
        Some(_) => Err(format!(
            "The key of {server} changed, this may not be the real server. Remove its line from {PIN_FILE} only if the owner confirms the new key"
        )),

        None =>
        {
            let pinned: std::io::Result<()> = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .and_then(|mut file| writeln!(file, "{server}\t{wanted}"));

            if let Err(error) = pinned
            {
                warning!("Could not save the key of {server} to {}: {error}", path.display());
            }

            Ok(())
        }
    }
}
