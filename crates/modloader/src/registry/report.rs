use super::{BlockEntry, EntityEntry, FuelEntry, ItemEntry, RecipeEntry, Registries, Registry, Row, TileEntityEntry};
use crate::log;
use mod_protocol::spec::{CreatureKind, DropSpec, RecipeSpec, SpawnSpec, StackSpec};
use std::fmt::Display;

// one debug line per entry a mod added or replaced so a modpack log shows exactly what each mod brought
pub fn report(registries: &Registries, owner: &str)
{
    section(&registries.m_blocks, owner, block);
    section(&registries.m_items, owner, item);
    section(&registries.m_entities, owner, entity);
    section(&registries.m_tileEntities, owner, tileEntity);
    section(&registries.m_recipes, owner, recipe);
    section(&registries.m_smelting, owner, recipe);
    section(&registries.m_fuel, owner, fuel);
    section(&registries.m_dimensions, owner, |_| String::new());
    section(&registries.m_biomes, owner, |_| String::new());
}

fn section<T>(registry: &Registry<T>, owner: &str, describe: fn(&T) -> String)
{
    let rows: Vec<&Row<T>> =
        registry.rows().filter(|row| row.m_owner == owner || row.m_replacedBy.as_deref() == Some(owner)).collect();
    if rows.is_empty()
    {
        return;
    }

    log::debug(owner, &format!("{} {} entries", rows.len(), registry.name()));
    for row in rows
    {
        let replaced: &str = if row.m_owner == owner
        {
            ""
        }
        else
        {
            " replaces vanilla"
        };
        log::debug(
            owner,
            &format!("  {} {} #{}{replaced}{}", registry.name(), row.m_id, row.m_numeric, describe(&row.m_value)),
        );
    }
}

fn part<T: Display>(label: &str, value: Option<T>) -> String
{
    value.map(|value| format!(" {label} {value}")).unwrap_or_default()
}

fn flag(label: &str, set: bool) -> &str { if set { label } else { "" } }

fn stack(spec: &StackSpec) -> String
{
    let meta: String = if spec.meta == 0
    {
        String::new()
    }
    else
    {
        format!(":{}", spec.meta)
    };
    format!("{}x {}{meta}", spec.count, spec.item)
}

fn drops(specs: Option<&Vec<DropSpec>>) -> String
{
    let items: Option<String> =
        specs.map(|specs| specs.iter().map(|spec| spec.item.to_string()).collect::<Vec<String>>().join(", "));
    part("drops", items)
}

fn block(entry: &BlockEntry) -> String
{
    if let Some((block, page)) = &entry.m_page
    {
        return format!(" state page {page} of {block}");
    }

    let Some(spec) = &entry.m_spec
    else
    {
        return String::new();
    };

    [
        part("name", spec.name.as_ref()),
        part("template", spec.template.as_ref()),
        part("material", spec.material.as_ref()),
        part("hardness", spec.hardness),
        part("resistance", spec.resistance),
        part("light", spec.light),
        part("tile entity", spec.tile_entity.as_ref()),
        part("fuel", spec.fuel),
        drops(spec.drops.as_ref()),
    ]
    .concat()
}

fn item(entry: &ItemEntry) -> String
{
    let form: String = part("block item of", entry.m_block.as_ref());
    let Some(spec) = &entry.m_spec
    else
    {
        return form;
    };

    [
        form,
        part("name", spec.name.as_ref()),
        part("template", spec.template.as_ref()),
        part("stack", spec.max_stack),
        part("durability", spec.max_damage),
        part("places", spec.places.as_ref()),
        part("fuel", spec.fuel),
        flag(" food", spec.food.is_some()).to_owned(),
        flag(" tool", spec.tool.is_some()).to_owned(),
        flag(" armor", spec.armor.is_some()).to_owned(),
        flag(" record", spec.record.is_some()).to_owned(),
    ]
    .concat()
}

fn creature(kind: CreatureKind) -> &'static str
{
    match kind
    {
        CreatureKind::Monster => "monster",
        CreatureKind::Creature => "creature",
        CreatureKind::Water => "water",
    }
}

fn spawn(spec: &SpawnSpec) -> String
{
    let biomes: String = if spec.biomes.is_empty()
    {
        String::from("all overworld biomes")
    }
    else
    {
        spec.biomes.iter().map(ToString::to_string).collect::<Vec<String>>().join(", ")
    };
    format!(" spawns as {} weight {} in {biomes}", creature(spec.kind), spec.weight)
}

fn entity(entry: &EntityEntry) -> String
{
    let key: String = format!(" save key {}", entry.m_saveKey);
    let Some(spec) = &entry.m_spec
    else
    {
        return key;
    };

    let size: Option<String> = spec.size.map(|[width, height]| format!("{width}x{height}"));
    [
        key,
        part("name", spec.name.as_ref()),
        part("template", spec.template.as_ref()),
        part("health", spec.health),
        part("speed", spec.speed),
        part("size", size),
        spec.spawn.as_ref().map(spawn).unwrap_or_default(),
        drops(spec.drops.as_ref()),
        flag(" fire immune", spec.fire_immune).to_owned(),
        flag(" persistent", spec.persistent).to_owned(),
        flag(" immovable", spec.immovable).to_owned(),
        flag(" scripted", spec.behavior.is_some()).to_owned(),
    ]
    .concat()
}

fn tileEntity(entry: &TileEntityEntry) -> String
{
    let key: String = format!(" save id {}", entry.m_saveId);
    let Some(spec) = &entry.m_spec
    else
    {
        return key;
    };

    [
        key,
        format!(" inventory {}", spec.inventory),
        flag(" ticks", spec.ticks).to_owned(),
        flag(" synced", spec.sync).to_owned(),
    ]
    .concat()
}

fn recipe(entry: &RecipeEntry) -> String
{
    match &entry.m_spec
    {
        Some(RecipeSpec::Shaped(spec)) =>
        {
            let key: Vec<String> =
                spec.key.iter().map(|(symbol, input)| format!("{symbol}={}", stack(input))).collect();
            format!(" shaped [{}] {} -> {}", spec.pattern.join("|"), key.join(", "), stack(&spec.result))
        }

        Some(RecipeSpec::Shapeless(spec)) =>
        {
            let inputs: Vec<String> = spec.ingredients.iter().map(stack).collect();
            format!(" shapeless {} -> {}", inputs.join(" + "), stack(&spec.result))
        }

        Some(RecipeSpec::Smelting(spec)) => format!(" smelts {} -> {}", spec.input, stack(&spec.result)),
        Some(RecipeSpec::Fuel(spec)) => format!(" fuel {} burns {} ticks", spec.item, spec.ticks),
        None => String::new(),
    }
}

fn fuel(entry: &FuelEntry) -> String { format!(" burns {} ticks", entry.m_ticks) }
