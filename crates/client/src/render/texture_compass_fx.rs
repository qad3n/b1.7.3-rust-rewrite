use crate::assets;
use crate::client::Minecraft;
use crate::render::{TILE_PIXELS, TextureFX, TextureFXFields};
use game::entity::Entity;
use game::item::Item;
use game::world::{ChunkCoordinates, World};
use jvm::image::{BufferedImage, ImageIO};
use jvm::io::{FileInputStream, IoResult};
use jvm::math;
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::Rc;

pub(super) fn readTile(path: &str, x: i32, y: i32, pixels: &mut [i32]) -> IoResult<()>
{
    let mut stream: Option<FileInputStream> = assets::getResourceAsStream(path);
    let image: BufferedImage = ImageIO::read(stream.as_mut())?.orNpe();
    image.getRGBRegion(x, y, 16, 16, pixels, 0, 16);
    Ok(())
}

pub(super) fn readItemIcon(iconIndex: i32, pixels: &mut [i32])
{
    if let Err(error) = readTile("/gui/items.png", iconIndex % 16 * 16, iconIndex / 16 * 16, pixels)
    {
        error.printStackTrace();
    }
}

pub(super) fn swing(angle: &Cell<f64>, angleDelta: &Cell<f64>, target: f64)
{
    let mut d: f64 = target - angle.get();
    while d < -PI
    {
        d += PI * 2.0;
    }
    while d >= PI
    {
        d -= PI * 2.0;
    }
    d = d.clamp(-1.0, 1.0);
    angleDelta.set(angleDelta.get() + d * 0.1);
    angleDelta.set(angleDelta.get() * 0.8);
    angle.set(angle.get() + angleDelta.get());
}

pub(super) fn netherSpin() -> f64 { math::random() * 3.1415927410125732 * 2.0 }

pub struct TextureCompassFX
{
    m_fx: TextureFXFields,
    m_mc: Rc<Minecraft>,
    m_compassIconImageData: RefCell<Vec<i32>>,
    m_angle: Cell<f64>,
    m_angleDelta: Cell<f64>,
}

impl TextureCompassFX
{
    pub fn new(minecraft: &Rc<Minecraft>) -> TextureCompassFX
    {
        let fx: TextureFXFields = TextureFXFields::new(Item::compass().getIconFromDamage(0));
        fx.m_tileImage.set(1);
        let mut icon: Vec<i32> = vec![0; TILE_PIXELS];
        readItemIcon(fx.m_iconIndex.get(), &mut icon);
        TextureCompassFX {
            m_fx: fx,
            m_mc: Rc::clone(minecraft),
            m_compassIconImageData: RefCell::new(icon),
            m_angle: Cell::new(0.0),
            m_angleDelta: Cell::new(0.0),
        }
    }

    fn drawNeedlePixel(&self, x: i32, y: i32, color: (i32, i32, i32))
    {
        let (r, g, b): (i32, i32, i32) = self.m_fx.tint(color.0, color.1, color.2);
        self.m_fx.setPixel((y * 16 + x) as usize, r, g, b, 255);
    }
}

impl TextureFX for TextureCompassFX
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        for (i, &pixel) in self.m_compassIconImageData.borrow().iter().enumerate()
        {
            let (r, g, b): (i32, i32, i32) = self.m_fx.tint(pixel >> 16 & 0xFF, pixel >> 8 & 0xFF, pixel & 0xFF);
            self.m_fx.setPixel(i, r, g, b, pixel >> 24 & 0xFF);
        }
        let mut d2: f64 = 0.0;
        if let (Some(world), Some(player)) = (self.m_mc.m_theWorld.get(), self.m_mc.m_thePlayer.get())
        {
            let world: Rc<World> = world;
            let player: Rc<Entity> = player;
            let chunkCoordinates: ChunkCoordinates = world.getSpawnPoint();
            let d3: f64 = chunkCoordinates.m_x as f64 - player.posX();
            let d4: f64 = chunkCoordinates.m_z as f64 - player.posZ();
            d2 = (player.m_rotationYaw.get() - 90.0) as f64 * PI / 180.0 - math::atan2(d4, d3);
            if world.worldProvider().m_isNether.get()
            {
                d2 = netherSpin();
            }
        }
        swing(&self.m_angle, &self.m_angleDelta, d2);
        let d5: f64 = math::sin(self.m_angle.get());
        let d6: f64 = math::cos(self.m_angle.get());
        for n11 in -4..=4
        {
            let n10: i32 = (8.5 + d6 * n11 as f64 * 0.3) as i32;
            let n9: i32 = (7.5 - d5 * n11 as f64 * 0.3 * 0.5) as i32;
            self.drawNeedlePixel(n10, n9, (100, 100, 100));
        }
        for n11 in -8..=16
        {
            let n10: i32 = (8.5 + d5 * n11 as f64 * 0.3) as i32;
            let n9: i32 = (7.5 + d6 * n11 as f64 * 0.3 * 0.5) as i32;
            let color: (i32, i32, i32) = if n11 >= 0
            {
                (255, 20, 20)
            }
            else
            {
                (100, 100, 100)
            };
            self.drawNeedlePixel(n10, n9, color);
        }
    }
}
