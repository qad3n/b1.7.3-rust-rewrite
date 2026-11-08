use crate::host::context::{blockCtx, withWorld};
use crate::state;
use common::nbt::{NBTBase, NBTTagCompound};
use game::block::{Block, BlockClass};
use game::entity::Entity;
use game::item::ItemStack;
use game::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use game::world::World;
use jvm::string::JString;
use mod_protocol::Identifier;
use mod_protocol::abi::{GuestCall, TileCall};
use mod_protocol::spec::{TileEntitySpec, TileMethod};
use std::cell::{OnceCell, RefCell};
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

thread_local! {
    static TYPES: RefCell<HashMap<Identifier, Rc<TileType>>> = RefCell::new(HashMap::new());
    // block id to the tile type its spec names
    static BLOCK_TYPES: RefCell<HashMap<i32, Rc<TileType>>> = RefCell::new(HashMap::new());
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

fn callTile(tile: &Rc<TileEntity>, call: TileCall)
{
    let tileType: Rc<TileType> = Rc::clone(&ext(tile).m_type);
    let Some(behavior) = tileType.m_spec.behavior.as_ref()
    else
    {
        return;
    };
    let Some(runtime) = state::modAt(tileType.m_owner)
    else
    {
        return;
    };
    let request: GuestCall = GuestCall::Tile { handle: behavior.handle, call };
    match tile.m_worldObj.get()
    {
        Some(world) => withWorld(&world, || runtime.call(&request)),
        None => runtime.call(&request),
    };
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
    tileType.m_class.get_or_init(|| {
        crate::classes::deriveTileEntityClass(&TILE_ENTITY, &tileType.m_id.to_string(), |class| {
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
    let ext: ModTileExt = ModTileExt {
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
    world.setBlockTileEntity(x, y, z, &this.getBlockEntity());
    crate::behavior::notifyAdded(this, world, x, y, z);
}

fn removed(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    crate::behavior::notifyRemoved(this, world, x, y, z);
    if let Some(tile) = world.getBlockTileEntity(x, y, z)
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

// a block that names a tile entity behaves like BlockContainer on top of its template, after any wasm patch
pub fn patchBlock(class: &mut BlockClass)
{
    class.getBlockEntity = blockEntity;
    class.onBlockAdded = added;
    class.onBlockRemoval = removed;
}
