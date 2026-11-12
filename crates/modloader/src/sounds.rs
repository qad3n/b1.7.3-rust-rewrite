use crate::state::{self, ModRuntime};
use game::resources::Source;
use std::path::{Path, PathBuf};

const SOUND_DIR: &str = "sounds";

// mod ogg files are copied out of the mod files once, the sound engine plays files from disk
const RECORD_DIR: &str = "record";

// holds the hash of the mod file the unpacked sounds came from
const STAMP_FILE: &str = ".sha256";

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
        found.extend(unpack(runtime, &paths.m_cache.join(SOUND_DIR).join(runtime.id())));
    }

    found
}

// a folder mod plays its files in place and an archive unpacks again only when its hash changes
fn unpack(runtime: &ModRuntime, dir: &Path) -> Vec<(String, PathBuf)>
{
    let id: &str = runtime.id();
    let prefix: String = format!("assets/{id}/{SOUND_DIR}/");
    let names: Vec<String> =
        runtime.m_source.borrow().list(&prefix).into_iter().filter(|name| name.ends_with(".ogg")).collect();

    if let Source::Folder(root) = &*runtime.m_source.borrow()
    {
        return names.iter().map(|name| (format!("{id}/{}", &name[prefix.len()..]), root.join(name))).collect();
    }

    let stamp: PathBuf = dir.join(STAMP_FILE);
    let current: bool = std::fs::read_to_string(&stamp).is_ok_and(|text| text.trim() == runtime.m_sha256);

    if !current
    {
        clear(id, dir);
    }

    let mut found: Vec<(String, PathBuf)> = Vec::new();
    let mut complete: bool = true;

    for name in &names
    {
        let relative: &str = &name[prefix.len()..];
        let target: PathBuf = dir.join(relative);

        if !current || !target.is_file()
        {
            let bytes: Option<Vec<u8>> = runtime.m_source.borrow_mut().read(name);
            let written: std::io::Result<()> = match bytes
            {
                Some(bytes) => target
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::write(&target, bytes)),
                None => Ok(()),
            };

            if let Err(error) = written
            {
                crate::log::warn(id, &format!("sound {relative} could not be unpacked: {error}"));
                complete = false;
                continue;
            }
        }

        found.push((format!("{id}/{relative}"), target));
    }

    if !current && complete
    {
        let written: std::io::Result<()> =
            std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&stamp, &runtime.m_sha256));

        if let Err(error) = written
        {
            crate::log::warn(id, &format!("sound cache stamp {} could not be written: {error}", stamp.display()));
        }
    }

    found
}

// sounds a previous version of the mod unpacked are dropped with the old stamp
fn clear(id: &str, dir: &Path)
{
    match std::fs::remove_dir_all(dir)
    {
        Ok(()) =>
        {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound =>
        {}
        Err(error) => crate::log::warn(id, &format!("old sounds in {} could not be removed: {error}", dir.display())),
    }
}

// mod discs stream from sounds/record like vanilla discs from resources/streaming
pub fn isRecord(path: &str) -> bool { path.split('/').nth(1) == Some(RECORD_DIR) }
