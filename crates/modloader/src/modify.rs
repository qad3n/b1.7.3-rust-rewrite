use crate::log;
use game::block::Block;
use game::item::Item;
use game::worldgen::{BiomeGenBase, SpawnListEntry};
use jvm::array_list::JArrayList;
use mod_protocol::Identifier;
use mod_protocol::spec::{BlockModifySpec, ItemModifySpec, SpawnModifySpec, ToolSpec};
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

// a field a block change can write, by name and whether a change sets it
struct FieldRow
{
    m_name: &'static str,
    m_present: fn(&BlockModifySpec) -> bool,
}

const FIELDS: [FieldRow; 7] = [
    FieldRow { m_name: "hardness", m_present: |spec| spec.hardness.is_some() },
    FieldRow { m_name: "resistance", m_present: |spec| spec.resistance.is_some() },
    FieldRow { m_name: "light", m_present: |spec| spec.light.is_some() },
    FieldRow { m_name: "opacity", m_present: |spec| spec.opacity.is_some() },
    FieldRow { m_name: "slipperiness", m_present: |spec| spec.slipperiness.is_some() },
    FieldRow { m_name: "flammability", m_present: |spec| spec.flammability.is_some() },
    FieldRow { m_name: "tool", m_present: |spec| spec.tool.is_some() },
];

// a block's or item's values before any change, so a server's changes can be taken back on leaving it
#[derive(Clone, Copy)]
enum Original
{
    Block
    {
        hardness: f32, resistance: f32, light: i32, opacity: i32, slipperiness: f32, fire: (i32, i32)
    },
    Item
    {
        stack: i32, damage: i32
    },
}

// both modify lists, what a server sends a client in place of the client's own
pub type ModifyWire = (Vec<(String, BlockModifySpec)>, Vec<(String, ItemModifySpec)>);

#[derive(Default)]
struct State
{
    // owner and change in the order mods registered them, which is load order
    m_blocks: Vec<(String, BlockModifySpec)>,
    m_spawns: Vec<(String, SpawnModifySpec)>,
    m_items: Vec<(String, ItemModifySpec)>,
    m_tools: HashMap<i32, ToolSpec>,
    m_conflicts: Vec<String>,
    m_originals: BTreeMap<(bool, i32), Original>,
    m_local: Option<ModifyWire>,
}

