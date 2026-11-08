use crate::render::{TILE_PIXELS, TextureFX, TextureFXFields};
use game::block::Block;
use std::cell::{Cell, RefCell};

pub struct WaterTuning
{
    pub m_neighbours: [(i32, i32); 3],
    pub m_spread: f32,
    pub m_cooling: f32,
    pub m_bubbleChance: f64,
}

const STILL_WATER: WaterTuning =
    WaterTuning { m_neighbours: [(-1, 0), (0, 0), (1, 0)], m_spread: 3.3, m_cooling: 0.1, m_bubbleChance: 0.05 };

pub struct WaterSimulation
{
    m_tuning: &'static WaterTuning,
    m_current: RefCell<Vec<f32>>,
    m_next: RefCell<Vec<f32>>,
    m_heat: RefCell<Vec<f32>>,
    m_heatDelta: RefCell<Vec<f32>>,
}

impl WaterSimulation
{
    pub fn new(tuning: &'static WaterTuning) -> WaterSimulation
    {
        WaterSimulation {
            m_tuning: tuning,
            m_current: RefCell::new(vec![0.0; TILE_PIXELS]),
            m_next: RefCell::new(vec![0.0; TILE_PIXELS]),
            m_heat: RefCell::new(vec![0.0; TILE_PIXELS]),
            m_heatDelta: RefCell::new(vec![0.0; TILE_PIXELS]),
        }
    }

    pub fn step(&self)
    {
        {
            let current = self.m_current.borrow();
            let mut next = self.m_next.borrow_mut();
            let heat = self.m_heat.borrow();
            for i in 0..16i32
            {
                for j in 0..16i32
                {
                    let mut f: f32 = 0.0;
                    for (dx, dy) in self.m_tuning.m_neighbours
                    {
                        let l: i32 = i + dx & 15;
                        let i1: i32 = j + dy & 15;
                        f += current[(l + i1 * 16) as usize];
                    }
                    let index: usize = (i + j * 16) as usize;
                    next[index] = f / self.m_tuning.m_spread + heat[index] * 0.8;
                }
            }
        }
        {
            let mut heat = self.m_heat.borrow_mut();
            let mut heatDelta = self.m_heatDelta.borrow_mut();
            for k2 in 0..16
            {
                for l2 in 0..16
                {
                    let index: usize = k2 + l2 * 16;
                    heat[index] += heatDelta[index] * 0.05;
                    if heat[index] < 0.0
                    {
                        heat[index] = 0.0;
                    }
                    heatDelta[index] -= self.m_tuning.m_cooling;
                    if jvm::math::random() < self.m_tuning.m_bubbleChance
                    {
                        heatDelta[index] = 0.5;
                    }
                }
            }
        }
        self.m_next.swap(&self.m_current);
    }

    pub fn paint(&self, fx: &TextureFXFields, source: impl Fn(i32) -> i32)
    {
        let current = self.m_current.borrow();
        for i3 in 0..TILE_PIXELS as i32
        {
            let f1: f32 = current[source(i3) as usize].clamp(0.0, 1.0);
            let f2: f32 = f1 * f1;
            let j3: i32 = (32.0 + f2 * 32.0) as i32;
            let k3: i32 = (50.0 + f2 * 64.0) as i32;
            let j1: i32 = 255;
            let k1: i32 = (146.0 + f2 * 50.0) as i32;
            let (r, g, b): (i32, i32, i32) = fx.tint(j3, k3, j1);
            fx.setPixel(i3 as usize, r, g, b, k1);
        }
    }
}

pub struct TextureWaterFX
{
    m_fx: TextureFXFields,
    m_simulation: WaterSimulation,
    m_tickCounter: Cell<i32>,
}

impl TextureWaterFX
{
    pub fn new() -> TextureWaterFX
    {
        TextureWaterFX {
            m_fx: TextureFXFields::new(Block::waterMoving().m_blockIndexInTexture.get()),
            m_simulation: WaterSimulation::new(&STILL_WATER),
            m_tickCounter: Cell::new(0),
        }
    }
}

impl TextureFX for TextureWaterFX
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        self.m_tickCounter.set(self.m_tickCounter.get() + 1);
        self.m_simulation.step();
        self.m_simulation.paint(&self.m_fx, |index| index);
    }
}
