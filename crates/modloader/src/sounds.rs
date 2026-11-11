use crate::state;
use std::path::PathBuf;

const SOUND_DIR: &str = "sounds";

// mod ogg files are copied out of the mod files once, the sound engine plays files from disk
const RECORD_DIR: &str = "record";

pub fn files() -> Vec<(String, PathBuf)>
{
    let Some(paths) = crate::paths()
    else
    {
        return Vec::new();
    };

    let mut found: Vec<(String, PathBuf)> = Vec::new();

    for runtime in state::mods().iter().filter(|runtime| runtime.isActive())
    {
        let id: String = runtime.id().to_owned();
        let prefix: String = format!("assets/{id}/{SOUND_DIR}/");
        let names: Vec<String> = runtime.m_source.borrow().list(&prefix);
        for name in names.into_iter().filter(|name| name.ends_with(".ogg"))
        {
            let relative: &str = &name[prefix.len()..];
            let target: PathBuf = paths.m_cache.join(SOUND_DIR).join(&id).join(relative);
            let bytes: Option<Vec<u8>> = runtime.m_source.borrow_mut().read(&name);
            let written: std::io::Result<()> = match bytes
            {
                Some(bytes) => target
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&target, bytes)),
                None => Ok(()),
            };
            match written
            {
                Ok(()) => found.push((format!("{id}/{relative}"), target)),
                Err(error) => crate::log::warn(&id, &format!("sound {relative} could not be unpacked: {error}")),
            }
        }
    }

    found
}

// mod discs stream from sounds/record like vanilla discs from resources/streaming
pub fn isRecord(path: &str) -> bool { path.split('/').nth(1) == Some(RECORD_DIR) }
