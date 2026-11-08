use crate::render::RenderManager;
use crate::render::particle::entity_fx::{self, extra, interpPosX, interpPosY, interpPosZ};
use game::entity::{Entity, EntityClass};
use game::math::math_helper::floor_double;
use game::world::World;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

pub struct PickupFields
{
    m_entity: Rc<Entity>,
    m_collector: Rc<Entity>,
    m_age: Cell<i32>,
    m_lifetime: Cell<i32>,
    m_offsetY: f32,
}

fn renderParticle(
    this: &Rc<Entity>,
    _target: &dyn std::any::Any,
    tickDelta: f32,
    _dx: f32,
    _dy: f32,
    _dz: f32,
    _forwards: f32,
    _sideways: f32,
)
{
    let pickup: &PickupFields = extra::<PickupFields>(this);
    let mut f: f32 = (pickup.m_age.get() as f32 + tickDelta) / pickup.m_lifetime.get() as f32;
    f *= f;
    let entity: &Rc<Entity> = &pickup.m_entity;
    let collector: &Rc<Entity> = &pickup.m_collector;
    let delta: f64 = tickDelta as f64;
    let d0: f64 = entity.posX();
    let d1: f64 = entity.posY();
    let d2: f64 = entity.posZ();
    let d3: f64 = collector.m_lastTickPosX.get() + (collector.posX() - collector.m_lastTickPosX.get()) * delta;
    let d4: f64 = collector.m_lastTickPosY.get()
        + (collector.posY() - collector.m_lastTickPosY.get()) * delta
        + pickup.m_offsetY as f64;
    let d5: f64 = collector.m_lastTickPosZ.get() + (collector.posZ() - collector.m_lastTickPosZ.get()) * delta;
    let mut d6: f64 = d0 + (d3 - d0) * f as f64;
    let mut d7: f64 = d1 + (d4 - d1) * f as f64;
    let mut d8: f64 = d2 + (d5 - d2) * f as f64;
    let i: i32 = floor_double(d6);
    let j: i32 = floor_double(d7 + (this.m_yOffset.get() / 2.0) as f64);
    let k: i32 = floor_double(d8);
    let f1: f32 = this.world().getLightBrightness(i, j, k);
    d6 -= interpPosX();
    d7 -= interpPosY();
    d8 -= interpPosZ();
    GL11::glColor4f(f1, f1, f1, 1.0);
    RenderManager::instance().renderEntityWithPosYaw(
        entity,
        d6 as f32 as f64,
        d7 as f32 as f64,
        d8 as f32 as f64,
        entity.m_rotationYaw.get(),
        tickDelta,
    );
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityPickupFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle,
    onUpdate: |this| {
        let pickup: &PickupFields = extra::<PickupFields>(this);
        pickup.m_age.set(pickup.m_age.get() + 1);
        if pickup.m_age.get() == pickup.m_lifetime.get()
        {
            this.setEntityDead();
        }
    },
    getFXLayer: |_| 3,
    ..entity_fx::CLASS
};

pub fn new(world: &Rc<World>, entity: &Rc<Entity>, collector: &Rc<Entity>, offsetY: f32) -> Rc<Entity>
{
    let pickup: PickupFields = PickupFields {
        m_entity: Rc::clone(entity),
        m_collector: Rc::clone(collector),
        m_age: Cell::new(0),
        m_lifetime: Cell::new(0),
        m_offsetY: offsetY,
    };
    let this: Rc<Entity> = entity_fx::construct(
        &CLASS,
        Box::new(pickup),
        world,
        entity.posX(),
        entity.posY(),
        entity.posZ(),
        entity.m_motionX.get(),
        entity.m_motionY.get(),
        entity.m_motionZ.get(),
    );
    extra::<PickupFields>(&this).m_lifetime.set(3);
    this
}
