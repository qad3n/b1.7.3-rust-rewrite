use crate::content::{self, BlockData, ItemData};
use crate::host::context::{blockCtx, stackData, stackFrom, withWorld};
use crate::state;
use game::block::{Block, BlockClass};
use game::entity::Entity;
use game::inventory::InventoryPlayer;
use game::item::{Item, ItemClass, ItemStack};
use game::world::{IBlockAccess, World};
use jvm::random::Random;
use mod_protocol::abi::{BlockCall, BlockCtx, BlockPos, FaceOverlay, GuestCall, GuestReply, ItemCall};
use mod_protocol::spec::{Behavior, BlockMethod, ItemMethod};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn parent(this: &Block) -> &'static BlockClass
{
    this.m_class.m_parent.expect("a derived block class keeps its template")
}

fn callBlock(this: &Block, world: Option<&Rc<World>>, call: BlockCall) -> Option<GuestReply>
{
    let data: Rc<BlockData> = content::blockData(this.m_blockID)?;
    let handle: u32 = data.m_spec.behavior.as_ref()?.handle;
    let runtime: Rc<state::ModRuntime> = state::modAt(data.m_owner)?;
    let request: GuestCall = GuestCall::Block { handle, call };

    match world
    {
        Some(world) => withWorld(world, || runtime.call(&request)),
        None => runtime.call(&request),
    }
}

fn ctx(this: &Block, world: &Rc<World>, x: i32, y: i32, z: i32) -> BlockCtx { blockCtx(world, this.m_blockID, x, y, z) }

fn asBool(reply: Option<GuestReply>) -> Option<bool>
{
    match reply
    {
        Some(GuestReply::Bool(value)) => Some(value),
        _ => None,
    }
}

fn asInt(reply: Option<GuestReply>) -> Option<i32>
{
    match reply
    {
        Some(GuestReply::Int(value)) => Some(value),
        _ => None,
    }
}

// void callbacks fall back to the template only when the mod failed to answer
fn handled(reply: Option<GuestReply>) -> bool { reply.is_some() }

fn activated(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>) -> bool
{
    let call: BlockCall = BlockCall::Activated { ctx: ctx(this, world, x, y, z), player: player.m_entityId.get() };
    asBool(callBlock(this, Some(world), call))
        .unwrap_or_else(|| (parent(this).blockActivated)(this, world, x, y, z, player))
}

fn clicked(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>)
{
    let call: BlockCall = BlockCall::Clicked { ctx: ctx(this, world, x, y, z), player: player.m_entityId.get() };
    if !handled(callBlock(this, Some(world), call))
    {
        (parent(this).onBlockClicked)(this, world, x, y, z, player);
    }
}

fn wantsMethod(this: &Block, method: BlockMethod) -> bool
{
    content::blockData(this.m_blockID)
        .is_some_and(|data| data.m_spec.behavior.as_ref().is_some_and(|behavior| behavior.methods.contains(&method)))
}

pub fn notifyAdded(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if wantsMethod(this, BlockMethod::Added)
    {
        callBlock(this, Some(world), BlockCall::Added { ctx: ctx(this, world, x, y, z) });
    }
}

pub fn notifyRemoved(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if wantsMethod(this, BlockMethod::Removed)
    {
        callBlock(this, Some(world), BlockCall::Removed { ctx: ctx(this, world, x, y, z) });
    }
}

fn added(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    (parent(this).onBlockAdded)(this, world, x, y, z);
    notifyAdded(this, world, x, y, z);
}

fn removed(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    notifyRemoved(this, world, x, y, z);
    (parent(this).onBlockRemoval)(this, world, x, y, z);
}

fn placed(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, face: i32)
{
    (parent(this).onBlockPlaced)(this, world, x, y, z, face);
    callBlock(this, Some(world), BlockCall::Placed { ctx: ctx(this, world, x, y, z), face });
}

