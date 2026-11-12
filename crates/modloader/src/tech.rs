use crate::host::context::{self, blockCtx, entityInfo, stackData, stackFrom};
use crate::host::{ok, unit};
use crate::state::ModRuntime;
use crate::tiles::{self, ModTileExt};
use game::block::Block;
use game::crafting::{CraftingGrid, CraftingManager};
use game::entity::Entity;
use game::item::{Item, ItemStack};
use game::math::AxisAlignedBB;
use game::tileentity::TileEntity;
use game::world::World;
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    BlockCtx, BlockPos, EnergyDemand, FluidBlock, FluidStack, GuestReply, HostCall, HostReply, ItemInfo, ItemModelSpec,
    RegistryKind, StackData, TileCall, capability,
};
use mod_protocol::spec::{ItemFilter, StateModelSpec, TileMethod};
use std::rc::Rc;

// the crafting grid a 3 by 3 recipe reads, in row order
const GRID_SIDE: i32 = 3;
const GRID_SLOTS: usize = 9;

// dropBlockAsItemWithChance's luck when every roll counts
const NO_LUCK: f32 = 1.0;
const PLACE_PITCH: f32 = 0.8;
const HALF: f64 = 0.5;

struct Grid
{
    m_slots: Vec<Option<Rc<ItemStack>>>,
}

impl CraftingGrid for Grid
{
    fn getItem(&self, column: i32, row: i32) -> Option<Rc<ItemStack>>
    {
        let inside: bool = (0..GRID_SIDE).contains(&column) && (0..GRID_SIDE).contains(&row);
        if !inside
        {
            return None;
        }

        self.m_slots.get((row * GRID_SIDE + column) as usize).cloned().flatten()
    }
}

fn world(dimension: i32) -> Result<Rc<World>, String>
{
    context::world(dimension).ok_or_else(|| format!("dimension {dimension} is not loaded"))
}

fn authoritative(world: &World) -> Result<(), String>
{
    if crate::side::isAuthority(world)
    {
        return Ok(());
    }

    Err("only the side that owns the world may change it".to_owned())
}

fn modTile(world: &World, pos: BlockPos) -> Option<Rc<TileEntity>>
{
    world.getBlockTileEntity(pos.x, pos.y, pos.z).filter(|tile| tile.tryExt::<ModTileExt>().is_some())
}

// a mod tile's answer to a transfer its behavior handles, None when the block has no such tile
fn ask(
    world: &Rc<World>,
    pos: BlockPos,
    method: TileMethod,
    call: impl FnOnce(BlockCtx) -> TileCall,
) -> Option<GuestReply>
{
    let tile: Rc<TileEntity> = modTile(world, pos).filter(|tile| tiles::wants(tile, method))?;
    let block: i32 = world.getBlockId(pos.x, pos.y, pos.z);
    tiles::callTile(&tile, call(blockCtx(world, block, pos.x, pos.y, pos.z)))
}

fn handles(world: &World, pos: BlockPos, method: TileMethod) -> bool
{
    modTile(world, pos).is_some_and(|tile| tiles::wants(&tile, method))
}

// World::insertItem, a mod tile's own hook first, its slots or a vanilla inventory otherwise
pub fn insert(world: &Rc<World>, pos: BlockPos, face: i32, stack: &Rc<ItemStack>, simulate: bool) -> i32
{
    let offered: StackData = stackData(stack);

    match ask(world, pos, TileMethod::InsertItem, |ctx| TileCall::InsertItem { ctx, face, stack: offered, simulate })
    {
        Some(GuestReply::Int(left)) => left.clamp(0, stack.m_stackSize.get()),
        Some(_) => stack.m_stackSize.get(),
        None => game::transfer::insert(world, pos.x, pos.y, pos.z, face, stack, simulate),
    }
}

pub fn extract(world: &Rc<World>, pos: BlockPos, face: i32, max: i32, simulate: bool) -> Option<Rc<ItemStack>>
{
    match ask(world, pos, TileMethod::ExtractItem, |ctx| TileCall::ExtractItem { ctx, face, max, simulate })
    {
        Some(GuestReply::Stack(taken)) => taken.as_ref().and_then(stackFrom),
        Some(_) => None,
        None => game::transfer::extract(world, pos.x, pos.y, pos.z, face, max, simulate),
    }
}

