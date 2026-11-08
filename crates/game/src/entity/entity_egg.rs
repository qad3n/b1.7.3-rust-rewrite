use crate::entity::entity_snowball::{self, alloc, initAt, initFromThrower, throwable, updateThrown};
use crate::entity::{ENTITY, Entity, EntityClass, entity_arrow};
use crate::world::World;
use std::rc::Rc;

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(&CLASS, world);
    entity.setSize(0.25, 0.25);
    entity
}

pub fn create(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(&CLASS, Some(Rc::clone(world)));
    initAt(&entity, x, y, z);
    entity
}

pub fn createFromThrower(world: &Rc<World>, thrower: &Rc<Entity>) -> Rc<Entity>
{
    let entity: Rc<Entity> = alloc(&CLASS, Some(Rc::clone(world)));
    initFromThrower(&entity, thrower);
    entity
}

pub fn setEggHeading(this: &Rc<Entity>, x: f64, y: f64, z: f64, speed: f32, spread: f32)
{
    entity_arrow::applyHeading(this, x, y, z, speed, spread);
    throwable(this).resetTicksInGround();
}

fn hatchChickens(this: &Rc<Entity>)
{
    let world: Rc<World> = this.world();
    if world.m_multiplayerWorld.get() || this.m_rand.nextIntBounded(8) != 0
    {
        return;
    }
    let mut b0: i32 = 1;
    if this.m_rand.nextIntBounded(32) == 0
    {
        b0 = 4;
    }
    for _ in 0..b0
    {
        let entitychicken: Rc<Entity> = crate::entity::entity_chicken::new(Some(Rc::clone(&world)));
        entitychicken.setLocationAndAngles(this.posX(), this.posY(), this.posZ(), this.m_rotationYaw.get(), 0.0);
        world.entityJoinedWorld(&entitychicken);
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityEgg",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    entityInit: |_| {},
    isInRangeToRenderDist: entity_snowball::isInRangeToRenderDist,
    setVelocity: entity_snowball::setVelocity,
    onUpdate: |this| updateThrown(this, hatchChickens),
    writeEntityToNBT: entity_snowball::writeEntityToNBT,
    readEntityFromNBT: entity_snowball::readEntityFromNBT,
    onCollideWithPlayer: entity_snowball::onCollideWithPlayer,
    getShadowSize: |_| 0.0,
    ..ENTITY
};
