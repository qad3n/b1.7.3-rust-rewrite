use game::block::block_pressure_plate::EnumMobType;
use game::block::{self as blocks, Block, BlockClass, Material};
use game::item::{self as items, EnumToolMaterial, Item};
use mod_protocol::Identifier;
use mod_protocol::spec::{ArmorSlot, FoodSpec, ItemSpec, ToolClass};
use std::rc::Rc;

// each template reuses a vanilla constructor so the block gets the right ext and setup, the material comes from the spec
pub struct BlockTemplate
{
    pub m_name: &'static str,
    pub m_create: fn(id: i32, sprite: i32, material: Rc<Material>) -> Rc<Block>,
}

// the vanilla class a template block gets, read from a block built by the same constructor
pub fn templateClass(template: &BlockTemplate) -> &'static BlockClass
{
    match template.m_name
    {
        "block" | "cobblestone" | "planks" | "iron_block" => &game::block::BLOCK,
        name => vanillaBlock(name).map_or(&game::block::BLOCK, |block| block.m_class),
    }
}

fn vanillaBlock(name: &str) -> Option<Rc<Block>>
{
    let id: Identifier = Identifier::minecraft(name);
    crate::registry::with(|registries| registries.blockId(&id)).and_then(Block::byId)
}

pub const BLOCK_TEMPLATES: [BlockTemplate; 29] = [
    BlockTemplate { m_name: "block", m_create: |id, sprite, material| Block::basic(id, sprite, material) },
    BlockTemplate { m_name: "stone", m_create: |id, sprite, _| blocks::block_stone::create(id, sprite) },
    BlockTemplate { m_name: "cobblestone", m_create: |id, sprite, _| Block::basic(id, sprite, Material::rock()) },
    BlockTemplate { m_name: "planks", m_create: |id, sprite, _| Block::basic(id, sprite, Material::wood()) },
    BlockTemplate { m_name: "iron_block", m_create: |id, sprite, _| Block::basic(id, sprite, Material::iron()) },
    BlockTemplate { m_name: "diamond_ore", m_create: |id, sprite, _| blocks::block_ore::create(id, sprite) },
    BlockTemplate { m_name: "dirt", m_create: |id, sprite, _| blocks::block_dirt::create(id, sprite) },
    BlockTemplate { m_name: "sand", m_create: |id, sprite, _| blocks::block_sand::create(id, sprite) },
    BlockTemplate { m_name: "gravel", m_create: |id, sprite, _| blocks::block_gravel::create(id, sprite) },
    BlockTemplate {
        m_name: "glass",
        m_create: |id, sprite, _| blocks::block_glass::create(id, sprite, Material::glass(), false),
    },
    BlockTemplate {
        m_name: "glowstone",
        m_create: |id, sprite, material| blocks::block_glow_stone::create(id, sprite, material),
    },
    BlockTemplate { m_name: "netherrack", m_create: |id, sprite, _| blocks::block_netherrack::create(id, sprite) },
    BlockTemplate { m_name: "obsidian", m_create: |id, sprite, _| blocks::block_obsidian::create(id, sprite) },
    BlockTemplate { m_name: "bookshelf", m_create: |id, sprite, _| blocks::block_bookshelf::create(id, sprite) },
    BlockTemplate { m_name: "ice", m_create: |id, sprite, _| blocks::block_ice::create(id, sprite) },
    BlockTemplate { m_name: "web", m_create: |id, sprite, _| blocks::block_web::create(id, sprite) },
    BlockTemplate { m_name: "torch", m_create: |id, sprite, _| blocks::block_torch::create(id, sprite) },
    BlockTemplate { m_name: "red_flower", m_create: |id, sprite, _| blocks::block_flower::create(id, sprite) },
    BlockTemplate { m_name: "brown_mushroom", m_create: |id, sprite, _| blocks::block_mushroom::create(id, sprite) },
    BlockTemplate { m_name: "ladder", m_create: |id, sprite, _| blocks::block_ladder::create(id, sprite) },
    BlockTemplate { m_name: "fence", m_create: |id, sprite, _| blocks::block_fence::create(id, sprite) },
    BlockTemplate { m_name: "leaves", m_create: |id, sprite, _| blocks::block_leaves::create(id, sprite) },
    BlockTemplate { m_name: "cactus", m_create: |id, sprite, _| blocks::block_cactus::create(id, sprite) },
    BlockTemplate { m_name: "stone_button", m_create: |id, sprite, _| blocks::block_button::create(id, sprite) },
    BlockTemplate { m_name: "lever", m_create: |id, sprite, _| blocks::block_lever::create(id, sprite) },
    BlockTemplate {
        m_name: "stone_pressure_plate",
        m_create: |id, sprite, _| blocks::block_pressure_plate::create(id, sprite, EnumMobType::Mobs, Material::rock()),
    },
    BlockTemplate { m_name: "tnt", m_create: |id, sprite, _| blocks::block_tnt::create(id, sprite) },
    BlockTemplate { m_name: "portal", m_create: |id, sprite, _| blocks::block_portal::create(id, sprite) },
    BlockTemplate { m_name: "bed", m_create: |id, _, _| blocks::block_bed::create(id) },
];