fn placedBy(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>)
{
    (parent(this).onBlockPlacedBy)(this, world, x, y, z, entity);
    callBlock(
        this,
        Some(world),
        BlockCall::PlacedBy { ctx: ctx(this, world, x, y, z), entity: entity.m_entityId.get() },
    );
}

fn neighborChanged(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, neighbor: i32)
{
    let call: BlockCall = BlockCall::NeighborChanged { ctx: ctx(this, world, x, y, z), neighbor };
    if !handled(callBlock(this, Some(world), call))
    {
        (parent(this).onNeighborBlockChange)(this, world, x, y, z, neighbor);
    }
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if !handled(callBlock(this, Some(world), BlockCall::UpdateTick { ctx: ctx(this, world, x, y, z) }))
    {
        (parent(this).updateTick)(this, world, x, y, z, random);
    }
}

fn randomDisplayTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if !handled(callBlock(this, Some(world), BlockCall::RandomDisplayTick { ctx: ctx(this, world, x, y, z) }))
    {
        (parent(this).randomDisplayTick)(this, world, x, y, z, random);
    }
}

fn blockEvent(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, kind: i32, data: i32)
{
    let call: BlockCall = BlockCall::BlockEvent { ctx: ctx(this, world, x, y, z), kind, data };
    if !handled(callBlock(this, Some(world), call))
    {
        (parent(this).playBlock)(this, world, x, y, z, kind, data);
    }
}

fn entityCollided(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>)
{
    let call: BlockCall = BlockCall::EntityCollided { ctx: ctx(this, world, x, y, z), entity: entity.m_entityId.get() };
    if !handled(callBlock(this, Some(world), call))
    {
        (parent(this).onEntityCollidedWithBlock)(this, world, x, y, z, entity);
    }
}

fn entityWalking(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>)
{
    let call: BlockCall = BlockCall::EntityWalking { ctx: ctx(this, world, x, y, z), entity: entity.m_entityId.get() };
    if !handled(callBlock(this, Some(world), call))
    {
        (parent(this).onEntityWalking)(this, world, x, y, z, entity);
    }
}

fn destroyedByPlayer(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32)
{
    let mut context: BlockCtx = ctx(this, world, x, y, z);
    context.meta = metadata;
    callBlock(this, Some(world), BlockCall::DestroyedByPlayer { ctx: context });
    (parent(this).onBlockDestroyedByPlayer)(this, world, x, y, z, metadata);
}

fn destroyedByExplosion(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    callBlock(this, Some(world), BlockCall::DestroyedByExplosion { ctx: ctx(this, world, x, y, z) });
    (parent(this).onBlockDestroyedByExplosion)(this, world, x, y, z);
}

fn harvest(this: &Rc<Block>, world: &Rc<World>, player: &Rc<Entity>, x: i32, y: i32, z: i32, metadata: i32)
{
    let mut context: BlockCtx = ctx(this, world, x, y, z);
    context.meta = metadata;

    if !handled(callBlock(this, Some(world), BlockCall::Harvest { ctx: context, player: player.m_entityId.get() }))
    {
        (parent(this).harvestBlock)(this, world, player, x, y, z, metadata);
    }
}

fn canPlaceAt(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    asBool(callBlock(this, Some(world), BlockCall::CanPlaceAt { ctx: ctx(this, world, x, y, z) }))
        .unwrap_or_else(|| (parent(this).canPlaceBlockAt)(this, world, x, y, z))
}

fn canStay(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    asBool(callBlock(this, Some(world), BlockCall::CanStay { ctx: ctx(this, world, x, y, z) }))
        .unwrap_or_else(|| (parent(this).canBlockStay)(this, world, x, y, z))
}

