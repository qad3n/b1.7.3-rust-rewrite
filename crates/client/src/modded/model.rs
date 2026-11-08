use crate::render::RenderBlocks;
use game::block::Block;
use modloader::atlas::Atlas;
use std::rc::Rc;

const MODEL_UNITS: f32 = 16.0;

// each cuboid renders as a standard block with its bounds, then the block gets its full bounds back
pub fn renderModel(renderer: &mut RenderBlocks, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
{
    let cuboids = modloader::content::blocks::activeCuboids(block.m_blockID, &*renderer.world(), x, y, z);
    let mut drawn: bool = false;
    for cuboid in &cuboids
    {
        block.setBlockBounds(
            cuboid.from[0] / MODEL_UNITS,
            cuboid.from[1] / MODEL_UNITS,
            cuboid.from[2] / MODEL_UNITS,
            cuboid.to[0] / MODEL_UNITS,
            cuboid.to[1] / MODEL_UNITS,
            cuboid.to[2] / MODEL_UNITS,
        );
        renderer.m_overrideBlockTexture = cuboid
            .texture
            .as_deref()
            .and_then(|texture| modloader::content::textureSlot(texture, Atlas::Terrain))
            .unwrap_or(-1);
        drawn |= renderer.renderStandardBlock(block, x, y, z);
    }
    renderer.m_overrideBlockTexture = -1;
    block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    drawn
}