// whether a filter takes a stack, by name, tag or the fluid it holds
pub fn accepts(filter: &ItemFilter, stack: &ItemStack) -> bool
{
    let data: StackData = stackData(stack);
    let name: Option<Identifier> =
        crate::registry::with(|registries| registries.m_items.byNumeric(data.id).map(|row| row.m_id.clone()));
    let tagged = |tag: &Identifier|
    {
        name.as_ref().is_some_and(|name| crate::content::tags::contains(RegistryKind::Item, tag, name))
    };

    match filter
    {
        ItemFilter::Any => true,
        ItemFilter::Item { item, meta } => name.as_ref() == Some(item) && (*meta < 0 || *meta == data.damage),
        ItemFilter::Tag(tag) => tagged(tag),
        ItemFilter::Except(tag) => !tagged(tag),
        ItemFilter::Holding(fluid) => crate::content::fluids::contained(&data)
            .is_some_and(|held| fluid.as_ref().is_none_or(|fluid| held.fluid == *fluid)),
        ItemFilter::Empty => crate::content::fluids::empty(&data),
    }
}

// World::extractMatching, a mod tile is asked what it would give before anything is taken
pub fn extractMatching(
    world: &Rc<World>,
    pos: BlockPos,
    face: i32,
    filter: &ItemFilter,
    max: i32,
    simulate: bool,
) -> Option<Rc<ItemStack>>
{
    if handles(world, pos, TileMethod::ExtractItem)
    {
        let offered: Rc<ItemStack> = extract(world, pos, face, max, true)?;
        if !accepts(filter, &offered)
        {
            return None;
        }

        return if simulate
        {
            Some(offered)
        }
        else
        {
            extract(world, pos, face, max, false)
        };
    }

    game::transfer::extractWhere(world, [pos.x, pos.y, pos.z], face, max, simulate, |stack| accepts(filter, stack))
}

fn capabilities(world: &Rc<World>, pos: BlockPos) -> i32
{
    let items: bool = game::transfer::inventoryAt(world, pos.x, pos.y, pos.z).is_some()
        || handles(world, pos, TileMethod::InsertItem)
        || handles(world, pos, TileMethod::ExtractItem);
    let mut bits: i32 = 0;

    if items
    {
        bits |= capability::ITEMS;
    }

    if handles(world, pos, TileMethod::InsertItem) || handles(world, pos, TileMethod::ExtractItem)
    {
        bits |= capability::HANDLER;
    }

    if handles(world, pos, TileMethod::Fluid)
    {
        bits |= capability::FLUIDS;
    }

    if handles(world, pos, TileMethod::Energy)
    {
        bits |= capability::ENERGY;
    }

    bits
}

// a block's demand in its own kind, converted into the asker's kind
fn energyDemand(world: &Rc<World>, pos: BlockPos, face: i32, kind: Identifier) -> Option<EnergyDemand>
{
    let native: EnergyDemand = match ask(world, pos, TileMethod::Energy, |ctx| TileCall::EnergyDemand { ctx, face })
    {
        Some(GuestReply::Demand(demand)) => demand?,
        _ => return None,
    };

    let min: i64 = crate::energy::cost(native.min, &kind, &native.kind)?;
    let max: i64 = crate::energy::cost(native.max, &kind, &native.kind)?;
    Some(EnergyDemand { kind, min, max })
}

// the amount converts into the kind the block takes, and what it took converts back into the sender's cost
fn sendEnergy(world: &Rc<World>, pos: BlockPos, face: i32, kind: Identifier, amount: i64, simulate: bool) -> i64
{
    let native: Identifier = match ask(world, pos, TileMethod::Energy, |ctx| TileCall::EnergyDemand { ctx, face })
    {
        Some(GuestReply::Demand(Some(demand))) => demand.kind,
        _ => return 0,
    };
    let Some(offered) = crate::energy::convert(amount.max(0), &kind, &native)
    else
    {
        return 0;
    };

    let call = |ctx: BlockCtx| TileCall::ReceiveEnergy { ctx, face, kind: native.clone(), amount: offered, simulate };
    let taken: i64 = match ask(world, pos, TileMethod::Energy, call)
    {
        Some(GuestReply::Long(taken)) => taken.clamp(0, offered),
        _ => 0,
    };
    crate::energy::cost(taken, &kind, &native).unwrap_or(0).min(amount)
}