thread_local!
{
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn queueBlock(owner: &str, spec: BlockModifySpec)
{
    STATE.with(|state| state.borrow_mut().m_blocks.push((owner.to_owned(), spec)));
}

pub fn queueItem(owner: &str, spec: ItemModifySpec)
{
    STATE.with(|state| state.borrow_mut().m_items.push((owner.to_owned(), spec)));
}

pub fn queueSpawn(owner: &str, spec: SpawnModifySpec)
{
    STATE.with(|state| state.borrow_mut().m_spawns.push((owner.to_owned(), spec)));
}

// the class name natural spawning keys entries by, vanilla mobs use their java class and mod mobs their identifier
thread_local!
{
    // spawn entries hold their entity class as a static name, built once per entity
    static CLASSES: RefCell<HashMap<Identifier, &'static str>> = RefCell::new(HashMap::new());
}

// a spawn list handler's answer as spawn entries, unknown entities and empty weights left out with a warning
pub fn spawnEntries(entries: &[mod_protocol::abi::SpawnEntry]) -> Vec<SpawnListEntry>
{
    entries
        .iter()
        .filter(|entry| entry.weight > 0)
        .filter_map(|entry| match cachedClass(&entry.entity)
        {
            Some(class) => Some(SpawnListEntry::new(class, entry.weight)),

            None =>
            {
                crate::log::warn(crate::log::LOADER, &format!("spawn list names unknown entity {}", entry.entity));
                None
            }
        })
        .collect()
}

fn cachedClass(entity: &Identifier) -> Option<&'static str>
{
    if let Some(class) = CLASSES.with(|classes| classes.borrow().get(entity).copied())
    {
        return Some(class);
    }

    let class: &'static str = spawnClass(entity)?;
    CLASSES.with(|classes| classes.borrow_mut().insert(entity.clone(), class));
    Some(class)
}

// the entity a spawn entry's class names, Entity<save key> for vanilla and the identifier for mods
pub fn spawnEntity(class: &str) -> Option<Identifier>
{
    if let Some(saveKey) = class.strip_prefix("Entity")
    {
        let vanilla: Option<Identifier> = crate::registry::with(|registries|
        {
            registries
                .m_entities
                .rows()
                .find(|row| row.m_id.isVanilla() && row.m_value.m_saveKey == saveKey)
                .map(|row| row.m_id.clone())
        });
        if vanilla.is_some()
        {
            return vanilla;
        }
    }

    Identifier::parse(class).ok()
}

fn spawnClass(entity: &Identifier) -> Option<&'static str>
{
    let saveKey: String =
        crate::registry::with(|registries| registries.m_entities.get(entity).map(|row| row.m_value.m_saveKey.clone()))?;

    let name: String = if entity.isVanilla()
    {
        format!("Entity{saveKey}")
    }
    else
    {
        entity.to_string()
    };
    Some(Box::leak(name.into_boxed_str()))
}

fn applySpawn(biome: &BiomeGenBase, class: &'static str, spec: &SpawnModifySpec)
{
    let list: &Rc<JArrayList<SpawnListEntry>> = crate::entities::spawnList(biome, spec.spawn.kind);
    for index in (0..list.size()).rev()
    {
        if list.get(index).m_entityClass == class
        {
            list.removeAt(index);
        }
    }

    if spec.spawn.weight > 0
    {
        list.add(SpawnListEntry::new(class, spec.spawn.weight));
    }
}

// vanilla biomes after the loader adds mod mobs, in load order so a later mod overrides an earlier one
pub fn applySpawns()
{
    STATE.with(|state|
    {
        for (owner, spec) in &state.borrow().m_spawns
        {
            let Some(class) = spawnClass(&spec.entity)
            else
            {
                log::warn(owner, &format!("spawn change names unknown entity {}", spec.entity));
                continue;
            };

            let biomes: Vec<Rc<BiomeGenBase>> = crate::entities::biomesFor(&spec.spawn);
            log::debug(
                owner,
                &format!("spawn change {} weight {} in {} biomes", spec.entity, spec.spawn.weight, biomes.len()),
            );

            for biome in biomes
            {
                applySpawn(&biome, class, spec);
            }
        }
    });
}

// a mod biome gets the changes that name it when its biome object is built
pub fn applySpawnsTo(biome: &BiomeGenBase, id: &Identifier)
{
    STATE.with(|state|
    {
        for (_, spec) in state.borrow().m_spawns.iter().filter(|(_, spec)| spec.spawn.biomes.contains(id))
        {
            if let Some(class) = spawnClass(&spec.entity)
            {
                applySpawn(biome, class, spec);
            }
        }
    });
}

fn remember(
    originals: &mut BTreeMap<(bool, i32), Original>,
    block: bool,
    numeric: i32,
    original: impl FnOnce() -> Original,
)
{
    originals.entry((block, numeric)).or_insert_with(original);
}

fn blockOriginal(block: &Block) -> Original
{
    let numeric: i32 = block.m_blockID;
    Original::Block
    {
        hardness: block.m_blockHardness.get(),
        resistance: block.m_blockResistance.get(),
        light: Block::lightValue(numeric),
        opacity: Block::lightOpacity(numeric),
        slipperiness: block.m_slipperiness.get(),
        fire: game::block::block_fire::burnRates(numeric),
    }
}

// every changed block and item gets its first recorded values back
fn revert(originals: &mut BTreeMap<(bool, i32), Original>)
{
    for ((_, numeric), original) in std::mem::take(originals)
    {
        match original
        {
            Original::Block { hardness, resistance, light, opacity, slipperiness, fire } =>
            {
                if let Some(block) = Block::byId(numeric)
                {
                    block.m_blockHardness.set(hardness);
                    block.m_blockResistance.set(resistance);
                    block.m_slipperiness.set(slipperiness);
                    Block::setLightEntries(numeric, light, opacity);
                    game::block::block_fire::setFlammable(numeric, fire.0, fire.1);
                }
            }

            Original::Item { stack, damage } =>
            {
                if let Some(item) = Item::byId(numeric)
                {
                    item.m_maxStackSize.set(stack);
                    item.setMaxDamage(damage);
                }
            }
        }
    }
}

pub fn snapshot() -> ModifyWire
{
    STATE.with(|state| (state.borrow().m_blocks.clone(), state.borrow().m_items.clone()))
}

// a modded server's changes stand in for the client's own until it disconnects
pub fn replace(wire: ModifyWire)
{
    STATE.with(|state|
    {
        let mut state = state.borrow_mut();
        let own: ModifyWire =
            (std::mem::replace(&mut state.m_blocks, wire.0), std::mem::replace(&mut state.m_items, wire.1));
        state.m_local.get_or_insert(own);
    });
    apply();
}

pub fn restore()
{
    let Some((blocks, items)) = STATE.with(|state| state.borrow_mut().m_local.take())
    else
    {
        return;
    };

    STATE.with(|state|
    {
        let mut state = state.borrow_mut();
        state.m_blocks = blocks;
        state.m_items = items;
    });
    apply();
}

fn applyItem(item: Rc<Item>, spec: &ItemModifySpec)
{
    let mut item: Rc<Item> = item;

    if let Some(size) = spec.max_stack
    {
        item = item.setMaxStackSize(size);
    }

    if let Some(damage) = spec.max_damage
    {
        item.setMaxDamage(damage);
    }
}

fn applyBlock(block: Rc<Block>, spec: &BlockModifySpec, tools: &mut HashMap<i32, ToolSpec>)
{
    let numeric: i32 = block.m_blockID;
    let mut block: Rc<Block> = block;

    if let Some(hardness) = spec.hardness
    {
        block = block.setHardness(hardness);
    }

    if let Some(resistance) = spec.resistance
    {
        block = block.setResistance(resistance);
    }

    if let Some(light) = spec.light
    {
        block = block.setLightValue(light);
    }

    if let Some(opacity) = spec.opacity
    {
        block = block.setLightOpacity(opacity);
    }

    if let Some(slipperiness) = spec.slipperiness
    {
        block.m_slipperiness.set(slipperiness);
    }

    if let Some(flammability) = spec.flammability
    {
        game::block::block_fire::setFlammable(numeric, flammability.encouragement, flammability.flammability);
    }

    if let Some(tool) = &spec.tool
    {
        tools.insert(numeric, tool.clone());
    }
}

// runs after every content build, so modded targets rebuilt at new ids get their changes again
pub fn apply()
{
    STATE.with(|state|
    {
        let mut state: RefMut<crate::modify::State> = state.borrow_mut();
        let state: &mut State = &mut state;
        state.m_tools.clear();
        state.m_conflicts.clear();
        revert(&mut state.m_originals);

        let mut writers: BTreeMap<(Identifier, &str), String> = BTreeMap::new();

        for (owner, spec) in &state.m_blocks
        {
            let Some(block) =
                crate::registry::with(|registries| registries.blockId(&spec.target)).and_then(Block::byId)
            else
            {
                log::warn(owner, &format!("modify names unknown block {}", spec.target));
                continue;
            };

            for field in FIELDS.iter().filter(|row| (row.m_present)(spec)).map(|row| row.m_name)
            {
                if let Some(previous) = writers.insert((spec.target.clone(), field), owner.clone())
                    && previous != *owner
                {
                    state.m_conflicts.push(format!("{} {field}: {previous} then {owner}, {owner} wins", spec.target));
                }
            }

            // mod blocks are built afresh on every content build, only vanilla ones keep a changed value
            if spec.target.isVanilla()
            {
                remember(&mut state.m_originals, true, block.m_blockID, || blockOriginal(&block));
            }

            applyBlock(block, spec, &mut state.m_tools);
        }

        for (owner, spec) in &state.m_items
        {
            let Some(item) = crate::registry::with(|registries| registries.itemId(&spec.target)).and_then(Item::byId)
            else
            {
                log::warn(owner, &format!("modify names unknown item {}", spec.target));
                continue;
            };

            let fields: [(&str, bool); 2] =
                [("max_stack", spec.max_stack.is_some()), ("max_damage", spec.max_damage.is_some())];
            for (field, _) in fields.iter().filter(|(_, present)| *present)
            {
                if let Some(previous) = writers.insert((spec.target.clone(), field), owner.clone())
                    && previous != *owner
                {
                    state.m_conflicts.push(format!("{} {field}: {previous} then {owner}, {owner} wins", spec.target));
                }
            }

            if spec.target.isVanilla()
            {
                let original: Original =
                    Original::Item { stack: item.m_maxStackSize.get(), damage: item.getMaxDamage() };
                remember(&mut state.m_originals, false, item.m_shiftedIndex, || original);
            }

            applyItem(item, spec);
        }

        for conflict in &state.m_conflicts
        {
            log::warn(log::LOADER, &format!("modify conflict {conflict}"));
        }
    });
}

pub fn tool(block: i32) -> Option<ToolSpec> { STATE.with(|state| state.borrow().m_tools.get(&block).cloned()) }

pub fn conflicts() -> Vec<String> { STATE.with(|state| state.borrow().m_conflicts.clone()) }
