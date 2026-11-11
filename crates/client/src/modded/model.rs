use crate::render::{RenderBlocks, Tessellator};
use game::block::Block;
use game::world::IBlockAccess;
use lwjgl::opengl::GL11;
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

// the shade beta gives each face, bottom top north south west east
const FACE_SHADE: [f32; 6] = [0.5, 1.0, 0.8, 0.8, 0.6, 0.6];
const FACE_STEP: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

// the sprites a mod block's behavior lays over its open faces, drawn after the block itself
pub fn renderOverlays(renderer: &mut RenderBlocks, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
{
    if !modloader::behavior::overlays(block.m_blockID)
    {
        return false;
    }

    let world: Rc<dyn IBlockAccess> = renderer.world();
    let tessellator: Rc<Tessellator> = crate::render::Tessellator::instance();
    let mut drawn: bool = false;

    for (face, [dx, dy, dz]) in FACE_STEP.into_iter().enumerate()
    {
        let face: i32 = face as i32;
        if !block.shouldSideBeRendered(&*world, x + dx, y + dy, z + dz, face)
        {
            continue;
        }
        let Some(overlay) = modloader::behavior::overlay(block, &*world, x, y, z, face)
        else
        {
            continue;
        };

        let light: f32 = if overlay.glow
        {
            1.0
        }
        else
        {
            block.getBlockBrightness(&*world, x + dx, y + dy, z + dz) * FACE_SHADE[face as usize]
        };
        let channel = |shift: u32| ((overlay.color >> shift) & 0xFF) as f32 / 255.0 * light;
        tessellator.setColorOpaque_F(channel(16), channel(8), channel(0));

        let lift: f64 = f64::from(overlay.lift / MODEL_UNITS);
        let [ox, oy, oz] = [x as f64 + dx as f64 * lift, y as f64 + dy as f64 * lift, z as f64 + dz as f64 * lift];

        match face
        {
            0 => renderer.renderBottomFace(block, ox, oy, oz, overlay.sprite),
            1 => renderer.renderTopFace(block, ox, oy, oz, overlay.sprite),
            2 => renderer.renderEastFace(block, ox, oy, oz, overlay.sprite),
            3 => renderer.renderWestFace(block, ox, oy, oz, overlay.sprite),
            4 => renderer.renderNorthFace(block, ox, oy, oz, overlay.sprite),
            _ => renderer.renderSouthFace(block, ox, oy, oz, overlay.sprite),
        }

        drawn = true;
    }

    drawn
}

// a moving block's model pieces as bounds in block units and the sprite each one overrides with, -1 for its faces
pub fn movingShapes(blockId: i32, meta: i32) -> Vec<([f32; 6], i32)>
{
    modloader::content::blocks::metaCuboids(blockId, meta)
        .into_iter()
        .map(|cuboid|
        {
            let [x0, y0, z0] = cuboid.from.map(|value| value / MODEL_UNITS);
            let [x1, y1, z1] = cuboid.to.map(|value| value / MODEL_UNITS);
            let sprite: i32 = cuboid
                .texture
                .as_deref()
                .and_then(|texture| modloader::content::textureSlot(texture, Atlas::Terrain))
                .unwrap_or(-1);
            ([x0, y0, z0, x1, y1, z1], sprite)
        })
        .collect()
}

// a block's item model in a slot or a hand: its cuboids as small blocks, then its tile's model posed once
pub fn renderItemModel(renderer: &mut RenderBlocks, block: &Rc<Block>, metadata: i32)
{
    let Some(model) = modloader::content::blocks::itemModel(block.m_blockID)
    else
    {
        return;
    };

    GL11::glTranslatef(-0.5, -0.5, -0.5);

    for cuboid in &model.cuboids
    {
        let [x0, y0, z0] = cuboid.from.map(|value| value / MODEL_UNITS);
        let [x1, y1, z1] = cuboid.to.map(|value| value / MODEL_UNITS);
        block.setBlockBounds(x0, y0, z0, x1, y1, z1);
        renderer.m_overrideBlockTexture = cuboid
            .texture
            .as_deref()
            .and_then(|texture| modloader::content::textureSlot(texture, Atlas::Terrain))
            .unwrap_or(-1);
        renderer.renderInventoryCube(block, Some(metadata), 0.0);
    }

    renderer.m_overrideBlockTexture = -1;
    block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);

    if let Some(tile) = &model.tile
    {
        super::entity_model::drawPosed(&tile.to_string(), &model.pose);
    }

    GL11::glTranslatef(0.5, 0.5, 0.5);
}
