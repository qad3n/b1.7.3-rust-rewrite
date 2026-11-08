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
use mod_protocol::abi::{BlockPos, HostCall, HostReply};
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
        HostCall::TileSlot { world: dimension, pos, slot } => ok(modTile(dimension, pos), |tile| {
            HostReply::Stack(tile.getStackInSlot(slot).map(|stack| stackData(&stack)))
        }),
        HostCall::SetTileSlot { world: dimension, pos, slot, stack } => unit(modTile(dimension, pos).map(|tile| {
            tile.setInventorySlotContents(slot, stack.as_ref().and_then(stackFrom));
        })),
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
        HostCall::Explode { world: dimension, pos, power } => unit(world(dimension).and_then(|world| {
            authoritative(&world)?;
            world.createExplosion(None, pos[0], pos[1], pos[2], power);
            Ok(())
        })),
        other => crate::extensions::handle(runtime, other),
    }
}
