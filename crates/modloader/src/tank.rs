use crate::tiles::ModTileExt;
use game::world::World;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, FluidStack, Holder, TankLevel};
use mod_protocol::spec::{AttachmentSpec, TankSpec};

// what a tile's loader-kept tank holds, an attachment so it is saved with the tile and reaches clients
const STORED: &str = "tank";
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

struct Tank
{
    m_spec: TankSpec,
    m_owner: usize,
}

fn tank(world: &World, pos: BlockPos) -> Option<Tank>
{
    let tile: std::rc::Rc<game::tileentity::TileEntity> = world.getBlockTileEntity(pos.x, pos.y, pos.z)?;
    let ext: &ModTileExt = tile.tryExt::<ModTileExt>()?;
    Some(Tank { m_spec: ext.m_type.m_spec.tank.clone()?, m_owner: ext.m_type.m_owner })
}

fn holder(world: &World, pos: BlockPos) -> Holder
{
    Holder::Tile { world: crate::host::context::dimension(world), pos }
}

fn held(world: &World, pos: BlockPos) -> Option<FluidStack>
{
    let bytes: Vec<u8> = crate::attach::fetch(holder(world, pos), &storedName()).ok().flatten()?;
    mod_protocol::wire::decodeNamed::<FluidStack>(&bytes).ok().filter(|stack| stack.amount > 0)
}

fn setHeld(world: &World, pos: BlockPos, stack: Option<FluidStack>) -> Result<(), String>
{
    let bytes: Option<Vec<u8>> =
        stack.filter(|stack| stack.amount > 0).map(|stack| mod_protocol::wire::encodeNamed(&stack));
    crate::attach::store(crate::log::LOADER, holder(world, pos), &storedName(), bytes)
}

// a face outside the six block faces is the tile's own code reaching in, which no mask shuts out
fn open(mask: u8, face: i32) -> bool { !(0..FACES).contains(&face) || mask & (1 << face) != 0 }

fn takes(spec: &TankSpec, fluid: &Identifier) -> bool
{
    spec.accepts.is_empty() || spec.accepts.iter().any(|name| *name == fluid.to_string())
}

pub fn kept(world: &World, pos: BlockPos) -> bool { tank(world, pos).is_some() }

// pours into the tank through a face, the amount it took: only a fluid it accepts, and only its own once it holds one
pub fn fill(world: &World, pos: BlockPos, face: i32, stack: &FluidStack, simulate: bool) -> i32
{
    let Some(tank) =
        tank(world, pos).filter(|tank| open(tank.m_spec.input_faces, face) && takes(&tank.m_spec, &stack.fluid))
    else
    {
        return 0;
    };

    let now: Option<FluidStack> = held(world, pos);
    if now.as_ref().is_some_and(|now| now.fluid != stack.fluid)
    {
        return 0;
    }

    let amount: i32 = now.map_or(0, |now| now.amount);
    let taken: i32 = stack.amount.clamp(0, tank.m_spec.capacity - amount);
    let filled: FluidStack = FluidStack { fluid: stack.fluid.clone(), amount: amount + taken };
    if taken > 0
        && !simulate
        && let Err(reason) = setHeld(world, pos, Some(filled))
    {
        crate::log::warn(crate::log::LOADER, &reason);
        return 0;
    }

    taken
}

// takes up to max out of the tank through a face
pub fn drain(world: &World, pos: BlockPos, face: i32, max: i32, simulate: bool) -> Option<FluidStack>
{
    tank(world, pos).filter(|tank| open(tank.m_spec.output_faces, face))?;

    let now: FluidStack = held(world, pos)?;
    let given: i32 = max.clamp(0, now.amount);
    if given == 0
    {
        return None;
    }

    let left: FluidStack = FluidStack { fluid: now.fluid.clone(), amount: now.amount - given };
    if !simulate && let Err(reason) = setHeld(world, pos, Some(left))
    {
        crate::log::warn(crate::log::LOADER, &reason);
        return None;
    }

    Some(FluidStack { fluid: now.fluid, amount: given })
}

pub fn level(world: &World, pos: BlockPos) -> Option<TankLevel>
{
    let tank: Tank = tank(world, pos)?;
    let now: Option<FluidStack> = held(world, pos);
    Some(TankLevel
    {
        amount: now.as_ref().map_or(0, |now| now.amount),
        fluid: now.map(|now| now.fluid),
        capacity: tank.m_spec.capacity,
    })
}

// the owning mod sets its tank directly
pub fn set(owner: usize, world: &World, pos: BlockPos, stack: Option<FluidStack>) -> Result<(), String>
{
    let tank: Tank = tank(world, pos).ok_or_else(|| format!("no loader-kept tank at {pos:?}"))?;
    if tank.m_owner != owner
    {
        return Err(format!("the tank at {pos:?} belongs to another mod"));
    }

    let capped: Option<FluidStack> =
        stack.map(|stack| FluidStack { amount: stack.amount.clamp(0, tank.m_spec.capacity), fluid: stack.fluid });
    setHeld(world, pos, capped)
}
