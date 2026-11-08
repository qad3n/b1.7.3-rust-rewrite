use crate::events;
use crate::host::context::{dimension, entityInfo, stackData, stackFrom, withWorld};
use game::entity::Entity;
use game::hooks::GameEvent;
use game::world::World;
use mod_protocol::abi::{BlockPos, EventData, EventEnvelope, EventKind, StackData};
use std::rc::Rc;

fn id(entity: &Entity) -> i32 { entity.m_entityId.get() }

fn post(world: Option<&Rc<World>>, kind: EventKind, data: EventData) -> EventEnvelope
{
    match world
    {
        Some(world) => withWorld(world, || events::post(kind, data)),
        None => events::post(kind, data),
    }
}

fn block(world: &Rc<World>, x: i32, y: i32, z: i32, block: i32, meta: i32, player: Option<&Rc<Entity>>) -> EventData
{
    EventData::Block {
        world: dimension(world),
        pos: BlockPos::new(x, y, z),
        block,
        meta,
        player: player.map(|player| id(player)),
    }
}

fn spawnDrops(world: &Rc<World>, x: i32, y: i32, z: i32, drops: &[StackData])
{
    for drop in drops
    {
        if let Some(stack) = stackFrom(drop)
        {
            let item: Rc<Entity> =
                game::entity::entity_item::create(world, x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5, stack);
            game::entity::entity_item::fields(&item).m_delayBeforeCanPickup.set(10);
            world.entityJoinedWorld(&item);
        }
    }
}

