use crate::host::context::{blockCtx, withWorld};
use crate::state;
use crate::state::ModRuntime;
use common::nbt::{NBTBase, NBTTagCompound};
use game::block::{Block, BlockClass};
use game::entity::Entity;
use game::item::ItemStack;
use game::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use game::world::World;
use jvm::string::JString;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, GuestCall, TileCall};
use mod_protocol::spec::{TileEntitySpec, TileMethod};
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

pub const MOD_DATA: &str = "ModData";
const ITEMS: &str = "Items";
const SLOT: &str = "Slot";
const STACK_LIMIT: i32 = 64;
const USE_DISTANCE_SQUARED: f64 = 64.0;

pub struct TileType
{
    pub m_id: Identifier,
    pub m_owner: usize,
    pub m_spec: TileEntitySpec,
    m_class: OnceCell<&'static TileEntityClass>,
}

pub struct ModTileExt
{
    pub m_type: Rc<TileType>,
    pub m_data: RefCell<Vec<u8>>,
    pub m_slots: RefCell<Vec<Option<Rc<ItemStack>>>>,
}

thread_local!
{
    static TYPES: RefCell<HashMap<Identifier, Rc<TileType>>> = RefCell::new(HashMap::new());
    // block id to the tile type its spec names
    static BLOCK_TYPES: RefCell<HashMap<i32, Rc<TileType>>> = RefCell::new(HashMap::new());
    // set while a swap replaces the block so the tile entity and its slots stay, as beta's keepFurnaceInventory
    static SWAPPING: Cell<bool> = const { Cell::new(false) };
    // tile entity type to the model its tiles draw with
    static MODELS: RefCell<HashMap<Identifier, Rc<mod_protocol::spec::EntityModelSpec>>> = RefCell::new(HashMap::new());
}

pub fn define(id: Identifier, owner: usize, spec: TileEntitySpec)
{
    let tileType: Rc<TileType> =
        Rc::new(TileType { m_id: id.clone(), m_owner: owner, m_spec: spec, m_class: OnceCell::new() });
    TYPES.with(|types| types.borrow_mut().insert(id, tileType));
}

pub fn bindBlock(block: i32, id: &Identifier) -> Result<(), String>
{
    let tileType: Rc<TileType> = TYPES
        .with(|types| types.borrow().get(id).cloned())
        .ok_or_else(|| format!("tile entity {id} is not registered"))?;
    BLOCK_TYPES.with(|types| types.borrow_mut().insert(block, tileType));
    Ok(())
}

pub fn ext(tile: &TileEntity) -> &ModTileExt { tile.ext::<ModTileExt>() }

fn callTile(tile: &Rc<TileEntity>, call: TileCall) -> Option<mod_protocol::abi::GuestReply>
{
    let tileType: Rc<TileType> = Rc::clone(&ext(tile).m_type);
    let behavior = tileType.m_spec.behavior.as_ref()?;
    let runtime: Rc<ModRuntime> = state::modAt(tileType.m_owner)?;
    let request: GuestCall = GuestCall::Tile { handle: behavior.handle, call };

    match tile.m_worldObj.get()
    {
        Some(world) => withWorld(&world, || runtime.call(&request)),
        None => runtime.call(&request),
    }
}

// only the mod that registered a tile entity type gives it a model, before any world
pub fn registerModel(modIndex: usize, id: Identifier, model: mod_protocol::spec::EntityModelSpec)
-> Result<(), String>
{
    let tileType: Rc<TileType> = TYPES
        .with(|types| types.borrow().get(&id).cloned())
        .ok_or_else(|| format!("tile entity {id} is not registered"))?;

    if tileType.m_owner != modIndex
    {
        return Err(format!("tile entity {id} belongs to another mod"));
    }

    MODELS.with(|models| models.borrow_mut().insert(id, Rc::new(model)));
    Ok(())
}

// the model a tile entity's type draws with, keyed by the type's class name as the renderer map looks it up
pub fn modelFor(className: &str) -> Option<Rc<mod_protocol::spec::EntityModelSpec>>
{
    let id: Identifier = Identifier::parse(className).ok()?;
    MODELS.with(|models| models.borrow().get(&id).cloned())
}

// one frame of a tile entity's model from its behavior, ticks run with the world clock for animations
pub fn pose(tile: &Rc<TileEntity>, partial: f32) -> Option<mod_protocol::abi::Pose>
{
    if tile.tryExt::<ModTileExt>().is_none() || !wants(tile, TileMethod::Pose)
    {
        return None;
    }

    let world: Rc<World> = tile.m_worldObj.get()?;
    let (x, y, z): (i32, i32, i32) = (tile.m_xCoord.get(), tile.m_yCoord.get(), tile.m_zCoord.get());
    let block: i32 = world.getBlockId(x, y, z);
    let ticks: i32 = world.getWorldTime() as i32;
    match callTile(tile, TileCall::Pose { ctx: blockCtx(&world, block, x, y, z), partial, ticks })
    {
        Some(mod_protocol::abi::GuestReply::Pose(pose)) => Some(pose),
        _ => None,
    }
}

