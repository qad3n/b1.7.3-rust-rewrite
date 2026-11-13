use game::entity::Entity;
use game::network::packet::Packet;
use game::network::packets::Packet51MapChunk;
use game::world::Chunk;
use std::any::Any;
use std::rc::Rc;

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

    super::behindChunks(||
    {
        for x in area.m_from.0..=area.m_to.0
        {
            for z in area.m_from.1..=area.m_to.1
            {
                let chunk: Rc<Chunk> = world.getChunkFromChunkCoords(x, z);
                crate::attach::chunkSight(player, &world, &chunk);
                super::sync::chunkTiles(player, &chunk);
            }
        }
    });
}
