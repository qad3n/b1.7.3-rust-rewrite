use game::entity::Entity;
use game::network::packet::Packet;
use game::network::packets::Packet51MapChunk;
use game::tileentity::TileEntity;
use game::world::{Chunk, World};
use mod_protocol::abi::{AttachmentHolder, BlockPos};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

pub const ATTACHMENTS: &str = "loader:attachments";

const CHUNK: i32 = 16;

// where one mod's attachment lives on the client
#[derive(Serialize, Deserialize)]
pub enum Target
{
    Tile
    {
        world: i32, pos: BlockPos
    },
    Chunk
    {
        world: i32, x: i32, z: i32
    },
    World
    {
        key: String
    },
}

#[derive(Serialize, Deserialize)]
pub struct AttachmentSync
{
    pub target: Target,
    pub owner: String,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct AttachmentBatch
{
    pub items: Vec<AttachmentSync>,
}

thread_local!
{
    // the holder kinds each mod asked to keep in step with clients
    static SYNCED: RefCell<BTreeSet<(AttachmentHolder, String)>> = const { RefCell::new(BTreeSet::new()) };
    // world data the server sent, read in place of the save a multiplayer client does not have
    static REMOTE_WORLD: RefCell<std::collections::BTreeMap<String, Vec<u8>>> = const { RefCell::new(std::collections::BTreeMap::new()) };
}

pub fn enable(holder: AttachmentHolder, owner: &str)
{
    SYNCED.with(|synced| synced.borrow_mut().insert((holder, owner.to_owned())));
}

fn synced(holder: AttachmentHolder, owner: &str) -> bool
{
    SYNCED.with(|synced| synced.borrow().contains(&(holder, owner.to_owned())))
}

fn any(holder: AttachmentHolder) -> bool
{
    SYNCED.with(|synced| synced.borrow().iter().any(|(kind, _)| *kind == holder))
}

fn encode(items: Vec<AttachmentSync>) -> Vec<u8> { mod_protocol::wire::encode(&AttachmentBatch { items }) }

fn channel() -> mod_protocol::Identifier { super::loader::id(ATTACHMENTS) }

pub fn tileChanged(tile: &TileEntity, owner: &str, data: &[u8])
{
    let Some(world) =
        tile.m_worldObj.get().filter(|_| crate::side::isServer() && synced(AttachmentHolder::Tile, owner))
    else
    {
        return;
    };

    let pos: BlockPos = BlockPos::new(tile.m_xCoord.get(), tile.m_yCoord.get(), tile.m_zCoord.get());
    let target: Target = Target::Tile { world: crate::host::context::dimension(&world), pos };
    let bytes: Vec<u8> = encode(vec![AttachmentSync { target, owner: owner.to_owned(), data: data.to_vec() }]);
    super::sendNear(&channel(), &world, [pos.x as f64, pos.y as f64, pos.z as f64], super::Reach::Column, &bytes);
}

pub fn chunkChanged(world: &World, x: i32, z: i32, owner: &str, data: &[u8])
{
    if !crate::side::isServer() || !synced(AttachmentHolder::Chunk, owner)
    {
        return;
    }

    let target: Target = Target::Chunk { world: crate::host::context::dimension(world), x, z };
    let bytes: Vec<u8> = encode(vec![AttachmentSync { target, owner: owner.to_owned(), data: data.to_vec() }]);
    let middle: [f64; 3] = [f64::from(x * CHUNK + CHUNK / 2), 0.0, f64::from(z * CHUNK + CHUNK / 2)];
    super::sendNear(&channel(), world, middle, super::Reach::Column, &bytes);
}

pub fn worldChanged(owner: &str, key: &str, data: &[u8])
{
    if !crate::side::isServer() || !synced(AttachmentHolder::World, owner)
    {
        return;
    }

    let item: AttachmentSync =
        AttachmentSync { target: Target::World { key: key.to_owned() }, owner: owner.to_owned(), data: data.to_vec() };
    super::broadcast(&channel(), &encode(vec![item]));
}

// the chunk columns a map chunk packet covers, taken before the packet leaves
pub struct ChunkArea
{
    m_from: (i32, i32),
    m_to: (i32, i32),
}

pub fn chunkArea(packet: &dyn Packet) -> Option<ChunkArea>
{
    let any: &dyn Any = packet;
    let sent: &Packet51MapChunk = any.downcast_ref()?;
    let from: (i32, i32) = (sent.m_xPosition >> 4, sent.m_zPosition >> 4);
    let to: (i32, i32) = ((sent.m_xPosition + sent.m_xSize - 1) >> 4, (sent.m_zPosition + sent.m_zSize - 1) >> 4);
    Some(ChunkArea { m_from: from, m_to: to })
}

// a player who starts seeing chunks gets their synced attachments and the data of synced mod tiles in them
pub fn chunkSent(player: &Rc<Entity>, area: &ChunkArea)
{
    let Some(world) = player.m_worldObj.get()
    else
    {
        return;
    };

    let dimension: i32 = crate::host::context::dimension(&world);
    let mut items: Vec<AttachmentSync> = Vec::new();

    for x in area.m_from.0..=area.m_to.0
    {
        for z in area.m_from.1..=area.m_to.1
        {
            let chunk: Rc<Chunk> = world.getChunkFromChunkCoords(x, z);
            chunkItems(&chunk, dimension, &mut items);
            super::sync::chunkTiles(player, &chunk);
        }
    }

    if !items.is_empty()
    {
        super::sendToPlayer(&channel(), player, encode(items));
    }
}

fn chunkItems(chunk: &Chunk, dimension: i32, items: &mut Vec<AttachmentSync>)
{
    if any(AttachmentHolder::Chunk)
    {
        for (owner, data) in
            chunk.m_attachments.borrow().iter().filter(|(owner, _)| synced(AttachmentHolder::Chunk, owner))
        {
            let target: Target = Target::Chunk { world: dimension, x: chunk.m_xPosition, z: chunk.m_zPosition };
            items.push(AttachmentSync { target, owner: owner.clone(), data: data.clone() });
        }
    }

    if !any(AttachmentHolder::Tile)
    {
        return;
    }

    for tile in chunk.m_chunkTileEntityMap.get().values()
    {
        let pos: BlockPos = BlockPos::new(tile.m_xCoord.get(), tile.m_yCoord.get(), tile.m_zCoord.get());
        for (owner, data) in
            tile.m_attachments.borrow().iter().filter(|(owner, _)| synced(AttachmentHolder::Tile, owner))
        {
            items.push(AttachmentSync
            {
                target: Target::Tile { world: dimension, pos },
                owner: owner.clone(),
                data: data.clone(),
            });
        }
    }
}

// a player's own client gets the synced world data on login, respawn and dimension change
pub fn syncWorld(player: &Rc<Entity>)
{
    if !crate::side::isServer() || !any(AttachmentHolder::World)
    {
        return;
    }

    let items: Vec<AttachmentSync> = crate::persist::worldEntries()
        .into_iter()
        .filter(|(owner, _, _)| synced(AttachmentHolder::World, owner))
        .map(|(owner, key, data)| AttachmentSync { target: Target::World { key }, owner, data })
        .collect();
    if !items.is_empty()
    {
        super::sendToPlayer(&channel(), player, encode(items));
    }
}

pub fn received(batch: AttachmentBatch)
{
    for AttachmentSync { target, owner, data } in batch.items
    {
        match target
        {
            Target::Tile { world, pos } =>
            {
                let tile: Option<Rc<TileEntity>> =
                    crate::host::context::world(world).and_then(|world| world.getBlockTileEntity(pos.x, pos.y, pos.z));
                if let Some(tile) = tile
                {
                    put(&tile.m_attachments, owner, data);
                }
            }

            Target::Chunk { world, x, z } =>
            {
                if let Some(world) = crate::host::context::world(world)
                {
                    put(&world.getChunkFromChunkCoords(x, z).m_attachments, owner, data);
                }
            }

            Target::World { key } =>
            {
                REMOTE_WORLD.with(|remote| remote.borrow_mut().insert(crate::persist::playerKey(&owner, &key), data));
            }
        }
    }
}

fn put(slots: &RefCell<common::nbt::attachments::Attachments>, owner: String, data: Vec<u8>)
{
    let mut slots = slots.borrow_mut();
    match data.is_empty()
    {
        true => slots.remove(&owner),
        false => slots.insert(owner, data),
    };
}

// a multiplayer client reads the server's copy of synced world data
pub fn remoteWorld(owner: &str, key: &str) -> Option<Vec<u8>>
{
    REMOTE_WORLD.with(|remote| remote.borrow().get(&crate::persist::playerKey(owner, key)).cloned())
}

pub fn clearRemote() { REMOTE_WORLD.with(|remote| remote.borrow_mut().clear()); }