fn fill(world: &Rc<World>, pos: BlockPos, face: i32, stack: FluidStack, simulate: bool) -> i32
{
    let amount: i32 = stack.amount;

    match ask(world, pos, TileMethod::Fluid, |ctx| TileCall::FillFluid { ctx, face, stack, simulate })
    {
        Some(GuestReply::Int(taken)) => taken.clamp(0, amount.max(0)),
        _ => 0,
    }
}

fn drain(world: &Rc<World>, pos: BlockPos, face: i32, max: i32, simulate: bool) -> Option<FluidStack>
{
    match ask(world, pos, TileMethod::Fluid, |ctx| TileCall::DrainFluid { ctx, face, max, simulate })
    {
        Some(GuestReply::Fluid(drained)) => drained.filter(|stack| stack.amount > 0),
        _ => None,
    }
}

// the fluid registry knows every liquid by its material, vanilla water and lava among them
fn fluidAt(world: &World, pos: BlockPos) -> Option<FluidBlock>
{
    let block: Rc<Block> = Block::byId(world.getBlockId(pos.x, pos.y, pos.z))?;
    let fluid: Rc<game::fluid::Fluid> = game::fluid::of(&block.m_blockMaterial)?;
    let level: i32 = world.getBlockMetadata(pos.x, pos.y, pos.z);
    let fluid: Identifier = Identifier::parse(&fluid.m_name).ok()?;
    Some(FluidBlock { fluid, level, source: level == 0 })
}

// Block.dropBlockAsItemWithChance at full luck, rolled into a list instead of spawned
// the rolls happen in the order old mods made them so the world's random advances the same way
fn blockDropStack(world: &Rc<World>, pos: BlockPos) -> Option<StackData>
{
    let block: Rc<Block> = Block::byId(world.getBlockId(pos.x, pos.y, pos.z))?;
    let meta: i32 = world.getBlockMetadata(pos.x, pos.y, pos.z);
    let id: i32 = block.idDropped(meta, &world.m_rand);
    let count: i32 = block.quantityDropped(&world.m_rand);
    let damage: i32 = block.damageDropped(meta);
    (id > 0 && count != 0).then(|| StackData { id: crate::unify::numericItem(id), count, damage })
}

fn blockDrops(world: &Rc<World>, pos: BlockPos) -> Vec<StackData>
{
    let Some(block) = Block::byId(world.getBlockId(pos.x, pos.y, pos.z))
    else
    {
        return Vec::new();
    };
    let meta: i32 = world.getBlockMetadata(pos.x, pos.y, pos.z);
    let table: Option<Rc<game::loot::LootTable>> = game::loot::block(block.m_blockID);
    let mut drops: Vec<StackData> = Vec::new();

    if table.as_ref().is_none_or(|table| table.m_keepBeta)
    {
        for _ in 0..block.quantityDropped(&world.m_rand)
        {
            if world.m_rand.nextFloat() > NO_LUCK
            {
                continue;
            }

            let id: i32 = block.idDropped(meta, &world.m_rand);
            if id > 0
            {
                drops.push(StackData
                {
                    id: crate::unify::numericItem(id),
                    count: 1,
                    damage: block.damageDropped(meta),
                });
            }
        }
    }

    if let Some(table) = table
    {
        let context: game::loot::LootContext = game::loot::LootContext
        {
            m_world: Some(world),
            m_pos: Some([pos.x, pos.y, pos.z]),
            m_block: block.m_blockID,
            m_blockMeta: meta,
            ..game::loot::LootContext::default()
        };
        let rolled: Vec<Rc<ItemStack>> = game::loot::roll(&table, &world.m_rand, &context);
        drops.extend(
            rolled
                .iter()
                .map(|stack| StackData { id: crate::unify::numericItem(stack.m_itemID.get()), ..stackData(stack) }),
        );
    }

    drops
}

