use crate::block::Block;
use crate::entity::{Entity, EntityClass, EntityExt, entity_weather_effect};
use crate::math::AxisAlignedBB;
use crate::math::math_helper::floor_double;
use crate::world::World;
use std::cell::Cell;
use std::rc::Rc;

pub struct LightningFields
{
    m_stage: Cell<i32>,
    pub m_seed: Cell<i64>,
    m_fireAttempts: Cell<i32>,
}

impl EntityExt for LightningFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &LightningFields { this.ext::<LightningFields>() }

fn igniteAt(world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if world.getBlockId(x, y, z) == 0 && Block::fire().canPlaceBlockAt(world, x, y, z)
    {
        world.setBlockWithNotify(x, y, z, Block::fire().m_blockID);
    }
}

pub fn create(world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(
        &CLASS,
        Box::new(LightningFields { m_stage: Cell::new(0), m_seed: Cell::new(0), m_fireAttempts: Cell::new(0) }),
    );
    entity_weather_effect::init(&entity, Some(Rc::clone(world)));
    let bolt: &LightningFields = fields(&entity);
    entity.setLocationAndAngles(x, y, z, 0.0, 0.0);
    bolt.m_stage.set(2);
    bolt.m_seed.set(entity.m_rand.nextLong());
    bolt.m_fireAttempts.set(entity.m_rand.nextIntBounded(3) + 1);
    if world.m_difficultySetting.get() >= 2
        && world.doChunksNearChunkExist(floor_double(x), floor_double(y), floor_double(z), 10)
    {
        igniteAt(world, floor_double(x), floor_double(y), floor_double(z));
        for _ in 0..4
        {
            let j: i32 = floor_double(x) + entity.m_rand.nextIntBounded(3) - 1;
            let k: i32 = floor_double(y) + entity.m_rand.nextIntBounded(3) - 1;
            let l: i32 = floor_double(z) + entity.m_rand.nextIntBounded(3) - 1;
            igniteAt(world, j, k, l);
        }
    }
    entity
}

fn onUpdate(this: &Rc<Entity>)
{
    (entity_weather_effect::CLASS.onUpdate)(this);
    let bolt: &LightningFields = fields(this);
    let world: Rc<World> = this.world();
    if bolt.m_stage.get() == 2
    {
        world.playSoundEffect(
            this.posX(),
            this.posY(),
            this.posZ(),
            "ambient.weather.thunder",
            10000.0,
            0.8 + this.m_rand.nextFloat() * 0.2,
        );
        world.playSoundEffect(
            this.posX(),
            this.posY(),
            this.posZ(),
            "random.explode",
            2.0,
            0.5 + this.m_rand.nextFloat() * 0.2,
        );
    }
    bolt.m_stage.set(bolt.m_stage.get() - 1);
    if bolt.m_stage.get() < 0
    {
        if bolt.m_fireAttempts.get() == 0
        {
            this.setEntityDead();
        }
        else if bolt.m_stage.get() < -this.m_rand.nextIntBounded(10)
        {
            bolt.m_fireAttempts.set(bolt.m_fireAttempts.get() - 1);
            bolt.m_stage.set(1);
            bolt.m_seed.set(this.m_rand.nextLong());
            if world.doChunksNearChunkExist(
                floor_double(this.posX()),
                floor_double(this.posY()),
                floor_double(this.posZ()),
                10,
            )
            {
                igniteAt(&world, floor_double(this.posX()), floor_double(this.posY()), floor_double(this.posZ()));
            }
        }
    }
    if bolt.m_stage.get() >= 0
    {
        let d0: f64 = 3.0;
        let list: Rc<jvm::array_list::JArrayList<Rc<Entity>>> = world.getEntitiesWithinAABBExcludingEntity(
            Some(this),
            &AxisAlignedBB::getBoundingBoxFromPool(
                this.posX() - d0,
                this.posY() - d0,
                this.posZ() - d0,
                this.posX() + d0,
                this.posY() + 6.0 + d0,
                this.posZ() + d0,
            ),
        );
        let mut l: i32 = 0;
        while l < list.size()
        {
            let entity: Rc<Entity> = list.get(l);
            entity.onStruckByLightning(this);
            l += 1;
        }
        world.m_lightningCooldown.set(2);
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityLightningBolt",
    m_parent: Some(&entity_weather_effect::CLASS),
    m_interfaces: &[],
    onUpdate,
    entityInit: |_| {},
    readEntityFromNBT: |_, _| {},
    writeEntityToNBT: |_, _| {},
    isInRangeToRenderVec3D: |this, _| fields(this).m_stage.get() >= 0,
    ..entity_weather_effect::CLASS
};
