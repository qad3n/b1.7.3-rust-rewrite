use crate::render::texture_lava_fx::LavaSimulation;
use crate::render::{TextureFX, TextureFXFields};
use game::block::Block;
use std::cell::Cell;

pub struct TextureLavaFlowFX
{
    m_fx: TextureFXFields,
    m_simulation: LavaSimulation,
    m_ticks: Cell<i32>,
}

impl TextureLavaFlowFX
{
    pub fn new() -> TextureLavaFlowFX
    {
        let fx: TextureFXFields = TextureFXFields::new(Block::lavaMoving().m_blockIndexInTexture.get() + 1);
        fx.m_tileSize.set(2);
        TextureLavaFlowFX { m_fx: fx, m_simulation: LavaSimulation::new(), m_ticks: Cell::new(0) }
    }
}

impl TextureFX for TextureLavaFlowFX
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        self.m_ticks.set(self.m_ticks.get() + 1);
        self.m_simulation.step();
        let ticks: i32 = self.m_ticks.get();
        self.m_simulation.paint(&self.m_fx, |index| index - ticks / 3 * 16 & 0xFF);
    }
}
