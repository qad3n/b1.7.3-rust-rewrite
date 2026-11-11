use crate::host::context;
use crate::host::{ok, unit};
use game::block::Block;
use game::entity::Entity;
use game::math::Vec3D;
use game::world::World;
use mod_protocol::abi::{BlockPos, HostCall, HostReply};
use std::rc::Rc;

// what mods need to run their own explosions the way Explosion does: ray strength per block, taking a block out
// with its explosion callback, and how much of an entity a blast point sees

fn world(dimension: i32) -> Result<Rc<World>, String>
{
    context::world(dimension).ok_or_else(|| format!("dimension {dimension} is not loaded"))
}

fn resistance(block: i32) -> f32 { Block::byId(block).map_or(0.0, |block| block.getExplosionResistance(None)) }

// Explosion.doExplosionB for one block, with the caller's drop chance instead of vanilla's 0.3
fn blast(world: &Rc<World>, pos: BlockPos, chance: f32) -> Result<(), String>
{
    if !crate::side::isAuthority(world)
    {
        return Err("only the side that owns the world may change it".to_owned());
    }

    let Some(block) = Block::byId(world.getBlockId(pos.x, pos.y, pos.z))
    else
    {
        return Ok(());
    };

    let meta: i32 = world.getBlockMetadata(pos.x, pos.y, pos.z);
    block.dropBlockAsItemWithChance(world, pos.x, pos.y, pos.z, meta, chance);
    world.setBlockWithNotify(pos.x, pos.y, pos.z, 0);
    block.onBlockDestroyedByExplosion(world, pos.x, pos.y, pos.z);
    Ok(())
}

fn density(world: &Rc<World>, from: [f64; 3], id: i32) -> Result<f32, String>
{
    let entity: Rc<Entity> = context::entityById(world, id).ok_or_else(|| format!("no entity {id}"))?;
    let point: Rc<Vec3D> = Vec3D::createVector(from[0], from[1], from[2]);
    Ok(world.getBlockDensity(&point, &entity.m_boundingBox))
}

pub fn handle(call: HostCall) -> HostReply
{
    match call
    {
        HostCall::ExplosionResistance { block } => HostReply::Float(resistance(block)),
        HostCall::BlastBlock { world: dimension, pos, chance } => unit(world(dimension).and_then(|world| blast(&world, pos, chance))),
        HostCall::BlockDensity { world: dimension, from, id } => ok(world(dimension).and_then(|world| density(&world, from, id)), HostReply::Float),
        other => HostReply::Error(format!("not an explosion call: {other:?}")),
    }
}
