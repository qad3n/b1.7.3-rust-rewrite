use crate::tiles::ModTileExt;
use game::world::World;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, EnergyDemand, Holder};
use mod_protocol::spec::{AttachmentSpec, EnergyBufferSpec, EnergySpec};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

thread_local!
{
    // every energy kind and its worth in one shared base unit, so a generator of one mod powers a machine of another
    static KINDS: RefCell<BTreeMap<Identifier, i64>> = const { RefCell::new(BTreeMap::new()) };
    // kind pairs already reported as unable to convert, each logged once
    static REPORTED: RefCell<BTreeSet<(Identifier, Identifier)>> = const { RefCell::new(BTreeSet::new()) };
}

pub fn register(owner: &str, id: Identifier, spec: EnergySpec) -> Result<(), String>
{
    if spec.worth < 0
    {
        return Err(format!("energy kind {id} has a negative worth {}", spec.worth));
    }

    if spec.worth == 0
    {
        crate::log::warn(
            owner,
            &format!("energy kind {id} has no worth, it only moves between blocks of its own kind"),
        );
    }

    crate::registry::claim(mod_protocol::abi::RegistryKind::Energy, &id)?;
    KINDS.with(|kinds| kinds.borrow_mut().insert(id, spec.worth));
    Ok(())
}

fn worth(id: &Identifier) -> Option<i64>
{
    KINDS.with(|kinds| kinds.borrow().get(id).copied().filter(|worth| *worth > 0))
}

// both worths in base units, None when the kinds differ and either has none
fn rates(from: &Identifier, to: &Identifier) -> Option<(i64, i64)>
{
    if from == to
    {
        return Some((1, 1));
    }

    let rates: Option<(i64, i64)> = worth(from).zip(worth(to));
    if rates.is_none()
    {
        report(from, to);
    }

    rates
}

fn report(from: &Identifier, to: &Identifier)
{
    let fresh: bool = REPORTED.with(|reported| reported.borrow_mut().insert((from.clone(), to.clone())));
    if !fresh
    {
        return;
    }

    let known: &str = match (worth(from).is_some(), worth(to).is_some())
    {
        (false, false) => "neither kind is registered with a worth",
        (false, true) => "the sending kind is not registered with a worth",
        _ => "the receiving kind is not registered with a worth",
    };
    crate::log::warn("loader", &format!("energy {from} cannot power a block taking {to}: {known}"));
}

// an amount of one kind in another, rounded down so conversion never makes energy
pub fn convert(amount: i64, from: &Identifier, to: &Identifier) -> Option<i64>
{
    let (fromWorth, toWorth): (i64, i64) = rates(from, to)?;
    Some(amount.saturating_mul(fromWorth) / toWorth)
}

// what an amount taken in the receiver's kind cost the sender, rounded up for the same reason
pub fn cost(taken: i64, from: &Identifier, to: &Identifier) -> Option<i64>
{
    let (fromWorth, toWorth): (i64, i64) = rates(from, to)?;
    let base: i64 = taken.saturating_mul(toWorth);
    Some((base + fromWorth - 1) / fromWorth)
}

// what a tile's loader-kept buffer holds, an attachment so it is saved with the tile and reaches clients like any other
const STORED: &str = "energy";
const FACES: i32 = 6;

fn storedName() -> Identifier { crate::network::loader::id(&format!("{}:{STORED}", crate::log::LOADER)) }

// once before mods load, on both sides, so a client knows the value the server sends
pub fn declareStore()
{
    let spec: AttachmentSpec = AttachmentSpec { saved: true, synced: true, kept: false };
    if let Err(reason) = crate::attach::declare(crate::log::LOADER, storedName(), spec)
    {
        crate::log::debug(crate::log::LOADER, &reason);
    }
}

struct Buffer
{
    m_kind: Identifier,
    m_spec: EnergyBufferSpec,
    m_owner: usize,
}