pub struct ItemTemplate
{
    pub m_name: &'static str,
    // takes the constructor id, numeric id minus 256
    pub m_create: fn(id: i32, spec: &ItemSpec) -> Rc<Item>,
}

fn tier(spec: &ItemSpec) -> EnumToolMaterial
{
    match spec.tool.as_ref().map_or(2, |tool| tool.level)
    {
        0 => EnumToolMaterial::Wood,
        1 => EnumToolMaterial::Stone,
        2 => EnumToolMaterial::Iron,
        _ => EnumToolMaterial::Emerald,
    }
}

fn armorSlot(spec: &ItemSpec) -> i32
{
    match spec.armor.as_ref().map(|armor| armor.slot)
    {
        Some(ArmorSlot::Chest) => 1,
        Some(ArmorSlot::Legs) => 2,
        Some(ArmorSlot::Feet) => 3,
        _ => 0,
    }
}

pub const ITEM_TEMPLATES: [ItemTemplate; 10] = [
    ItemTemplate { m_name: "stick", m_create: |id, _| Item::basic(id) },
    ItemTemplate {
        m_name: "apple",
        m_create: |id, spec| {
            let food: Option<&FoodSpec> = spec.food.as_ref();
            items::item_food::create(id, food.map_or(4, |food| food.heal), food.is_some_and(|food| food.wolf_food))
        },
    },
    ItemTemplate { m_name: "iron_pickaxe", m_create: |id, spec| items::item_pickaxe::create(id, tier(spec)) },
    ItemTemplate { m_name: "iron_shovel", m_create: |id, spec| items::item_spade::create(id, tier(spec)) },
    ItemTemplate { m_name: "iron_axe", m_create: |id, spec| items::item_axe::create(id, tier(spec)) },
    ItemTemplate { m_name: "iron_sword", m_create: |id, spec| items::item_sword::create(id, tier(spec)) },
    ItemTemplate { m_name: "iron_hoe", m_create: |id, spec| items::item_hoe::create(id, tier(spec)) },
    ItemTemplate { m_name: "snowball", m_create: |id, _| items::item_snowball::create(id) },
    ItemTemplate { m_name: "shears", m_create: |id, _| items::item_shears::create(id) },
    ItemTemplate { m_name: "iron_helmet", m_create: |id, spec| items::item_armor::create(id, 2, 2, armorSlot(spec)) },
];

// the template a spec gets when it names none
pub fn defaultItemTemplate(spec: &ItemSpec) -> &'static str
{
    if let Some(tool) = &spec.tool
    {
        return match tool.class
        {
            ToolClass::Pickaxe => "iron_pickaxe",
            ToolClass::Shovel => "iron_shovel",
            ToolClass::Axe => "iron_axe",
            ToolClass::Sword => "iron_sword",
            ToolClass::Hoe => "iron_hoe",
            ToolClass::Shears => "shears",
        };
    }
    if spec.food.is_some()
    {
        return "apple";
    }
    if spec.armor.is_some()
    {
        return "iron_helmet";
    }
    "stick"
}

pub const DEFAULT_BLOCK_TEMPLATE: &str = "block";

pub fn blockTemplate(name: &str) -> Option<&'static BlockTemplate>
{
    BLOCK_TEMPLATES.iter().find(|row| row.m_name == name)
}

pub fn itemTemplate(name: &str) -> Option<&'static ItemTemplate>
{
    ITEM_TEMPLATES.iter().find(|row| row.m_name == name)
}

pub struct MaterialRow
{
    pub m_name: &'static str,
    pub m_get: fn() -> Rc<Material>,
}

pub const MATERIALS: [MaterialRow; 14] = [
    MaterialRow { m_name: "rock", m_get: Material::rock },
    MaterialRow { m_name: "wood", m_get: Material::wood },
    MaterialRow { m_name: "iron", m_get: Material::iron },
    MaterialRow { m_name: "ground", m_get: Material::ground },
    MaterialRow { m_name: "grass", m_get: Material::grassMaterial },
    MaterialRow { m_name: "sand", m_get: Material::sand },
    MaterialRow { m_name: "glass", m_get: Material::glass },
    MaterialRow { m_name: "cloth", m_get: Material::cloth },
    MaterialRow { m_name: "leaves", m_get: Material::leaves },
    MaterialRow { m_name: "plants", m_get: Material::plants },
    MaterialRow { m_name: "circuits", m_get: Material::circuits },
    MaterialRow { m_name: "ice", m_get: Material::ice },
    MaterialRow { m_name: "snow", m_get: Material::builtSnow },
    MaterialRow { m_name: "tnt", m_get: Material::tnt },
];

pub fn material(name: &str) -> Option<Rc<Material>>
{
    MATERIALS.iter().find(|row| row.m_name == name).map(|row| (row.m_get)())
}
