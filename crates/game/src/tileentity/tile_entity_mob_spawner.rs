use crate::entity::{Entity, EntityList};
use crate::math::AxisAlignedBB;
use crate::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use crate::world::World;
use jvm::field::Field;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

pub struct SpawnerExt
{
    pub m_delay: Cell<i32>,
    m_mobID: Field<Option<JString>>,
    pub m_yaw: Cell<f64>,
    pub m_yaw2: Cell<f64>,
}

pub fn fields(this: &TileEntity) -> &SpawnerExt { this.ext::<SpawnerExt>() }

pub fn getMobID(this: &TileEntity) -> Option<JString> { fields(this).m_mobID.get() }

pub fn setMobID(this: &TileEntity, kind: Option<JString>) { fields(this).m_mobID.set(kind); }

pub fn anyPlayerInRange(this: &TileEntity) -> bool
{
    this.world()
        .getClosestPlayer(
            this.m_xCoord.get() as f64 + 0.5,
            this.m_yCoord.get() as f64 + 0.5,
            this.m_zCoord.get() as f64 + 0.5,
            16.0,
        )
        .is_some()
}

fn updateEntity(this: &Rc<TileEntity>)
{
    let spawner: &SpawnerExt = fields(this);
    let world: Rc<World> = this.world();
    let (x, y, z): (i32, i32, i32) = (this.m_xCoord.get(), this.m_yCoord.get(), this.m_zCoord.get());
    spawner.m_yaw2.set(spawner.m_yaw.get());
    if !anyPlayerInRange(this)
    {
        return;
    }
    let mut d0: f64 = (x as f32 + world.m_rand.nextFloat()) as f64;
    let mut d1: f64 = (y as f32 + world.m_rand.nextFloat()) as f64;
    let mut d2: f64 = (z as f32 + world.m_rand.nextFloat()) as f64;
    world.spawnParticle("smoke", d0, d1, d2, 0.0, 0.0, 0.0);
    world.spawnParticle("flame", d0, d1, d2, 0.0, 0.0, 0.0);
    spawner.m_yaw.set(spawner.m_yaw.get() + (1000.0f32 / (spawner.m_delay.get() as f32 + 200.0)) as f64);
    while spawner.m_yaw.get() > 360.0
    {
        spawner.m_yaw.set(spawner.m_yaw.get() - 360.0);
        spawner.m_yaw2.set(spawner.m_yaw2.get() - 360.0);
    }
    if !world.m_multiplayerWorld.get()
    {
        if spawner.m_delay.get() == -1
        {
            updateDelay(this);
        }
        if spawner.m_delay.get() > 0
        {
            spawner.m_delay.set(spawner.m_delay.get() - 1);
            return;
        }
        let b0: i8 = 4;
        for _ in 0..b0
        {
            let created: Option<Rc<Entity>> = EntityList::createEntityInWorld(spawner.m_mobID.get().as_ref(), &world);
            if let Some(entity) = &created
                && !entity.instanceOf("EntityLiving")
            {
                jvm::throwable::throw(
                    &jvm::throwable::CLASS_CAST,
                    &format!(
                        "net.minecraft.src.{} cannot be cast to net.minecraft.src.EntityLiving",
                        entity.className()
                    ),
                );
            }
            let Some(entityliving) = created
            else
            {
                return;
            };
            let j: usize = world
                .getEntitiesWithinAABB(
                    entityliving.className(),
                    &AxisAlignedBB::getBoundingBoxFromPool(
                        x as f64,
                        y as f64,
                        z as f64,
                        (x + 1) as f64,
                        (y + 1) as f64,
                        (z + 1) as f64,
                    )
                    .expand(8.0, 4.0, 8.0),
                )
                .len();
            if j >= 6
            {
                updateDelay(this);
                return;
            }
            let d3: f64 = x as f64 + (world.m_rand.nextDouble() - world.m_rand.nextDouble()) * 4.0;
            let d4: f64 = (y + world.m_rand.nextIntBounded(3) - 1) as f64;
            let d5: f64 = z as f64 + (world.m_rand.nextDouble() - world.m_rand.nextDouble()) * 4.0;
            entityliving.setLocationAndAngles(d3, d4, d5, world.m_rand.nextFloat() * 360.0, 0.0);
            if entityliving.getCanSpawnHere()
            {
                world.entityJoinedWorld(&entityliving);
                for _ in 0..20
                {
                    d0 = x as f64 + 0.5 + (world.m_rand.nextFloat() as f64 - 0.5) * 2.0;
                    d1 = y as f64 + 0.5 + (world.m_rand.nextFloat() as f64 - 0.5) * 2.0;
                    d2 = z as f64 + 0.5 + (world.m_rand.nextFloat() as f64 - 0.5) * 2.0;
                    world.spawnParticle("smoke", d0, d1, d2, 0.0, 0.0, 0.0);
                    world.spawnParticle("flame", d0, d1, d2, 0.0, 0.0, 0.0);
                }
                crate::entity::entity_living::spawnExplosionParticle(&entityliving);
                updateDelay(this);
            }
        }
    }
    (TILE_ENTITY.updateEntity)(this);
}

fn updateDelay(this: &TileEntity) { fields(this).m_delay.set(200 + this.world().m_rand.nextIntBounded(600)); }

pub const CLASS: TileEntityClass = TileEntityClass {
    m_name: "TileEntityMobSpawner",
    m_parent: Some(&TILE_ENTITY),
    updateEntity,
    readFromNBT: |this, nbt| {
        (TILE_ENTITY.readFromNBT)(this, nbt);
        fields(this).m_mobID.set(nbt.getString("EntityId"));
        fields(this).m_delay.set(nbt.getShort("Delay") as i32);
    },
    writeToNBT: |this, nbt| {
        (TILE_ENTITY.writeToNBT)(this, nbt);
        nbt.setString("EntityId", fields(this).m_mobID.get());
        nbt.setShort("Delay", fields(this).m_delay.get() as i16);
    },
    ..TILE_ENTITY
};

pub fn create() -> Rc<TileEntity>
{
    let ext: SpawnerExt =
        SpawnerExt { m_delay: Cell::new(-1), m_mobID: Field::new(None), m_yaw: Cell::new(0.0), m_yaw2: Cell::new(0.0) };
    let tileEntity: Rc<TileEntity> = TileEntity::new(&CLASS, Box::new(ext));
    fields(&tileEntity).m_mobID.set(Some(JString::from("Pig")));
    fields(&tileEntity).m_delay.set(20);
    tileEntity
}
