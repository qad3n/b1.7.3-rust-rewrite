use crate::entity::Entity;
use crate::math::Vec3D;
use crate::math::math_helper::{self, floor_double};
use crate::world::{World, WorldInfo};
use crate::worldgen::BiomeGenBase;
use std::rc::Rc;

const PI_F: f32 = std::f32::consts::PI;

pub fn setSpawnLocation(this: &Rc<World>)
{
    let info: Rc<WorldInfo> = this.worldInfo();
    if info.getSpawnY() <= 0
    {
        info.setSpawnY(64);
    }
    let mut i: i32 = info.getSpawnX();
    let mut j: i32 = info.getSpawnZ();
    while this.getFirstUncoveredBlock(i, j) == 0
    {
        i += this.m_rand.nextIntBounded(8) - this.m_rand.nextIntBounded(8);
        j += this.m_rand.nextIntBounded(8) - this.m_rand.nextIntBounded(8);
    }
    info.setSpawnX(i);
    info.setSpawnZ(j);
}

pub fn tick(this: &Rc<World>)
{
    this.updateWeather();
    if this.isAllPlayersFullyAsleep()
    {
        let mut flag: bool = false;
        if this.m_spawnHostileMobs.get() && this.m_difficultySetting.get() >= 1
        {
            flag = crate::entity::SpawnerAnimals::performSleepSpawning(this, &this.m_playerEntities.toVec());
        }
        if !flag
        {
            let i: i64 = this.worldInfo().getWorldTime() + 24000;
            this.worldInfo().setWorldTime(i - i % 24000);
            this.wakeUpAllPlayers();
        }
    }

    crate::entity::SpawnerAnimals::performSpawning(this, this.m_spawnHostileMobs.get(), this.m_spawnPeacefulMobs.get());
    this.chunkProvider().unload100OldestChunks();
    let j: i32 = this.calculateSkylightSubtracted(1.0);
    if j != this.m_skylightSubtracted.get()
    {
        this.m_skylightSubtracted.set(j);
        let mut k: i32 = 0;
        while k < this.m_worldAccesses.size()
        {
            this.m_worldAccesses.get(k).updateAllRenderers();
            k += 1;
        }
    }

    let l: i64 = this.worldInfo().getWorldTime() + 1;
    if l % this.m_autosavePeriod.get() as i64 == 0
    {
        this.saveWorld(false, None);
    }
    this.worldInfo().setWorldTime(l);
    this.TickUpdates(false);
    this.updateBlocksAndPlayCaveSounds();
}

pub fn updateWeather(this: &Rc<World>)
{
    if this.worldProvider().m_hasNoSky.get()
    {
        return;
    }

    let info: Rc<WorldInfo> = this.worldInfo();
    if this.m_thunderCooldown.get() > 0
    {
        this.m_thunderCooldown.set(this.m_thunderCooldown.get() - 1);
    }

    let mut i: i32 = info.getThunderTime();
    if i <= 0
    {
        if info.getThundering()
        {
            info.setThunderTime(this.m_rand.nextIntBounded(12000) + 3600);
        }
        else
        {
            info.setThunderTime(this.m_rand.nextIntBounded(168000) + 12000);
        }
    }
    else
    {
        i -= 1;
        info.setThunderTime(i);
        if i <= 0
        {
            info.setThundering(!info.getThundering());
        }
    }

    let mut j: i32 = info.getRainTime();
    if j <= 0
    {
        if info.getRaining()
        {
            info.setRainTime(this.m_rand.nextIntBounded(12000) + 12000);
        }
        else
        {
            info.setRainTime(this.m_rand.nextIntBounded(168000) + 12000);
        }
    }
    else
    {
        j -= 1;
        info.setRainTime(j);
        if j <= 0
        {
            info.setRaining(!info.getRaining());
        }
    }

    this.m_prevRainingStrength.set(this.m_rainingStrength.get());
    if info.getRaining()
    {
        this.m_rainingStrength.set((this.m_rainingStrength.get() as f64 + 0.01) as f32);
    }
    else
    {
        this.m_rainingStrength.set((this.m_rainingStrength.get() as f64 - 0.01) as f32);
    }
    if this.m_rainingStrength.get() < 0.0
    {
        this.m_rainingStrength.set(0.0);
    }
    if this.m_rainingStrength.get() > 1.0
    {
        this.m_rainingStrength.set(1.0);
    }

    this.m_prevThunderingStrength.set(this.m_thunderingStrength.get());
    if info.getThundering()
    {
        this.m_thunderingStrength.set((this.m_thunderingStrength.get() as f64 + 0.01) as f32);
    }
    else
    {
        this.m_thunderingStrength.set((this.m_thunderingStrength.get() as f64 - 0.01) as f32);
    }
    if this.m_thunderingStrength.get() < 0.0
    {
        this.m_thunderingStrength.set(0.0);
    }
    if this.m_thunderingStrength.get() > 1.0
    {
        this.m_thunderingStrength.set(1.0);
    }
}

