use crate::render::{TILE_PIXELS, TextureFX, TextureFXFields};
use game::block::Block;
use game::math::math_helper;
use std::cell::RefCell;

const PI_F: f32 = std::f64::consts::PI as f32;

pub struct LavaSimulation
{
    m_current: RefCell<Vec<f32>>,
    m_next: RefCell<Vec<f32>>,
    m_heat: RefCell<Vec<f32>>,
    m_heatDelta: RefCell<Vec<f32>>,
}

impl LavaSimulation
{
    pub fn new() -> LavaSimulation
    {
        LavaSimulation {
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
            let mut heat = self.m_heat.borrow_mut();
            let mut heatDelta = self.m_heatDelta.borrow_mut();
            for i in 0..16i32
            {
                for j in 0..16i32
                {
                    let mut f: f32 = 0.0;
                    let k: i32 = (math_helper::sin(j as f32 * PI_F * 2.0 / 16.0) * 1.2) as i32;
                    let l: i32 = (math_helper::sin(i as f32 * PI_F * 2.0 / 16.0) * 1.2) as i32;
                    for i1 in i - 1..=i + 1
                    {
                        for j1 in j - 1..=j + 1
                        {
                            let k1: i32 = i1 + k & 15;
                            let l1: i32 = j1 + l & 15;
                            f += current[(k1 + l1 * 16) as usize];
                        }
                    }
                    let at = |x: i32, y: i32| -> usize { ((x & 15) + (y & 15) * 16) as usize };
                    let index: usize = (i + j * 16) as usize;
                    next[index] = f / 10.0
                        + (heat[at(i, j)] + heat[at(i + 1, j)] + heat[at(i + 1, j + 1)] + heat[at(i, j + 1)]) / 4.0
                            * 0.8;
                    heat[index] += heatDelta[index] * 0.01;
                    if heat[index] < 0.0
                    {
                        heat[index] = 0.0;
                    }
                    heatDelta[index] -= 0.06;
                    if jvm::math::random() < 0.005
                    {
                        heatDelta[index] = 1.5;
                    }
                }
            }
        }
        self.m_next.swap(&self.m_current);
    }

    pub fn paint(&self, fx: &TextureFXFields, source: impl Fn(i32) -> i32)
    {
        let current = self.m_current.borrow();
        for j2 in 0..TILE_PIXELS as i32
        {
            let f1: f32 = (current[source(j2) as usize] * 2.0).clamp(0.0, 1.0);
            let f2: f32 = f1;
            let k2: i32 = (f2 * 100.0 + 155.0) as i32;
            let l2: i32 = (f2 * f2 * 255.0) as i32;
            let i3: i32 = (f2 * f2 * f2 * f2 * 128.0) as i32;
            let (r, g, b): (i32, i32, i32) = fx.tint(k2, l2, i3);
            fx.setPixel(j2 as usize, r, g, b, -1);
        }
    }
}

pub struct TextureLavaFX
{
    m_fx: TextureFXFields,
    m_simulation: LavaSimulation,
}

impl TextureLavaFX
{
    pub fn new() -> TextureLavaFX
    {
        TextureLavaFX {
            m_fx: TextureFXFields::new(Block::lavaMoving().m_blockIndexInTexture.get()),
            m_simulation: LavaSimulation::new(),
        }
    }
}

impl TextureFX for TextureLavaFX
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        self.m_simulation.step();
        self.m_simulation.paint(&self.m_fx, |index| index);
    }
}
