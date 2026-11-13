mod encode;
mod import;
mod page;
mod tool;
mod transfer;

use crate::host::{context, unit};
use game::block::{Block, block_juke_box};
use game::entity::{Entity, entity_player};
use game::item::ItemStack;
use game::world::World;
use mod_protocol::abi::{BlockPos, HostReply, MediaLimit, MediaRequest};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::rc::Rc;

// songs sit in the media folder of the world that imported them and in the client cache of everyone who heard them
const DIR: &str = "media";
const EXTENSION: &str = "ogg";
const HASH_LENGTH: usize = 64;
const EMPTY_JUKEBOX: i32 = 0;

pub fn register() { transfer::register(); }

// the side that owns the world sends the chunks clients asked for
pub fn tick() { transfer::sendChunks(); }

pub fn handle(call: MediaRequest) -> HostReply
{
    match call
    {
        MediaRequest::Import { text, limit } => match importSong(text, limit)
        {
            Ok(job) => HostReply::Int(job),
            Err(reason) => HostReply::Error(reason),
        },
        MediaRequest::Status { job } => HostReply::Media(import::status(job)),
        MediaRequest::Play { world, pos, song } =>
        {
            let world: Option<Rc<World>> = context::world(world);
            unit(world.ok_or_else(|| "no such world".to_owned()).and_then(|world| playSong(&world, pos, song)))
        }
        MediaRequest::InsertHeld { world, pos, player } => unit(insertHeld(world, pos, player)),
    }
}

pub fn songPath(dir: &Path, hash: &str) -> PathBuf { dir.join(format!("{hash}.{EXTENSION}")) }

fn worldSongs(world: &World) -> Option<PathBuf> { crate::persist::worldDir(world).map(|dir| dir.join(DIR)) }

fn cacheSongs() -> Option<PathBuf> { crate::paths().map(|paths| paths.m_cache.join(DIR)) }

fn hashOf(bytes: &[u8]) -> String { Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect() }

// a song name is its hash, which keeps it from naming any other file
fn validHash(hash: &str) -> bool
{
    hash.len() == HASH_LENGTH && hash.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn importSong(text: String, limit: MediaLimit) -> Result<i32, String>
{
    let text: String = text.trim().to_owned();
    if text.is_empty()
    {
        return Err("give a link or a song name".to_owned());
    }

    if page::isLink(&text)
    {
        page::check(&text)?;
    }

    let world: Rc<World> = context::world(0).ok_or("no world is loaded")?;
    let dir: PathBuf = worldSongs(&world).ok_or("the world has no folder")?;
    let cache: PathBuf = crate::paths().ok_or("the loader has no cache folder")?.m_cache;
    std::fs::create_dir_all(&dir).map_err(|error| format!("could not make {}: {error}", dir.display()))?;

    Ok(import::start(dir, cache, text, limit))
}

fn playSong(world: &Rc<World>, pos: BlockPos, song: mod_protocol::abi::Song) -> Result<(), String>
{
    if !validHash(&song.hash)
    {
        return Err(format!("{} is not a song name", song.hash));
    }

    transfer::play(world, pos, song)
}

// as ItemRecord.onItemUse but the whole stack goes in so its components come back out
fn insertHeld(dimension: i32, pos: BlockPos, player: i32) -> Result<(), String>
{
    let world: Rc<World> = context::world(dimension).ok_or("no such world")?;
    let player: Rc<Entity> = context::entityById(&world, player).ok_or("no such player")?;

    let empty: bool = world.getBlockId(pos.x, pos.y, pos.z) == Block::JUKEBOX
        && world.getBlockMetadata(pos.x, pos.y, pos.z) == EMPTY_JUKEBOX;
    if !empty
    {
        return Err("there is no empty jukebox there".to_owned());
    }

    let stack: Rc<ItemStack> = entity_player::inventory(&player).getCurrentItem().ok_or("the player holds nothing")?;
    block_juke_box::insertStack(&world, pos.x, pos.y, pos.z, &stack);
    stack.m_stackSize.set(stack.m_stackSize.get() - 1);
    Ok(())
}
