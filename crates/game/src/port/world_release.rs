use crate::world::{Chunk, World};
use std::rc::Rc;

pub fn releaseWorld(world: &World)
{
    if let Some(provider) = world.m_chunkProvider.get()
    {
        for chunk in provider.releaseChunks()
        {
            releaseChunk(&chunk);
        }
    }
    world.m_loadedEntityList.clear();
    world.m_unloadedEntityList.clear();
    world.m_loadedTileEntityList.clear();
    world.m_pendingBlockEntities.clear();
    world.m_playerEntities.clear();
    world.m_weatherEffects.clear();
    world.m_worldAccesses.clear();
    world.m_lightingToUpdate.clear();
    world.m_collidingBoundingBoxes.clear();
    world.m_entitiesInArea.clear();
}

fn releaseChunk(chunk: &Rc<Chunk>)
{
    for list in &chunk.m_entities
    {
        list.clear();
    }
    chunk.m_chunkTileEntityMap.update(|map| map.clear());
}
