use super::{ItemData, templates};
use crate::atlas::Atlas;
use crate::registry::{ItemEntry, Row};
use game::block::Block;
use game::item::item_sword::SwordExt;
use game::item::item_tool::ToolExt;
use game::item::{EnumToolMaterial, Item, ItemClass, ItemStack};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::{ItemSpec, ItemToolSpec, ToolClass};
use std::rc::Rc;

const DEFAULT_ICON: i32 = 0;

fn data(this: &Item) -> Rc<ItemData>
{
    super::itemData(this.m_shiftedIndex).expect("a data driven item class only runs for registered mod items")
}

// vanilla tools report their class and level from their class name and tier
pub fn toolOf(item: &Rc<Item>) -> Option<(ToolClass, i32)>
{
    if let Some(data) = super::itemData(item.m_shiftedIndex)
    {
        return data.m_tool.as_ref().map(|tool| (tool.class, tool.level));
    }
    let class: ToolClass = match item.className()
    {
        "ItemPickaxe" => ToolClass::Pickaxe,
        "ItemSpade" => ToolClass::Shovel,
        "ItemAxe" => ToolClass::Axe,
        "ItemSword" => ToolClass::Sword,
        "ItemShears" => return Some((ToolClass::Shears, 0)),
        _ => return None,
    };
    Some((class, materialOf(item)?.getHarvestLevel()))
}

// swords are not ItemTool in vanilla so their tier sits in their own ext
pub fn materialOf(item: &Item) -> Option<EnumToolMaterial>
{
    if let Some(tool) = item.tryExt::<ToolExt>()
    {
        return Some(tool.m_toolMaterial);
    }
    item.tryExt::<SwordExt>().map(|sword| sword.m_toolMaterial)
}

fn template(this: &Item) -> &'static ItemClass
{
    this.m_class.m_parent.expect("a derived item class keeps its template")
}

fn iconColor(this: &Rc<Item>, _metadata: i32) -> i32 { data(this).m_spec.color.map_or(0xFFFFFF, |color| color as i32) }

// blocks without a tool component get the vanilla answer of the template tool at the same tier
fn toolSpeed(this: &Rc<Item>, stack: &Rc<ItemStack>, block: &Rc<Block>) -> f32
{
    let data: Rc<ItemData> = data(this);
    match (&data.m_tool, super::blockData(block.m_blockID).and_then(|block| block.m_tool.clone()))
    {
        (Some(tool), Some(wanted)) =>
        {
            if wanted.class == tool.class
            {
                tool.speed
            }
            else
            {
                1.0
            }
        }
        _ => (template(this).getStrVsBlock)(this, stack, block),
    }
}

fn toolHarvest(this: &Rc<Item>, block: &Rc<Block>) -> bool
{
    let data: Rc<ItemData> = data(this);
    match (&data.m_tool, super::blockData(block.m_blockID).and_then(|block| block.m_tool.clone()))
    {
        (Some(tool), Some(wanted)) => wanted.class == tool.class && tool.level >= wanted.level,
        _ => (template(this).canHarvestBlock)(this, block),
    }
}

fn toolDamage(this: &Rc<Item>, _target: Option<&Rc<game::entity::Entity>>) -> i32
{
    data(this).m_tool.as_ref().map_or(1, |tool| tool.damage)
}

fn patch(class: &mut ItemClass, spec: &ItemSpec)
{
    if spec.color.is_some()
    {
        class.getColorFromDamage = iconColor;
    }
    if spec.tool.is_some()
    {
        class.getStrVsBlock = toolSpeed;
        class.canHarvestBlock = toolHarvest;
        class.getDamageVsEntity = toolDamage;
    }
    if let Some(behavior) = &spec.behavior
    {
        crate::behavior::patchItem(class, behavior);
    }
}

pub fn validate(spec: &ItemSpec) -> Result<(), String>
{
    let name: String = super::blocks::templateName(spec.template.as_ref(), templates::defaultItemTemplate(spec))?;
    if templates::itemTemplate(&name).is_none()
    {
        let known: Vec<&str> = templates::ITEM_TEMPLATES.iter().map(|row| row.m_name).collect();
        return Err(format!("unknown item template minecraft:{name}, known: {}", known.join(", ")));
    }
    Ok(())
}

pub fn requestTextures(spec: &ItemSpec)
{
    if let Some(texture) = &spec.texture
    {
        crate::atlas::request(Atlas::Items, texture);
    }
}

