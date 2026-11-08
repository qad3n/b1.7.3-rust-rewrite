use crate::render::particle::{entity_rain_fx, entity_smoke_fx};
use crate::render::{EntityRenderer, Tessellator};
use game::block::{Block, Material};
use game::entity::Entity;
use game::math::math_helper;
use game::world::World;
use game::worldgen::BiomeGenBase;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::rc::Rc;

fn columnSeed(x: i32, z: i32) -> i32
{
    x.wrapping_mul(x)
        .wrapping_mul(3121)
        .wrapping_add(x.wrapping_mul(45238971))
        .wrapping_add(z.wrapping_mul(z).wrapping_mul(418711))
        .wrapping_add(z.wrapping_mul(13761))
}

impl EntityRenderer
{
    pub(super) fn addRainParticles(&self)
    {
        let world: Rc<World> = self.theWorld();
        let mut f: f32 = world.getRain(1.0);
        if !self.gameSettings().m_fancyGraphics.get()
        {
            f /= 2.0;
        }
        if f == 0.0
        {
            return;
        }
        self.m_random.setSeed(self.m_rendererUpdateCount.get() as i64 * 312987231);
        let entityLiving: Rc<Entity> = self.renderViewEntity();
        let n: i32 = math_helper::floor_double(entityLiving.posX());
        let n2: i32 = math_helper::floor_double(entityLiving.posY());
        let n3: i32 = math_helper::floor_double(entityLiving.posZ());
        let n4: i32 = 10;
        let mut d: f64 = 0.0;
        let mut d2: f64 = 0.0;
        let mut d3: f64 = 0.0;
        let mut n5: i32 = 0;
        let effectRenderer = self.m_mc.m_effectRenderer.get().orNpe();
        for _ in 0..(100.0 * f * f) as i32
        {
            let n6: i32 = n + self.m_random.nextIntBounded(n4) - self.m_random.nextIntBounded(n4);
            let n7: i32 = n3 + self.m_random.nextIntBounded(n4) - self.m_random.nextIntBounded(n4);
            let n8: i32 = world.findTopSolidBlock(n6, n7);
            let n9: i32 = world.getBlockId(n6, n8 - 1, n7);
            if n8 > n2 + n4
                || n8 < n2 - n4
                || !world.getWorldChunkManager().getBiomeGenAt(n6, n7).canSpawnLightningBolt()
            {
                continue;
            }
            let f2: f32 = self.m_random.nextFloat();
            let f3: f32 = self.m_random.nextFloat();
            if n9 <= 0
            {
                continue;
            }
            let block: Rc<Block> = Block::byId(n9).orNpe();
            let x: f64 = (n6 as f32 + f2) as f64;
            let y: f64 = (n8 as f32 + 0.1) as f64 - block.m_minY.get();
            let z: f64 = (n7 as f32 + f3) as f64;
            if Rc::ptr_eq(&block.m_blockMaterial, &Material::lava())
            {
                effectRenderer.addEffect(entity_smoke_fx::new(&world, x, y, z, 0.0, 0.0, 0.0));
                continue;
            }
            n5 += 1;
            if self.m_random.nextIntBounded(n5) == 0
            {
                d = x;
                d2 = y;
                d3 = z;
            }
            effectRenderer.addEffect(entity_rain_fx::new(&world, x, y, z));
        }
        if n5 > 0
        {
            let counter: i32 = self.m_rainSoundCounter.get();
            self.m_rainSoundCounter.set(counter + 1);
            if self.m_random.nextIntBounded(3) < counter
            {
                self.m_rainSoundCounter.set(0);
                let top: i32 = world.findTopSolidBlock(
                    math_helper::floor_double(entityLiving.posX()),
                    math_helper::floor_double(entityLiving.posZ()),
                );
                if d2 > entityLiving.posY() + 1.0 && top > math_helper::floor_double(entityLiving.posY())
                {
                    self.theWorld().playSoundEffect(d, d2, d3, "ambient.weather.rain", 0.1, 0.5);
                }
                else
                {
                    self.theWorld().playSoundEffect(d, d2, d3, "ambient.weather.rain", 0.2, 1.0);
                }
            }
        }
    }

