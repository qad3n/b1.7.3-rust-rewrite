use crate::client::Minecraft;
use crate::render::tile::TileEntitySpecialRenderer;
use crate::render::{RenderBlocks, RenderHelper, Tessellator};
use game::block::{Block, block_piston_base, block_piston_extension};
use game::tileentity::{TileEntity, tile_entity_piston};
use game::world::{IBlockAccess, World};
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::RefCell;
use std::rc::Rc;

pub struct TileEntityRendererPiston
{
    m_blockRenderer: RefCell<Option<RenderBlocks>>,
}

impl TileEntityRendererPiston
{
    pub fn new() -> TileEntityRendererPiston { TileEntityRendererPiston { m_blockRenderer: RefCell::new(None) } }
}

impl TileEntitySpecialRenderer for TileEntityRendererPiston
{
    fn renderTileEntityAt(&self, piston: &Rc<TileEntity>, d: f64, d2: f64, d3: f64, f: f32)
    {
        let Some(block) = Block::byId(tile_entity_piston::getStoredBlockID(piston))
        else
        {
            return;
        };
        if tile_entity_piston::getProgress(piston, f) >= 1.0
        {
            return;
        }
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        self.bindTextureByName("/terrain.png");
        RenderHelper::disableStandardItemLighting();
        GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
        GL11::glEnable(GL11::GL_BLEND);
        GL11::glDisable(GL11::GL_CULL_FACE);
        GL11::glShadeModel(
            if Minecraft::isAmbientOcclusionEnabled()
            {
                GL11::GL_SMOOTH
            }
            else
            {
                GL11::GL_FLAT
            },
        );
        tessellator.startDrawingQuads();
        let (x, y, z): (i32, i32, i32) = (piston.m_xCoord.get(), piston.m_yCoord.get(), piston.m_zCoord.get());
        tessellator.setTranslationD(
            (d as f32 - x as f32 + tile_entity_piston::getRenderOffsetX(piston, f)) as f64,
            (d2 as f32 - y as f32 + tile_entity_piston::getRenderOffsetY(piston, f)) as f64,
            (d3 as f32 - z as f32 + tile_entity_piston::getRenderOffsetZ(piston, f)) as f64,
        );
        tessellator.setColorOpaque(1, 1, 1);
        let mut renderer = self.m_blockRenderer.borrow_mut();
        let renderer: &mut RenderBlocks = renderer.as_mut().orNpe();
        let extension: Rc<Block> = Block::pistonExtension();
        if Rc::ptr_eq(&block, &extension) && tile_entity_piston::getProgress(piston, f) < 0.5
        {
            renderer.renderPistonHead(&block, x, y, z, false);
        }
        else if tile_entity_piston::isSource(piston) && !tile_entity_piston::isExtending(piston)
        {
            block_piston_extension::setForcedSprite(&extension, block_piston_base::getTopSprite(&block));
            renderer.renderPistonHead(&extension, x, y, z, tile_entity_piston::getProgress(piston, f) < 0.5);
            block_piston_extension::clearForcedSprite(&extension);
            tessellator.setTranslationD(
                (d as f32 - x as f32) as f64,
                (d2 as f32 - y as f32) as f64,
                (d3 as f32 - z as f32) as f64,
            );
            renderer.renderExtendedPistonBase(&block, x, y, z);
        }
        else
        {
            renderer.renderMovingBlock(&block, x, y, z);
        }
        tessellator.setTranslationD(0.0, 0.0, 0.0);
        tessellator.draw();
        RenderHelper::enableStandardItemLighting();
    }

    fn setWorld(&self, world: Option<Rc<World>>)
    {
        let renderer: RenderBlocks = match world
        {
            Some(world) => RenderBlocks::new(world as Rc<dyn IBlockAccess>),
            None => RenderBlocks::empty(),
        };
        *self.m_blockRenderer.borrow_mut() = Some(renderer);
    }
}
