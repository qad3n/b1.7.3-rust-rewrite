use crate::render::particle::
{
    entity_bubble_fx, entity_explode_fx, entity_flame_fx, entity_foot_step_fx, entity_heart_fx, entity_lava_fx,
    entity_note_fx, entity_portal_fx, entity_reddust_fx, entity_slime_fx, entity_smoke_fx, entity_snow_shovel_fx,
    entity_splash_fx,
};
use crate::render::{EffectRenderer, ImageBufferDownload, RenderGlobal};
use game::block::{Block, StepSound};
use game::entity::Entity;
use game::item::{Item, item_record};
use game::tileentity::TileEntity;
use game::world::{IWorldAccess, World};
use jvm::random::Random;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::rc::Rc;

impl RenderGlobal
{
    fn obtainSkin(&self, url: Option<JString>)
    {
        if let Some(url) = url
        {
            self.m_renderEngine.obtainImageData(&url.to_string(), Some(Box::new(ImageBufferDownload::new())));
        }
    }

    fn releaseSkin(&self, url: Option<JString>)
    {
        if let Some(url) = url
        {
            self.m_renderEngine.releaseImageData(&url.to_string());
        }
    }

    fn smokeBurst(&self, random: &Random, x: i32, y: i32, z: i32, data: i32)
    {
        let n: i32 = data % 3 - 1;
        let n2: i32 = data / 3 % 3 - 1;
        let d: f64 = x as f64 + n as f64 * 0.6 + 0.5;
        let d2: f64 = y as f64 + 0.5;
        let d3: f64 = z as f64 + n2 as f64 * 0.6 + 0.5;

        for _ in 0..10
        {
            let d4: f64 = random.nextDouble() * 0.2 + 0.01;
            let d5: f64 = d + n as f64 * 0.01 + (random.nextDouble() - 0.5) * n2 as f64 * 0.5;
            let d6: f64 = d2 + (random.nextDouble() - 0.5) * 0.5;
            let d7: f64 = d3 + n2 as f64 * 0.01 + (random.nextDouble() - 0.5) * n as f64 * 0.5;
            let d8: f64 = n as f64 * d4 + random.nextGaussian() * 0.01;
            let d9: f64 = -0.03 + random.nextGaussian() * 0.01;
            let d10: f64 = n2 as f64 * d4 + random.nextGaussian() * 0.01;
            self.spawnParticle("smoke", d5, d6, d7, d8, d9, d10);
        }
    }
}

type Make = fn(&RenderGlobal, &Rc<World>, [f64; 3], [f64; 3]) -> Rc<Entity>;

struct ParticleRow
{
    m_name: &'static str,
    m_make: Make,
}

// beta's particles by the name RenderGlobal.spawnParticle takes, mod particles build on one of these
const PARTICLES: [ParticleRow; 15] = [
    ParticleRow
    {
        m_name: "bubble",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_bubble_fx::new(w, x, y, z, vx, vy, vz),
    },
    ParticleRow
    {
        m_name: "smoke",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_smoke_fx::new(w, x, y, z, vx, vy, vz),
    },
    ParticleRow { m_name: "note", m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_note_fx::new(w, x, y, z, vx, vy, vz) },
    ParticleRow
    {
        m_name: "portal",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_portal_fx::new(w, x, y, z, vx, vy, vz),
    },
    ParticleRow
    {
        m_name: "explode",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_explode_fx::new(w, x, y, z, vx, vy, vz),
    },
    ParticleRow
    {
        m_name: "flame",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_flame_fx::new(w, x, y, z, vx, vy, vz),
    },
    ParticleRow { m_name: "lava", m_make: |_, w, [x, y, z], _| entity_lava_fx::new(w, x, y, z) },
    ParticleRow
    {
        m_name: "footstep",
        m_make: |global, w, [x, y, z], _| entity_foot_step_fx::new(&global.m_renderEngine, w, x, y, z),
    },
    ParticleRow
    {
        m_name: "splash",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_splash_fx::new(w, x, y, z, vx, vy, vz),
    },
    ParticleRow
    {
        m_name: "largesmoke",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_smoke_fx::newScaled(w, x, y, z, vx, vy, vz, 2.5),
    },
    ParticleRow
    {
        m_name: "reddust",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_reddust_fx::new(w, x, y, z, vx as f32, vy as f32, vz as f32),
    },
    ParticleRow
    {
        m_name: "snowballpoof",
        m_make: |_, w, [x, y, z], _| entity_slime_fx::new(w, x, y, z, &Item::snowball()),
    },
    ParticleRow
    {
        m_name: "snowshovel",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_snow_shovel_fx::new(w, x, y, z, vx, vy, vz),
    },
    ParticleRow { m_name: "slime", m_make: |_, w, [x, y, z], _| entity_slime_fx::new(w, x, y, z, &Item::slimeBall()) },
    ParticleRow
    {
        m_name: "heart",
        m_make: |_, w, [x, y, z], [vx, vy, vz]| entity_heart_fx::new(w, x, y, z, vx, vy, vz),
    },
];