fn wants(tile: &TileEntity, method: TileMethod) -> bool
{
    ext(tile).m_type.m_spec.behavior.as_ref().is_some_and(|behavior| behavior.methods.contains(&method))
}

fn tick(this: &Rc<TileEntity>)
{
    if !wants(this, TileMethod::Tick)
    {
        return;
    }

    let world: Rc<World> = this.world();
    let (x, y, z): (i32, i32, i32) = (this.m_xCoord.get(), this.m_yCoord.get(), this.m_zCoord.get());
    let block: i32 = world.getBlockId(x, y, z);
    callTile(this, TileCall::Tick { ctx: blockCtx(&world, block, x, y, z) });
}

fn writeNbt(this: &Rc<TileEntity>, nbt: &NBTTagCompound)
{
    nbt.setString("id", Some(JString::from(ext(this).m_type.m_id.to_string().as_str())));
    nbt.setInteger("x", this.m_xCoord.get());
    nbt.setInteger("y", this.m_yCoord.get());
    nbt.setInteger("z", this.m_zCoord.get());
    nbt.setByteArray(MOD_DATA, Rc::new(ext(this).m_data.borrow().iter().map(|byte| *byte as i8).collect()));

    let items: Rc<NBTBase> = NBTTagCompound::create();

    for (slot, stack) in ext(this).m_slots.borrow().iter().enumerate()
    {
        if let Some(stack) = stack
        {
            let entry: Rc<NBTBase> = NBTTagCompound::create();
            stack.writeToNBT(entry.compound());
            items.compound().setCompoundTag(&format!("{SLOT}{slot}"), &entry);
        }
    }

    nbt.setCompoundTag(ITEMS, &items);
    common::nbt::attachments::write(&this.m_attachments.borrow(), nbt);
}

fn readNbt(this: &Rc<TileEntity>, nbt: &NBTTagCompound)
{
    (TILE_ENTITY.readFromNBT)(this, nbt);
    *ext(this).m_data.borrow_mut() =
        nbt.getByteArray(MOD_DATA).map(|bytes| bytes.iter().map(|byte| *byte as u8).collect()).unwrap_or_default();
    let items: Rc<NBTBase> = nbt.getCompoundTag(ITEMS);

    for (slot, stack) in ext(this).m_slots.borrow_mut().iter_mut().enumerate()
    {
        let key: String = format!("{SLOT}{slot}");
        *stack =
            items.compound().hasKey(&key).then(|| ItemStack::withNbt(items.compound().getCompoundTag(&key).compound()));
    }
}

fn slotAt(this: &Rc<TileEntity>, slot: i32) -> Option<Rc<ItemStack>>
{
    ext(this).m_slots.borrow().get(slot as usize).cloned().flatten()
}

fn setSlot(this: &Rc<TileEntity>, slot: i32, stack: Option<Rc<ItemStack>>)
{
    if let Some(entry) = ext(this).m_slots.borrow_mut().get_mut(slot as usize)
    {
        *entry = stack;
    }

    this.onInventoryChanged();
}

fn decrSlot(this: &Rc<TileEntity>, slot: i32, amount: i32) -> Option<Rc<ItemStack>>
{
    let stack: Rc<ItemStack> = slotAt(this, slot)?;
    if stack.m_stackSize.get() <= amount
    {
        setSlot(this, slot, None);
        return Some(stack);
    }

    let split: Rc<ItemStack> = stack.splitStack(amount);

    this.onInventoryChanged();
    Some(split)
}

fn canInteract(this: &Rc<TileEntity>, player: &Rc<Entity>) -> bool
{
    let world: Rc<World> = this.world();
    let (x, y, z): (i32, i32, i32) = (this.m_xCoord.get(), this.m_yCoord.get(), this.m_zCoord.get());
    let present: bool = world.getBlockTileEntity(x, y, z).is_some_and(|tile| Rc::ptr_eq(&tile, this));
    present && player.getDistanceSq(x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5) <= USE_DISTANCE_SQUARED
}