fn accessCtx(this: &Block, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> BlockCtx
{
    let dimension: i32 = crate::host::context::currentDimension();

    BlockCtx
    {
        world: dimension,
        pos: BlockPos::new(x, y, z),
        block: this.m_blockID,
        meta: world.getBlockMetadata(x, y, z),
        authority: true,
    }
}

// a place by dimension, side and block coordinates
type Place = (i32, bool, i32, i32, i32);

thread_local!
{
    // connection masks answered this tick by place, models and collision ask many times a tick
    static CONNECTIONS: RefCell<HashMap<Place, u8>> = RefCell::new(HashMap::new());
}

// every tick starts with fresh answers, so neighbours that changed show next tick at the latest
pub fn clearConnections() { CONNECTIONS.with(|cache| cache.borrow_mut().clear()); }

// the faces a connecting block joins, None for a block whose behavior does not answer
pub fn connections(this: &Block, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> Option<u8>
{
    if !wantsMethod(this, BlockMethod::Connections)
    {
        return None;
    }

    let ctx: BlockCtx = accessCtx(this, world, x, y, z);
    let key: Place = (ctx.world, crate::side::isServer(), x, y, z);
    if let Some(mask) = CONNECTIONS.with(|cache| cache.borrow().get(&key).copied())
    {
        return Some(mask);
    }

    let mask: u8 = asInt(callBlock(this, None, BlockCall::Connections { ctx })).unwrap_or(0) as u8;
    CONNECTIONS.with(|cache| cache.borrow_mut().insert(key, mask));
    Some(mask)
}

fn brightness(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> f32
{
    let base: f32 = (parent(this).getBlockBrightness)(this, world, x, y, z);

    match callBlock(this, None, BlockCall::Brightness { ctx: accessCtx(this, world, x, y, z), base })
    {
        Some(GuestReply::Float(shown)) if shown >= 0.0 => shown,
        _ => base,
    }
}

fn poweringTo(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, direction: i32) -> bool
{
    asBool(callBlock(this, None, BlockCall::PoweringTo { ctx: accessCtx(this, world, x, y, z), direction }))
        .unwrap_or_else(|| (parent(this).isPoweringTo)(this, world, x, y, z, direction))
}

fn indirectlyPoweringTo(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, direction: i32) -> bool
{
    let call: BlockCall = BlockCall::IndirectlyPoweringTo { ctx: ctx(this, world, x, y, z), direction };
    asBool(callBlock(this, Some(world), call))
        .unwrap_or_else(|| (parent(this).isIndirectlyPoweringTo)(this, world, x, y, z, direction))
}

fn canProvidePower(this: &Rc<Block>) -> bool
{
    asBool(callBlock(this, None, BlockCall::CanProvidePower { block: this.m_blockID }))
        .unwrap_or_else(|| (parent(this).canProvidePower)(this))
}

fn texture(this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    asInt(callBlock(this, None, BlockCall::Texture { face, meta: metadata }))
        .unwrap_or_else(|| (parent(this).getBlockTextureFromSideAndMetadata)(this, face, metadata))
}

// a named state's sprite, else the metadata callback when the behavior has one, else the spec's textures
fn stateTexture(this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    let sprite: Option<i32> = crate::states::describe(this.m_blockID, metadata)
        .and_then(|state| asInt(callBlock(this, None, BlockCall::StateTexture { face, state })))
        .filter(|slot| *slot >= 0);
    if let Some(sprite) = sprite
    {
        return sprite;
    }

    let metaCallback: bool = content::blockData(this.m_blockID).is_some_and(|data|
    {
        data.m_spec.behavior.as_ref().is_some_and(|behavior| behavior.methods.contains(&BlockMethod::Texture))
    });
    if metaCallback
    {
        return texture(this, face, metadata);
    }

    (parent(this).getBlockTextureFromSideAndMetadata)(this, face, metadata)
}

fn faceTexture(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> i32
{
    asInt(callBlock(this, None, BlockCall::FaceTexture { ctx: accessCtx(this, world, x, y, z), face }))
        .filter(|slot| *slot >= 0)
        .unwrap_or_else(|| (parent(this).getBlockTexture)(this, world, x, y, z, face))
}

fn strength(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>) -> f32
{
    let call: BlockCall = BlockCall::Strength { ctx: ctx(this, world, x, y, z), player: player.m_entityId.get() };
    match callBlock(this, Some(world), call)
    {
        Some(GuestReply::Float(share)) if share >= 0.0 => share,
        _ => (parent(this).blockStrengthAt)(this, world, x, y, z, player),
    }
}

// whether a block's behavior draws sprites over its faces, asked before every face so plain blocks cost one lookup
pub fn overlays(blockId: i32) -> bool
{
    content::blockData(blockId)
        .and_then(|data| data.m_spec.behavior.as_ref().map(|behavior| behavior.methods.contains(&BlockMethod::Overlay)))
        .unwrap_or(false)
}

// quarter turns of each face's sprite by face index, asked only of blocks whose behavior turns faces
pub fn faceRotations(this: &Block, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> Option<[i32; 6]>
{
    let turns: bool = content::blockData(this.m_blockID)
        .and_then(|data|
        {
            data.m_spec.behavior.as_ref().map(|behavior| behavior.methods.contains(&BlockMethod::FaceRotation))
        })
        .unwrap_or(false);

    if !turns
    {
        return None;
    }

    match callBlock(this, None, BlockCall::FaceRotation { ctx: accessCtx(this, world, x, y, z) })
    {
        Some(GuestReply::Bytes(bytes)) if bytes.len() == 6 =>
        {
            Some(std::array::from_fn(|face| i32::from(bytes[face] & 3)))
        }

        _ => None,
    }
}

pub fn overlay(this: &Block, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> Option<FaceOverlay>
{
    match callBlock(this, None, BlockCall::Overlay { ctx: accessCtx(this, world, x, y, z), face })
    {
        Some(GuestReply::Overlay(overlay)) => overlay,
        _ => None,
    }
}

fn colorMultiplier(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> i32
{
    asInt(callBlock(this, None, BlockCall::ColorMultiplier { ctx: accessCtx(this, world, x, y, z) }))
        .unwrap_or_else(|| (parent(this).colorMultiplier)(this, world, x, y, z))
}

// only the methods a mod overrides cross into wasm, every other entry keeps the template fn pointer
pub fn patchBlock(class: &mut BlockClass, behavior: &Behavior<BlockMethod>)
{
    for method in &behavior.methods
    {
        match method
        {
            BlockMethod::Activated => class.blockActivated = activated,
            BlockMethod::Clicked => class.onBlockClicked = clicked,
            BlockMethod::Added => class.onBlockAdded = added,
            BlockMethod::Removed => class.onBlockRemoval = removed,
            BlockMethod::Placed => class.onBlockPlaced = placed,
            BlockMethod::PlacedBy => class.onBlockPlacedBy = placedBy,
            BlockMethod::NeighborChanged => class.onNeighborBlockChange = neighborChanged,
            BlockMethod::UpdateTick => class.updateTick = updateTick,
            BlockMethod::RandomDisplayTick => class.randomDisplayTick = randomDisplayTick,
            BlockMethod::BlockEvent => class.playBlock = blockEvent,
            BlockMethod::EntityCollided => class.onEntityCollidedWithBlock = entityCollided,
            BlockMethod::EntityWalking => class.onEntityWalking = entityWalking,
            BlockMethod::DestroyedByPlayer => class.onBlockDestroyedByPlayer = destroyedByPlayer,
            BlockMethod::DestroyedByExplosion => class.onBlockDestroyedByExplosion = destroyedByExplosion,
            BlockMethod::Harvest => class.harvestBlock = harvest,
            BlockMethod::CanPlaceAt => class.canPlaceBlockAt = canPlaceAt,
            BlockMethod::CanStay => class.canBlockStay = canStay,
            BlockMethod::PoweringTo => class.isPoweringTo = poweringTo,
            BlockMethod::IndirectlyPoweringTo => class.isIndirectlyPoweringTo = indirectlyPoweringTo,
            BlockMethod::CanProvidePower => class.canProvidePower = canProvidePower,
            // a state texture stands over the metadata one whichever the behavior lists first
            BlockMethod::Texture if !behavior.methods.contains(&BlockMethod::StateTexture) =>
            {
                class.getBlockTextureFromSideAndMetadata = texture;
            }
            BlockMethod::Texture =>
            {}
            BlockMethod::StateTexture => class.getBlockTextureFromSideAndMetadata = stateTexture,
            BlockMethod::ColorMultiplier => class.colorMultiplier = colorMultiplier,
            BlockMethod::FaceTexture => class.getBlockTexture = faceTexture,
            BlockMethod::Strength => class.blockStrengthAt = strength,
            // the renderer asks for overlays and face turns itself, the class keeps its draw
            BlockMethod::Brightness => class.getBlockBrightness = brightness,
            BlockMethod::Overlay | BlockMethod::FaceRotation | BlockMethod::Connections =>
            {}
        }
    }
}

fn itemParent(this: &Item) -> &'static ItemClass
{
    this.m_class.m_parent.expect("a derived item class keeps its template")
}

fn callItem(this: &Item, world: Option<&Rc<World>>, call: ItemCall) -> Option<GuestReply>
{
    let data: Rc<ItemData> = content::itemData(this.m_shiftedIndex)?;
    let handle: u32 = data.m_spec.behavior.as_ref()?.handle;
    let runtime: Rc<state::ModRuntime> = state::modAt(data.m_owner)?;
    let request: GuestCall = GuestCall::Item { handle, call };

    match world
    {
        Some(world) => withWorld(world, || runtime.call(&request)),
        None => runtime.call(&request),
    }
}

fn dimensionOf(world: &World) -> i32 { crate::host::context::dimension(world) }

// a stack the guest returns replaces the held one, its size and damage are written back in place
fn applyStack(stack: &Rc<ItemStack>, reply: &Option<mod_protocol::abi::StackData>) -> Option<Rc<ItemStack>>
{
    match reply
    {
        Some(data) if data.id == stack.m_itemID.get() =>
        {
            stack.m_stackSize.set(data.count);
            stack.setItemDamage(data.damage);
            Some(Rc::clone(stack))
        }

        Some(data) => stackFrom(data),
        None => None,
    }
}

fn rightClick(this: &Rc<Item>, stack: &Rc<ItemStack>, world: &Rc<World>, player: &Rc<Entity>) -> Option<Rc<ItemStack>>
{
    let call: ItemCall =
        ItemCall::RightClick { world: dimensionOf(world), player: player.m_entityId.get(), stack: stackData(stack) };

    match callItem(this, Some(world), call)
    {
        Some(GuestReply::Use { handled: true, stack: reply }) => applyStack(stack, &reply),
        Some(GuestReply::Stack(reply)) => applyStack(stack, &reply),
        _ => (itemParent(this).onItemRightClick)(this, stack, world, player),
    }
}

fn useOnBlock(
    this: &Rc<Item>,
    stack: &Rc<ItemStack>,
    player: &Rc<Entity>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    face: i32,
) -> bool
{
    let call: ItemCall = ItemCall::UseOnBlock
    {
        world: dimensionOf(world),
        player: player.m_entityId.get(),
        stack: stackData(stack),
        pos: BlockPos::new(x, y, z),
        face,
    };
    match callItem(this, Some(world), call)
    {
        Some(GuestReply::Use { handled, stack: reply }) =>
        {
            applyStack(stack, &reply);
            handled
        }

        Some(GuestReply::Bool(handled)) => handled,
        _ => (itemParent(this).onItemUse)(this, stack, player, world, x, y, z, face),
    }
}

fn hitEntity(this: &Rc<Item>, stack: &Rc<ItemStack>, target: &Rc<Entity>, attacker: &Rc<Entity>) -> bool
{
    let call: ItemCall = ItemCall::HitEntity
    {
        stack: stackData(stack),
        target: target.m_entityId.get(),
        attacker: attacker.m_entityId.get(),
    };

    let world: Option<Rc<World>> = target.m_worldObj.get();
    asBool(callItem(this, world.as_ref(), call))
        .unwrap_or_else(|| (itemParent(this).hitEntity)(this, stack, target, attacker))
}

fn interactEntity(this: &Rc<Item>, stack: &Rc<ItemStack>, entity: &Rc<Entity>)
{
    let call: ItemCall =
        ItemCall::InteractEntity { stack: stackData(stack), target: entity.m_entityId.get(), player: 0 };
    let world: Option<Rc<World>> = entity.m_worldObj.get();
    if !handled(callItem(this, world.as_ref(), call))
    {
        (itemParent(this).saddleEntity)(this, stack, entity);
    }
}

fn blockDestroyed(
    this: &Rc<Item>,
    stack: &Rc<ItemStack>,
    block: i32,
    x: i32,
    y: i32,
    z: i32,
    entity: &Rc<Entity>,
) -> bool
{
    let call: ItemCall = ItemCall::BlockDestroyed
    {
        stack: stackData(stack),
        block,
        pos: BlockPos::new(x, y, z),
        player: entity.m_entityId.get(),
    };

    let world: Option<Rc<World>> = entity.m_worldObj.get();
    asBool(callItem(this, world.as_ref(), call))
        .unwrap_or_else(|| (itemParent(this).onBlockDestroyed)(this, stack, block, x, y, z, entity))
}

// Item.getItemNameIS hands back a key, so a stack's own name goes into the lang table under a key of its own
fn stackName(this: &Rc<Item>, stack: &Rc<ItemStack>) -> Option<String>
{
    const PREFIX: &str = "loader.stack_name.";

    match callItem(this, None, ItemCall::Name { stack: stackData(stack) })
    {
        Some(GuestReply::Lines(lines)) if !lines.is_empty() =>
        {
            let name: &str = &lines[0];
            let key: String = format!("{PREFIX}{}", content::lang::keyFor(name));
            content::lang::addDefault(&format!("{key}.name"), name);
            Some(key)
        }

        _ => (itemParent(this).getItemNameIS)(this, stack),
    }
}

fn icon(this: &Rc<Item>, damage: i32) -> i32
{
    asInt(callItem(this, None, ItemCall::Icon { damage }))
        .unwrap_or_else(|| (itemParent(this).getIconFromDamage)(this, damage))
}

fn strVsBlock(this: &Rc<Item>, stack: &Rc<ItemStack>, block: &Rc<Block>) -> f32
{
    match callItem(this, None, ItemCall::StrVsBlock { stack: stackData(stack), block: block.m_blockID })
    {
        Some(GuestReply::Float(speed)) => speed,
        _ => (itemParent(this).getStrVsBlock)(this, stack, block),
    }
}

fn canHarvest(this: &Rc<Item>, block: &Rc<Block>) -> bool
{
    asBool(callItem(this, None, ItemCall::CanHarvest { block: block.m_blockID }))
        .unwrap_or_else(|| (itemParent(this).canHarvestBlock)(this, block))
}

fn useDuration(this: &Rc<Item>, stack: &Rc<ItemStack>) -> i32
{
    asInt(callItem(this, None, ItemCall::UseDuration { stack: stackData(stack) }))
        .unwrap_or_else(|| (itemParent(this).getMaxItemUseDuration)(this, stack))
}

fn useAction(this: &Rc<Item>, stack: &Rc<ItemStack>) -> i32
{
    asInt(callItem(this, None, ItemCall::UseAction { stack: stackData(stack) }))
        .unwrap_or_else(|| (itemParent(this).getItemUseAction)(this, stack))
}

fn usingTick(this: &Rc<Item>, stack: &Rc<ItemStack>, player: &Rc<Entity>, left: i32)
{
    let world: Rc<World> = player.world();

    let call: ItemCall = ItemCall::UsingTick
    {
        world: dimensionOf(&world),
        player: player.m_entityId.get(),
        stack: stackData(stack),
        left,
    };
    callItem(this, Some(&world), call);
}

// a stack the guest gives back replaces the held one, none empties the slot it was in
fn heldAfter(stack: &Rc<ItemStack>, player: &Rc<Entity>, reply: &Option<mod_protocol::abi::StackData>)
{
    let result: Option<Rc<ItemStack>> = applyStack(stack, reply);
    if result.as_ref().is_some_and(|result| Rc::ptr_eq(result, stack))
    {
        return;
    }

    let inventory: Rc<InventoryPlayer> = game::entity::entity_player::inventory(player);
    let slot: usize = inventory.m_currentItem.get() as usize;
    let held: bool = inventory.m_mainInventory.borrow()[slot].as_ref().is_some_and(|held| Rc::ptr_eq(held, stack));
    if held
    {
        inventory.m_mainInventory.borrow_mut()[slot] = result;
    }
}

fn stoppedUsing(this: &Rc<Item>, stack: &Rc<ItemStack>, world: &Rc<World>, player: &Rc<Entity>, left: i32)
{
    let call: ItemCall = ItemCall::StoppedUsing
    {
        world: dimensionOf(world),
        player: player.m_entityId.get(),
        stack: stackData(stack),
        left,
    };
    match callItem(this, Some(world), call)
    {
        Some(GuestReply::Use { handled: true, stack: reply }) => heldAfter(stack, player, &reply),
        _ => (itemParent(this).onPlayerStoppedUsing)(this, stack, world, player, left),
    }
}

fn useFinish(this: &Rc<Item>, stack: &Rc<ItemStack>, world: &Rc<World>, player: &Rc<Entity>) -> Option<Rc<ItemStack>>
{
    let call: ItemCall =
        ItemCall::UseFinish { world: dimensionOf(world), player: player.m_entityId.get(), stack: stackData(stack) };

    match callItem(this, Some(world), call)
    {
        Some(GuestReply::Use { handled: true, stack: reply }) => applyStack(stack, &reply),
        _ => (itemParent(this).onItemUseFinish)(this, stack, world, player),
    }
}

fn useIcon(this: &Rc<Item>, stack: &Rc<ItemStack>, used: i32) -> i32
{
    asInt(callItem(this, None, ItemCall::UseIcon { stack: stackData(stack), used }))
        .unwrap_or_else(|| (itemParent(this).getUseIcon)(this, stack, used))
}

fn crafted(this: &Rc<Item>, stack: &Rc<ItemStack>, world: &Rc<World>, player: &Rc<Entity>)
{
    (itemParent(this).onCreated)(this, stack, world, player);
    callItem(this, Some(world), ItemCall::Crafted { stack: stackData(stack), player: player.m_entityId.get() });
}

pub fn patchItem(class: &mut ItemClass, behavior: &Behavior<ItemMethod>)
{
    for method in &behavior.methods
    {
        match method
        {
            ItemMethod::RightClick => class.onItemRightClick = rightClick,
            ItemMethod::UseOnBlock => class.onItemUse = useOnBlock,
            ItemMethod::HitEntity => class.hitEntity = hitEntity,
            ItemMethod::InteractEntity => class.saddleEntity = interactEntity,
            ItemMethod::BlockDestroyed => class.onBlockDestroyed = blockDestroyed,
            ItemMethod::Icon => class.getIconFromDamage = icon,
            ItemMethod::StrVsBlock => class.getStrVsBlock = strVsBlock,
            ItemMethod::CanHarvest => class.canHarvestBlock = canHarvest,
            ItemMethod::Crafted => class.onCreated = crafted,
            ItemMethod::UseDuration => class.getMaxItemUseDuration = useDuration,
            ItemMethod::UseAction => class.getItemUseAction = useAction,
            ItemMethod::UsingTick => class.onUsingItemTick = usingTick,
            ItemMethod::StoppedUsing => class.onPlayerStoppedUsing = stoppedUsing,
            ItemMethod::UseFinish => class.onItemUseFinish = useFinish,
            ItemMethod::UseIcon => class.getUseIcon = useIcon,
            ItemMethod::Name => class.getItemNameIS = stackName,
        }
    }
}

pub fn patchTileBlock(class: &mut BlockClass) { crate::tiles::patchBlock(class); }