// the beta particle names join the particle registry beside mod particles
pub(crate) fn nameParticles()
{
    for row in &PARTICLES
    {
        let id: modloader::Identifier = modloader::Identifier::minecraft(row.m_name);
        modloader::registry::addClientName(modloader::mod_protocol::abi::RegistryKind::Particle, id);
    }
}

pub(crate) fn vanillaParticle(
    global: &RenderGlobal,
    name: &str,
    world: &Rc<World>,
    pos: [f64; 3],
    motion: [f64; 3],
) -> Option<Rc<Entity>>
{
    PARTICLES.iter().find(|row| row.m_name == name).map(|row| (row.m_make)(global, world, pos, motion))
}

impl IWorldAccess for RenderGlobal
{
    fn markBlockAndNeighborsNeedsUpdate(&self, x: i32, y: i32, z: i32)
    {
        self.markDirty(x - 1, y - 1, z - 1, x + 1, y + 1, z + 1);
    }

    fn markBlockRangeNeedsUpdate(&self, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32)
    {
        self.markDirty(minX - 1, minY - 1, minZ - 1, maxX + 1, maxY + 1, maxZ + 1);
    }

    fn playSound(&self, name: &str, x: f64, y: f64, z: f64, volume: f32, pitch: f32)
    {
        let mut f: f32 = 16.0;

        if volume > 1.0
        {
            f *= volume;
        }

        if self.renderViewEntity().getDistanceSq(x, y, z) < (f * f) as f64
        {
            self.m_mc.m_sndManager.get().orNpe().playSound(name, x as f32, y as f32, z as f32, volume, pitch);
        }
    }

    fn spawnParticle(&self, kind: &str, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64)
    {
        let (Some(view), Some(effectRenderer)): (Option<Rc<Entity>>, Option<Rc<EffectRenderer>>) =
            (self.m_mc.m_renderViewEntity.get(), self.m_mc.m_effectRenderer.get())
        else
        {
            return;
        };
        let d: f64 = view.posX() - x;
        let d2: f64 = view.posY() - y;
        let d3: f64 = view.posZ() - z;
        let d4: f64 = 16.0;
        if d * d + d2 * d2 + d3 * d3 > d4 * d4
        {
            return;
        }

        let world: Rc<World> = self.worldObj();
        let pos: [f64; 3] = [x, y, z];
        let motion: [f64; 3] = [velocityX, velocityY, velocityZ];
        // rewrite: mod particles, item crack bits and tinted portal particles after the beta names
        let particle: Option<Rc<Entity>> = vanillaParticle(self, kind, &world, pos, motion)
            .or_else(|| crate::modded::particles::spawn(self, kind, &world, pos, motion));

        if let Some(particle) = particle
        {
            effectRenderer.addEffect(particle);
        }
    }

