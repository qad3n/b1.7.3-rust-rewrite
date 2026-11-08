use super::context::{self, entityInfo, stackData, stackFrom};
use super::{ok, unit};
use crate::network::loader;
use crate::state::ModRuntime;
use crate::tiles::{self, ModTileExt};
use game::entity::{Entity, entity_player};
use game::inventory::IInventory;
use game::item::ItemStack;
use game::math::AxisAlignedBB;
use game::tileentity::TileEntity;
use game::world::World;
use mod_protocol::abi::{BlockInfo, BlockPos, EntityField, HostCall, HostReply};
use std::rc::Rc;

fn world(dimension: i32) -> Result<Rc<World>, String>
{
    context::world(dimension).ok_or_else(|| format!("dimension {dimension} is not loaded"))
}

fn entity(dimension: i32, id: i32) -> Result<Rc<Entity>, String>
{
    context::entityIn(dimension, id).ok_or_else(|| format!("no entity {id} in dimension {dimension}"))
}

// players are looked up in every loaded world, a guest only knows their entity id
pub fn player(id: i32) -> Result<Rc<Entity>, String>
{
    context::worlds()
        .iter()
        .flat_map(|world| world.m_playerEntities.toVec())
        .find(|player| player.m_entityId.get() == id)
        .ok_or_else(|| format!("no player with entity id {id}"))
}

fn authoritative(world: &World) -> Result<(), String>
{
    if crate::side::isAuthority(world)
    {
        Ok(())
    }
    else
    {
        Err("only the side that owns the world may change it".to_owned())
    }
}

fn modTile(dimension: i32, pos: BlockPos) -> Result<Rc<TileEntity>, String>
{
    let tile: Rc<TileEntity> = world(dimension)?
        .getBlockTileEntity(pos.x, pos.y, pos.z)
        .ok_or_else(|| format!("no tile entity at {pos:?}"))?;
    if tile.tryExt::<ModTileExt>().is_none()
    {
        return Err(format!("the tile entity at {pos:?} is not a mod tile entity"));
    }
    Ok(tile)
}

// any tile with an inventory, a mod tile or a vanilla chest, furnace or dispenser
fn inventoryTile(dimension: i32, pos: BlockPos) -> Result<Rc<TileEntity>, String>
{
    world(dimension)?
        .getBlockTileEntity(pos.x, pos.y, pos.z)
        .filter(|tile| tile.instanceOf("IInventory"))
        .ok_or_else(|| format!("no inventory at {pos:?}"))
}

fn blockInfo(block: &Rc<game::block::Block>) -> BlockInfo
{
    BlockInfo {
        opacity: game::block::Block::lightOpacity(block.m_blockID),
        light: game::block::Block::lightValue(block.m_blockID),
        hardness: block.m_blockHardness.get(),
        opaque: block.isOpaqueCube(),
        solid: block.m_blockMaterial.isSolid(),
        liquid: block.m_blockMaterial.getIsLiquid(),
        replaceable: block.m_blockMaterial.getIsGroundCover(),
    }
}

fn living(entity: &Entity) -> Option<&game::entity::entity_living::LivingFields>
{
    entity.instanceOf("EntityLiving").then(|| game::entity::entity_living::fields(entity))
}

fn creature(entity: &Entity) -> Option<&game::entity::entity_creature::CreatureFields>
{
    entity.instanceOf("EntityCreature").then(|| game::entity::entity_creature::fields(entity))
}

fn mob(entity: &Entity) -> Option<&game::entity::entity_mob::MobFields>
{
    entity.instanceOf("EntityMob").then(|| game::entity::entity_mob::fields(entity))
}

// World.doEvent 2001 packs the block id and its metadata times 256
const BLOCK_BREAK_EVENT: i32 = 2001;
const META_SHIFT: i32 = 256;

fn movingBlock(entity: &Entity) -> Option<&game::entity::entity_falling_sand::FallingSandFields>
{
    entity.instanceOf("EntityFallingSand").then(|| game::entity::entity_falling_sand::fields(entity))
}

