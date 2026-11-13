use game::world::{ISaveHandler, World};
use mod_protocol::wire;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const LOADER_DIR: &str = "loader";
const WORLD_DATA_FILE: &str = "moddata.dat";
const PROBE: &str = "loader_probe";

thread_local!
{
    // mod id and key to bytes for the world in play, loaded on world load and written on save
    static WORLD_DATA: RefCell<BTreeMap<String, Vec<u8>>> = const { RefCell::new(BTreeMap::new()) };
}

// the world folder is the parent of its data folder, which the save handler already knows
pub fn worldDir(world: &World) -> Option<PathBuf>
{
    let handler: Rc<dyn ISaveHandler> = world.m_saveHandler.get()?;
    Some(handler.getDataFile(PROBE)?.parent()?.parent()?.to_path_buf())
}

pub fn loaderDir(world: &World) -> Option<PathBuf> { worldDir(world).map(|dir| dir.join(LOADER_DIR)) }

mod format
{
    // loader files start with these bytes and a format number, files from before the header have neither
    pub const MAGIC: [u8; 4] = *b"B173";
    pub const VERSION: u8 = 1;
    pub const HEADER: usize = MAGIC.len() + 1;
    pub const SET_ASIDE: &str = "unreadable";
}

// a file that cannot be used is kept under another name so the next save never destroys what it held
fn setAside(path: &Path, reason: &str)
{
    let name: String = path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    let kept: PathBuf =
        path.with_file_name(format!("{name}.{}-{}", format::SET_ASIDE, jvm::system::currentTimeMillis()));
    let outcome: String = match std::fs::rename(path, &kept)
    {
        Ok(()) => format!("kept as {}", kept.display()),
        Err(error) => format!("and it could not be moved aside: {error}"),
    };
    let message: String = format!("{} {reason}, {outcome}", path.display());

    crate::log::error(crate::log::LOADER, &message);
    crate::state::problem(crate::log::LOADER, &name, &message);
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String>
{
    let Some(payload) = bytes.strip_prefix(&format::MAGIC)
    else
    {
        // written before files had a header
        return wire::decode(bytes).map_err(|error| format!("does not decode: {error}"));
    };

    match payload.split_first()
    {
        Some((version, body)) if *version <= format::VERSION =>
        {
            wire::decode(body).map_err(|error| format!("does not decode: {error}"))
        }

        Some((version, _)) => Err(format!("has format {version}, this loader reads up to {}", format::VERSION)),
        None => Err("ends after its first bytes".to_owned()),
    }
}

// None when the file does not exist yet, and also after an unusable file was reported and set aside
pub fn loadFrom<T: DeserializeOwned>(dir: &Path, name: &str) -> Option<T>
{
    let path: PathBuf = dir.join(name);
    let bytes: Vec<u8> = match std::fs::read(&path)
    {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,

        Err(error) =>
        {
            let message: String = format!("{} cannot be read: {error}", path.display());
            crate::log::error(crate::log::LOADER, &message);
            crate::state::problem(crate::log::LOADER, name, &message);
            return None;
        }
    };

    match decode(&bytes)
    {
        Ok(value) => Some(value),

        Err(reason) =>
        {
            setAside(&path, &reason);
            None
        }
    }
}

pub fn saveTo<T: Serialize>(dir: &Path, name: &str, value: &T) -> std::io::Result<()>
{
    std::fs::create_dir_all(dir)?;

    let mut bytes: Vec<u8> = Vec::with_capacity(format::HEADER);
    bytes.extend_from_slice(&format::MAGIC);
    bytes.push(format::VERSION);
    bytes.extend_from_slice(&wire::encode(value));

    // written beside the target and renamed so a crash never leaves half a file
    let temp: PathBuf = dir.join(format!("{name}.tmp"));
    std::fs::write(&temp, bytes)?;
    std::fs::rename(temp, dir.join(name))
}

pub fn load<T: DeserializeOwned>(world: &World, name: &str) -> Option<T> { loadFrom(&loaderDir(world)?, name) }

pub fn save<T: Serialize>(world: &World, name: &str, value: &T) -> std::io::Result<()>
{
    match loaderDir(world)
    {
        Some(dir) => saveTo(&dir, name, value),
        None => Ok(()),
    }
}

pub fn loadWorld(world: &World)
{
    let loaded: BTreeMap<String, Vec<u8>> = load(world, WORLD_DATA_FILE).unwrap_or_default();
    crate::log::debug(crate::log::LOADER, &format!("loaded world data for {} mod keys", loaded.len()));
    WORLD_DATA.with(|data| *data.borrow_mut() = loaded);
    crate::structures::load(world);
    crate::biomes::load(world);
}

pub fn saveWorld(world: &World)
{
    let saved: std::io::Result<()> = WORLD_DATA.with(|data| save(world, WORLD_DATA_FILE, &*data.borrow()));
    if let Err(error) = saved
    {
        crate::log::error(crate::log::LOADER, &format!("could not save mod world data: {error}"));
    }

    crate::structures::save(world);
    crate::biomes::save(world);
}

fn key(modId: &str, key: &str) -> String { format!("{modId}:{key}") }

pub fn worldData(modId: &str, name: &str) -> Vec<u8>
{
    WORLD_DATA.with(|data| data.borrow().get(&key(modId, name)).cloned().unwrap_or_default())
}

pub fn setWorldData(modId: &str, name: &str, bytes: Vec<u8>)
{
    WORLD_DATA.with(|data| data.borrow_mut().insert(key(modId, name), bytes));
}

pub fn playerKey(modId: &str, name: &str) -> String { key(modId, name) }

// a world value under its full name, none when it was never set
pub fn worldValue(name: &str) -> Option<Vec<u8>> { WORLD_DATA.with(|data| data.borrow().get(name).cloned()) }

pub fn setWorldValue(name: &str, bytes: Option<Vec<u8>>)
{
    WORLD_DATA.with(|data|
    {
        let mut data = data.borrow_mut();
        match bytes
        {
            Some(bytes) => data.insert(name.to_owned(), bytes),
            None => data.remove(name),
        };
    });
}

pub fn worldValues() -> Vec<(String, Vec<u8>)>
{
    WORLD_DATA.with(|data| data.borrow().iter().map(|(name, bytes)| (name.clone(), bytes.clone())).collect())
}

// owner, key and bytes of every world data entry
pub fn worldEntries() -> Vec<(String, String, Vec<u8>)>
{
    WORLD_DATA.with(|data|
    {
        data.borrow()
            .iter()
            .filter_map(|(stored, bytes)|
            {
                stored.split_once(':').map(|(owner, key)| (owner.to_owned(), key.to_owned(), bytes.clone()))
            })
            .collect()
    })
}