// ItemBlock.onItemUse with no player: the same checks and hooks, and no onBlockPlacedBy
fn placeBlock(world: &Rc<World>, pos: BlockPos, face: i32, data: &StackData) -> Result<bool, String>
{
    authoritative(world)?;

    let stack: Rc<ItemStack> = stackFrom(data).ok_or("cannot place an empty or unknown stack")?;
    let item: Rc<Item> = stack.getItem();
    if item.tryExt::<game::item::item_block::BlockItemExt>().is_none()
    {
        return Err("the stack holds no block".to_owned());
    }

    let id: i32 = game::item::item_block::blockID(&item);
    if !world.canBlockBePlacedAt(id, pos.x, pos.y, pos.z, false, face)
    {
        return Ok(false);
    }

    let meta: i32 = item.getPlacedBlockMetadata(stack.getItemDamage());
    let refused: bool = game::hooks::fire(game::hooks::GameEvent::BlockPlace
    {
        world,
        x: pos.x,
        y: pos.y,
        z: pos.z,
        block: id,
        meta,
        player: None,
    });
    if refused || !world.setBlockAndMetadataWithNotify(pos.x, pos.y, pos.z, id, meta)
    {
        return Ok(false);
    }

    let block: Rc<Block> = Block::byId(id).ok_or("the placed block is not registered")?;
    block.onBlockPlaced(world, pos.x, pos.y, pos.z, face);
    placeSound(world, &block, pos);
    Ok(true)
}

// the step sound ItemBlock.onItemUse plays
fn placeSound(world: &World, block: &Block, pos: BlockPos)
{
    let sound: Rc<game::block::StepSound> = block.m_stepSound.get();
    let volume: f32 = (sound.getVolume() + 1.0) / 2.0;
    let pitch: f32 = sound.getPitch() * PLACE_PITCH;
    let center: [f64; 3] = [pos.x as f64 + HALF, pos.y as f64 + HALF, pos.z as f64 + HALF];
    world.playSoundEffect(center[0], center[1], center[2], &sound.getStepping(), volume, pitch);
}

fn craftResult(grid: &[Option<StackData>]) -> Option<StackData>
{
    let slots: Vec<Option<Rc<ItemStack>>> =
        (0..GRID_SLOTS).map(|slot| grid.get(slot).cloned().flatten().as_ref().and_then(stackFrom)).collect();
    let result: Rc<ItemStack> = CraftingManager::getInstance().findMatchingRecipe(&Grid { m_slots: slots })?;
    Some(stackData(&result))
}

fn containerItem(data: &StackData) -> Option<StackData>
{
    let container: Rc<Item> = Item::byId(data.id)?.getContainerItem()?;
    Some(StackData { id: container.m_shiftedIndex, count: 1, damage: 0 })
}

fn itemInfo(data: &StackData) -> Option<ItemInfo>
{
    let item: Rc<Item> = Item::byId(data.id)?;
    // item ids run past the block table, so the range check comes before the lookup
    let block: Option<Rc<Block>> = Block::isBlockItem(data.id).then(|| Block::byId(data.id)).flatten();
    Some(ItemInfo
    {
        max_stack: item.getItemStackLimit(),
        max_damage: item.getMaxDamage(),
        subtypes: item.getHasSubtypes(),
        block: block.is_some(),
        cube: block.is_some_and(|block| block.renderAsNormalBlock()),
    })
}

fn dropItem(world: &Rc<World>, pos: [f64; 3], data: &StackData, motion: [f64; 3], delay: i32) -> Result<i32, String>
{
    authoritative(world)?;

    let stack: Rc<ItemStack> = stackFrom(data).ok_or("cannot drop an empty or unknown stack")?;
    let item: Rc<Entity> = game::entity::entity_item::create(world, pos[0], pos[1], pos[2], stack);
    item.m_motionX.set(motion[0]);
    item.m_motionY.set(motion[1]);
    item.m_motionZ.set(motion[2]);
    game::entity::entity_item::fields(&item).m_delayBeforeCanPickup.set(delay);
    world.entityJoinedWorld(&item);
    Ok(item.m_entityId.get())
}

// the inventory an entity carries itself, a minecart's chest
fn entityInventory(dimension: i32, id: i32) -> Result<Rc<dyn game::inventory::IInventory>, String>
{
    let entity: Rc<Entity> =
        context::entityIn(dimension, id).ok_or_else(|| format!("no entity {id} in dimension {dimension}"))?;
    if !entity.instanceOf("EntityMinecart")
    {
        return Err(format!("entity {id} has no inventory"));
    }

    Ok(game::entity::entity_minecart::inventory(&entity))
}

fn setItemStack(dimension: i32, id: i32, data: &StackData) -> Result<(), String>
{
    let entity: Rc<Entity> =
        context::entityIn(dimension, id).ok_or_else(|| format!("no entity {id} in dimension {dimension}"))?;
    authoritative(&entity.world())?;

    let fields: &game::entity::entity_item::ItemFields =
        entity.m_ext.asAny().downcast_ref().ok_or("the entity is not a dropped item")?;
    if data.count <= 0
    {
        entity.setEntityDead();
        return Ok(());
    }

    fields.m_item.set(Some(stackFrom(data).ok_or("cannot give a dropped item an unknown stack")?));
    Ok(())
}