fn applyCommon(mut item: Rc<Item>, spec: &ItemSpec, id: &Identifier) -> Result<Rc<Item>, String>
{
    if let Some(texture) = &spec.texture
    {
        item = item.setIconIndex(super::textureSlot(texture, Atlas::Items).unwrap_or(DEFAULT_ICON));
    }
    if let Some(size) = spec.max_stack
    {
        item = item.setMaxStackSize(size);
    }
    let durability: Option<i32> =
        spec.max_damage.or(spec.tool.as_ref().map(|tool| tool.durability).filter(|value| *value > 0));
    if let Some(damage) = durability.or(spec.armor.as_ref().map(|armor| armor.durability))
    {
        item = item.setMaxDamage(damage);
    }
    if spec.full3d == Some(true)
    {
        item = item.setFull3D();
    }
    let key: String = format!("{}.{}", id.namespace(), id.path().replace('/', "."));
    item = item.setItemName(&key);
    if let Some(name) = &spec.name
    {
        super::lang::addDefault(&format!("item.{key}.name"), name);
    }
    if let Some(container) = &spec.container_item
    {
        let numeric: i32 = crate::registry::with(|registries| registries.itemId(container))
            .ok_or_else(|| format!("container item {container} is not registered"))?;
        item = item.setContainerItem(Item::byId(numeric));
    }
    Ok(item)
}

pub fn build(row: &Row<ItemEntry>, owner: usize) -> Result<(), String>
{
    if row.m_value.m_block.is_some()
    {
        return Ok(());
    }
    let spec: ItemSpec = row.m_value.m_spec.clone().ok_or("a modded item row has a spec")?;
    let templateId: String =
        super::blocks::templateName(spec.template.as_ref(), templates::defaultItemTemplate(&spec))?;
    let template: &templates::ItemTemplate =
        templates::itemTemplate(&templateId).ok_or_else(|| format!("unknown item template {templateId}"))?;
    let tool: Option<ItemToolSpec> = spec.tool.clone();
    super::putItem(
        row.m_numeric,
        ItemData { m_id: row.m_id.clone(), m_owner: owner, m_spec: spec.clone(), m_tool: tool },
    );
    let template_class: &'static ItemClass = templateClass(template, &spec);
    let class: &'static ItemClass =
        crate::classes::deriveItemClass(template_class, &row.m_id.to_string(), |class| patch(class, &spec));
    let flowing: Option<i32> = super::fluids::bucketFlowing(&row.m_id);
    let item: Rc<Item> = game::hooks::withItemClass(class, || match flowing
    {
        Some(flowing) => game::item::item_bucket::create(row.m_numeric - 256, flowing),
        None => (template.m_create)(row.m_numeric - 256, &spec),
    });
    applyCommon(item, &spec, &row.m_id)?;
    if let Some(ticks) = spec.fuel
    {
        super::putFuel(row.m_numeric, ticks);
    }
    if spec.food.as_ref().is_some_and(|food| food.wolf_food)
    {
        super::tags::flag(RegistryKind::Item, "wolf_food", &row.m_id);
    }
    Ok(())
}

// the class a template item gets, read from one vanilla item built by the same constructor
fn templateClass(template: &templates::ItemTemplate, spec: &ItemSpec) -> &'static ItemClass
{
    let id: Identifier = Identifier::minecraft(template.m_name);
    let _ = spec;
    crate::registry::with(|registries| registries.itemId(&id))
        .and_then(Item::byId)
        .map_or(&game::item::ITEM, |item| item.m_class)
}

// the item form of a mod block shares its numeric id, a spec item block renames, restacks or gives it a behavior
pub fn buildBlockItem(block: &Rc<Block>, spec: Option<&ItemSpec>, owner: usize, id: &Identifier) -> Result<(), String>
{
    let numeric: i32 = block.m_blockID;
    if Item::byId(numeric).is_some()
    {
        return Ok(());
    }
    let class: &'static ItemClass = match spec.filter(|spec| spec.behavior.is_some())
    {
        Some(spec) =>
        {
            crate::classes::deriveItemClass(&game::item::item_block::CLASS, &id.to_string(), |class| patch(class, spec))
        }
        None => &game::item::item_block::CLASS,
    };
    let item: Rc<Item> = game::item::item_block::construct(class, numeric - 256);
    if let Some(spec) = spec
    {
        super::putItem(numeric, ItemData { m_id: id.clone(), m_owner: owner, m_spec: spec.clone(), m_tool: None });
        if let Some(size) = spec.max_stack
        {
            item.setMaxStackSize(size);
        }
        if let Some(ticks) = spec.fuel
        {
            super::putFuel(numeric, ticks);
        }
    }
    Ok(())
}
