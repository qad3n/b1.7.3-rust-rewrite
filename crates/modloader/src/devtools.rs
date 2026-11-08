use crate::host::context;
use game::block::Block;
use game::inventory::IInventory;
use game::world::World;
use mod_protocol::abi::Sender;
use std::rc::Rc;
use std::time::Instant;

// console and op tools for checking ported content without a client: generate terrain, map it, read columns
pub const COMMAND: &str = "loader";
const USAGE: &str = "loader gen <dim> <chunkX> <chunkZ> <radius> | map <dim> <chunkX> <chunkZ> <radius> <file.png> | column <dim> <x> <z> | tp <player> <dim> <x> <y> <z> [yaw pitch] | fill <dim> <x1> <y1> <z1> <x2> <y2> <z2> <block[key=value,...]> | use <player> <item> <x> <y> <z> <face> | entities <dim> | summon <dim> <entity> <x> <y> <z> | slot <dim> <x> <y> <z> <slot> <item|air> [count] [damage] | tile <dim> <x> <y> <z> | slice <dim> <x|z> <at> <from> <to> <file.png> | interact <player> <entity> | close <player>";

mod map
{
    pub const HEIGHT: i32 = 128;
    // height shading spans this many blocks either side of the middle of the world
    pub const SHADE_SPAN: f32 = 64.0;
    pub const MIN_SHADE: f32 = 0.45;
    pub const VOID: u32 = 0x000000;
    pub const CHUNK: i32 = 16;
    pub const CHANNELS: usize = 3;
    pub const GLOW: f32 = 1.6;
}

const MILLIS_PER_SECOND: f64 = 1000.0;

fn channels(color: u32) -> [u8; map::CHANNELS] { [(color >> 16) as u8, (color >> 8) as u8, color as u8] }

fn shaded(color: u32, shade: f32) -> u32
{
    let [red, green, blue] = channels(color).map(|channel| (channel as f32 * shade).min(u8::MAX as f32) as u32);
    red << 16 | green << 8 | blue
}

fn number(args: &[String], index: usize) -> Result<i32, String>
{
    let text: &String = args.get(index).ok_or_else(|| format!("missing argument {index}, usage: {USAGE}"))?;
    text.parse::<i32>().map_err(|_| format!("{text} is not a number"))
}

fn world(dimension: i32) -> Result<Rc<World>, String>
{
    context::world(dimension).ok_or_else(|| format!("dimension {dimension} is not loaded on this side"))
}

// loads every chunk in the square, which generates and populates the inner ones; prepareChunk loads where a
// dedicated server's getChunk would hand back its empty stand in
fn generate(world: &World, centerX: i32, centerZ: i32, radius: i32) -> usize
{
    let provider = world.chunkProvider();
    let mut count: usize = 0;
    for chunkX in centerX - radius..=centerX + radius
    {
        for chunkZ in centerZ - radius..=centerZ + radius
        {
            provider.prepareChunk(chunkX, chunkZ);
            count += 1;
        }
    }
    count
}

fn topColor(world: &World, x: i32, z: i32) -> u32
{
    for y in (0..map::HEIGHT).rev()
    {
        let id: i32 = world.getBlockId(x, y, z);
        if id == 0
        {
            continue;
        }
        let Some(block) = Block::byId(id)
        else
        {
            continue;
        };
        let base: u32 = block.m_blockMaterial.m_materialMapColor.m_colorValue as u32;
        let shade: f32 = (1.0 + (y as f32 - map::SHADE_SPAN) / (map::SHADE_SPAN * 2.0)).max(map::MIN_SHADE).min(1.0);
        return shaded(base, shade);
    }
    map::VOID
}