    fn obtainEntitySkin(&self, entity: &Rc<Entity>)
    {
        entity.updateCloak();
        self.obtainSkin(entity.m_skinUrl.get());
        self.obtainSkin(entity.m_cloakUrl.get());
    }

    fn releaseEntitySkin(&self, entity: &Rc<Entity>)
    {
        self.releaseSkin(entity.m_skinUrl.get());
        self.releaseSkin(entity.m_cloakUrl.get());
    }

    fn updateAllRenderers(&self) { Self::updateAllRenderers(self); }

    fn playRecord(&self, record: Option<&str>, x: i32, y: i32, z: i32)
    {
        if let Some(record) = record
        {
            // rewrite: mod discs name their own artist and title
            let title: String = modloader::content::recordTitle(record).unwrap_or_else(|| format!("C418 - {record}"));
            self.m_mc.m_ingameGUI.get().orNpe().setRecordPlayingMessage(&title);
        }

        self.m_mc.m_sndManager.get().orNpe().playStreaming(record, x as f32, y as f32, z as f32, 1.0, 1.0);
    }

    fn doNothingWithTileEntity(&self, _x: i32, _y: i32, _z: i32, _blockEntity: &Rc<TileEntity>) {}

    fn doEvent(&self, _source: Option<&Rc<Entity>>, kind: i32, x: i32, y: i32, z: i32, data: i32)
    {
        let world: Rc<World> = self.worldObj();
        let random: &Random = &world.m_rand;
        let center: (f64, f64, f64) = (x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5);

        match kind
        {
            1001 => world.playSoundEffect(x as f64, y as f64, z as f64, "random.click", 1.0, 1.2),
            1000 => world.playSoundEffect(x as f64, y as f64, z as f64, "random.click", 1.0, 1.0),
            1002 => world.playSoundEffect(x as f64, y as f64, z as f64, "random.bow", 1.0, 1.2),
            2000 => self.smokeBurst(random, x, y, z, data),

            game::block::BREAK_EVENT =>
            {
                let (id, meta): (i32, i32) = game::block::unpackBreak(data);
                if id > 0
                {
                    let block: Rc<Block> = Block::byId(id).orNpe();
                    let stepSound: Rc<StepSound> = block.m_stepSound.get();
                    self.m_mc.m_sndManager.get().orNpe().playSound(
                        &stepSound.stepSoundDir(),
                        x as f32 + 0.5,
                        y as f32 + 0.5,
                        z as f32 + 0.5,
                        (stepSound.getVolume() + 1.0) / 2.0,
                        stepSound.getPitch() * 0.8,
                    );
                }

                self.m_mc.m_effectRenderer.get().orNpe().addBlockDestroyEffects(x, y, z, id, meta);
            }

            1003 =>
            {
                let sound: &str = if jvm::math::random() < 0.5
                {
                    "random.door_open"
                }
                else
                {
                    "random.door_close"
                };
                world.playSoundEffect(center.0, center.1, center.2, sound, 1.0, world.m_rand.nextFloat() * 0.1 + 0.9);
            }

            1004 =>
            {
                let pitch: f32 = 2.6 + (random.nextFloat() - random.nextFloat()) * 0.8;
                world.playSoundEffect(
                    (x as f32 + 0.5) as f64,
                    (y as f32 + 0.5) as f64,
                    (z as f32 + 0.5) as f64,
                    "random.fizz",
                    0.5,
                    pitch,
                );
            }

            1005 =>
            {
                let item: Option<Rc<Item>> = Item::byId(data);
                match item.filter(|item| item.instanceOf("ItemRecord"))
                {
                    Some(record) => world.playRecord(Some(item_record::recordName(&record)), x, y, z),
                    None => world.playRecord(None, x, y, z),
                }
            }

            _ =>
            {}
        }
    }
}
