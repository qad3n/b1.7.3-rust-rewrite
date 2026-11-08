use crate::render::particle::entity_fx::{self, FXFields, fx};
use game::block::Material;
use game::entity::{Entity, EntityClass};
use game::math::math_helper::floor_double;
use game::world::World;
use jvm::math;
use std::rc::Rc;

fn onUpdate(this: &Rc<Entity>)
{
    entity_fx::setPrevPosition(this);
    this.m_motionY.set(this.m_motionY.get() + 0.002);
    entity_fx::moveByMotion(this);
    entity_fx::scaleMotion(this, 0.85f32 as f64);
    let material: Rc<Material> =
        this.world().getBlockMaterial(floor_double(this.posX()), floor_double(this.posY()), floor_double(this.posZ()));
    if !Rc::ptr_eq(&material, &Material::water())
    {
        this.setEntityDead();
    }
    let fields: &FXFields = fx(this);
    let maxAge: i32 = fields.m_particleMaxAge.get();
    fields.m_particleMaxAge.set(maxAge - 1);
    if maxAge <= 0
    {
        this.setEntityDead();
    }
}

pub const CLASS: EntityClass =
    EntityClass { m_name: "EntityBubbleFX", m_parent: Some(&entity_fx::CLASS), onUpdate, ..entity_fx::CLASS };

fn velocity(value: f64) -> f64 { value * 0.2f32 as f64 + ((math::random() * 2.0 - 1.0) as f32 * 0.02) as f64 }

pub fn new(world: &Rc<World>, x: f64, y: f64, z: f64, velocityX: f64, velocityY: f64, velocityZ: f64) -> Rc<Entity>
{
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(()), world, x, y, z, velocityX, velocityY, velocityZ);
    let fields: &FXFields = fx(&this);
    fields.m_particleRed.set(1.0);
    fields.m_particleGreen.set(1.0);
    fields.m_particleBlue.set(1.0);
    fields.m_particleTextureIndex.set(32);
    this.setSize(0.02, 0.02);
    fields.m_particleScale.set(fields.m_particleScale.get() * (this.m_rand.nextFloat() * 0.6 + 0.2));
    this.m_motionX.set(velocity(velocityX));
    this.m_motionY.set(velocity(velocityY));
    this.m_motionZ.set(velocity(velocityZ));
    fields.m_particleMaxAge.set((8.0 / (math::random() * 0.8 + 0.2)) as i32);
    this
}
