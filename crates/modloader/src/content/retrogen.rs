use game::world::{Chunk, World};
use mod_protocol::Identifier;
use std::cell::OnceCell;
use std::collections::BTreeSet;
use std::rc::Rc;

// the chunk attachment listing the stable keys of every feature that ran in the chunk
const RECORD: &str = "loader:features";

thread_local!
{
    static RETROGEN: OnceCell<Vec<Identifier>> = const { OnceCell::new() };
}

fn recorded(chunk: &Chunk) -> BTreeSet<i64>
{
    let bytes: Option<Vec<u8>> = chunk.m_attachments.borrow().get(RECORD).cloned();
    bytes.and_then(|bytes| mod_protocol::wire::decode(&bytes).ok()).unwrap_or_default()
}

fn write(chunk: &Rc<Chunk>, keys: &BTreeSet<i64>)
{
    chunk.m_attachments.borrow_mut().insert(RECORD.to_owned(), mod_protocol::wire::encode(keys));
    chunk.setChunkModified();
}

// a populated chunk remembers which features it got, so a feature added later can be run there once
pub fn record(world: &Rc<World>, chunkX: i32, chunkZ: i32)
{
    let keys: BTreeSet<i64> =
        super::features::keys(crate::host::context::dimension(world)).into_iter().map(|(_, key)| key).collect();
    if keys.is_empty()
    {
        return;
    }

    let chunk: Rc<Chunk> = world.getChunkFromChunkCoords(chunkX, chunkZ);
    let mut done: BTreeSet<i64> = recorded(&chunk);
    done.extend(keys);
    write(&chunk, &done);
}

// a loaded chunk that was populated without a feature loader.toml lists under [retrogen] gets it now
pub fn backfill(world: &Rc<World>, chunkX: i32, chunkZ: i32)
{
    let wanted: Vec<Identifier> = RETROGEN.with(|cell| cell.get_or_init(super::features::retrogen).clone());
    if wanted.is_empty() || !crate::side::isAuthority(world)
    {
        return;
    }

    let chunk: Rc<Chunk> = world.getChunkFromChunkCoords(chunkX, chunkZ);
    // populate writes into the next chunks east and south, so those must exist as they did for populate
    let ready: bool = chunk.m_isTerrainPopulated.get()
        && world.chunkExists(chunkX + 1, chunkZ)
        && world.chunkExists(chunkX, chunkZ + 1)
        && world.chunkExists(chunkX + 1, chunkZ + 1);
    if !ready
    {
        return;
    }

    let mut done: BTreeSet<i64> = recorded(&chunk);
    let missing: Vec<i64> = super::features::keys(crate::host::context::dimension(world))
        .into_iter()
        .filter(|(id, key)| wanted.contains(id) && !done.contains(key))
        .map(|(_, key)| key)
        .collect();
    if missing.is_empty()
    {
        return;
    }

    for key in &missing
    {
        super::features::populateOne(world, chunkX, chunkZ, *key);
    }

    done.extend(missing);
    write(&chunk, &done);
}