fn clamp01(value: f32) -> f32
{
    let mut value: f32 = value;
    if value < 0.0
    {
        value = 0.0;
    }
    if value > 1.0
    {
        value = 1.0;
    }
    value
}

impl World
{
    pub fn initWeather(&self)
    {
        let info: Rc<WorldInfo> = self.worldInfo();
        if info.getRaining()
        {
            self.m_rainingStrength.set(1.0);
            if info.getThundering()
            {
                self.m_thunderingStrength.set(1.0);
            }
        }
    }

    pub fn stopPrecipitation(&self)
    {
        let info: Rc<WorldInfo> = self.worldInfo();
        info.setRainTime(0);
        info.setRaining(false);
        info.setThunderTime(0);
        info.setThundering(false);
    }

    pub fn calculateSkylightSubtracted(&self, tickDelta: f32) -> i32
    {
        let f: f32 = self.getCelestialAngle(tickDelta);
        let mut f1: f32 = clamp01(1.0 - (math_helper::cos(f * PI_F * 2.0) * 2.0 + 0.5));
        f1 = 1.0 - f1;
        f1 = (f1 as f64 * (1.0 - (self.getRain(tickDelta) * 5.0) as f64 / 16.0)) as f32;
        f1 = (f1 as f64 * (1.0 - (self.getThunder(tickDelta) * 5.0) as f64 / 16.0)) as f32;
        f1 = 1.0 - f1;
        (f1 * 11.0) as i32
    }

    pub fn getSkyColor(&self, entity: &Entity, tickDelta: f32) -> Rc<Vec3D>
    {
        let f: f32 = self.getCelestialAngle(tickDelta);
        let f1: f32 = clamp01(math_helper::cos(f * PI_F * 2.0) * 2.0 + 0.5);
        let i: i32 = floor_double(entity.posX());
        let j: i32 = floor_double(entity.posZ());
        let f2: f32 = self.getWorldChunkManager().getTemperature(i, j) as f32;
        let biome: Rc<BiomeGenBase> = self.getWorldChunkManager().getBiomeGenAt(i, j);
        let k: i32 = biome.getSkyColorByTemp(f2);
        let mut f3: f32 = (k >> 16 & 0xFF) as f32 / 255.0;
        let mut f4: f32 = (k >> 8 & 0xFF) as f32 / 255.0;
        let mut f5: f32 = (k & 0xFF) as f32 / 255.0;
        f3 *= f1;
        f4 *= f1;
        f5 *= f1;
        let f6: f32 = self.getRain(tickDelta);
        if f6 > 0.0
        {
            let f7: f32 = (f3 * 0.3 + f4 * 0.59 + f5 * 0.11) * 0.6;
            let f8: f32 = 1.0 - f6 * 0.75;
            f3 = f3 * f8 + f7 * (1.0 - f8);
            f4 = f4 * f8 + f7 * (1.0 - f8);
            f5 = f5 * f8 + f7 * (1.0 - f8);
        }
        let f10: f32 = self.getThunder(tickDelta);
        if f10 > 0.0
        {
            let f11: f32 = (f3 * 0.3 + f4 * 0.59 + f5 * 0.11) * 0.2;
            let f9: f32 = 1.0 - f10 * 0.75;
            f3 = f3 * f9 + f11 * (1.0 - f9);
            f4 = f4 * f9 + f11 * (1.0 - f9);
            f5 = f5 * f9 + f11 * (1.0 - f9);
        }
        if self.m_lightningCooldown.get() > 0
        {
            let mut f12: f32 = self.m_lightningCooldown.get() as f32 - tickDelta;
            if f12 > 1.0
            {
                f12 = 1.0;
            }
            f12 *= 0.45;
            f3 = f3 * (1.0 - f12) + 0.8 * f12;
            f4 = f4 * (1.0 - f12) + 0.8 * f12;
            f5 = f5 * (1.0 - f12) + 1.0 * f12;
        }
        Vec3D::createVector(f3 as f64, f4 as f64, f5 as f64)
    }

