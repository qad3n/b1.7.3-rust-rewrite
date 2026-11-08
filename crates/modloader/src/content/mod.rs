pub mod blocks;
pub mod data;
pub mod harvest;
pub mod items;
pub mod lang;
pub mod recipes;
pub mod tags;
pub mod templates;
pub mod worldgen;

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
    pub m_drops: Vec<ResolvedDrop>,
    pub m_tool: Option<ToolSpec>,
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

thread_local! {
    static TABLES: RefCell<Tables> = RefCell::new(Tables::default());
}

pub fn blockData(id: i32) -> Option<Rc<BlockData>> { TABLES.with(|tables| tables.borrow().m_blocks.get(&id).cloned()) }

pub fn itemData(id: i32) -> Option<Rc<ItemData>> { TABLES.with(|tables| tables.borrow().m_items.get(&id).cloned()) }

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

// a registered texture path such as ruby:block/ruby_ore, or a vanilla slot as minecraft:terrain/50
pub fn textureSlot(texture: &str, atlas: crate::atlas::Atlas) -> Option<i32>
{
    let id: Identifier = Identifier::parse(texture).ok()?;
    if id.isVanilla()
    {
        let (sheet, slot) = id.path().split_once('/')?;
        let expected: &str = match atlas
        {
            crate::atlas::Atlas::Terrain => "terrain",
            crate::atlas::Atlas::Items => "items",
        };
        return if sheet == expected
        {
            slot.parse().ok()
        }
        else
        {
            None
        };
    }
    crate::atlas::slotOf(atlas, &id)
}

// drops every numeric keyed table before content is rebuilt under a new id map
pub fn reset()
{
    TABLES.with(|tables| *tables.borrow_mut() = Tables::default());
    worldgen::clear();
    recipes::removeModded();
}

// mod armor textures are <texture>_1.png and <texture>_2.png, like the vanilla armor sheets
pub fn armorTexture(item: i32, layer: i32) -> Option<String>
{
    let data: Rc<ItemData> = itemData(item)?;
    let texture: &str = &data.m_spec.armor.as_ref()?.texture;
    crate::entities::modTexturePath(&format!("{texture}_{}", if layer == 2 { 2 } else { 1 }))
}
