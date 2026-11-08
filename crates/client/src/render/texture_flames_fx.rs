use crate::render::{TILE_PIXELS, TextureFX, TextureFXFields};
use game::block::Block;
use jvm::math;
use std::cell::RefCell;

const FIRE_CELLS: usize = 320;
const FIRE_ROWS: i32 = 20;

pub struct TextureFlamesFX
{
    m_fx: TextureFXFields,
    m_current: RefCell<Vec<f32>>,
    m_next: RefCell<Vec<f32>>,
}

impl TextureFlamesFX
{
    pub fn new(variant: i32) -> TextureFlamesFX
    {
        TextureFlamesFX {
            m_fx: TextureFXFields::new(Block::fire().m_blockIndexInTexture.get() + variant * 16),
            m_current: RefCell::new(vec![0.0; FIRE_CELLS]),
            m_next: RefCell::new(vec![0.0; FIRE_CELLS]),
        }
    }
}

impl TextureFX for TextureFlamesFX
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        {
            let current = self.m_current.borrow();
            let mut next = self.m_next.borrow_mut();
            for i in 0..16i32
            {
                for j in 0..FIRE_ROWS
                {
                    let mut k: i32 = 18;
                    let mut f: f32 = current[(i + (j + 1) % FIRE_ROWS * 16) as usize] * k as f32;
                    for l in i - 1..=i + 1
                    {
                        for i1 in j..=j + 1
                        {
                            if l >= 0 && i1 >= 0 && l < 16 && i1 < FIRE_ROWS
                            {
                                f += current[(l + i1 * 16) as usize];
                            }
                            k += 1;
                        }
                    }
                    let index: usize = (i + j * 16) as usize;
                    next[index] = f / (k as f32 * 1.06);
                    if j >= FIRE_ROWS - 1
                    {
                        next[index] = (math::random() * math::random() * math::random() * 4.0
                            + math::random() * 0.1f32 as f64
                            + 0.2f32 as f64) as f32;
                    }
                }
            }
        }
        self.m_next.swap(&self.m_current);
        let current = self.m_current.borrow();
        for k2 in 0..TILE_PIXELS
        {
            let f1: f32 = (current[k2] * 1.8).clamp(0.0, 1.0);
            let f2: f32 = f1;
            let l2: i32 = (f2 * 155.0 + 100.0) as i32;
            let i3: i32 = (f2 * f2 * 255.0) as i32;
            let j3: i32 = (f2 * f2 * f2 * f2 * f2 * f2 * f2 * f2 * f2 * f2 * 255.0) as i32;
            let alpha: i32 = if f2 < 0.5 { 0 } else { 255 };
            let (r, g, b): (i32, i32, i32) = self.m_fx.tint(l2, i3, j3);
            self.m_fx.setPixel(k2, r, g, b, alpha);
        }
    }
}
