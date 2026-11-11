use super::context::{self, entityInfo, stackData, stackFrom};
use super::{ok, unit};
use crate::network::loader;
use crate::state::ModRuntime;
use crate::tiles::{self, ModTileExt};
use game::damage::{DamageKind, DamageSource};
use game::entity::{Entity, entity_player};
use game::inventory::IInventory;
use game::item::ItemStack;
use game::math::AxisAlignedBB;
use game::tileentity::TileEntity;
use game::world::World;
use mod_protocol::abi::{BlockInfo, BlockPos, EntityField, HostCall, HostReply, StackRef};
use std::rc::Rc;

// ItemDye's bone meal damage
const BONE_MEAL: i32 = 15;

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

fn anyTile(dimension: i32, pos: BlockPos) -> Result<Rc<TileEntity>, String>
{
    world(dimension)?.getBlockTileEntity(pos.x, pos.y, pos.z).ok_or_else(|| format!("no tile entity at {pos:?}"))
}

fn attachment(slots: &std::cell::RefCell<common::nbt::attachments::Attachments>, owner: &str) -> HostReply
{
    HostReply::Bytes(slots.borrow().get(owner).cloned().unwrap_or_default())
}

// an empty value removes the slot, so a cleared attachment leaves no trace in the save
fn setAttachment(slots: &std::cell::RefCell<common::nbt::attachments::Attachments>, owner: &str, data: Vec<u8>)
{
    let mut slots = slots.borrow_mut();
    if data.is_empty()
    {
        slots.remove(owner);
    }
    else
    {
        slots.insert(owner.to_owned(), data);
    }
}

