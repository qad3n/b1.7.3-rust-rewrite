use crate::render::{TILE_BYTES, TILE_PIXELS, TextureFX, TextureFXFields};
use game::block::Block;
use game::math::math_helper;
use jvm::random::Random;
use std::cell::Cell;

const PI_F: f32 = std::f64::consts::PI as f32;
const FRAMES: usize = 32;
const PORTAL_SEED: i64 = 100;

pub struct TexturePortalFX
{
    m_fx: TextureFXFields,
    m_portalTickCounter: Cell<i32>,
    m_portalTextureData: Vec<Vec<i8>>,
}

fn renderFrame(random: &Random, frame: usize) -> Vec<i8>
{
    let mut data: Vec<i8> = vec![0; TILE_BYTES];
    for j in 0..16
    {
        for k in 0..16
        {
            let mut f: f32 = 0.0;
            for l in 0..2
            {
                let f1: f32 = (l * 8) as f32;
                let f2: f32 = (l * 8) as f32;
                let mut f3: f32 = (j as f32 - f1) / 16.0 * 2.0;
                let mut f4: f32 = (k as f32 - f2) / 16.0 * 2.0;
                if f3 < -1.0
                {
                    f3 += 2.0;
                }
                if f3 >= 1.0
                {
                    f3 -= 2.0;
                }
                if f4 < -1.0
                {
                    f4 += 2.0;
                }
                if f4 >= 1.0
                {
                    f4 -= 2.0;
                }
                let f5: f32 = f3 * f3 + f4 * f4;
                let mut f6: f32 = jvm::math::atan2(f4 as f64, f3 as f64) as f32
                    + (frame as f32 / 32.0 * PI_F * 2.0 - f5 * 10.0 + (l * 2) as f32) * (l * 2 - 1) as f32;
                f6 = (math_helper::sin(f6) + 1.0) / 2.0;
                f6 /= f5 + 1.0;
                f += f6 * 0.5;
            }
            f += random.nextFloat() * 0.1;
            let i1: i32 = (f * 100.0 + 155.0) as i32;
            let j1: i32 = (f * f * 200.0 + 55.0) as i32;
            let k1: i32 = (f * f * f * f * 255.0) as i32;
            let l1: i32 = (f * 100.0 + 155.0) as i32;
            let i2: usize = k * 16 + j;
            data[i2 * 4] = j1 as i8;
            data[i2 * 4 + 1] = k1 as i8;
            data[i2 * 4 + 2] = i1 as i8;
            data[i2 * 4 + 3] = l1 as i8;
        }
    }
    data
}

impl TexturePortalFX
{
    pub fn new() -> TexturePortalFX
    {
        let fx: TextureFXFields = TextureFXFields::new(Block::portal().m_blockIndexInTexture.get());
        let random: Random = Random::withSeed(PORTAL_SEED);
        let frames: Vec<Vec<i8>> = (0..FRAMES).map(|frame| renderFrame(&random, frame)).collect();
        TexturePortalFX { m_fx: fx, m_portalTickCounter: Cell::new(0), m_portalTextureData: frames }
    }
}

impl TextureFX for TexturePortalFX
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        self.m_portalTickCounter.set(self.m_portalTickCounter.get() + 1);
        let abyte: &[i8] = &self.m_portalTextureData[(self.m_portalTickCounter.get() & 31) as usize];
        for i in 0..TILE_PIXELS
        {
            let j: i32 = abyte[i * 4] as i32 & 255;
            let k: i32 = abyte[i * 4 + 1] as i32 & 255;
            let l: i32 = abyte[i * 4 + 2] as i32 & 255;
            let i1: i32 = abyte[i * 4 + 3] as i32 & 255;
            let (r, g, b): (i32, i32, i32) = self.m_fx.tint(j, k, l);
            self.m_fx.setPixel(i, r, g, b, i1);
        }
    }
}
