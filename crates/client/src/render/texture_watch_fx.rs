use crate::client::Minecraft;
use crate::render::texture_compass_fx::{netherSpin, readTile, swing};
use crate::render::{TILE_PIXELS, TextureFX, TextureFXFields};
use game::item::Item;
use game::world::World;
use jvm::math;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const PI_F: f32 = std::f64::consts::PI as f32;

pub struct TextureWatchFX
{
    m_fx: TextureFXFields,
    m_mc: Rc<Minecraft>,
    m_watchIconImageData: RefCell<Vec<i32>>,
    m_dialImageData: RefCell<Vec<i32>>,
    m_angle: Cell<f64>,
    m_angleDelta: Cell<f64>,
}

impl TextureWatchFX
{
    pub fn new(minecraft: &Rc<Minecraft>) -> TextureWatchFX
    {
        let fx: TextureFXFields = TextureFXFields::new(Item::pocketSundial().getIconFromDamage(0));
        fx.m_tileImage.set(1);
        let mut icon: Vec<i32> = vec![0; TILE_PIXELS];
        let mut dial: Vec<i32> = vec![0; TILE_PIXELS];
        let n: i32 = fx.m_iconIndex.get() % 16 * 16;
        let n2: i32 = fx.m_iconIndex.get() / 16 * 16;
        if let Err(error) =
            readTile("/gui/items.png", n, n2, &mut icon).and_then(|()| readTile("/misc/dial.png", 0, 0, &mut dial))
        {
            error.printStackTrace();
        }
        TextureWatchFX {
            m_fx: fx,
            m_mc: Rc::clone(minecraft),
            m_watchIconImageData: RefCell::new(icon),
            m_dialImageData: RefCell::new(dial),
            m_angle: Cell::new(0.0),
            m_angleDelta: Cell::new(0.0),
        }
    }
}

impl TextureFX for TextureWatchFX
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        let mut d2: f64 = 0.0;
        if let (Some(world), Some(_)) = (self.m_mc.m_theWorld.get(), self.m_mc.m_thePlayer.get())
        {
            let world: Rc<World> = world;
            let f: f32 = world.getCelestialAngle(1.0);
            d2 = (-f * PI_F * 2.0) as f64;
            if world.worldProvider().m_isNether.get()
            {
                d2 = netherSpin();
            }
        }
        swing(&self.m_angle, &self.m_angleDelta, d2);
        let d3: f64 = math::sin(self.m_angle.get());
        let d4: f64 = math::cos(self.m_angle.get());
        let dial = self.m_dialImageData.borrow();
        for (i, &pixel) in self.m_watchIconImageData.borrow().iter().enumerate()
        {
            let mut n: i32 = pixel >> 24 & 0xFF;
            let mut n2: i32 = pixel >> 16 & 0xFF;
            let mut n3: i32 = pixel >> 8 & 0xFF;
            let mut n4: i32 = pixel & 0xFF;
            if n2 == n4 && n3 == 0 && n4 > 0
            {
                let d5: f64 = -((i % 16) as f64 / 15.0 - 0.5);
                let d6: f64 = (i / 16) as f64 / 15.0 - 0.5;
                let n5: i32 = n2;
                let n6: i32 = ((d5 * d4 + d6 * d3 + 0.5) * 16.0) as i32;
                let n7: i32 = ((d6 * d4 - d5 * d3 + 0.5) * 16.0) as i32;
                let face: i32 = dial[((n6 & 0xF) + (n7 & 0xF) * 16) as usize];
                n = face >> 24 & 0xFF;
                n2 = (face >> 16 & 0xFF) * n5 / 255;
                n3 = (face >> 8 & 0xFF) * n5 / 255;
                n4 = (face & 0xFF) * n5 / 255;
            }
            let (r, g, b): (i32, i32, i32) = self.m_fx.tint(n2, n3, n4);
            self.m_fx.setPixel(i, r, g, b, n);
        }
    }
}