fn entityField(entity: &Rc<Entity>, field: EntityField) -> HostReply
{
    match field
    {
        EntityField::FallDistance => HostReply::Float(entity.m_fallDistance.get()),
        EntityField::OnGround => HostReply::Bool(entity.m_onGround.get()),
        EntityField::Sneaking => HostReply::Bool(entity.isSneaking()),
        EntityField::InWater => HostReply::Bool(entity.m_inWater.get()),
        EntityField::Riding => HostReply::Bool(entity.m_ridingEntity.get().is_some()),
        EntityField::FireTicks => HostReply::Float(entity.m_fire.get() as f32),
        EntityField::Air => HostReply::Float(entity.m_air.get() as f32),
        EntityField::TicksExisted => HostReply::Float(entity.m_ticksExisted.get() as f32),
        EntityField::RidingId =>
        {
            HostReply::Float(entity.m_ridingEntity.get().map_or(-1, |other| other.m_entityId.get()) as f32)
        }
        EntityField::RiderId =>
        {
            HostReply::Float(entity.m_riddenByEntity.get().map_or(-1, |other| other.m_entityId.get()) as f32)
        }
        EntityField::MoveForward => HostReply::Float(living(entity).map_or(0.0, |fields| fields.m_moveForward.get())),
        EntityField::MoveStrafe => HostReply::Float(living(entity).map_or(0.0, |fields| fields.m_moveStrafing.get())),
        EntityField::Jumping => HostReply::Bool(living(entity).is_some_and(|fields| fields.m_isJumping.get())),
        EntityField::Yaw => HostReply::Float(entity.m_rotationYaw.get()),
        EntityField::Pitch => HostReply::Float(entity.m_rotationPitch.get()),
        EntityField::AirJumps => HostReply::Float(crate::steering::airJumps(entity) as f32),
        EntityField::MountSpeed => HostReply::Float(crate::steering::speed(entity)),
        EntityField::TargetId => HostReply::Float(
            creature(entity)
                .and_then(|fields| fields.m_playerToAttack.get())
                .map_or(-1, |target| target.m_entityId.get()) as f32,
        ),
        EntityField::AttackStrength =>
        {
            HostReply::Float(mob(entity).map_or(0, |fields| fields.m_attackStrength.get()) as f32)
        }
        EntityField::Width => HostReply::Float(entity.m_width.get()),
        EntityField::Height => HostReply::Float(entity.m_height.get()),
        EntityField::SwingProgress =>
        {
            HostReply::Float(living(entity).map_or(0.0, |fields| fields.m_swingProgress.get()))
        }
        EntityField::Block => HostReply::Float(movingBlock(entity).map_or(0, |fields| fields.m_blockID.get()) as f32),
        EntityField::Collided => HostReply::Bool(entity.m_isCollided.get()),
        EntityField::Health => HostReply::Float(living(entity).map_or(0, |fields| fields.m_health.get()) as f32),
        EntityField::Bottom => HostReply::Float(entity.m_boundingBox.m_minY.get() as f32),
        EntityField::MoveSpeed => HostReply::Float(living(entity).map_or(0.0, |fields| fields.m_moveSpeed.get())),
        EntityField::StepHeight => HostReply::Float(entity.m_stepHeight.get()),
    }
}

