use crate::gui::FontRenderer;
use crate::render::{RenderEngine, TileEntityRenderer};
use game::tileentity::TileEntity;
use game::world::World;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub trait TileEntitySpecialRenderer
{
    fn renderTileEntityAt(&self, tileEntity: &Rc<TileEntity>, dx: f64, dy: f64, dz: f64, tickDelta: f32);

    fn setWorld(&self, _world: Option<Rc<World>>) {}

    fn bindTextureByName(&self, path: &str)
    {
        let renderEngine: Rc<RenderEngine> = TileEntityRenderer::instance().m_renderEngine.get().orNpe();
        renderEngine.bindTexture(renderEngine.getTexture(path));
    }

    fn getFontRenderer(&self) -> Option<Rc<FontRenderer>> { TileEntityRenderer::instance().getFontRenderer() }
}
