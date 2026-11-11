use crate::host::context::{dimension, entityKind};
use crate::{log, registry};
use game::damage::DamageSource;
use game::entity::{Entity, entity_player};
use game::hooks::GameEvent;
use game::item::ItemStack;
use game::world::World;
use std::rc::Rc;

const FIRST_ITEM: i32 = 256;

// one debug line per player or world action, events that fire every tick or per block update stay out
// entity spawns stay out too since the world already logs them
pub fn describe(event: &GameEvent<'_>) -> Option<String>
{
    let line: String = match event
    {
        GameEvent::BlockPlace { world, x, y, z, block, meta, player } =>
        {
            format!("{} placed {} at {}", actor(*player), blockName(*block, *meta), at(world, *x, *y, *z))
        }

        GameEvent::BlockBreak { world, x, y, z, block, meta, player } =>
        {
            format!("{} broke {} at {}", actor(*player), blockName(*block, *meta), at(world, *x, *y, *z))
        }

        GameEvent::BlockActivate { world, x, y, z, block, player } =>
        {
            let meta: i32 = world.getBlockMetadata(*x, *y, *z);
            format!("{} used {} at {}", name(player), blockName(*block, meta), at(world, *x, *y, *z))
        }

        GameEvent::ItemUse { player, stack, .. } => format!("{} used {}", name(player), stackName(stack)),

        GameEvent::ItemUseOnBlock { world, player, stack, x, y, z, face } =>
        {
            format!("{} used {} on {} face {face}", name(player), stackName(stack), at(world, *x, *y, *z))
        }

        GameEvent::ItemDestroyed { player, stack } => format!("{} wore out {}", name(player), stackName(stack)),
        GameEvent::ItemPickup { player, stack, .. } => format!("{} picked up {}", name(player), stackName(stack)),
        GameEvent::ItemToss { player, stack } => format!("{} dropped {}", name(player), stackName(stack)),
        GameEvent::ItemCrafted { player, stack } => format!("{} crafted {}", name(player), stackName(stack)),
        GameEvent::ItemSmelted { player, stack } => format!("{} took smelted {}", name(player), stackName(stack)),
        GameEvent::BucketFill { player, stack, .. } => format!("{} filled {}", name(player), stackName(stack)),

        // environment damage fires every tick before the hurt cooldown so only attacks by an entity log
        GameEvent::LivingAttack
        {
            entity,
            damage: DamageSource { m_attacker: Some(attacker), m_kind: kind, .. },
            amount,
        } =>
        {
            format!("{} attacked {} for {amount} by {}", name(attacker), name(entity), kind.name())
        }

        GameEvent::LivingDeath { entity, source } =>
        {
            format!("{} died at {} killed by {}", name(entity), position(entity), actor(*source))
        }

        GameEvent::PlayerSleep { player, x, y, z } =>
        {
            let world: Option<Rc<World>> = player.m_worldObj.get();
            let place: String = world.map_or_else(String::new, |world| at(&world, *x, *y, *z));
            format!("{} went to bed at {place}", name(player))
        }

        _ => return None,
    };
    Some(line)
}

pub fn write(line: &str, canceled: bool)
{
    let outcome: &str = if canceled { " canceled by a mod" } else { "" };
    log::debug(log::GAME, &format!("{line}{outcome}"));
}

pub fn name(entity: &Rc<Entity>) -> String
{
    if entity.instanceOf("EntityPlayer")
    {
        let username: String =
            entity_player::fields(entity).m_username.get().map(|name| name.to_string()).unwrap_or_default();
        return format!("player {username}");
    }

    let kind: String = entityKind(entity).map_or_else(|| String::from("unknown entity"), |kind| kind.to_string());
    format!("{kind} #{}", entity.m_entityId.get())
}

fn actor(entity: Option<&Rc<Entity>>) -> String { entity.map_or_else(|| String::from("nothing"), name) }

fn at(world: &Rc<World>, x: i32, y: i32, z: i32) -> String
{
    format!("({x}, {y}, {z}) in dimension {}", dimension(world))
}

fn position(entity: &Entity) -> String
{
    let world: i32 = entity.m_worldObj.get().map_or(0, |world| dimension(&world));
    format!("({:.1}, {:.1}, {:.1}) in dimension {world}", entity.m_posX.get(), entity.m_posY.get(), entity.m_posZ.get())
}

fn blockName(block: i32, meta: i32) -> String
{
    let id: String = registry::with(|registries|
    {
        registries.m_blocks.byNumeric(block).map_or_else(|| format!("block {block}"), |row| row.m_id.to_string())
    });

    if meta == 0
    {
        id
    }
    else
    {
        format!("{id}:{meta}")
    }
}

fn stackName(stack: &ItemStack) -> String
{
    let item: i32 = stack.m_itemID.get();
    let id: String = if item < FIRST_ITEM
    {
        blockName(item, 0)
    }
    else
    {
        registry::with(|registries|
        {
            registries.m_items.byNumeric(item).map_or_else(|| format!("item {item}"), |row| row.m_id.to_string())
        })
    };
    let damage: i32 = stack.getItemDamage();
    let suffix: String = if damage == 0
    {
        String::new()
    }
    else
    {
        format!(":{damage}")
    };
    format!("{}x {id}{suffix}", stack.m_stackSize.get())
}