// riding is read only, a mount changes through the riding api
fn setEntityField(entity: &Rc<Entity>, field: EntityField, value: f64)
{
    match field
    {
        EntityField::FallDistance => entity.m_fallDistance.set(value as f32),
        EntityField::OnGround => entity.m_onGround.set(value > 0.0),
        EntityField::Sneaking => entity.setSneaking(value > 0.0),
        EntityField::InWater => entity.m_inWater.set(value > 0.0),
        EntityField::Riding =>
        {}
        EntityField::FireTicks => entity.m_fire.set(value as i32),
        EntityField::Air => entity.m_air.set(value as i32),
        EntityField::TicksExisted => entity.m_ticksExisted.set(value as i32),
        EntityField::RidingId | EntityField::RiderId =>
        {}
        EntityField::MoveForward =>
        {
            living(entity).into_iter().for_each(|fields| fields.m_moveForward.set(value as f32))
        }
        EntityField::MoveStrafe =>
        {
            living(entity).into_iter().for_each(|fields| fields.m_moveStrafing.set(value as f32))
        }
        EntityField::Jumping => living(entity).into_iter().for_each(|fields| fields.m_isJumping.set(value > 0.0)),
        EntityField::Yaw => entity.m_rotationYaw.set(value as f32),
        EntityField::Pitch => entity.m_rotationPitch.set(value as f32),
        EntityField::AirJumps => crate::steering::setAirJumps(entity, value as i32),
        EntityField::MountSpeed => crate::steering::setSpeed(entity, value as f32),
        EntityField::TargetId =>
        {
            if let Some(fields) = creature(entity)
            {
                let target: Option<Rc<Entity>> =
                    entity.m_worldObj.get().and_then(|world| context::entityById(&world, value as i32));
                fields.m_playerToAttack.set(target);
            }
        }
        EntityField::AttackStrength =>
        {
            mob(entity).into_iter().for_each(|fields| fields.m_attackStrength.set(value as i32))
        }
        EntityField::Width => entity.setSize(value as f32, entity.m_height.get()),
        EntityField::Height => entity.setSize(entity.m_width.get(), value as f32),
        EntityField::SwingProgress =>
        {}
        EntityField::Block => movingBlock(entity).into_iter().for_each(|fields| fields.m_blockID.set(value as i32)),
        EntityField::Collided | EntityField::Bottom =>
        {}
        EntityField::Health => living(entity).into_iter().for_each(|fields| fields.m_health.set(value as i32)),
        EntityField::MoveSpeed => living(entity).into_iter().for_each(|fields| fields.m_moveSpeed.set(value as f32)),
        EntityField::StepHeight => entity.m_stepHeight.set(value as f32),
    }
}

fn heldStack(player: &Rc<Entity>) -> Option<Rc<ItemStack>> { entity_player::inventory(player).getCurrentItem() }

