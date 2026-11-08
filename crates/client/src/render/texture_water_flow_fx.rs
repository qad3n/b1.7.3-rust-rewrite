use crate::render::texture_water_fx::{WaterSimulation, WaterTuning};
use crate::render::{TextureFX, TextureFXFields};
use game::block::Block;
use std::cell::Cell;

const FLOWING_WATER: WaterTuning =
    WaterTuning { m_neighbours: [(0, -2), (0, -1), (0, 0)], m_spread: 3.2, m_cooling: 0.3, m_bubbleChance: 0.2 };

pub struct TextureWaterFlowFX
{
    m_fx: TextureFXFields,
    m_simulation: WaterSimulation,
    m_ticks: Cell<i32>,
}

impl TextureWaterFlowFX
{
    pub fn new() -> TextureWaterFlowFX
    {
        let fx: TextureFXFields = TextureFXFields::new(Block::waterMoving().m_blockIndexInTexture.get() + 1);
        fx.m_tileSize.set(2);
        TextureWaterFlowFX { m_fx: fx, m_simulation: WaterSimulation::new(&FLOWING_WATER), m_ticks: Cell::new(0) }
    }
}

impl TextureFX for TextureWaterFlowFX
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        self.m_ticks.set(self.m_ticks.get() + 1);
        self.m_simulation.step();
        let ticks: i32 = self.m_ticks.get();
        self.m_simulation.paint(&self.m_fx, |index| index - ticks * 16 & 0xFF);
    }
}
