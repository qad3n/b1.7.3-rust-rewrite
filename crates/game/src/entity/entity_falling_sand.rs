use crate::block::block_sand;
use crate::entity::{ENTITY, Entity, EntityClass, EntityExt};
use crate::math::math_helper::floor_double;
use crate::world::World;
use std::cell::Cell;
use std::rc::Rc;

pub struct FallingSandFields
{
    pub m_blockID: Cell<i32>,
    pub m_fallTime: Cell<i32>,
}

impl EntityExt for FallingSandFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &FallingSandFields { this.ext::<FallingSandFields>() }

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> =
        Entity::alloc(&CLASS, Box::new(FallingSandFields { m_blockID: Cell::new(0), m_fallTime: Cell::new(0) }));
    Entity::init(&entity, world);
    entity
}

pub fn create(world: &Rc<World>, x: f64, y: f64, z: f64, block: i32) -> Rc<Entity>
{
    let entity: Rc<Entity> = new(Some(Rc::clone(world)));
    fields(&entity).m_blockID.set(block);
    entity.m_preventEntitySpawning.set(true);
    entity.setSize(0.98, 0.98);
    entity.m_yOffset.set(entity.m_height.get() / 2.0);
    entity.setPosition(x, y, z);
    entity.m_motionX.set(0.0);
    entity.m_motionY.set(0.0);
    entity.m_motionZ.set(0.0);
    entity.m_prevPosX.set(x);
    entity.m_prevPosY.set(y);
    entity.m_prevPosZ.set(z);
    entity
}

fn onUpdate(this: &Rc<Entity>)
{
    let sand: &FallingSandFields = fields(this);
    let blockID: i32 = sand.m_blockID.get();
    if blockID == 0
    {
        this.setEntityDead();
        return;
    }
    this.m_prevPosX.set(this.posX());
    this.m_prevPosY.set(this.posY());
    this.m_prevPosZ.set(this.posZ());
    sand.m_fallTime.set(sand.m_fallTime.get() + 1);
    this.m_motionY.set(this.m_motionY.get() - 0.04f32 as f64);
    this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
    this.m_motionX.set(this.m_motionX.get() * 0.98f32 as f64);
    this.m_motionY.set(this.m_motionY.get() * 0.98f32 as f64);
    this.m_motionZ.set(this.m_motionZ.get() * 0.98f32 as f64);
    let n: i32 = floor_double(this.posX());
    let n2: i32 = floor_double(this.posY());
    let n3: i32 = floor_double(this.posZ());
    let world: Rc<World> = this.world();
    if world.getBlockId(n, n2, n3) == blockID
    {
        world.setBlockWithNotify(n, n2, n3, 0);
    }
    if this.m_onGround.get()
    {
        this.m_motionX.set(this.m_motionX.get() * 0.7f32 as f64);
        this.m_motionZ.set(this.m_motionZ.get() * 0.7f32 as f64);
        this.m_motionY.set(this.m_motionY.get() * -0.5);
        this.setEntityDead();
        if world.canBlockBePlacedAt(blockID, n, n2, n3, true, 1)
            && !block_sand::canFallBelow(&world, n, n2 - 1, n3)
            && world.setBlockWithNotify(n, n2, n3, blockID)
            || world.m_multiplayerWorld.get()
        {
            return;
        }
        this.dropItem(blockID, 1);
    }
    else
    {
        if sand.m_fallTime.get() <= 100 || world.m_multiplayerWorld.get()
        {
            return;
        }
        this.dropItem(blockID, 1);
        this.setEntityDead();
    }
}

pub fn getWorld(this: &Entity) -> Option<Rc<World>> { this.m_worldObj.get() }

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityFallingSand",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    canTriggerWalking: |_| false,
    entityInit: |_| {},
    canBeCollidedWith: |this| !this.m_isDead.get(),
    onUpdate,
    writeEntityToNBT: |this, nbt| nbt.setByte("Tile", fields(this).m_blockID.get() as i8),
    readEntityFromNBT: |this, nbt| fields(this).m_blockID.set(nbt.getByte("Tile") as i32 & 0xFF),
    getShadowSize: |_| 0.0,
    ..ENTITY
};
