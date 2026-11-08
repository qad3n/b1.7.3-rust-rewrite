use super::{PlayerFields, addStat, fields, inventory};
use crate::block::Material;
use crate::entity::{Entity, entity_living};
use crate::math::math_helper;
use crate::stats::{AchievementList, StatList};
use crate::world::ChunkCoordinates;
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList};
use jvm::math;
use std::rc::Rc;

pub fn readEntityFromNBT(this: &Rc<Entity>, nbt: &NBTTagCompound)
{
    (entity_living::CLASS.readEntityFromNBT)(this, nbt);
    let player: &PlayerFields = fields(this);
    let nbttaglist: Rc<NBTBase> = nbt.getTagList("Inventory");
    inventory(this).readFromNBT(&nbttaglist);
    player.m_dimension.set(nbt.getInteger("Dimension"));
    player.m_sleeping.set(nbt.getBoolean("Sleeping"));
    player.m_sleepTimer.set(nbt.getShort("SleepTimer") as i32);
    if player.m_sleeping.get()
    {
        player.m_bedChunkCoordinates.set(Some(ChunkCoordinates::new(
            math_helper::floor_double(this.posX()),
            math_helper::floor_double(this.posY()),
            math_helper::floor_double(this.posZ()),
        )));
        this.wakeUpPlayer(true, true, false);
    }
    if nbt.hasKey("SpawnX") && nbt.hasKey("SpawnY") && nbt.hasKey("SpawnZ")
    {
        player.m_playerSpawnCoordinate.set(Some(ChunkCoordinates::new(
            nbt.getInteger("SpawnX"),
            nbt.getInteger("SpawnY"),
            nbt.getInteger("SpawnZ"),
        )));
    }
}

pub fn writeEntityToNBT(this: &Rc<Entity>, nbt: &NBTTagCompound)
{
    (entity_living::CLASS.writeEntityToNBT)(this, nbt);
    let player: &PlayerFields = fields(this);
    nbt.setTag("Inventory", &inventory(this).writeToNBT(&NBTTagList::create()));
    nbt.setInteger("Dimension", player.m_dimension.get());
    nbt.setBoolean("Sleeping", player.m_sleeping.get());
    nbt.setShort("SleepTimer", player.m_sleepTimer.get() as i16);
    if let Some(spawn) = player.m_playerSpawnCoordinate.get()
    {
        nbt.setInteger("SpawnX", spawn.m_x);
        nbt.setInteger("SpawnY", spawn.m_y);
        nbt.setInteger("SpawnZ", spawn.m_z);
    }
}

pub fn addMovementStat(this: &Rc<Entity>, x: f64, y: f64, z: f64)
{
    if this.m_ridingEntity.with(Option::is_some)
    {
        return;
    }
    if this.isInsideOfMaterial(&Material::water())
    {
        let i: i32 = math::roundFloat(math_helper::sqrt_double(x * x + y * y + z * z) * 100.0);
        if i > 0
        {
            addStat(this, StatList::distanceDoveStat(), i);
        }
    }
    else if this.isInWater()
    {
        let j: i32 = math::roundFloat(math_helper::sqrt_double(x * x + z * z) * 100.0);
        if j > 0
        {
            addStat(this, StatList::distanceSwumStat(), j);
        }
    }
    else if this.isOnLadder()
    {
        if y > 0.0
        {
            addStat(this, StatList::distanceClimbedStat(), math::roundDouble(y * 100.0) as i32);
        }
    }
    else if this.m_onGround.get()
    {
        let k: i32 = math::roundFloat(math_helper::sqrt_double(x * x + z * z) * 100.0);
        if k > 0
        {
            addStat(this, StatList::distanceWalkedStat(), k);
        }
    }
    else
    {
        let l: i32 = math::roundFloat(math_helper::sqrt_double(x * x + z * z) * 100.0);
        if l > 25
        {
            addStat(this, StatList::distanceFlownStat(), l);
        }
    }
}

pub fn addMountedMovementStat(this: &Rc<Entity>, x: f64, y: f64, z: f64)
{
    let Some(riding) = this.m_ridingEntity.get()
    else
    {
        return;
    };
    let player: &PlayerFields = fields(this);
    let i: i32 = math::roundFloat(math_helper::sqrt_double(x * x + y * y + z * z) * 100.0);
    if i <= 0
    {
        return;
    }
    if riding.instanceOf("EntityMinecart")
    {
        addStat(this, StatList::distanceByMinecartStat(), i);
        let here: (i32, i32, i32) = (
            math_helper::floor_double(this.posX()),
            math_helper::floor_double(this.posY()),
            math_helper::floor_double(this.posZ()),
        );
        match player.m_startMinecartRidingCoordinate.get()
        {
            None => player.m_startMinecartRidingCoordinate.set(Some(ChunkCoordinates::new(here.0, here.1, here.2))),
            Some(start) =>
            {
                if start.getSqDistanceTo(here.0, here.1, here.2) >= 1000.0
                {
                    addStat(this, AchievementList::onARail(), 1);
                }
            }
        }
    }
    else if riding.instanceOf("EntityBoat")
    {
        addStat(this, StatList::distanceByBoatStat(), i);
    }
    else if riding.instanceOf("EntityPig")
    {
        addStat(this, StatList::distanceByPigStat(), i);
    }
}