pub fn handle(runtime: &Rc<ModRuntime>, call: HostCall) -> HostReply
{
    match call
    {
        HostCall::IsAuthority { world: dimension } =>
        {
            ok(world(dimension), |world| HostReply::Bool(crate::side::isAuthority(&world)))
        }
        HostCall::GetBlock { world: dimension, pos } =>
        {
            ok(world(dimension), |world| HostReply::Int(world.getBlockId(pos.x, pos.y, pos.z)))
        }
        HostCall::Height { world: dimension, x, z } =>
        {
            ok(world(dimension), |world| HostReply::Int(world.getHeightValue(x, z)))
        }
        HostCall::SyncedData { world: dimension, id } =>
        {
            ok(entity(dimension, id), |entity| HostReply::Bytes(crate::entities::synced(&entity)))
        }
        // only the dedicated server broadcasts, single player and a client keep the bytes local
        HostCall::SetSyncedData { world: dimension, id, data } =>
        {
            unit(entity(dimension, id).map(|entity| crate::entities::publishSynced(&entity, data)))
        }
        HostCall::EntityField { world: dimension, id, field } =>
        {
            ok(entity(dimension, id), |entity| entityField(&entity, field))
        }
        HostCall::SetEntityField { world: dimension, id, field, value } =>
        {
            unit(entity(dimension, id).map(|entity| setEntityField(&entity, field, value)))
        }
        HostCall::RayTrace { world: dimension, from, to, liquids } => ok(world(dimension), |world| {
            let start = game::math::Vec3D::createVector(from[0], from[1], from[2]);
            let end = game::math::Vec3D::createVector(to[0], to[1], to[2]);
            HostReply::Hit(world.rayTraceBlocks_do(&start, &end, liquids).map(|hit| mod_protocol::abi::RayHit {
                pos: BlockPos::new(hit.m_blockX, hit.m_blockY, hit.m_blockZ),
                face: hit.m_sideHit,
                point: [hit.m_hitVec.x(), hit.m_hitVec.y(), hit.m_hitVec.z()],
            }))
        }),
        HostCall::EntitiesAlong { world: dimension, from, to, radius, exclude } => ok(world(dimension), |world| {
            let start = game::math::Vec3D::createVector(from[0], from[1], from[2]);
            let end = game::math::Vec3D::createVector(to[0], to[1], to[2]);
            let sweep: Rc<AxisAlignedBB> = AxisAlignedBB::getBoundingBoxFromPool(
                from[0].min(to[0]),
                from[1].min(to[1]),
                from[2].min(to[2]),
                from[0].max(to[0]),
                from[1].max(to[1]),
                from[2].max(to[2]),
            )
            .expand(radius + 1.0, radius + 1.0, radius + 1.0);
            let mut hits: Vec<(f64, Rc<Entity>)> = world
                .getEntitiesWithinAABB("Entity", &sweep)
                .into_iter()
                .filter(|entity| Some(entity.m_entityId.get()) != exclude && entity.canBeCollidedWith())
                .filter_map(|entity| {
                    let hit = entity.m_boundingBox.expand(radius, radius, radius).clip(&start, &end)?;
                    Some((start.distanceTo(&hit.m_hitVec), entity))
                })
                .collect();
            hits.sort_by(|a, b| a.0.total_cmp(&b.0));
            HostReply::Entities(hits.iter().map(|(_, entity)| entityInfo(entity)).collect())
        }),
        HostCall::Mount { world: dimension, id, vehicle } => unit(entity(dimension, id).and_then(|rider| {
            let vehicle: Option<Rc<Entity>> = match vehicle
            {
                Some(vehicle) => Some(entity(dimension, vehicle).or_else(|_| player(vehicle))?),
                None => None,
            };
            rider.mountEntity(vehicle.as_ref());
            Ok(())
        })),
        HostCall::BlockInfo { block } =>
        {
            HostReply::BlockInfo(game::block::Block::byId(block).map(|block| blockInfo(&block)))
        }
        HostCall::GetMeta { world: dimension, pos } =>
        {
            ok(world(dimension), |world| HostReply::Int(world.getBlockMetadata(pos.x, pos.y, pos.z)))
        }
        HostCall::SetBlock { world: dimension, pos, block, meta, notify } =>
        {
            ok(world(dimension).and_then(|world| authoritative(&world).map(|()| world)), |world| {
                let changed: bool = if notify
                {
                    world.setBlockAndMetadataWithNotify(pos.x, pos.y, pos.z, block, meta)
                }
                else
                {
                    world.setBlockAndMetadata(pos.x, pos.y, pos.z, block, meta)
                };
                HostReply::Bool(changed)
            })
        }
        HostCall::BlockEffect { world: dimension, pos, block, meta } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            world.doEvent(BLOCK_BREAK_EVENT, pos.x, pos.y, pos.z, block + meta * META_SHIFT);
            Ok(())
        })),
        HostCall::SetMeta { world: dimension, pos, meta } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            world.setBlockMetadataWithNotify(pos.x, pos.y, pos.z, meta);
            Ok(())
        })),
        HostCall::NotifyNeighbors { world: dimension, pos, block } => unit(world(dimension).map(|world| {
            world.notifyBlocksOfNeighborChange(pos.x, pos.y, pos.z, block);
        })),
        HostCall::ScheduleTick { world: dimension, pos, block, delay } => unit(world(dimension).map(|world| {
            world.scheduleBlockUpdate(pos.x, pos.y, pos.z, block, delay);
        })),
        HostCall::MarkForRender { world: dimension, pos } => unit(world(dimension).map(|world| {
            world.markBlockNeedsUpdate(pos.x, pos.y, pos.z);
        })),
        HostCall::Light { world: dimension, pos } =>
        {
            ok(world(dimension), |world| HostReply::Int(world.getBlockLightValue(pos.x, pos.y, pos.z)))
        }
        HostCall::IsPowered { world: dimension, pos } =>
        {
            ok(world(dimension), |world| HostReply::Bool(world.isBlockIndirectlyGettingPowered(pos.x, pos.y, pos.z)))
        }
        HostCall::WorldTime { world: dimension } => ok(world(dimension), |world| HostReply::Long(world.getWorldTime())),
        HostCall::WorldRandom { world: dimension, bound } =>
        {
            ok(world(dimension), |world| HostReply::Int(world.m_rand.nextIntBounded(bound.max(1))))
        }
        HostCall::TileData { world: dimension, pos } =>
        {
            ok(modTile(dimension, pos), |tile| HostReply::Bytes(tiles::ext(&tile).m_data.borrow().clone()))
        }
        HostCall::SetTileData { world: dimension, pos, data } => unit(modTile(dimension, pos).map(|tile| {
            *tiles::ext(&tile).m_data.borrow_mut() = data;
            tile.onInventoryChanged();
            crate::network::sync::tileChanged(&tile);
        })),
        HostCall::TileSlot { world: dimension, pos, slot } => ok(inventoryTile(dimension, pos), |tile| {
            HostReply::Stack(tile.getStackInSlot(slot).map(|stack| stackData(&stack)))
        }),
        HostCall::SetTileSlot { world: dimension, pos, slot, stack } =>
        {
            unit(inventoryTile(dimension, pos).map(|tile| {
                tile.setInventorySlotContents(slot, stack.as_ref().and_then(stackFrom));
            }))
        }
        HostCall::SpawnItem { world: dimension, pos, stack } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            let stack: Rc<ItemStack> = stackFrom(&stack).ok_or("cannot spawn an empty or unknown stack")?;
            let item: Rc<Entity> = game::entity::entity_item::create(&world, pos[0], pos[1], pos[2], stack);
            game::entity::entity_item::fields(&item).m_delayBeforeCanPickup.set(10);
            world.entityJoinedWorld(&item);
            Ok(())
        })),
        HostCall::SpawnEntity { world: dimension, kind, pos, data } => ok(
            world(dimension).and_then(|world| {
                authoritative(&world)?;
                crate::entities::spawn(&world, &kind, pos, data)
            }),
            HostReply::Int,
        ),
        HostCall::Entity { world: dimension, id } =>
        {
            HostReply::Entity(context::entityIn(dimension, id).map(|entity| entityInfo(&entity)))
        }
        HostCall::EntitiesNear { world: dimension, pos, radius } => ok(world(dimension), |world| {
            let bounds: Rc<AxisAlignedBB> = AxisAlignedBB::getBoundingBoxFromPool(
                pos[0] - radius,
                pos[1] - radius,
                pos[2] - radius,
                pos[0] + radius,
                pos[1] + radius,
                pos[2] + radius,
            );
            let found: Vec<Rc<Entity>> = world.getEntitiesWithinAABB("Entity", &bounds);
            HostReply::Entities(found.iter().map(entityInfo).collect())
        }),
        HostCall::EntityData { world: dimension, id } =>
        {
            ok(entity(dimension, id), |entity| HostReply::Bytes(crate::entities::data(&entity, runtime.id())))
        }
        HostCall::SetEntityData { world: dimension, id, data } =>
        {
            unit(entity(dimension, id).map(|entity| crate::entities::setData(&entity, runtime.id(), data)))
        }
        HostCall::MoveEntity { world: dimension, id, pos, motion, rotation } =>
        {
            unit(entity(dimension, id).map(|entity| {
                let [yaw, pitch] = rotation.unwrap_or([entity.m_rotationYaw.get(), entity.m_rotationPitch.get()]);
                if let Some([x, y, z]) = pos
                {
                    entity.setLocationAndAngles(x, y, z, yaw, pitch);
                }
                else if rotation.is_some()
                {
                    entity.m_rotationYaw.set(yaw);
                    entity.m_rotationPitch.set(pitch);
                }
                if let Some([x, y, z]) = motion
                {
                    entity.setVelocity(x, y, z);
                }
            }))
        }
        HostCall::DamageEntity { world: dimension, id, source, amount } => ok(entity(dimension, id), |target| {
            let attacker: Option<Rc<Entity>> = source.and_then(|source| context::entityIn(dimension, source));
            HostReply::Bool(target.attackEntityFrom(attacker.as_ref(), amount))
        }),
        HostCall::KillEntity { world: dimension, id } =>
        {
            unit(entity(dimension, id).map(|entity| entity.setEntityDead()))
        }
        HostCall::MoveTowards { world: dimension, id, pos, speed } => unit(entity(dimension, id).map(|entity| {
            crate::entities::moveTowards(&entity, pos, speed);
        })),
        HostCall::Players { world: dimension } =>
        {
            let worlds: Vec<Rc<World>> = match dimension
            {
                Some(dimension) => context::world(dimension).into_iter().collect(),
                None => context::worlds(),
            };
            HostReply::Entities(
                worlds
                    .iter()
                    .flat_map(|world| world.m_playerEntities.toVec())
                    .map(|player| entityInfo(&player))
                    .collect(),
            )
        }
        HostCall::Chat { player: target, message } => unit(match target
        {
            Some(id) => player(id).map(|player| player.addChatMessage(&message)),
            None =>
            {
                for world in context::worlds()
                {
                    for player in world.m_playerEntities.toVec()
                    {
                        player.addChatMessage(&message);
                    }
                }
                Ok(())
            }
        }),
        HostCall::Give { player: id, stack } => unit(player(id).and_then(|player| {
            let stack: Rc<ItemStack> = stackFrom(&stack).ok_or("cannot give an empty or unknown stack")?;
            if !entity_player::inventory(&player).addItemStackToInventory(&stack)
            {
                entity_player::dropPlayerItem(&player, Some(stack));
            }
            Ok(())
        })),
        HostCall::HeldStack { player: id } =>
        {
            ok(player(id), |player| HostReply::Stack(heldStack(&player).map(|stack| stackData(&stack))))
        }
        HostCall::SetHeldStack { player: id, stack } => unit(player(id).map(|player| {
            let inventory = entity_player::inventory(&player);
            inventory.setInventorySlotContents(inventory.m_currentItem.get(), stack.as_ref().and_then(stackFrom));
        })),
        HostCall::PlaySound { world: dimension, pos, sound, volume, pitch } => unit(world(dimension).map(|world| {
            world.playSoundEffect(pos[0], pos[1], pos[2], &sound, volume, pitch);
            let message = loader::Sound { world: dimension, pos, sound, volume, pitch };
            loader::broadcastNear(loader::SOUND, &world, pos, &message);
        })),
        HostCall::Particle { world: dimension, name, pos, motion } => unit(world(dimension).map(|world| {
            world.spawnParticle(&name, pos[0], pos[1], pos[2], motion[0], motion[1], motion[2]);
            let message = loader::Particle { world: dimension, name, pos, motion };
            loader::broadcastNear(loader::PARTICLE, &world, pos, &message);
        })),
        HostCall::Lightning { world: dimension, pos } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            world.addWeatherEffect(game::entity::entity_lightning_bolt::create(&world, pos[0], pos[1], pos[2]));
            Ok(())
        })),
        HostCall::InventorySlot { player: id, slot } => ok(player(id), |player| {
            HostReply::Stack(entity_player::inventory(&player).getStackInSlot(slot).map(|stack| stackData(&stack)))
        }),
        HostCall::SetInventorySlot { player: id, slot, stack } => unit(player(id).map(|player| {
            entity_player::inventory(&player).setInventorySlotContents(slot, stack.as_ref().and_then(stackFrom));
        })),
        HostCall::CursorStack { player: id } => ok(player(id), |player| {
            HostReply::Stack(entity_player::inventory(&player).getItemStack().map(|stack| stackData(&stack)))
        }),
        HostCall::SetCursorStack { player: id, stack } => unit(player(id).map(|player| {
            entity_player::inventory(&player).setItemStack(stack.as_ref().and_then(stackFrom));
            crate::netplay::syncCursor(&player);
        })),
        HostCall::TextWidth { text } => HostReply::Int(crate::netplay::textWidth(&text)),
        HostCall::Explode { world: dimension, pos, power } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            world.createExplosion(None, pos[0], pos[1], pos[2], power);
            Ok(())
        })),
        other => crate::extensions::handle(runtime, other),
    }
}
