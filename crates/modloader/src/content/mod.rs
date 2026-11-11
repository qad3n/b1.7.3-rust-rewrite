pub mod blocks;
pub mod data;
pub mod effects;
pub mod fluids;
pub mod harvest;
pub mod items;
pub mod lang;
pub mod loot;
pub mod machines;
pub mod particles;
pub mod progress;
pub mod recipes;
pub mod tags;
pub mod templates;
pub mod worldgen;

use crate::atlas::Atlas;
use mod_protocol::Identifier;
use mod_protocol::spec::{BlockSpec, ItemSpec, ItemToolSpec, ToolSpec};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct ResolvedDrop
{
    pub m_item: i32,
    pub m_min: i32,
    pub m_max: i32,
    pub m_meta: i32,
    pub m_chance: f32,
}

pub struct BlockData
{
    pub m_id: Identifier,
    pub m_owner: usize,
    pub m_spec: BlockSpec,
    // bottom, top, then the four sides, the front face is side index 3
    pub m_faces: [i32; 6],
    // top head, top foot, side head, side foot, end head, end foot and bottom sprites of a bed
    pub m_bed: Option<[i32; 7]>,
    pub m_drops: Vec<ResolvedDrop>,
    pub m_tool: Option<ToolSpec>,
    // faces of every named state, indexed by state, for blocks whose states change their textures
    pub m_stateFaces: Option<Vec<[i32; 6]>>,
    pub m_page: usize,
}

pub struct ItemData
{
    pub m_id: Identifier,
    pub m_owner: usize,
    pub m_spec: ItemSpec,
    pub m_tool: Option<ItemToolSpec>,
}

#[derive(Default)]
struct Tables
{
    m_blocks: HashMap<i32, Rc<BlockData>>,
    m_items: HashMap<i32, Rc<ItemData>>,
    m_fuel: HashMap<i32, i32>,
}

thread_local!
{
    static TABLES: RefCell<Tables> = RefCell::new(Tables::default());
}

pub fn blockData(id: i32) -> Option<Rc<BlockData>> { TABLES.with(|tables| tables.borrow().m_blocks.get(&id).cloned()) }

pub fn itemData(id: i32) -> Option<Rc<ItemData>> { TABLES.with(|tables| tables.borrow().m_items.get(&id).cloned()) }

// the now playing line of a mod disc, by its streaming sound
pub fn recordTitle(sound: &str) -> Option<String>
{
    TABLES.with(|tables|
    {
        tables.borrow().m_items.values().find_map(|data|
        {
            data.m_spec.record.as_ref().filter(|record| record.sound == sound).map(|record| record.title.clone())
        })
    })
}

pub fn fuel(id: i32) -> Option<i32> { TABLES.with(|tables| tables.borrow().m_fuel.get(&id).copied()) }

pub fn hasBlockTools() -> bool
{
    TABLES.with(|tables| tables.borrow().m_blocks.values().any(|data| data.m_tool.is_some()))
}

pub(crate) fn putBlock(id: i32, data: BlockData)
{
    TABLES.with(|tables| tables.borrow_mut().m_blocks.insert(id, Rc::new(data)));
}

pub(crate) fn putItem(id: i32, data: ItemData)
{
    TABLES.with(|tables| tables.borrow_mut().m_items.insert(id, Rc::new(data)));
}

pub(crate) fn putFuel(id: i32, ticks: i32) { TABLES.with(|tables| tables.borrow_mut().m_fuel.insert(id, ticks)); }

// a registered texture path such as ruby:block/ruby_ore, or a vanilla one as minecraft:terrain/50 or minecraft:block/diamond_ore
pub fn textureSlot(texture: &str, atlas: Atlas) -> Option<i32>
{
    let id: Identifier = Identifier::parse(texture).ok()?;
    if id.isVanilla()
    {
        return vanillaSlot(atlas, id.path());
    }

    crate::atlas::slotOf(atlas, &id)
}

// a beta sheet slot by number or by its modern sprite name
pub(crate) fn vanillaSlot(atlas: Atlas, path: &str) -> Option<i32>
{
    let (sheet, name) = path.split_once('/')?;

    match (atlas, sheet)
    {
        (Atlas::Terrain, "terrain") | (Atlas::Items, "items") =>
        {
            name.parse().ok().filter(|slot| (0..crate::atlas::VANILLA_TILES).contains(slot))
        }
        (Atlas::Terrain, "block") => game::resources::terrainSlot(name),
        (Atlas::Items, "item") => game::resources::itemSlot(name),
        _ => None,
    }
}

// a vanilla texture that names no slot fails the spec, a mod texture is checked when the atlas reads it
pub(crate) fn checkTexture(texture: &str, atlas: Atlas) -> Result<(), String>
{
    let id: Identifier = Identifier::parse(texture).map_err(|error| format!("texture {texture}: {error:?}"))?;
    if !id.isVanilla() || vanillaSlot(atlas, id.path()).is_some()
    {
        return Ok(());
    }

    let (number, named): (&str, &str) = match atlas
    {
        Atlas::Terrain => ("terrain", "block"),
        Atlas::Items => ("items", "item"),
    };
    Err(format!("unknown vanilla texture {texture}, use minecraft:{number}/<slot> or minecraft:{named}/<sprite name>"))
}

// drops every numeric keyed table before content is rebuilt under a new id map
pub fn reset()
{
    TABLES.with(|tables| *tables.borrow_mut() = Tables::default());
    crate::states::clearBindings();
    worldgen::clear();
    recipes::removeModded();
}

// mod armor textures are <texture>_1.png and <texture>_2.png, like the vanilla armor sheets
// white for vanilla armor and untinted mod armor
pub fn armorColor(item: i32) -> u32
{
    itemData(item).and_then(|data| data.m_spec.armor.as_ref().and_then(|armor| armor.color)).unwrap_or(0xFFFFFF)
}

pub fn armorTexture(item: i32, layer: i32) -> Option<String>
{
    let data: Rc<ItemData> = itemData(item)?;
    let texture: &str = &data.m_spec.armor.as_ref()?.texture;
    crate::entities::modTexturePath(&format!("{texture}_{}", if layer == 2 { 2 } else { 1 }))
}
