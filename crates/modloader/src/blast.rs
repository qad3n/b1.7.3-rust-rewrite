use crate::host::{context, ok, unit};
use common::world::ChunkPosition;
use game::block::Block;
use game::entity::Entity;
use game::math::Vec3D;
use game::world::{Explosion, World};
use mod_protocol::abi::{BlockPos, HostCall, HostReply, RegistryRequest, WorldRequest};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

pub const BLAST: &str = "loader:blast";

// what Packet60Explosion carries, for an explosion the loader ran in place of World.newExplosion
#[derive(Serialize, Deserialize)]
pub struct Blast
{
    pub world: i32,
    pub at: [f64; 3],
    pub power: f32,
    pub blocks: Vec<BlockPos>,
}

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

// a whole explosion run here, a ray pass over thousands of blocks would overrun a guest call's time budget
fn explode(world: &Rc<World>, at: [f64; 3], power: f32, drop: f32, damage: f32, fire: bool) -> Result<(), String>
{
    let explosion: Rc<Explosion> = Explosion::new(world, None, at[0], at[1], at[2], power);
    explosion.m_isFlaming.set(fire);
    explosion.m_dropChance.set(drop);
    explosion.m_damageScale.set(damage);
    explosion.doExplosionA();
    explosion.doExplosionB(true);

    if crate::side::isServer()
    {
        announce(world, &explosion, at, power);
    }

    Ok(())
}

// WorldServer.newExplosion sends the blast to players within 64 blocks, this path has to do the same
fn announce(world: &Rc<World>, explosion: &Explosion, at: [f64; 3], power: f32)
{
    let blocks: Vec<BlockPos> = explosion
        .m_destroyedBlockPositions
        .with(|positions| positions.items())
        .iter()
        .map(|position| BlockPos::new(position.m_x, position.m_y, position.m_z))
        .collect();
    let message: Blast = Blast { world: context::dimension(world), at, power, blocks };
    let bytes: Vec<u8> = mod_protocol::wire::encode(&message);
    crate::network::sendNear(&crate::network::loader::id(BLAST), world, at, crate::network::Reach::Sphere, &bytes);
}

// NetClientHandler.handleExplosion: the client clears the blocks it was told and plays the sound and particles
pub fn received(message: Blast)
{
    let Some(world) = context::world(message.world)
    else
    {
        return;
    };

    crate::log::debug(crate::log::LOADER, &format!("blast received with {} blocks", message.blocks.len()));

    let [x, y, z]: [f64; 3] = message.at;
    let explosion: Rc<Explosion> = Explosion::new(&world, None, x, y, z, message.power);
    explosion.m_destroyedBlockPositions.update(|positions|
    {
        for pos in &message.blocks
        {
            positions.add(ChunkPosition::new(pos.x, pos.y, pos.z));
        }
    });
    explosion.doExplosionB(true);
}

pub fn handle(call: HostCall) -> HostReply
{
    match call
    {
        HostCall::Registry(RegistryRequest::ExplosionResistance { block }) => HostReply::Float(resistance(block)),

        HostCall::World(WorldRequest::BlastBlock { world: dimension, pos, chance }) =>
        {
            unit(world(dimension).and_then(|world| blast(&world, pos, chance)))
        }

        HostCall::World(WorldRequest::ExplodeWith { world: dimension, at, power, drop, damage, fire }) =>
        {
            unit(world(dimension).and_then(|world| explode(&world, at, power, drop, damage, fire)))
        }

        HostCall::World(WorldRequest::BlockDensity { world: dimension, from, id }) =>
        {
            ok(world(dimension).and_then(|world| density(&world, from, id)), HostReply::Float)
        }

        other => HostReply::Error(format!("not an explosion call: {other:?}")),
    }
}