fn entitiesIn(world: &World, min: [f64; 3], max: [f64; 3]) -> HostReply
{
    let bounds: Rc<AxisAlignedBB> =
        AxisAlignedBB::getBoundingBoxFromPool(min[0], min[1], min[2], max[0], max[1], max[2]);
    let found: Vec<Rc<Entity>> = world.getEntitiesWithinAABBExcludingEntity(None, &bounds).toVec();
    HostReply::Entities(found.iter().map(entityInfo).collect())
}

// <game folder>/moddata/<modid>/<key>.dat, outside every save for data such as blueprints that outlives one save
fn sharedPath(runtime: &Rc<ModRuntime>, key: &str) -> Result<std::path::PathBuf, String>
{
    const FOLDER: &str = "moddata";
    const EXTENSION: &str = "dat";

    let valid: bool =
        !key.is_empty() && key.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if !valid
    {
        return Err(format!("shared data key '{key}' must be lowercase letters, digits and underscores"));
    }

    let paths: crate::paths::LoaderPaths = crate::paths().ok_or("the loader has no game folder")?;
    Ok(paths.m_root.join(FOLDER).join(runtime.id()).join(format!("{key}.{EXTENSION}")))
}

fn writeShared(path: &std::path::Path, data: &[u8]) -> Result<(), String>
{
    if data.is_empty()
    {
        return match std::fs::remove_file(path)
        {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound =>
            {
                Err(format!("could not remove {}: {error}", path.display()))
            }
            _ => Ok(()),
        };
    }

    let folder: &std::path::Path = path.parent().ok_or("shared data has no folder")?;
    std::fs::create_dir_all(folder).map_err(|error| format!("could not create {}: {error}", folder.display()))?;
    std::fs::write(path, data).map_err(|error| format!("could not write {}: {error}", path.display()))
}

fn itemModel(runtime: &Rc<ModRuntime>, block: Identifier, model: ItemModelSpec) -> Result<(), String>
{
    if crate::state::phase() != crate::state::Phase::Register
    {
        return Err("item models register during the register phase".to_owned());
    }

    let id: Identifier = runtime.own(&block.to_string())?;
    crate::content::blocks::setItemModel(id, model);
    Ok(())
}

fn stateModel(runtime: &Rc<ModRuntime>, block: Identifier, model: StateModelSpec) -> Result<(), String>
{
    if crate::state::phase() != crate::state::Phase::Register
    {
        return Err("state models register during the register phase".to_owned());
    }

    let id: Identifier = runtime.own(&block.to_string())?;
    crate::content::blocks::setStateModel(id, model);
    Ok(())
}

