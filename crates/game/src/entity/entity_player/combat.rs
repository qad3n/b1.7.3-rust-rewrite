use super::{
    PlayerFields, addStat, destroyCurrentEquippedItem, dropPlayerItemWithRandomChoice, fields, getCurrentEquippedItem,
    inventory,
};
use crate::entity::{Entity, entity_arrow, entity_creature, entity_living, entity_wolf};
use crate::item::{Item, ItemStack};
use crate::math::{AxisAlignedBB, math_helper};
use crate::stats::StatList;
use crate::world::World;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::rc::Rc;

const PI_F: f32 = std::f64::consts::PI as f32;

pub fn onDeath(this: &Rc<Entity>, killer: Option<&Rc<Entity>>)
{
    (entity_living::CLASS.onDeath)(this, killer);
    this.setSize(0.2, 0.2);
    this.setPosition(this.posX(), this.posY(), this.posZ());
    this.m_motionY.set(0.1f32 as f64);
    if fields(this).m_username.get().orNpe() == JString::from("Notch")
    {
        dropPlayerItemWithRandomChoice(this, Some(ItemStack::withItemCount(&Item::appleRed(), 1)), true);
    }
    inventory(this).dropAllItems();
    if killer.is_some()
    {
        let angle: f32 = (entity_living::fields(this).m_attackedAtYaw.get() + this.m_rotationYaw.get()) * PI_F / 180.0;
        this.m_motionX.set((-math_helper::cos(angle) * 0.1) as f64);
        this.m_motionZ.set((-math_helper::sin(angle) * 0.1) as f64);
    }
    else
    {
        this.m_motionZ.set(0.0);
        this.m_motionX.set(0.0);
    }
    this.m_yOffset.set(0.1);
    addStat(this, StatList::deathsStat(), 1);
}

pub fn attackEntityFrom(this: &Rc<Entity>, source: Option<&Rc<Entity>>, mut amount: i32) -> bool
{
    let living: &entity_living::LivingFields = entity_living::fields(this);
    let world: Rc<World> = this.world();
    living.m_entityAge.set(0);
    if living.m_health.get() <= 0
    {
        return false;
    }
    if this.isPlayerSleeping() && !world.m_multiplayerWorld.get()
    {
        this.wakeUpPlayer(true, true, false);
    }
    if source.is_some_and(|source| source.instanceOf("EntityMob") || source.instanceOf("EntityArrow"))
    {
        if world.m_difficultySetting.get() == 0
        {
            amount = 0;
        }
        if world.m_difficultySetting.get() == 1
        {
            amount = amount / 3 + 1;
        }
        if world.m_difficultySetting.get() == 3
        {
            amount = amount * 3 / 2;
        }
    }
    if amount == 0
    {
        return false;
    }
    let mut entity: Option<Rc<Entity>> = source.cloned();
    if let Some(arrow) = &entity
        && arrow.instanceOf("EntityArrow")
        && let Some(owner) = entity_arrow::fields(arrow).m_owner.get()
    {
        entity = Some(owner);
    }
    if let Some(attacker) = &entity
        && attacker.instanceOf("EntityLiving")
    {
        alertWolves(this, attacker, false);
    }
    addStat(this, StatList::damageTakenStat(), amount);
    (entity_living::CLASS.attackEntityFrom)(this, source, amount)
}

pub(super) fn pvpEnabled(_this: &Rc<Entity>) -> bool { false }

pub fn alertWolves(this: &Rc<Entity>, target: &Rc<Entity>, wasAttackedByTarget: bool)
{
    if target.instanceOf("EntityCreeper") || target.instanceOf("EntityGhast")
    {
        return;
    }
    let ownedBy = |wolf: &Rc<Entity>| Some(fields(this).m_username.get().orNpe()) == entity_wolf::getWolfOwner(wolf);
    if target.instanceOf("EntityWolf") && entity_wolf::isWolfTamed(target) && ownedBy(target)
    {
        return;
    }
    if !target.instanceOf("EntityPlayer") || this.pvpEnabled()
    {
        let bounds: Rc<AxisAlignedBB> = AxisAlignedBB::getBoundingBoxFromPool(
            this.posX(),
            this.posY(),
            this.posZ(),
            this.posX() + 1.0,
            this.posY() + 1.0,
            this.posZ() + 1.0,
        )
        .expand(16.0, 4.0, 16.0);
        for wolf in this.world().getEntitiesWithinAABB("EntityWolf", &bounds)
        {
            if entity_wolf::isWolfTamed(&wolf)
                && entity_creature::getTarget(&wolf).is_none()
                && ownedBy(&wolf)
                && (!wasAttackedByTarget || !entity_wolf::isWolfSitting(&wolf))
            {
                entity_wolf::setWolfSitting(&wolf, false);
                entity_creature::setTarget(&wolf, Some(Rc::clone(target)));
            }
        }
    }
}

pub fn damageEntity(this: &Rc<Entity>, mut amount: i32)
{
    let player: &PlayerFields = fields(this);
    let i: i32 = 25 - inventory(this).getTotalArmorValue();
    let j: i32 = amount * i + player.m_damageRemainder.get();
    inventory(this).damageArmor(amount);
    amount = j / 25;
    player.m_damageRemainder.set(j % 25);
    (entity_living::CLASS.damageEntity)(this, amount);
}

pub fn useCurrentItemOnEntity(this: &Rc<Entity>, target: &Rc<Entity>)
{
    if !target.interact(this)
        && let Some(itemstack) = getCurrentEquippedItem(this)
        && target.instanceOf("EntityLiving")
    {
        itemstack.useItemOnEntity(target);
        if itemstack.m_stackSize.get() <= 0
        {
            itemstack.onRemoved(this);
            destroyCurrentEquippedItem(this);
        }
    }
}

pub fn attackTargetEntityWithCurrentItem(this: &Rc<Entity>, target: &Rc<Entity>)
{
    let mut i: i32 = inventory(this).getDamageVsEntity(target);
    if i > 0
    {
        if this.m_motionY.get() < 0.0
        {
            i += 1;
        }
        target.attackEntityFrom(Some(this), i);
        if let Some(itemstack) = getCurrentEquippedItem(this)
            && target.instanceOf("EntityLiving")
        {
            itemstack.hitEntity(target, this);
            if itemstack.m_stackSize.get() <= 0
            {
                itemstack.onRemoved(this);
                destroyCurrentEquippedItem(this);
            }
        }
        if target.instanceOf("EntityLiving")
        {
            if target.isEntityAlive()
            {
                alertWolves(this, target, true);
            }
            addStat(this, StatList::damageDealtStat(), i);
        }
    }
}
