use super::{Channel, Handler};
use crate::host::context::{self, dimension};
use game::entity::Entity;
use game::network::packet::Packet;
use game::network::packets::{Packet51MapChunk, Packet52MultiBlockChange, Packet53BlockChange};
use game::world::World;
use mod_protocol::handshake::ChannelDirection;
use mod_protocol::{Identifier, wire};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::rc::Rc;

pub const CHUNK_EXT: &str = "loader:chunk_ext";
const VANILLA_LIMIT: i32 = 256;
const FALLBACK_BLOCK: i32 = 1;

// block ids above 255 that a vanilla chunk or block change packet cannot carry
#[derive(Serialize, Deserialize, Default)]
pub struct ChunkExt
{
    pub world: i32,
    pub blocks: Vec<(i32, i32, i32, i32, i32)>,
}

fn id() -> Identifier { Identifier::parse(CHUNK_EXT).expect("the loader channel name is valid") }

fn apply(message: ChunkExt)
{
    let Some(world) = context::world(message.world)
    else
    {
        return;
    };

    for (x, y, z, block, meta) in message.blocks
    {
        world.setBlockAndMetadata(x, y, z, block, meta);
        world.markBlockNeedsUpdate(x, y, z);
    }
}

pub fn register()
{
    let handler: Handler = Rc::new(|_: &Rc<Entity>, bytes: &[u8]| match wire::decode::<ChunkExt>(bytes)
    {
        Ok(message) => apply(message),
        Err(error) => crate::log::warn(crate::log::LOADER, &format!("bad chunk extension: {error}")),
    });

    super::registerChannel(Channel
    {
        m_id: id(),
        m_version: "1".to_owned(),
        m_direction: ChannelDirection::ToClient,
        m_onServer: None,
        m_onClient: Some(handler),
    });
}

fn collect(
    world: &World,
    minX: i32,
    minY: i32,
    minZ: i32,
    sizeX: i32,
    sizeY: i32,
    sizeZ: i32,
) -> Vec<(i32, i32, i32, i32, i32)>
{
    let mut found: Vec<(i32, i32, i32, i32, i32)> = Vec::new();

    for x in minX..minX + sizeX
    {
        for z in minZ..minZ + sizeZ
        {
            for y in minY.max(0)..(minY + sizeY).min(world.height())
            {
                let block: i32 = world.getBlockId(x, y, z);
                if block >= VANILLA_LIMIT
                {
                    found.push((x, y, z, block, world.getBlockMetadata(x, y, z)));
                }
            }
        }
    }

    found
}

fn fallback(block: i32) -> i32
{
    crate::content::blockData(block)
        .and_then(|data| data.m_spec.vanilla_fallback.clone())
        .and_then(|fallback| crate::registry::with(|registries| registries.blockId(&fallback)))
        .unwrap_or(FALLBACK_BLOCK)
}

// the low byte of a 12-bit id would show another block, so the packet carries a fallback and the extension follows
pub fn outgoing(player: &Rc<Entity>, mut packet: Box<dyn Packet>) -> (Box<dyn Packet>, Option<ChunkExt>)
{
    if crate::lifecycle::blockLimit() <= VANILLA_LIMIT
    {
        return (packet, None);
    }
    let Some(world) = player.m_worldObj.get()
    else
    {
        return (packet, None);
    };

    let any: &mut dyn Any = &mut *packet;
    let blocks: Vec<(i32, i32, i32, i32, i32)> = if let Some(change) = any.downcast_mut::<Packet53BlockChange>()
    {
        if change.m_type < VANILLA_LIMIT
        {
            return (packet, None);
        }

        let entry: (i32, i32, i32, i32, i32) =
            (change.m_xPosition, change.m_yPosition, change.m_zPosition, change.m_type, change.m_metadata);
        change.m_type = fallback(change.m_type);
        vec![entry]
    }
    else if let Some(chunk) = any.downcast_ref::<Packet51MapChunk>()
    {
        collect(
            &world,
            chunk.m_xPosition,
            chunk.m_yPosition,
            chunk.m_zPosition,
            chunk.m_xSize,
            chunk.m_ySize,
            chunk.m_zSize,
        )
    }
    else if let Some(multi) = any.downcast_ref::<Packet52MultiBlockChange>()
    {
        let coordinates: Vec<i16> = multi.m_coordinateArray.clone().unwrap_or_default();
        coordinates
            .iter()
            .take(multi.m_size.max(0) as usize)
            .filter_map(|packed|
            {
                let packed: i32 = *packed as i32;
                let x: i32 = multi.m_xPosition * 16 + (packed >> 12 & 15);
                let z: i32 = multi.m_zPosition * 16 + (packed >> 8 & 15);
                let y: i32 = packed & 255;
                let block: i32 = world.getBlockId(x, y, z);
                (block >= VANILLA_LIMIT).then(|| (x, y, z, block, world.getBlockMetadata(x, y, z)))
            })
            .collect()
    }
    else
    {
        return (packet, None);
    };

    if blocks.is_empty()
    {
        return (packet, None);
    }

    (packet, Some(ChunkExt { world: dimension(&world), blocks }))
}

pub fn encodeExtension(player: &Rc<Entity>, message: &ChunkExt) -> Option<(u32, Vec<u8>)>
{
    crate::netsync::encode(player.m_entityId.get(), &id(), &wire::encode(message))
}