// the tech capability calls of api 1.33, anything else goes on to the extension systems
pub fn handle(runtime: &Rc<ModRuntime>, call: HostCall) -> HostReply
{
    match call
    {
        HostCall::Capabilities { world: dimension, pos, .. } =>
        {
            ok(world(dimension), |world| HostReply::Int(capabilities(&world, pos)))
        }

        HostCall::EnergyDemand { world: dimension, pos, face, kind } =>
        {
            ok(world(dimension), |world| HostReply::Demand(energyDemand(&world, pos, face, kind)))
        }

        HostCall::SendEnergy { world: dimension, pos, face, kind, amount, simulate } =>
        {
            ok(world(dimension), |world| HostReply::Long(sendEnergy(&world, pos, face, kind, amount, simulate)))
        }

        HostCall::FillFluid { world: dimension, pos, face, stack, simulate } =>
        {
            ok(world(dimension), |world| HostReply::Int(fill(&world, pos, face, stack, simulate)))
        }

        HostCall::DrainFluid { world: dimension, pos, face, max, simulate } =>
        {
            ok(world(dimension), |world| HostReply::Fluid(drain(&world, pos, face, max, simulate)))
        }

        HostCall::FluidAt { world: dimension, pos } =>
        {
            ok(world(dimension), |world| HostReply::FluidBlock(fluidAt(&world, pos)))
        }

        HostCall::BlockDrops { world: dimension, pos } =>
        {
            ok(world(dimension), |world| HostReply::Stacks(blockDrops(&world, pos)))
        }

        HostCall::BlockDropStack { world: dimension, pos } =>
        {
            ok(world(dimension), |world| HostReply::Stack(blockDropStack(&world, pos)))
        }

        HostCall::PlaceBlock { world: dimension, pos, face, stack } =>
        {
            ok(world(dimension).and_then(|world| placeBlock(&world, pos, face, &stack)), HostReply::Bool)
        }

        HostCall::CraftResult { grid } => HostReply::Stack(craftResult(&grid)),
        HostCall::ContainerItem { stack } => HostReply::Stack(containerItem(&stack)),

        HostCall::FuelTicks { stack } =>
        {
            HostReply::Int(game::tileentity::tile_entity_furnace::getItemBurnTime(stackFrom(&stack)))
        }

        HostCall::SetItemStack { world: dimension, id, stack } => unit(setItemStack(dimension, id, &stack)),

        HostCall::EntitiesIn { world: dimension, min, max } =>
        {
            ok(world(dimension), |world| entitiesIn(&world, min, max))
        }

        HostCall::InventorySize { world: dimension, pos } => ok(world(dimension), |world|
        {
            let tile: Option<Rc<TileEntity>> =
                world.getBlockTileEntity(pos.x, pos.y, pos.z).filter(|tile| tile.instanceOf("IInventory"));
            HostReply::Int(tile.map_or(0, |tile| tile.getSizeInventory()))
        }),

        HostCall::SharedData { key } =>
        {
            ok(sharedPath(runtime, &key), |path| HostReply::Bytes(std::fs::read(path).unwrap_or_default()))
        }
        HostCall::SetSharedData { key, data } =>
        {
            unit(sharedPath(runtime, &key).and_then(|path| writeShared(&path, &data)))
        }

        HostCall::BiomeAt { world: dimension, x, z } =>
        {
            ok(world(dimension), |world| HostReply::Id(crate::biomes::biomeAt(&world, x, z)))
        }

        HostCall::ItemInfo { stack } => HostReply::ItemInfo(itemInfo(&stack)),

        HostCall::DropItem { world: dimension, pos, stack, motion, delay } =>
        {
            ok(world(dimension).and_then(|world| dropItem(&world, pos, &stack, motion, delay)), HostReply::Int)
        }

        HostCall::Unified { stack } =>
        {
            HostReply::Stack(Some(StackData { id: crate::unify::numericItem(stack.id), ..stack }))
        }
        HostCall::EntityInventorySize { world: dimension, id } =>
        {
            HostReply::Int(entityInventory(dimension, id).map_or(0, |inventory| inventory.getSizeInventory()))
        }

        HostCall::EntitySlot { world: dimension, id, slot } => ok(entityInventory(dimension, id), |inventory|
        {
            HostReply::Stack(inventory.getStackInSlot(slot).map(|stack| stackData(&stack)))
        }),

        HostCall::SetEntitySlot { world: dimension, id, slot, stack } =>
        {
            unit(entityInventory(dimension, id).and_then(|inventory|
            {
                if !(0..inventory.getSizeInventory()).contains(&slot)
                {
                    return Err(format!("the entity has no slot {slot}"));
                }

                inventory.setInventorySlotContents(slot, stack.as_ref().and_then(stackFrom));
                Ok(())
            }))
        }

        HostCall::PaintSprite { texture, size, pixels } =>
        {
            if !crate::side::isClient()
            {
                return HostReply::Unit;
            }

            unit(crate::atlas::paint(&texture, size, pixels.into_iter().map(|pixel| pixel as i32).collect()))
        }

        HostCall::SpritePixels { texture } =>
        {
            let (size, pixels): (u32, Vec<u32>) = crate::atlas::pixels(&texture).unwrap_or_default();
            HostReply::Pixels { size, pixels }
        }

        HostCall::BlockItemModel { block, model } => unit(itemModel(runtime, block, model)),
        HostCall::SetStateModel { block, model } => unit(stateModel(runtime, block, model)),

        HostCall::ScreenCraftSlot { screen, first, x, y } => unit(runtime.own(&screen).map(|id|
        {
            crate::containers::setCraftSlot(id, crate::containers::CraftSlot { m_first: first, m_x: x, m_y: y })
        })),

        HostCall::TileStackLimit { tile, limit } => unit(tiles::setStackLimit(runtime.m_index, &tile, limit)),
        other => crate::extensions::handle(runtime, other),
    }
}