fn writePng(path: &str, width: i32, height: i32, pixels: &[u8]) -> Result<(), String>
{
    let file: std::fs::File = std::fs::File::create(path).map_err(|error| format!("{path}: {error}"))?;
    let mut encoder: png::Encoder<std::fs::File> = png::Encoder::new(file, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer.write_image_data(pixels).map_err(|error| error.to_string())
}

fn writeMap(world: &World, centerX: i32, centerZ: i32, radius: i32, path: &str) -> Result<String, String>
{
    let size: i32 = (radius * 2 + 1) * map::CHUNK;
    let originX: i32 = (centerX - radius) * map::CHUNK;
    let originZ: i32 = (centerZ - radius) * map::CHUNK;
    let mut pixels: Vec<u8> = Vec::with_capacity((size * size) as usize * map::CHANNELS);
    for z in 0..size
    {
        for x in 0..size
        {
            let color: u32 = topColor(world, originX + x, originZ + z);
            pixels.extend_from_slice(&channels(color));
        }
    }
    writePng(path, size, size, &pixels)?;
    Ok(format!("wrote {size}x{size} map of blocks {originX},{originZ} to {path}"))
}

// a block's map color, glowing blocks drawn brighter so lit stone stands out in a slice
fn blockColor(world: &World, x: i32, y: i32, z: i32) -> u32
{
    let id: i32 = world.getBlockId(x, y, z);
    let Some(block) = Block::byId(id).filter(|_| id != 0)
    else
    {
        return map::VOID;
    };
    let base: u32 = block.m_blockMaterial.m_materialMapColor.m_colorValue as u32;
    if Block::lightValue(id) > 0
    {
        shaded(base, map::GLOW)
    }
    else
    {
        base
    }
}

// a vertical cut through the world along x or z, top of the world at the top of the image
fn slice(world: &World, args: &[String]) -> Result<String, String>
{
    let axis: &str = args.get(2).map(String::as_str).ok_or(USAGE)?;
    let [at, from, to]: [i32; 3] = [number(args, 3)?, number(args, 4)?, number(args, 5)?];
    let path: &String = args.get(6).ok_or(USAGE)?;
    let width: i32 = to - from + 1;
    let mut pixels: Vec<u8> = Vec::with_capacity((width * map::HEIGHT) as usize * map::CHANNELS);
    for y in (0..map::HEIGHT).rev()
    {
        for along in from..=to
        {
            let color: u32 = match axis
            {
                "x" => blockColor(world, at, y, along),
                _ => blockColor(world, along, y, at),
            };
            pixels.extend_from_slice(&channels(color));
        }
    }
    writePng(path, width, map::HEIGHT, &pixels)?;
    Ok(format!("wrote {width}x{} slice at {axis}={at} to {path}", map::HEIGHT))
}

fn column(world: &World, x: i32, z: i32) -> String
{
    let mut runs: Vec<String> = Vec::new();
    let mut y: i32 = map::HEIGHT - 1;
    while y >= 0
    {
        let (id, meta): (i32, i32) = (world.getBlockId(x, y, z), world.getBlockMetadata(x, y, z));
        let top: i32 = y;
        while y >= 0 && world.getBlockId(x, y, z) == id && world.getBlockMetadata(x, y, z) == meta
        {
            y -= 1;
        }
        if id != 0
        {
            runs.push(format!("{} {}..{top}", stateName(id, meta), y + 1));
        }
    }
    if runs.is_empty()
    {
        "empty column".to_owned()
    }
    else
    {
        runs.join(", ")
    }
}

// a block by name with its named state, the numeric id when nothing names it
fn stateName(id: i32, meta: i32) -> String
{
    let Some(state) = crate::states::describe(id, meta)
    else
    {
        return format!("{id}:{meta}");
    };
    let values: Vec<String> = state.properties.iter().map(|(key, value)| format!("{key}={value}")).collect();
    format!("{}[{}]", state.block, values.join(","))
}

// ns:path or ns:path[key=value,...], the state syntax the column listing prints
fn parseState(text: &str) -> Result<mod_protocol::abi::StateData, String>
{
    let (name, values): (&str, &str) = match text.split_once('[')
    {
        Some((name, rest)) => (name, rest.strip_suffix(']').ok_or_else(|| format!("{text} misses its ]"))?),
        None => (text, ""),
    };
    let block: mod_protocol::Identifier = mod_protocol::Identifier::parse(name).map_err(|error| error.to_string())?;
    let properties: std::collections::BTreeMap<String, String> = values
        .split(',')
        .filter(|pair| !pair.is_empty())
        .map(|pair| pair.split_once('=').map(|(key, value)| (key.to_owned(), value.to_owned())))
        .collect::<Option<_>>()
        .ok_or_else(|| format!("{text}: properties are key=value"))?;
    Ok(mod_protocol::abi::StateData { block, properties })
}

fn namedPlayer(name: &str) -> Result<Rc<game::entity::Entity>, String>
{
    context::worlds()
        .iter()
        .flat_map(|world| world.m_playerEntities.toVec())
        .find(|player| {
            game::entity::entity_player::fields(player)
                .m_username
                .get()
                .is_some_and(|username| username.to_string() == name)
        })
        .ok_or_else(|| format!("no player named {name}"))
}

// puts an item in the player's hand and uses it on a block face, the same path a right click takes on the server
fn useItem(args: &[String]) -> Result<String, String>
{
    let name: &String = args.get(1).ok_or(USAGE)?;
    let player: Rc<game::entity::Entity> = namedPlayer(name)?;
    let itemName: &String = args.get(2).ok_or(USAGE)?;
    let id: mod_protocol::Identifier = mod_protocol::Identifier::parse(itemName).map_err(|error| error.to_string())?;
    let item: i32 =
        crate::registry::with(|registries| registries.itemId(&id)).ok_or_else(|| format!("no item {id}"))?;
    let world: Rc<World> = player.m_worldObj.get().ok_or("the player has no world")?;
    let [x, y, z, face]: [i32; 4] = [number(args, 3)?, number(args, 4)?, number(args, 5)?, number(args, 6)?];
    let inventory = game::entity::entity_player::inventory(&player);
    inventory
        .setInventorySlotContents(inventory.m_currentItem.get(), Some(game::item::ItemStack::withDamage(item, 1, 0)));
    let stack = inventory.getCurrentItem().ok_or("the hand stayed empty")?;
    let used: bool = stack.useItem(&player, &world, x, y, z, face);
    let held: String = inventory
        .getCurrentItem()
        .and_then(|held| {
            crate::registry::with(|registries| {
                registries.m_items.byNumeric(held.m_itemID.get()).map(|row| row.m_id.to_string())
            })
        })
        .unwrap_or_else(|| "nothing".to_owned());
    Ok(format!("{name} used {id} on {x} {y} {z} face {face}: used {used}, now holding {held}"))
}

// right clicks an entity as a player would, the server side of the interaction
fn interact(args: &[String]) -> Result<String, String>
{
    let name: &String = args.get(1).ok_or(USAGE)?;
    let player: Rc<game::entity::Entity> = namedPlayer(name)?;
    let world: Rc<World> = player.m_worldObj.get().ok_or("the player has no world")?;
    let id: i32 = number(args, 2)?;
    let entity: Rc<game::entity::Entity> = context::entityById(&world, id).ok_or_else(|| format!("no entity {id}"))?;
    let handled: bool = entity.interact(&player);
    Ok(format!("{name} interacted with {id}: {handled}"))
}

// sends a player to any loaded dimension through the same queued move mods use
fn teleport(args: &[String]) -> Result<String, String>
{
    let name: &String = args.get(1).ok_or(USAGE)?;
    let player: Rc<game::entity::Entity> = namedPlayer(name)?;
    let dimension: i32 = number(args, 2)?;
    let pos: [f64; 3] = [number(args, 3)? as f64 + 0.5, number(args, 4)? as f64, number(args, 5)? as f64 + 0.5];
    if let (Ok(yaw), Ok(pitch)) = (number(args, 6), number(args, 7))
    {
        player.m_rotationYaw.set(yaw as f32);
        player.m_rotationPitch.set(pitch as f32);
    }
    crate::netplay::teleport(player.m_entityId.get(), dimension, pos)?;
    Ok(format!("sending {name} to dimension {dimension} at {pos:?}"))
}

// places a box of one block by registry name, for building test setups from the console
fn fill(world: &Rc<World>, args: &[String]) -> Result<String, String>
{
    let corners: Vec<i32> = (2..8).map(|index| number(args, index)).collect::<Result<_, _>>()?;
    let name: &String = args.get(8).ok_or(USAGE)?;
    let state: mod_protocol::abi::StateData = parseState(name)?;
    let (block, meta): (i32, i32) = if state.block == mod_protocol::Identifier::minecraft("air")
    {
        (0, 0)
    }
    else
    {
        crate::states::resolve(&state).ok_or_else(|| format!("no block {}", state.block))?
    };
    let mut count: i32 = 0;
    for x in corners[0].min(corners[3])..=corners[0].max(corners[3])
    {
        for y in corners[1].min(corners[4])..=corners[1].max(corners[4])
        {
            for z in corners[2].min(corners[5])..=corners[2].max(corners[5])
            {
                world.setBlockAndMetadataWithNotify(x, y, z, block, meta);
                count += 1;
            }
        }
    }
    Ok(format!("placed {count} {name}"))
}

fn itemId(name: &str) -> Result<i32, String>
{
    let id: mod_protocol::Identifier = mod_protocol::Identifier::parse(name).map_err(|error| error.to_string())?;
    crate::registry::with(|registries| registries.itemId(&id)).ok_or_else(|| format!("no item {id}"))
}

fn itemName(id: i32) -> String
{
    crate::registry::with(|registries| registries.m_items.byNumeric(id).map(|row| row.m_id.to_string()))
        .unwrap_or_else(|| id.to_string())
}

fn inventoryTile(world: &World, args: &[String]) -> Result<Rc<game::tileentity::TileEntity>, String>
{
    let [x, y, z]: [i32; 3] = [number(args, 2)?, number(args, 3)?, number(args, 4)?];
    world
        .getBlockTileEntity(x, y, z)
        .filter(|tile| tile.instanceOf("IInventory"))
        .ok_or_else(|| format!("no inventory at {x} {y} {z}"))
}

// fills one slot of a mod tile entity, for feeding machines from the console
fn slot(world: &World, args: &[String]) -> Result<String, String>
{
    let tile: Rc<game::tileentity::TileEntity> = inventoryTile(world, args)?;
    let index: i32 = number(args, 5)?;
    let name: &String = args.get(6).ok_or(USAGE)?;
    let count: i32 = number(args, 7).unwrap_or(1);
    let damage: i32 = number(args, 8).unwrap_or(0);
    let stack: Option<Rc<game::item::ItemStack>> = if name == "air"
    {
        None
    }
    else
    {
        Some(game::item::ItemStack::withDamage(itemId(name)?, count, damage))
    };
    tile.setInventorySlotContents(index, stack);
    Ok(format!("slot {index} set to {count} {name}"))
}

// lists an inventory's slots with their components, and mod data and attachments
fn tile(world: &World, args: &[String]) -> Result<String, String>
{
    let tile: Rc<game::tileentity::TileEntity> = inventoryTile(world, args)?;
    let slots: Vec<String> = (0..tile.getSizeInventory())
        .map(|slot| {
            tile.getStackInSlot(slot).map_or("empty".to_owned(), |stack| {
                let components: Vec<String> = stack.m_components.borrow().keys().cloned().collect();
                format!(
                    "{} {}@{} {components:?}",
                    stack.m_stackSize.get(),
                    itemName(stack.m_itemID.get()),
                    stack.getItemDamage()
                )
            })
        })
        .collect();
    let data: Vec<u8> =
        tile.tryExt::<crate::tiles::ModTileExt>().map(|ext| ext.m_data.borrow().clone()).unwrap_or_default();
    let attachments: Vec<String> = tile.m_attachments.borrow().keys().cloned().collect();
    Ok(format!("slots [{}] data {data:?} attachments {attachments:?}", slots.join(", ")))
}

fn entities(world: &World) -> String
{
    let describe = |entity: &Rc<game::entity::Entity>| {
        let info = context::entityInfo(entity);
        format!(
            "{} {} {:.1},{:.1},{:.1}",
            info.id,
            info.kind.map_or("?".to_owned(), |kind| kind.to_string()),
            info.pos[0],
            info.pos[1],
            info.pos[2]
        )
    };
    let loaded: Vec<String> = world.m_loadedEntityList.toVec().iter().map(describe).collect();
    let players: Vec<String> = world.m_playerEntities.toVec().iter().map(describe).collect();
    format!("loaded [{}] players [{}]", loaded.join("; "), players.join("; "))
}

fn run(args: &[String]) -> Result<String, String>
{
    let tool: &str = args.first().map(String::as_str).ok_or(USAGE)?;
    match tool
    {
        "tp" => return teleport(args),
        "use" => return useItem(args),
        "interact" => return interact(args),
        "close" =>
        {
            let name: &String = args.get(1).ok_or(USAGE)?;
            namedPlayer(name)?.closeScreen();
            return Ok(format!("closed {name}'s screen"));
        }
        _ =>
        {}
    }
    let dimension: i32 = number(args, 1)?;
    let world: Rc<World> = world(dimension)?;
    match tool
    {
        "gen" =>
        {
            let started: Instant = Instant::now();
            let count: usize = generate(&world, number(args, 2)?, number(args, 3)?, number(args, 4)?);
            let elapsed: f64 = started.elapsed().as_secs_f64() * MILLIS_PER_SECOND;
            Ok(format!("loaded {count} chunks in {elapsed:.0} ms, {:.2} ms each", elapsed / count.max(1) as f64))
        }
        "map" =>
        {
            let (x, z, radius): (i32, i32, i32) = (number(args, 2)?, number(args, 3)?, number(args, 4)?);
            generate(&world, x, z, radius + 1);
            let path: &String = args.get(5).ok_or(USAGE)?;
            writeMap(&world, x, z, radius, path)
        }
        "column" => Ok(column(&world, number(args, 2)?, number(args, 3)?)),
        "slice" => slice(&world, args),
        "fill" => fill(&world, args),
        "entities" => Ok(entities(&world)),
        "slot" => slot(&world, args),
        "tile" => tile(&world, args),
        "summon" =>
        {
            let kind: &String = args.get(2).ok_or(USAGE)?;
            let id: mod_protocol::Identifier =
                mod_protocol::Identifier::parse(kind).map_err(|error| error.to_string())?;
            let pos: [f64; 3] = [number(args, 3)? as f64 + 0.5, number(args, 4)? as f64, number(args, 5)? as f64 + 0.5];
            let entity: i32 = crate::entities::spawn(&world, &id, pos, Vec::new())?;
            Ok(format!("summoned {id} as entity {entity}"))
        }
        _ => Err(USAGE.to_owned()),
    }
}

pub fn execute(args: &[String], sender: &Sender) -> String
{
    if sender.player.is_some() && !sender.op
    {
        return "\u{a7}cOnly ops and the console use /loader".to_owned();
    }
    run(args).unwrap_or_else(|reason| format!("\u{a7}c{reason}"))
}