    pub(super) fn renderRainSnow(&self, tickDelta: f32)
    {
        let world: Rc<World> = self.theWorld();
        let f2: f32 = world.getRain(tickDelta);
        if f2 <= 0.0
        {
            return;
        }
        let entityLiving: Rc<Entity> = self.renderViewEntity();
        let n6: i32 = math_helper::floor_double(entityLiving.posX());
        let n7: i32 = math_helper::floor_double(entityLiving.posY());
        let n8: i32 = math_helper::floor_double(entityLiving.posZ());
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let renderEngine = self.m_mc.m_renderEngine.get().orNpe();
        GL11::glDisable(2884);
        GL11::glNormal3f(0.0, 1.0, 0.0);
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        GL11::glAlphaFunc(516, 0.01);
        GL11::glBindTexture(3553, renderEngine.getTexture("/environment/snow.png"));
        let delta: f64 = tickDelta as f64;
        let d: f64 =
            entityLiving.m_lastTickPosX.get() + (entityLiving.posX() - entityLiving.m_lastTickPosX.get()) * delta;
        let d2: f64 =
            entityLiving.m_lastTickPosY.get() + (entityLiving.posY() - entityLiving.m_lastTickPosY.get()) * delta;
        let d3: f64 =
            entityLiving.m_lastTickPosZ.get() + (entityLiving.posZ() - entityLiving.m_lastTickPosZ.get()) * delta;
        let n9: i32 = math_helper::floor_double(d2);
        let n10: i32 = if self.gameSettings().m_fancyGraphics.get()
        {
            10
        }
        else
        {
            5
        };
        let biomes: Vec<Option<Rc<BiomeGenBase>>> =
            world.getWorldChunkManager().getBiomes(n6 - n10, n8 - n10, n10 * 2 + 1, n10 * 2 + 1);
        let updateCount: i32 = self.m_rendererUpdateCount.get();
        let mut n11: usize = 0;
        for n5 in (n6 - n10)..=(n6 + n10)
        {
            for n4 in (n8 - n10)..=(n8 + n10)
            {
                let biome: Rc<BiomeGenBase> = biomes[n11].clone().orNpe();
                n11 += 1;
                if !biome.getEnableSnow()
                {
                    continue;
                }
                let mut n3: i32 = world.findTopSolidBlock(n5, n4);
                if n3 < 0
                {
                    n3 = 0;
                }
                let n2: i32 = n3.max(n9);
                let mut n: i32 = n7 - n10;
                let mut n12: i32 = n7 + n10;
                if n < n3
                {
                    n = n3;
                }
                if n12 < n3
                {
                    n12 = n3;
                }
                let f: f32 = 1.0;
                if n == n12
                {
                    continue;
                }
                self.m_random.setSeed(columnSeed(n5, n4) as i64);
                let f3: f32 = updateCount as f32 + tickDelta;
                let f4: f32 = ((updateCount & 0x1FF) as f32 + tickDelta) / 512.0;
                let f5: f32 = self.m_random.nextFloat() + f3 * 0.01 * self.m_random.nextGaussian() as f32;
                let f6: f32 = self.m_random.nextFloat() + f3 * self.m_random.nextGaussian() as f32 * 0.001;
                let d4: f64 = (n5 as f32 + 0.5) as f64 - entityLiving.posX();
                let d5: f64 = (n4 as f32 + 0.5) as f64 - entityLiving.posZ();
                let f7: f32 = math_helper::sqrt_double(d4 * d4 + d5 * d5) / n10 as f32;
                tessellator.startDrawingQuads();
                let f8: f32 = world.getLightBrightness(n5, n2, n4);
                GL11::glColor4f(f8, f8, f8, ((1.0 - f7 * f7) * 0.3 + 0.5) * f2);
                tessellator.setTranslationD(-d * 1.0, -d2 * 1.0, -d3 * 1.0);
                let v0: f64 = (n as f32 * f / 4.0 + f4 * f + f6) as f64;
                let v1: f64 = (n12 as f32 * f / 4.0 + f4 * f + f6) as f64;
                let u0: f64 = (0.0 * f + f5) as f64;
                let u1: f64 = (1.0 * f + f5) as f64;
                Self::drawColumn(&tessellator, n5, n4, n, n12, u0, u1, v0, v1);
            }
        }
        GL11::glBindTexture(3553, renderEngine.getTexture("/environment/rain.png"));
        let n10: i32 = if self.gameSettings().m_fancyGraphics.get()
        {
            10
        }
        else
        {
            n10
        };
        let mut n11: usize = 0;
        for n5 in (n6 - n10)..=(n6 + n10)
        {
            for n4 in (n8 - n10)..=(n8 + n10)
            {
                let biome: Rc<BiomeGenBase> = biomes[n11].clone().orNpe();
                n11 += 1;
                if !biome.canSpawnLightningBolt()
                {
                    continue;
                }
                let n3: i32 = world.findTopSolidBlock(n5, n4);
                let mut n2: i32 = n7 - n10;
                let mut n: i32 = n7 + n10;
                if n2 < n3
                {
                    n2 = n3;
                }
                if n < n3
                {
                    n = n3;
                }
                let f9: f32 = 1.0;
                if n2 == n
                {
                    continue;
                }
                let seed: i32 = columnSeed(n5, n4);
                self.m_random.setSeed(seed as i64);
                let f: f32 = ((updateCount.wrapping_add(seed) & 0x1F) as f32 + tickDelta) / 32.0
                    * (3.0 + self.m_random.nextFloat());
                let d6: f64 = (n5 as f32 + 0.5) as f64 - entityLiving.posX();
                let d7: f64 = (n4 as f32 + 0.5) as f64 - entityLiving.posZ();
                let f10: f32 = math_helper::sqrt_double(d6 * d6 + d7 * d7) / n10 as f32;
                tessellator.startDrawingQuads();
                let f11: f32 = world.getLightBrightness(n5, 128, n4) * 0.85 + 0.15;
                GL11::glColor4f(f11, f11, f11, ((1.0 - f10 * f10) * 0.5 + 0.5) * f2);
                tessellator.setTranslationD(-d * 1.0, -d2 * 1.0, -d3 * 1.0);
                let v0: f64 = (n2 as f32 * f9 / 4.0 + f * f9) as f64;
                let v1: f64 = (n as f32 * f9 / 4.0 + f * f9) as f64;
                Self::drawColumn(&tessellator, n5, n4, n2, n, (0.0 * f9) as f64, (1.0 * f9) as f64, v0, v1);
            }
        }
        GL11::glEnable(2884);
        GL11::glDisable(3042);
        GL11::glAlphaFunc(516, 0.1);
    }

    fn drawColumn(tessellator: &Tessellator, x: i32, z: i32, bottom: i32, top: i32, u0: f64, u1: f64, v0: f64, v1: f64)
    {
        let (x, z): (f64, f64) = (x as f64, z as f64);
        let (bottom, top): (f64, f64) = (bottom as f64, top as f64);
        tessellator.addVertexWithUV(x + 0.0, bottom, z + 0.5, u0, v0);
        tessellator.addVertexWithUV(x + 1.0, bottom, z + 0.5, u1, v0);
        tessellator.addVertexWithUV(x + 1.0, top, z + 0.5, u1, v1);
        tessellator.addVertexWithUV(x + 0.0, top, z + 0.5, u0, v1);
        tessellator.addVertexWithUV(x + 0.5, bottom, z + 0.0, u0, v0);
        tessellator.addVertexWithUV(x + 0.5, bottom, z + 1.0, u1, v0);
        tessellator.addVertexWithUV(x + 0.5, top, z + 1.0, u1, v1);
        tessellator.addVertexWithUV(x + 0.5, top, z + 0.0, u0, v1);
        tessellator.setTranslationD(0.0, 0.0, 0.0);
        tessellator.draw();
    }
}
