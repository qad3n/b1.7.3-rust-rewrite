use crate::render::particle::{entity_digging_fx, entity_fx};
use crate::render::{RenderEngine, Tessellator};
use game::block::Block;
use game::entity::Entity;
use game::math::math_helper;
use game::world::World;
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::f64::consts::PI;
use std::rc::Rc;

pub struct EffectRenderer
{
    m_worldObj: Field<Option<Rc<World>>>,
    m_fxLayers: [JArrayList<Rc<Entity>>; 4],
    m_renderer: Rc<RenderEngine>,
    m_rand: Random,
}

impl EffectRenderer
{
    pub fn new(world: Option<Rc<World>>, textureManager: &Rc<RenderEngine>) -> EffectRenderer
    {
        let effectRenderer: EffectRenderer = EffectRenderer {
            m_worldObj: Field::new(None),
            m_fxLayers: [JArrayList::new(), JArrayList::new(), JArrayList::new(), JArrayList::new()],
            m_renderer: Rc::clone(textureManager),
            m_rand: Random::new(),
        };
        if world.is_some()
        {
            effectRenderer.m_worldObj.set(world);
        }
        effectRenderer
    }

    fn worldObj(&self) -> Rc<World> { self.m_worldObj.get().orNpe() }

    pub fn addEffect(&self, particle: Rc<Entity>)
    {
        let layer: &JArrayList<Rc<Entity>> = &self.m_fxLayers[particle.getFXLayer() as usize];
        if layer.size() >= 4000
        {
            layer.removeAt(0);
        }
        layer.add(particle);
    }

    pub fn updateEffects(&self)
    {
        for layer in &self.m_fxLayers
        {
            let mut j: i32 = 0;
            while j < layer.size()
            {
                let entityfx: Rc<Entity> = layer.get(j);
                entityfx.onUpdate();
                if entityfx.m_isDead.get()
                {
                    layer.removeAt(j);
                    j -= 1;
                }
                j += 1;
            }
        }
    }

    pub fn renderParticles(&self, camera: &Rc<Entity>, tickDelta: f32)
    {
        let yaw: f32 = camera.m_rotationYaw.get() * PI as f32 / 180.0;
        let pitch: f32 = camera.m_rotationPitch.get() * PI as f32 / 180.0;
        let f: f32 = math_helper::cos(yaw);
        let f2: f32 = math_helper::sin(yaw);
        let f3: f32 = -f2 * math_helper::sin(pitch);
        let f4: f32 = f * math_helper::sin(pitch);
        let f5: f32 = math_helper::cos(pitch);
        let delta: f64 = tickDelta as f64;
        entity_fx::setInterpPos(
            camera.m_lastTickPosX.get() + (camera.posX() - camera.m_lastTickPosX.get()) * delta,
            camera.m_lastTickPosY.get() + (camera.posY() - camera.m_lastTickPosY.get()) * delta,
            camera.m_lastTickPosZ.get() + (camera.posZ() - camera.m_lastTickPosZ.get()) * delta,
        );
        let textures: [&str; 3] = ["/particles.png", "/terrain.png", "/gui/items.png"];
        for (layer, texture) in self.m_fxLayers.iter().zip(textures)
        {
            if layer.size() == 0
            {
                continue;
            }
            GL11::glBindTexture(3553, self.m_renderer.getTexture(texture));
            let tessellator: Rc<Tessellator> = Tessellator::instance();
            tessellator.startDrawingQuads();
            for j in 0..layer.size()
            {
                layer.get(j).renderParticle(&*tessellator, tickDelta, f, f5, f2, f3, f4);
            }
            tessellator.draw();
        }
    }

    pub fn renderLit(&self, _camera: &Rc<Entity>, tickDelta: f32)
    {
        let layer: &JArrayList<Rc<Entity>> = &self.m_fxLayers[3];
        if layer.size() == 0
        {
            return;
        }
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        for i in 0..layer.size()
        {
            layer.get(i).renderParticle(&*tessellator, tickDelta, 0.0, 0.0, 0.0, 0.0, 0.0);
        }
    }

    pub fn clearEffects(&self, world: Option<Rc<World>>)
    {
        self.m_worldObj.set(world);
        for layer in &self.m_fxLayers
        {
            layer.clear();
        }
    }

    pub fn addBlockDestroyEffects(&self, x: i32, y: i32, z: i32, block: i32, metadata: i32)
    {
        if block == 0
        {
            return;
        }
        let block2: Option<Rc<Block>> = Block::byId(block);
        let n: i32 = 4;
        for i in 0..n
        {
            for j in 0..n
            {
                for k in 0..n
                {
                    let d: f64 = x as f64 + (i as f64 + 0.5) / n as f64;
                    let d2: f64 = y as f64 + (j as f64 + 0.5) / n as f64;
                    let d3: f64 = z as f64 + (k as f64 + 0.5) / n as f64;
                    let n2: i32 = self.m_rand.nextIntBounded(6);
                    let particle: Rc<Entity> = entity_digging_fx::new(
                        &self.worldObj(),
                        d,
                        d2,
                        d3,
                        d - x as f64 - 0.5,
                        d2 - y as f64 - 0.5,
                        d3 - z as f64 - 0.5,
                        block2.as_ref().orNpe(),
                        n2,
                        metadata,
                    );
                    self.addEffect(entity_digging_fx::init(&particle, x, y, z));
                }
            }
        }
    }

    pub fn addBlockHitEffects(&self, x: i32, y: i32, z: i32, face: i32)
    {
        let world: Rc<World> = self.worldObj();
        let n: i32 = world.getBlockId(x, y, z);
        if n == 0
        {
            return;
        }
        let block: Rc<Block> = Block::byId(n).orNpe();
        let f: f32 = 0.1;
        let inset: f64 = (f * 2.0) as f64;
        let mut d: f64 = x as f64
            + self.m_rand.nextDouble() * (block.m_maxX.get() - block.m_minX.get() - inset)
            + f as f64
            + block.m_minX.get();
        let mut d2: f64 = y as f64
            + self.m_rand.nextDouble() * (block.m_maxY.get() - block.m_minY.get() - inset)
            + f as f64
            + block.m_minY.get();
        let mut d3: f64 = z as f64
            + self.m_rand.nextDouble() * (block.m_maxZ.get() - block.m_minZ.get() - inset)
            + f as f64
            + block.m_minZ.get();
        match face
        {
            0 => d2 = y as f64 + block.m_minY.get() - f as f64,
            1 => d2 = y as f64 + block.m_maxY.get() + f as f64,
            2 => d3 = z as f64 + block.m_minZ.get() - f as f64,
            3 => d3 = z as f64 + block.m_maxZ.get() + f as f64,
            4 => d = x as f64 + block.m_minX.get() - f as f64,
            5 => d = x as f64 + block.m_maxX.get() + f as f64,
            _ =>
            {}
        }
        let particle: Rc<Entity> =
            entity_digging_fx::new(&world, d, d2, d3, 0.0, 0.0, 0.0, &block, face, world.getBlockMetadata(x, y, z));
        let particle: Rc<Entity> = entity_digging_fx::init(&particle, x, y, z);
        let particle: Rc<Entity> = entity_fx::multiplyVelocity(&particle, 0.2);
        self.addEffect(entity_fx::multiplySize(&particle, 0.6));
    }

    pub fn getStatistics(&self) -> String
    {
        format!("{}", self.m_fxLayers[0].size() + self.m_fxLayers[1].size() + self.m_fxLayers[2].size())
    }
}
