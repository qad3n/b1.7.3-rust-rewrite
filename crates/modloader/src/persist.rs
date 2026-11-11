use game::world::{ISaveHandler, World};
use mod_protocol::wire;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
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

pub fn read(world: &World, name: &str) -> Option<Vec<u8>> { std::fs::read(loaderDir(world)?.join(name)).ok() }

// a saved file that no longer decodes is reported and then treated as missing
pub fn decode<T: serde::de::DeserializeOwned>(name: &str, bytes: &[u8]) -> Option<T>
{
    match wire::decode(bytes)
    {
        Ok(value) => Some(value),

        Err(error) =>
        {
            crate::log::warn(crate::log::LOADER, &format!("could not decode {name}, ignoring it: {error}"));
            None
        }
    }
}

pub fn write(world: &World, name: &str, bytes: &[u8]) -> std::io::Result<()>
{
    let Some(dir) = loaderDir(world)
    else
    {
        return Ok(());
    };

    std::fs::create_dir_all(&dir)?;

    let target: PathBuf = dir.join(name);
    let temp: PathBuf = dir.join(format!("{name}.tmp"));
    std::fs::write(&temp, bytes)?;
    std::fs::rename(temp, target)
}

pub fn loadWorld(world: &World)
{
    let loaded: BTreeMap<String, Vec<u8>> =
        read(world, WORLD_DATA_FILE).and_then(|bytes| decode(WORLD_DATA_FILE, &bytes)).unwrap_or_default();
    crate::log::debug(crate::log::LOADER, &format!("loaded world data for {} mod keys", loaded.len()));
    WORLD_DATA.with(|data| *data.borrow_mut() = loaded);
    crate::structures::load(world);
    crate::biomes::load(world);
}

pub fn saveWorld(world: &World)
{
    let bytes: Vec<u8> = WORLD_DATA.with(|data| wire::encode(&*data.borrow()));
    if let Err(error) = write(world, WORLD_DATA_FILE, &bytes)
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