fn stackAt(at: StackRef) -> Result<Rc<game::item::ItemStack>, String>
{
    let stack: Option<Rc<game::item::ItemStack>> = match at
    {
        StackRef::Held { player: id } => heldStack(&player(id)?),
        StackRef::Inventory { player: id, slot } =>
        {
            let owner: Rc<Entity> = player(id)?;
            entity_player::inventory(&owner).getStackInSlot(slot)
        }
        StackRef::Tile { world: dimension, pos, slot } => inventoryTile(dimension, pos)?.getStackInSlot(slot),
    };
    stack.ok_or_else(|| format!("no stack at {at:?}"))
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

fn movingBlock(entity: &Entity) -> Option<&game::entity::entity_falling_sand::FallingSandFields>
{
    entity.instanceOf("EntityFallingSand").then(|| game::entity::entity_falling_sand::fields(entity))
}

mod shot
{
    pub const NOBODY: i32 = -1;
    pub const NOT_A_PROJECTILE: i32 = -2;
    pub const THROWN: [&str; 3] = ["EntitySnowball", "EntityEgg", "EntityFireball"];
}

fn shooter(entity: &Entity) -> i32
{
    let owner: Option<Rc<Entity>> = if entity.instanceOf("EntityArrow")
    {
        game::entity::entity_arrow::fields(entity).m_owner.get()
    }
    else if shot::THROWN.iter().any(|class| entity.instanceOf(class))
    {
        game::entity::entity_snowball::throwable(entity).m_thrower.get()
    }
    else
    {
        return shot::NOT_A_PROJECTILE;
    };
    owner.map_or(shot::NOBODY, |owner| owner.m_entityId.get())
}

fn setShooter(entity: &Entity, id: i32)
{
    let owner: Option<Rc<Entity>> =
        entity.m_worldObj.get().and_then(|world| context::entityIn(context::dimension(&world), id));
    if entity.instanceOf("EntityArrow")
    {
        let arrow: &game::entity::entity_arrow::ArrowFields = game::entity::entity_arrow::fields(entity);
        // EntityArrow's player constructor, only a player's arrow can be picked up
        arrow.m_doesArrowBelongToPlayer.set(owner.as_ref().is_some_and(|owner| owner.instanceOf("EntityPlayer")));
        arrow.m_owner.set(owner);
    }
    else if shot::THROWN.iter().any(|class| entity.instanceOf(class))
    {
        game::entity::entity_snowball::throwable(entity).m_thrower.set(owner);
    }
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
        EntityField::MaxHealth => HostReply::Float(game::hooks::maxHealth(entity) as f32),
        EntityField::DeathTime => HostReply::Float(living(entity).map_or(0, |fields| fields.m_deathTime.get()) as f32),
        EntityField::BlockMeta =>
        {
            HostReply::Float(movingBlock(entity).map_or(0, |fields| fields.m_metadata.get()) as f32)
        }
        EntityField::Invisible => HostReply::Bool(context::invisible(entity.m_entityId.get())),
        EntityField::ItemInUse =>
        {
            HostReply::Float(entity.m_ext.player().map_or(0, |player| player.m_itemInUseCount.get()) as f32)
        }
        EntityField::Shooter => HostReply::Float(shooter(entity) as f32),
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
        EntityField::MaxHealth => context::setMaxHealth(entity.m_entityId.get(), value as i32),
        EntityField::Invisible => context::setInvisible(entity.m_entityId.get(), value != 0.0),
        EntityField::ItemInUse =>
        {}
        EntityField::Shooter => setShooter(entity, value as i32),
        EntityField::DeathTime => living(entity).into_iter().for_each(|fields| fields.m_deathTime.set(value as i32)),
        EntityField::BlockMeta =>
        {
            movingBlock(entity).into_iter().for_each(|fields| fields.m_metadata.set(value as i32))
        }
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
            ok(entity(dimension, id), |entity| HostReply::Bytes(crate::entities::synced(&entity, runtime.id())))
        }
        // only the dedicated server broadcasts, single player and a client keep the bytes local
        HostCall::SetSyncedData { world: dimension, id, data } =>
        {
            unit(entity(dimension, id).map(|entity| crate::entities::publishSynced(&entity, runtime.id(), data)))
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
        HostCall::SetItemInUse { player: id, ticks } => unit(player(id).and_then(|player| {
            let held = entity_player::inventory(&player).getCurrentItem().ok_or("the player holds nothing to use")?;
            entity_player::setItemInUse(&player, &held, ticks);
            Ok(())
        })),
        HostCall::StopUsingItem { player: id } => unit(player(id).map(|player| entity_player::stopUsingItem(&player))),
        HostCall::DigSpeed { player: id, block } => ok(
            player(id).and_then(|player| {
                game::block::Block::byId(block).map(|block| (player, block)).ok_or(format!("no block {block}"))
            }),
            |(player, block)| {
                HostReply::Float(game::entity::entity_player::getCurrentPlayerStrVsBlock(&player, &block))
            },
        ),
        HostCall::SetBiome { world: dimension, x, z, biome } => unit(
            world(dimension)
                .and_then(|world| authoritative(&world).map(|()| world))
                .and_then(|world| crate::biomes::setBiome(&world, x, z, biome)),
        ),
        HostCall::Weather { world: dimension, raining, thundering, ticks } =>
        {
            ok(world(dimension).and_then(|world| authoritative(&world).map(|()| world)), |world| {
                let info = world.worldInfo();
                info.setRaining(raining);
                info.setRainTime(ticks);
                info.setThundering(raining && thundering);
                info.setThunderTime(ticks);
                HostReply::Unit
            })
        }
        HostCall::SwapBlock { world: dimension, pos, block, meta } => ok(
            world(dimension)
                .and_then(|world| authoritative(&world).map(|()| world))
                .and_then(|world| tiles::swap(&world, pos, block, meta)),
            HostReply::Bool,
        ),
        HostCall::BlockState { world: dimension, pos } => ok(world(dimension), |world| {
            let block: i32 = world.getBlockId(pos.x, pos.y, pos.z);
            HostReply::State(crate::states::describe(block, world.getBlockMetadata(pos.x, pos.y, pos.z)))
        }),
        HostCall::SetBlockState { world: dimension, pos, state, notify } =>
        {
            let target: Result<(i32, i32), String> =
                crate::states::resolve(&state).ok_or_else(|| format!("{} is not a registered block", state.block));
            ok(
                world(dimension)
                    .and_then(|world| authoritative(&world).map(|()| world))
                    .and_then(|world| target.map(|target| (world, target))),
                |(world, (block, meta))| {
                    let changed: bool = if notify
                    {
                        world.setBlockAndMetadataWithNotify(pos.x, pos.y, pos.z, block, meta)
                    }
                    else
                    {
                        world.setBlockAndMetadata(pos.x, pos.y, pos.z, block, meta)
                    };
                    HostReply::Bool(changed)
                },
            )
        }
        HostCall::BlockEffect { world: dimension, pos, block, meta } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            world.doEvent(game::block::BREAK_EVENT, pos.x, pos.y, pos.z, game::block::packBreak(block, meta));
            Ok(())
        })),
        HostCall::DropBlockItems { world: dimension, pos, block, meta } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            let block = game::block::Block::byId(block).ok_or_else(|| format!("no block {block}"))?;
            block.dropBlockAsItem(&world, pos.x, pos.y, pos.z, meta);
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
        HostCall::TileAttachment { world: dimension, pos } =>
        {
            ok(anyTile(dimension, pos), |tile| attachment(&tile.m_attachments, runtime.id()))
        }
        HostCall::SetTileAttachment { world: dimension, pos, data } => unit(anyTile(dimension, pos).map(|tile| {
            setAttachment(&tile.m_attachments, runtime.id(), data);
            if let Some(world) = tile.m_worldObj.get()
            {
                world.getChunkFromBlockCoords(pos.x, pos.z).setChunkModified();
            }
        })),
        HostCall::ChunkAttachment { world: dimension, chunk_x, chunk_z } => ok(world(dimension), |world| {
            attachment(&world.getChunkFromChunkCoords(chunk_x, chunk_z).m_attachments, runtime.id())
        }),
        HostCall::SetChunkAttachment { world: dimension, chunk_x, chunk_z, data } =>
        {
            unit(world(dimension).map(|world| {
                let chunk = world.getChunkFromChunkCoords(chunk_x, chunk_z);
                setAttachment(&chunk.m_attachments, runtime.id(), data);
                chunk.setChunkModified();
            }))
        }
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
                    // a player moves itself, the tracker sends it new motion only flagged as a knockback
                    if entity.instanceOf("EntityPlayer")
                    {
                        entity.setBeenAttacked();
                    }
                }
            }))
        }
        HostCall::DamageEntity { world: dimension, id, source, amount } => ok(entity(dimension, id), |target| {
            let attacker: Option<Rc<Entity>> = source.and_then(|source| context::entityIn(dimension, source));
            HostReply::Bool(
                target.attackEntityFrom(&DamageSource::withAttacker(DamageKind::Generic, attacker.as_ref()), amount),
            )
        }),
        HostCall::DamageEntityAs { world: dimension, id, source, amount, kind } =>
        {
            let Some(kind) = crate::damage::kind(&kind)
            else
            {
                return HostReply::Error(format!("{kind} is not a damage type"));
            };
            ok(entity(dimension, id), |target| {
                let attacker: Option<Rc<Entity>> = source.and_then(|source| context::entityIn(dimension, source));
                HostReply::Bool(target.attackEntityFrom(&DamageSource::withAttacker(kind, attacker.as_ref()), amount))
            })
        }
        HostCall::DamageType => HostReply::Id(crate::damage::current()),
        HostCall::InsertItem { world: dimension, pos, face, stack, simulate } =>
        {
            ok(world(dimension), |world| match stackFrom(&stack)
            {
                Some(stack) =>
                {
                    HostReply::Int(game::transfer::insert(&world, pos.x, pos.y, pos.z, face, &stack, simulate))
                }
                None => HostReply::Error("cannot insert an empty or unknown stack".to_owned()),
            })
        }
        HostCall::ExtractItem { world: dimension, pos, face, max, simulate } => ok(world(dimension), |world| {
            let taken: Option<Rc<ItemStack>> =
                game::transfer::extract(&world, pos.x, pos.y, pos.z, face, max, simulate);
            HostReply::Stack(taken.map(|stack| stackData(&stack)))
        }),
        HostCall::UnlockAchievement { player: id, achievement } =>
        {
            ok(player(id), |target| HostReply::Bool(crate::content::progress::unlock(&target, &achievement)))
        }
        HostCall::HasAchievement { player: id, achievement } =>
        {
            ok(player(id), |target| HostReply::Bool(game::stats::named::unlocked(&target, &achievement.to_string())))
        }
        HostCall::AddStat { player: id, stat, amount } =>
        {
            unit(player(id).map(|target| crate::content::progress::addStat(&target, &stat, amount)))
        }
        HostCall::StatValue { player: id, stat } =>
        {
            ok(player(id), |target| HostReply::Int(game::stats::named::value(&target, &stat)))
        }
        HostCall::AddEffect { world: dimension, id, effect, level, ticks } =>
        {
            unit(entity(dimension, id).and_then(|target| crate::content::effects::add(&target, &effect, level, ticks)))
        }
        HostCall::RemoveEffect { world: dimension, id, effect } =>
        {
            unit(entity(dimension, id).map(|target| crate::content::effects::remove(&target, &effect)))
        }
        HostCall::EffectLevel { world: dimension, id, effect } =>
        {
            ok(entity(dimension, id), |target| HostReply::Int(crate::content::effects::level(&target, &effect)))
        }
        HostCall::SetModifier { world: dimension, id, attribute, modifier, amount, operation } => unit(
            entity(dimension, id)
                .and_then(|target| crate::attributes::set(&target, &attribute, &modifier, amount, operation)),
        ),
        HostCall::RemoveModifier { world: dimension, id, attribute, modifier } =>
        {
            unit(entity(dimension, id).and_then(|target| crate::attributes::remove(&target, &attribute, &modifier)))
        }
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
            entity_player::inventory(&player).addItemStackToInventory(&stack);
            // a partial fit leaves the rest in the stack
            if stack.m_stackSize.get() > 0
            {
                entity_player::dropPlayerItem(&player, Some(stack));
            }
            Ok(())
        })),
        HostCall::StackComponent { at, component } => ok(stackAt(at), |stack| {
            HostReply::Bytes(stack.m_components.borrow().get(&component.to_string()).cloned().unwrap_or_default())
        }),
        HostCall::SetStackComponent { at, component, data } =>
        {
            let owned: Result<(), String> = if component.namespace() == runtime.id()
            {
                Ok(())
            }
            else
            {
                Err(format!("{component} is outside the namespace {}", runtime.id()))
            };
            unit(
                owned
                    .and_then(|()| stackAt(at))
                    .map(|stack| setAttachment(&stack.m_components, &component.to_string(), data)),
            )
        }
        HostCall::ItemEntityStack { world: dimension, id } => ok(entity(dimension, id), |entity| {
            let fields: Option<&game::entity::entity_item::ItemFields> = entity.m_ext.asAny().downcast_ref();
            HostReply::Stack(fields.and_then(|fields| fields.m_item.get()).map(|stack| stackData(&stack)))
        }),
        HostCall::HeldStack { player: id } =>
        {
            ok(player(id), |player| HostReply::Stack(heldStack(&player).map(|stack| stackData(&stack))))
        }
        HostCall::SetHeldStack { player: id, stack } => unit(player(id).map(|player| {
            let inventory = entity_player::inventory(&player);
            inventory.setInventorySlotContents(inventory.m_currentItem.get(), stack.as_ref().and_then(stackFrom));
        })),
        HostCall::SetSpawner { world: dimension, pos, entity } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            world.setBlockWithNotify(pos.x, pos.y, pos.z, game::block::Block::MOB_SPAWNER);
            let tile: Rc<TileEntity> =
                world.getBlockTileEntity(pos.x, pos.y, pos.z).ok_or_else(|| format!("no spawner tile at {pos:?}"))?;
            game::tileentity::tile_entity_mob_spawner::setMobID(
                &tile,
                Some(jvm::string::JString::from(entity.as_str())),
            );
            Ok(())
        })),
        HostCall::RandomTick { world: dimension, pos } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            let id: i32 = world.getBlockId(pos.x, pos.y, pos.z);
            if id != 0
                && game::block::Block::tickOnLoad(id)
                && let Some(block) = game::block::Block::byId(id)
            {
                block.updateTick(&world, pos.x, pos.y, pos.z, &world.m_rand);
            }
            Ok(())
        })),
        HostCall::MaxDamage { item } =>
        {
            HostReply::Int(game::item::Item::byId(item).map_or(0, |item| item.getMaxDamage()))
        }
        HostCall::SetDynamicTexture { name, width, height, pixels } =>
        {
            let fits: bool = pixels.len() == (width as usize) * (height as usize);
            if !fits
            {
                return HostReply::Error(format!("texture {name} has {} pixels for {width} by {height}", pixels.len()));
            }
            crate::textures::set(name.to_string(), width, height, pixels);
            HostReply::Unit
        }
        HostCall::BlockMapColor { block } => HostReply::Int(
            game::block::Block::byId(block).map_or(0, |block| block.m_blockMaterial.m_materialMapColor.m_colorValue),
        ),
        HostCall::Fertilize { world: dimension, pos } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            // vanilla bone meal used on the block, with a throwaway item entity as the holder it never reads
            let stack: Rc<ItemStack> = ItemStack::withItemDamage(&game::item::Item::dyePowder(), 1, BONE_MEAL);
            let holder: Rc<Entity> = game::entity::entity_item::create(
                &world,
                f64::from(pos.x),
                f64::from(pos.y),
                f64::from(pos.z),
                Rc::clone(&stack),
            );
            stack.getItem().onItemUse(&stack, &holder, &world, pos.x, pos.y, pos.z, 1);
            Ok(())
        })),
        HostCall::DamageHeld { player: id, amount } => unit(player(id).map(|player| {
            let Some(stack) = heldStack(&player)
            else
            {
                return;
            };
            stack.damageItem(amount, &player);
            if stack.m_stackSize.get() <= 0
            {
                entity_player::destroyCurrentEquippedItem(&player);
            }
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
        HostCall::SetWorldTime { time } =>
        {
            context::worlds().iter().for_each(|world| world.setWorldTime(time));
            HostReply::Unit
        }
        HostCall::SetSkyTime { world: dimension, time } =>
        {
            crate::worlds::setSkyTime(dimension, time);
            HostReply::Unit
        }
        HostCall::IsOp { player: id } => ok(player(id), |player| HostReply::Bool(crate::netplay::isOp(&player))),
        HostCall::OpenSlot { player: id, slot } => ok(player(id), |player| {
            let open = entity_player::fields(&player).m_craftingInventory.get();
            let stack: Option<Rc<ItemStack>> = open
                .filter(|container| slot >= 0 && slot < container.m_slots.size())
                .and_then(|container| container.m_slots.get(slot).getStack());
            HostReply::Stack(stack.map(|stack| stackData(&stack)))
        }),
        HostCall::DisplayName { stack } =>
        {
            HostReply::Text(stackFrom(&stack).map(|stack| crate::viewer::displayName(&stack)).unwrap_or_default())
        }
        HostCall::Explode { world: dimension, pos, power } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            world.createExplosion(None, pos[0], pos[1], pos[2], power);
            Ok(())
        })),
        other => crate::extensions::handle(runtime, other),
    }
}
