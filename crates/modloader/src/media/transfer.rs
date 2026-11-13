use crate::host::context;
use crate::network;
use game::block::Block;
use game::entity::Entity;
use game::world::{Stream, World};
use mod_protocol::abi::{BlockPos, Song};
use mod_protocol::handshake::ChannelDirection;
use mod_protocol::{Identifier, wire};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub const REQUEST: &str = "loader:song_request";
pub const CHUNK: &str = "loader:song_chunk";

mod limits
{
    pub const CHUNK_BYTES: usize = 32768;
    // about 2.6 MB a second so a song arrives in seconds without starving the game's own packets
    pub const CHUNKS_PER_TICK: usize = 4;
}

const TICKS_PER_SECOND: f32 = 20.0;
const RECORD_IN: i32 = 1;

#[derive(Serialize, Deserialize)]
pub struct Chunk
{
    pub hash: String,
    pub index: u32,
    pub total: u32,
    pub data: Vec<u8>,
}

struct Upload
{
    m_player: Rc<Entity>,
    m_hash: String,
    m_bytes: Vec<u8>,
    m_next: usize,
}

struct Pending
{
    m_dimension: i32,
    m_pos: BlockPos,
    m_song: Song,
}

thread_local!
{
    static UPLOADS: RefCell<Vec<Upload>> = const { RefCell::new(Vec::new()) };
    static DOWNLOADS: RefCell<HashMap<String, Vec<u8>>> = RefCell::new(HashMap::new());
    static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

pub fn register()
{
    network::registerChannel(network::Channel
    {
        m_id: id(REQUEST),
        m_version: "1".to_owned(),
        m_direction: ChannelDirection::ToServer,
        m_onServer: Some(Rc::new(requested)),
        m_onClient: None,
    });

    network::registerChannel(network::Channel
    {
        m_id: id(CHUNK),
        m_version: "1".to_owned(),
        m_direction: ChannelDirection::ToClient,
        m_onServer: None,
        m_onClient: Some(network::loader::clientHandler(received)),
    });
}

fn id(name: &str) -> Identifier { Identifier::parse(name).expect("loader channel names are valid identifiers") }

// a client lacking the song asks the server for it and starts it once the last chunk is in
pub fn play(world: &Rc<World>, pos: BlockPos, song: Song) -> Result<(), String>
{
    if let Some(file) = local(world, &song.hash)
    {
        start(world, pos, &song, &file);
        return Ok(());
    }

    if crate::side::isAuthority(world)
    {
        return Err(format!("song {} is not in this world", song.hash));
    }

    let asked: bool = DOWNLOADS.with(|downloads| downloads.borrow().contains_key(&song.hash));
    if !asked
    {
        DOWNLOADS.with(|downloads| downloads.borrow_mut().insert(song.hash.clone(), Vec::new()));

        if !network::sendToServer(&id(REQUEST), wire::encode(&song.hash))
        {
            DOWNLOADS.with(|downloads| downloads.borrow_mut().remove(&song.hash));
            return Err("the server does not send songs".to_owned());
        }
    }

    PENDING.with(|pending|
    {
        *pending.borrow_mut() = Some(Pending { m_dimension: context::dimension(world), m_pos: pos, m_song: song })
    });
    Ok(())
}

fn local(world: &World, hash: &str) -> Option<PathBuf>
{
    let dir: PathBuf = if crate::side::isAuthority(world)
    {
        super::worldSongs(world)
    }
    else
    {
        super::cacheSongs()
    }?;
    Some(super::songPath(&dir, hash)).filter(|path| path.is_file())
}

// every listener counts from the same world time so all hear the same moment
fn start(world: &World, pos: BlockPos, song: &Song, file: &Path)
{
    let offset: f32 = (world.getWorldTime() - song.start) as f32 / TICKS_PER_SECOND;
    let stream: Stream = Stream { m_file: file, m_title: &song.title, m_offset: offset };
    world.playStream(&stream, pos.x, pos.y, pos.z);
}

fn requested(player: &Rc<Entity>, bytes: &[u8])
{
    let hash: String = match wire::decode::<String>(bytes)
    {
        Ok(hash) if super::validHash(&hash) => hash,
        Ok(hash) => return crate::log::warn(crate::log::LOADER, &format!("song request for a bad name {hash}")),
        Err(error) => return crate::log::warn(crate::log::LOADER, &format!("bad song request: {error}")),
    };

    let Some(dir) = super::worldSongs(&player.world())
    else
    {
        return;
    };

    match std::fs::read(super::songPath(&dir, &hash))
    {
        Ok(bytes) =>
        {
            let upload: Upload = Upload { m_player: Rc::clone(player), m_hash: hash, m_bytes: bytes, m_next: 0 };
            UPLOADS.with(|uploads| uploads.borrow_mut().push(upload));
        }
        Err(error) => crate::log::warn(crate::log::LOADER, &format!("song {hash} could not be sent: {error}")),
    }
}

pub fn sendChunks() { UPLOADS.with(|uploads| uploads.borrow_mut().retain_mut(Upload::send)); }

impl Upload
{
    // false once every chunk went out or the player left
    fn send(&mut self) -> bool
    {
        let total: u32 = u32::try_from(self.m_bytes.len().div_ceil(limits::CHUNK_BYTES)).unwrap_or(u32::MAX);

        for _ in 0..limits::CHUNKS_PER_TICK
        {
            let begin: usize = self.m_next * limits::CHUNK_BYTES;
            if begin >= self.m_bytes.len()
            {
                return false;
            }

            let end: usize = (begin + limits::CHUNK_BYTES).min(self.m_bytes.len());
            let chunk: Chunk = Chunk
            {
                hash: self.m_hash.clone(),
                index: u32::try_from(self.m_next).unwrap_or(u32::MAX),
                total,
                data: self.m_bytes[begin..end].to_vec(),
            };

            if !network::loader::send(CHUNK, &self.m_player, &chunk)
            {
                return false;
            }

            self.m_next += 1;
        }

        true
    }
}

fn received(chunk: Chunk)
{
    let complete: Option<Vec<u8>> = DOWNLOADS.with(|downloads|
    {
        let mut downloads: std::cell::RefMut<HashMap<String, Vec<u8>>> = downloads.borrow_mut();
        downloads.get_mut(&chunk.hash)?.extend_from_slice(&chunk.data);
        if chunk.index + 1 < chunk.total
        {
            return None;
        }

        downloads.remove(&chunk.hash)
    });

    let Some(bytes) = complete
    else
    {
        return;
    };

    if super::hashOf(&bytes) != chunk.hash
    {
        return crate::log::warn(crate::log::LOADER, &format!("song {} arrived damaged", chunk.hash));
    }

    let Some(dir) = super::cacheSongs()
    else
    {
        return;
    };

    let path: PathBuf = super::songPath(&dir, &chunk.hash);
    let written: std::io::Result<()> = std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&path, &bytes));
    if let Err(error) = written
    {
        return crate::log::warn(crate::log::LOADER, &format!("song {} could not be kept: {error}", chunk.hash));
    }

    let pending: Option<Pending> = PENDING.with(|pending|
    {
        let mut pending: std::cell::RefMut<Option<Pending>> = pending.borrow_mut();
        pending.take_if(|waiting| waiting.m_song.hash == chunk.hash)
    });
    let Some(pending) = pending
    else
    {
        return;
    };

    let Some(world) = context::world(pending.m_dimension)
    else
    {
        return;
    };

    // the disc may have come out while the song was on its way
    let pos: BlockPos = pending.m_pos;
    let playing: bool = world.getBlockId(pos.x, pos.y, pos.z) == Block::JUKEBOX
        && world.getBlockMetadata(pos.x, pos.y, pos.z) == RECORD_IN;
    if playing
    {
        start(&world, pos, &pending.m_song, &path);
    }
}