fn buffer(world: &World, pos: BlockPos) -> Option<Buffer>
{
    let tile: std::rc::Rc<game::tileentity::TileEntity> = world.getBlockTileEntity(pos.x, pos.y, pos.z)?;
    let ext: &ModTileExt = tile.tryExt::<ModTileExt>()?;
    let spec: EnergyBufferSpec = ext.m_type.m_spec.energy.clone()?;
    let kind: Identifier = Identifier::parseIn(&spec.kind, &crate::state::modId(ext.m_type.m_owner)).ok()?;
    Some(Buffer { m_kind: kind, m_spec: spec, m_owner: ext.m_type.m_owner })
}

fn holder(world: &World, pos: BlockPos) -> Holder
{
    Holder::Tile { world: crate::host::context::dimension(world), pos }
}

fn stored(world: &World, pos: BlockPos) -> i64
{
    crate::attach::fetch(holder(world, pos), &storedName())
        .ok()
        .flatten()
        .and_then(|bytes| bytes.try_into().ok())
        .map_or(0, i64::from_le_bytes)
}

fn setStored(world: &World, pos: BlockPos, amount: i64) -> Result<(), String>
{
    crate::attach::store(crate::log::LOADER, holder(world, pos), &storedName(), Some(amount.to_le_bytes().to_vec()))
}

// a face outside the six block faces is the tile's own code reaching in, which no mask shuts out
fn open(mask: u8, face: i32) -> bool { !(0..FACES).contains(&face) || mask & (1 << face) != 0 }

pub fn kept(world: &World, pos: BlockPos) -> bool { buffer(world, pos).is_some() }

// what the buffer would take through a face right now, in its own kind
pub fn demand(world: &World, pos: BlockPos, face: i32) -> Option<EnergyDemand>
{
    let buffer: Buffer = buffer(world, pos)?;
    let room: i64 = buffer.m_spec.capacity - stored(world, pos);
    let max: i64 = if open(buffer.m_spec.input_faces, face)
    {
        room.min(buffer.m_spec.input).max(0)
    }
    else
    {
        0
    };
    Some(EnergyDemand { kind: buffer.m_kind, min: 0, max })
}

// puts an amount of the buffer's own kind in through a face, the part it took
pub fn receive(world: &World, pos: BlockPos, face: i32, amount: i64, simulate: bool) -> i64
{
    let Some(max) = demand(world, pos, face).map(|demand| demand.max)
    else
    {
        return 0;
    };

    let taken: i64 = amount.clamp(0, max);
    if taken > 0
        && !simulate
        && let Err(reason) = setStored(world, pos, stored(world, pos) + taken)
    {
        crate::log::warn(crate::log::LOADER, &reason);
        return 0;
    }

    taken
}

// kind, stored and capacity of a loader-kept buffer
pub fn level(world: &World, pos: BlockPos) -> Option<(Identifier, i64, i64)>
{
    let buffer: Buffer = buffer(world, pos)?;
    Some((buffer.m_kind, stored(world, pos), buffer.m_spec.capacity))
}

// draws an amount of the buffer's own kind out through a face, the part it gave
pub fn take(world: &World, pos: BlockPos, face: i32, amount: i64, simulate: bool) -> i64
{
    let Some(buffer) = buffer(world, pos).filter(|buffer| open(buffer.m_spec.output_faces, face))
    else
    {
        return 0;
    };

    let held: i64 = stored(world, pos);
    let given: i64 = amount.clamp(0, held.min(buffer.m_spec.output));
    if given > 0
        && !simulate
        && let Err(reason) = setStored(world, pos, held - given)
    {
        crate::log::warn(crate::log::LOADER, &reason);
        return 0;
    }

    given
}

// the owning mod sets its buffer directly, as a generator adding what it made
pub fn set(owner: usize, world: &World, pos: BlockPos, amount: i64) -> Result<(), String>
{
    let buffer: Buffer = buffer(world, pos).ok_or_else(|| format!("no loader-kept energy buffer at {pos:?}"))?;
    if buffer.m_owner != owner
    {
        return Err(format!("the energy buffer at {pos:?} belongs to another mod"));
    }

    setStored(world, pos, amount.clamp(0, buffer.m_spec.capacity))
}