fn kindOf(event: &GameEvent<'_>) -> EventKind
{
    match event
    {
        GameEvent::WorldSave { .. } => EventKind::WorldSave,
        GameEvent::ChunkLoad { .. } => EventKind::ChunkLoad,
        GameEvent::ChunkUnload { .. } => EventKind::ChunkUnload,
        GameEvent::ChunkGenerate { .. } => EventKind::ChunkGenerate,
        GameEvent::BlockPlace { .. } => EventKind::BlockPlace,
        GameEvent::BlockBreak { .. } => EventKind::BlockBreak,
        GameEvent::BlockActivate { .. } => EventKind::BlockActivate,
        GameEvent::NeighborChange { .. } => EventKind::NeighborChange,
        GameEvent::ExplosionAffect { .. } => EventKind::ExplosionAffect,
        GameEvent::HarvestDrops { .. } => EventKind::BlockHarvestDrops,
        GameEvent::ItemUse { .. } => EventKind::ItemUse,
        GameEvent::ItemUseOnBlock { .. } => EventKind::ItemUseOnBlock,
        GameEvent::ItemDestroyed { .. } => EventKind::ItemDestroyed,
        GameEvent::ItemPickup { .. } => EventKind::ItemPickup,
        GameEvent::ItemToss { .. } => EventKind::ItemToss,
        GameEvent::EntityJoinWorld { .. } => EventKind::EntityJoinWorld,
        GameEvent::EntityTick { .. } => EventKind::EntityTick,
        GameEvent::LivingAttack { .. } => EventKind::LivingAttack,
        GameEvent::LivingDeath { .. } => EventKind::LivingDeath,
        GameEvent::LivingDrops { .. } => EventKind::LivingDrops,
        GameEvent::LivingSpawnCheck { .. } => EventKind::LivingSpawnCheck,
        GameEvent::PlayerSleep { .. } => EventKind::PlayerSleep,
        GameEvent::PlayerBreakSpeed { .. } => EventKind::PlayerBreakSpeed,
        GameEvent::PlayerCanHarvest { .. } => EventKind::PlayerCanHarvest,
        GameEvent::ItemCrafted { .. } => EventKind::ItemCrafted,
        GameEvent::ItemSmelted { .. } => EventKind::ItemSmelted,
        GameEvent::BucketFill { .. } => EventKind::BucketFill,
    }
}

// true cancels the game action, mutable fields of the event carry listener changes back
pub fn fire(event: GameEvent<'_>) -> bool
{
    if let GameEvent::WorldSave { world } = &event
    {
        // a vanilla world open after a modded one in the same game gets no loader folder
        if crate::side::isAuthority(world) && crate::play::active()
        {
            crate::persist::saveWorld(world);
        }
    }
    let kind: EventKind = kindOf(&event);
    let tracked: bool = matches!(kind, EventKind::LivingAttack | EventKind::LivingHurt);
    if !events::hasListeners(kind) && !(tracked && events::hasListeners(EventKind::LivingHurt))
    {
        return false;
    }
    match event
    {
        GameEvent::WorldSave { world } =>
        {
            post(Some(world), kind, EventData::World { world: dimension(world) }).canceled
        }
        GameEvent::ChunkLoad { world, x, z }
        | GameEvent::ChunkUnload { world, x, z }
        | GameEvent::ChunkGenerate { world, x, z } =>
        {
            post(Some(world), kind, EventData::Chunk { world: dimension(world), x, z }).canceled
        }
        GameEvent::BlockPlace { world, x, y, z, block: id, meta, player }
        | GameEvent::BlockBreak { world, x, y, z, block: id, meta, player } =>
        {
            post(Some(world), kind, block(world, x, y, z, id, meta, player)).canceled
        }
        GameEvent::BlockActivate { world, x, y, z, block: id, player } =>
        {
            let meta: i32 = world.getBlockMetadata(x, y, z);
            post(Some(world), kind, block(world, x, y, z, id, meta, Some(player))).canceled
        }
        GameEvent::NeighborChange { world, x, y, z, block: id, neighbor } =>
        {
            let data: EventData =
                EventData::Neighbor { world: dimension(world), pos: BlockPos::new(x, y, z), block: id, neighbor };
            post(Some(world), kind, data).canceled
        }
        GameEvent::ExplosionAffect { world, x, y, z, block: id } =>
        {
            post(Some(world), kind, block(world, x, y, z, id, world.getBlockMetadata(x, y, z), None)).canceled
        }
        GameEvent::HarvestDrops { world, x, y, z, block: id, meta } =>
        {
            let data: EventData = EventData::Drops {
                world: dimension(world),
                pos: BlockPos::new(x, y, z),
                block: id,
                meta,
                drops: Vec::new(),
            };
            let envelope: EventEnvelope = post(Some(world), kind, data);
            if let EventData::Drops { drops, .. } = &envelope.data
            {
                spawnDrops(world, x, y, z, drops);
            }
            envelope.canceled
        }
        GameEvent::ItemUse { world, player, stack } =>
        {
            let data: EventData = EventData::ItemUse {
                world: dimension(world),
                player: id(player),
                stack: Some(stackData(stack)),
                pos: None,
                face: -1,
            };
            post(Some(world), kind, data).canceled
        }
        GameEvent::ItemUseOnBlock { world, player, stack, x, y, z, face } =>
        {
            let data: EventData = EventData::ItemUse {
                world: dimension(world),
                player: id(player),
                stack: Some(stackData(stack)),
                pos: Some(BlockPos::new(x, y, z)),
                face,
            };
            post(Some(world), kind, data).canceled
        }
        GameEvent::ItemDestroyed { player, stack } =>
        {
            let world: Option<Rc<World>> = player.m_worldObj.get();
            let data: EventData = EventData::Item {
                world: world.as_ref().map_or(0, |world| dimension(world)),
                player: id(player),
                stack: stackData(stack),
            };
            post(world.as_ref(), kind, data).canceled
        }
        GameEvent::ItemPickup { player, stack, .. }
        | GameEvent::ItemToss { player, stack }
        | GameEvent::ItemCrafted { player, stack }
        | GameEvent::ItemSmelted { player, stack } =>
        {
            let world: Option<Rc<World>> = player.m_worldObj.get();
            let data: EventData = EventData::Item {
                world: world.as_ref().map_or(0, |world| dimension(world)),
                player: id(player),
                stack: stackData(stack),
            };
            post(world.as_ref(), kind, data).canceled
        }
        GameEvent::BucketFill { world, player, stack } =>
        {
            let data: EventData = EventData::ItemUse {
                world: dimension(world),
                player: id(player),
                stack: Some(stackData(stack)),
                pos: None,
                face: -1,
            };
            post(Some(world), kind, data).canceled
        }
        GameEvent::EntityJoinWorld { world, entity } =>
        {
            post(Some(world), kind, EventData::Entity { entity: entityInfo(entity) }).canceled
        }
        GameEvent::EntityTick { entity } =>
        {
            post(entity.m_worldObj.get().as_ref(), kind, EventData::Entity { entity: entityInfo(entity) }).canceled
        }
        GameEvent::LivingAttack { entity, source, amount } =>
        {
            let world: Option<Rc<World>> = entity.m_worldObj.get();
            let data: EventData = EventData::Attack {
                entity: entityInfo(entity),
                source: source.map(|source| id(source)),
                amount: *amount,
            };
            let attack: EventEnvelope = post(world.as_ref(), EventKind::LivingAttack, data.clone());
            if attack.canceled
            {
                return true;
            }
            // hurt runs after attack and may change the damage that lands
            let hurt: EventEnvelope = post(world.as_ref(), EventKind::LivingHurt, data);
            if let EventData::Attack { amount: changed, .. } = hurt.data
            {
                *amount = changed;
            }
            hurt.canceled
        }
        GameEvent::LivingDeath { entity, source } =>
        {
            let data: EventData = EventData::Death {
                entity: entityInfo(entity),
                source: source.map(|source| id(source)),
                drops: Vec::new(),
            };
            post(entity.m_worldObj.get().as_ref(), kind, data).canceled
        }
        GameEvent::LivingDrops { entity, source } =>
        {
            let world: Option<Rc<World>> = entity.m_worldObj.get();
            let data: EventData = EventData::Death {
                entity: entityInfo(entity),
                source: source.map(|source| id(source)),
                drops: Vec::new(),
            };
            let envelope: EventEnvelope = post(world.as_ref(), kind, data);
            if let (Some(world), EventData::Death { drops, .. }) = (&world, &envelope.data)
            {
                spawnDrops(
                    world,
                    entity.m_posX.get() as i32,
                    entity.m_posY.get() as i32,
                    entity.m_posZ.get() as i32,
                    drops,
                );
            }
            envelope.canceled
        }
        GameEvent::LivingSpawnCheck { entity, allowed } =>
        {
            let pos: BlockPos =
                BlockPos::new(entity.m_posX.get() as i32, entity.m_posY.get() as i32, entity.m_posZ.get() as i32);
            let envelope: EventEnvelope =
                post(entity.m_worldObj.get().as_ref(), kind, EventData::Spawn { entity: entityInfo(entity), pos });
            if let Some(result) = envelope.result
            {
                *allowed = result != 0;
            }
            envelope.canceled
        }
        GameEvent::PlayerSleep { player, x, y, z } =>
        {
            let world: Option<Rc<World>> = player.m_worldObj.get();
            let dimensionId: i32 = world.as_ref().map_or(0, |world| dimension(world));
            let data: EventData = EventData::Block {
                world: dimensionId,
                pos: BlockPos::new(x, y, z),
                block: 0,
                meta: 0,
                player: Some(id(player)),
            };
            let envelope: EventEnvelope = post(world.as_ref(), kind, data);
            envelope.canceled || envelope.result == Some(0)
        }
        GameEvent::PlayerBreakSpeed { player, block: target, speed } =>
        {
            let world: Option<Rc<World>> = player.m_worldObj.get();
            let data: EventData = EventData::BreakSpeed {
                player: id(player),
                world: world.as_ref().map_or(0, |world| dimension(world)),
                pos: BlockPos::default(),
                block: target.m_blockID,
                speed: *speed,
            };
            if let EventData::BreakSpeed { speed: changed, .. } = post(world.as_ref(), kind, data).data
            {
                *speed = changed;
            }
            false
        }
        GameEvent::PlayerCanHarvest { player, block: target, allowed } =>
        {
            let data: EventData = EventData::Harvest { player: id(player), block: target.m_blockID, meta: 0 };
            if let Some(result) = post(player.m_worldObj.get().as_ref(), kind, data).result
            {
                *allowed = result != 0;
            }
            false
        }
    }
}