    pub fn getCelestialAngle(&self, tickDelta: f32) -> f32
    {
        self.worldProvider().calculateCelestialAngle(self.worldInfo().getWorldTime(), tickDelta)
    }

    pub fn getCloudColor(&self, tickDelta: f32) -> Rc<Vec3D>
    {
        let f: f32 = self.getCelestialAngle(tickDelta);
        let f1: f32 = clamp01(math_helper::cos(f * PI_F * 2.0) * 2.0 + 0.5);
        let cloud: i64 = self.m_cloudColor.get();
        let mut f2: f32 = (cloud >> 16 & 255) as f32 / 255.0;
        let mut f3: f32 = (cloud >> 8 & 255) as f32 / 255.0;
        let mut f4: f32 = (cloud & 255) as f32 / 255.0;
        let f5: f32 = self.getRain(tickDelta);
        if f5 > 0.0
        {
            let f6: f32 = (f2 * 0.3 + f3 * 0.59 + f4 * 0.11) * 0.6;
            let f7: f32 = 1.0 - f5 * 0.95;
            f2 = f2 * f7 + f6 * (1.0 - f7);
            f3 = f3 * f7 + f6 * (1.0 - f7);
            f4 = f4 * f7 + f6 * (1.0 - f7);
        }
        f2 *= f1 * 0.9 + 0.1;
        f3 *= f1 * 0.9 + 0.1;
        f4 *= f1 * 0.85 + 0.15;
        let f9: f32 = self.getThunder(tickDelta);
        if f9 > 0.0
        {
            let f10: f32 = (f2 * 0.3 + f3 * 0.59 + f4 * 0.11) * 0.2;
            let f8: f32 = 1.0 - f9 * 0.95;
            f2 = f2 * f8 + f10 * (1.0 - f8);
            f3 = f3 * f8 + f10 * (1.0 - f8);
            f4 = f4 * f8 + f10 * (1.0 - f8);
        }
        Vec3D::createVector(f2 as f64, f3 as f64, f4 as f64)
    }

    pub fn getFogColor(&self, tickDelta: f32) -> Rc<Vec3D>
    {
        let f: f32 = self.getCelestialAngle(tickDelta);
        self.worldProvider().getFogColor(f, tickDelta)
    }

    pub fn getStarBrightness(&self, tickDelta: f32) -> f32
    {
        let f: f32 = self.getCelestialAngle(tickDelta);
        let f1: f32 = clamp01(1.0 - (math_helper::cos(f * PI_F * 2.0) * 2.0 + 0.75));
        f1 * f1 * 0.5
    }

    pub fn getThunder(&self, tickDelta: f32) -> f32
    {
        (self.m_prevThunderingStrength.get()
            + (self.m_thunderingStrength.get() - self.m_prevThunderingStrength.get()) * tickDelta)
            * self.getRain(tickDelta)
    }

    pub fn getRain(&self, tickDelta: f32) -> f32
    {
        self.m_prevRainingStrength.get() + (self.m_rainingStrength.get() - self.m_prevRainingStrength.get()) * tickDelta
    }

    pub fn setRain(&self, rain: f32)
    {
        self.m_prevRainingStrength.set(rain);
        self.m_rainingStrength.set(rain);
    }

    pub fn isThundering(&self) -> bool { self.getThunder(1.0) as f64 > 0.9 }

    pub fn isRaining(&self) -> bool { self.getRain(1.0) as f64 > 0.2 }

    pub fn canBlockBeRainedOn(&self, x: i32, y: i32, z: i32) -> bool
    {
        if !self.isRaining() || !self.canBlockSeeTheSky(x, y, z) || self.findTopSolidBlock(x, z) > y
        {
            return false;
        }
        let biomegenbase: Rc<BiomeGenBase> = self.getWorldChunkManager().getBiomeGenAt(x, z);
        if biomegenbase.getEnableSnow()
        {
            false
        }
        else
        {
            biomegenbase.canSpawnLightningBolt()
        }
    }
}