fn tileClass(tileType: &Rc<TileType>) -> &'static TileEntityClass
{
    tileType.m_class.get_or_init(||
    {
        crate::classes::deriveTileEntityClass(&TILE_ENTITY, &tileType.m_id.to_string(), |class|
        {
            class.updateEntity = tick;
            class.writeToNBT = writeNbt;
            class.readFromNBT = readNbt;
            class.getSizeInventory = |this| ext(this).m_slots.borrow().len() as i32;
            class.getStackInSlot = slotAt;
            class.decrStackSize = decrSlot;
            class.setInventorySlotContents = setSlot;
            class.getInvName = |_| "container.mod";
            class.getInventoryStackLimit = |_| STACK_LIMIT;
            class.canInteractWith = canInteract;

            if tileType.m_spec.inventory > 0
            {
                class.m_interfaces = &["IInventory"];
            }
        })
    })
}

pub fn create(tileType: &Rc<TileType>) -> Rc<TileEntity>
{
    let slots: usize = tileType.m_spec.inventory.max(0) as usize;

    let ext: ModTileExt = ModTileExt
    {
        m_type: Rc::clone(tileType),
        m_data: RefCell::new(Vec::new()),
        m_slots: RefCell::new(vec![None; slots]),
    };
    TileEntity::new(tileClass(tileType), Box::new(ext))
}

pub fn createById(id: &str) -> Option<Rc<TileEntity>>
{
    let id: Identifier = crate::remap::rename(&Identifier::parse(id).ok()?);
    let tileType: Rc<TileType> = TYPES.with(|types| types.borrow().get(&id).cloned())?;
    Some(create(&tileType))
}

pub fn clearBindings() { BLOCK_TYPES.with(|types| types.borrow_mut().clear()); }

pub fn forBlock(block: i32) -> Option<Rc<TileType>> { BLOCK_TYPES.with(|types| types.borrow().get(&block).cloned()) }

fn blockParent(this: &Block) -> &'static BlockClass
{
    this.m_class.m_parent.expect("a derived block class keeps its template")
}

fn blockEntity(this: &Rc<Block>) -> Rc<TileEntity>
{
    match forBlock(this.m_blockID)
    {
        Some(tileType) => create(&tileType),

        None =>
        {
            jvm::throwable::throw(&jvm::throwable::ERROR, &format!("block {} has no tile entity type", this.m_blockID))
        }
    }
}

fn added(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    (blockParent(this).onBlockAdded)(this, world, x, y, z);

    if !SWAPPING.with(Cell::get)
    {
        world.setBlockTileEntity(x, y, z, &this.getBlockEntity());
    }

    crate::behavior::notifyAdded(this, world, x, y, z);
}

fn removed(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    crate::behavior::notifyRemoved(this, world, x, y, z);

    let swapping: bool = SWAPPING.with(Cell::get);
    if !swapping
        && let Some(tile) = world.getBlockTileEntity(x, y, z)
        && tile.tryExt::<ModTileExt>().is_some()
    {
        if wants(&tile, TileMethod::Removed)
        {
            callTile(&tile, TileCall::Removed { ctx: blockCtx(world, this.m_blockID, x, y, z) });
        }

        for stack in ext(&tile).m_slots.borrow().iter().flatten()
        {
            this.dropBlockAsItem_do(world, x, y, z, stack.copy());
        }
    }

    (blockParent(this).onBlockRemoval)(this, world, x, y, z);
    world.removeBlockTileEntity(x, y, z);
}

// BlockFurnace.updateFurnaceBlockState for mod tiles, the new block must hold the same tile type
pub fn swap(world: &Rc<World>, pos: BlockPos, block: i32, meta: i32) -> Result<bool, String>
{
    let BlockPos { x, y, z } = pos;
    let tile: Rc<TileEntity> = world
        .getBlockTileEntity(x, y, z)
        .filter(|tile| tile.tryExt::<ModTileExt>().is_some())
        .ok_or_else(|| format!("no mod tile entity at {x} {y} {z}"))?;
    let tileType: &Rc<TileType> = &ext(&tile).m_type;
    if !forBlock(block).is_some_and(|target| Rc::ptr_eq(&target, tileType))
    {
        return Err(format!("block {block} does not hold tile entity {}", tileType.m_id));
    }

    SWAPPING.with(|swapping| swapping.set(true));

    let changed: bool = world.setBlockAndMetadataWithNotify(x, y, z, block, meta);

    SWAPPING.with(|swapping| swapping.set(false));

    if !changed
    {
        return Ok(false);
    }

    tile.cancelRemoval();
    world.setBlockTileEntity(x, y, z, &tile);
    Ok(true)
}

// a block that names a tile entity behaves like BlockContainer on top of its template, after any wasm patch
pub fn patchBlock(class: &mut BlockClass)
{
    class.getBlockEntity = blockEntity;
    class.onBlockAdded = added;
    class.onBlockRemoval = removed;
}
