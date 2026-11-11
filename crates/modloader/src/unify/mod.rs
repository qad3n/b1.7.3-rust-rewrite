mod infer;

use crate::content::tags;
use crate::host::context;
use crate::{log, registry, state};
use game::entity::Entity;
use game::inventory::IInventory;
use game::item::ItemStack;
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::identifier::MINECRAFT;
use mod_protocol::spec::{IngredientSpec, RecipeSpec, StackSpec, WorldgenKind, WorldgenSpec};
use serde::Deserialize;
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::rc::Rc;

// one material many mods add, the members of a shared c: tag, and the member everything turns into
const FILE: &str = "unification.toml";
const COMMON: &str = "c";
const UNIFY: &str = "unify";

// the settings file written on first run, every key optional
const DEFAULTS: &str = r#"# merges the same material added by several mods into one item, as OreDictionaryConverter does for Forge
# a group is a shared tag in the c namespace whose first path part is listed in prefixes and whose members come
# from two or more mods; every member turns into the group's preferred member in recipes, drops, pickups, machine
# output and player inventories
enabled = true
prefixes = ["ores", "ingots", "nuggets", "dusts", "gems", "plates", "gears", "rods", "storage_blocks", "raw_materials"]
# mods whose members win a group, first wins; vanilla minecraft members always win; among unlisted mods the one
# adding the most kinds of a material (ore, ingot, nugget, block...) wins all of it, then load order
mod_priority = []
# only one ore vein feature per group and dimension generates, with the preferred block
dedupe_ore_worldgen = true
# the loader also reads materials from names such as copper_ingot, ore_tin or golden_gear, for mods that tag nothing
infer_from_names = true

# a group's winner by hand, the tag on the left and the member on the right
[preferred]
# "c:ingots/copper" = "somemod:copper_ingot"
"#;

#[derive(Deserialize, Clone)]
#[serde(default)]
struct Settings
{
    enabled: bool,
    prefixes: Vec<String>,
    mod_priority: Vec<String>,
    dedupe_ore_worldgen: bool,
    infer_from_names: bool,
    preferred: BTreeMap<String, String>,
}

// the kinds of shared tag that merge, the same list DEFAULTS writes
const PREFIXES: [&str; 10] =
    ["ores", "ingots", "nuggets", "dusts", "gems", "plates", "gears", "rods", "storage_blocks", "raw_materials"];

// built by hand, the container level serde default calls this for every parse
impl Default for Settings
{
    fn default() -> Self
    {
        Settings
        {
            enabled: true,
            prefixes: PREFIXES.map(str::to_owned).to_vec(),
            mod_priority: Vec::new(),
            dedupe_ore_worldgen: true,
            infer_from_names: true,
            preferred: BTreeMap::new(),
        }
    }
}

struct Group
{
    m_tag: Identifier,
    m_members: Vec<Identifier>,
    m_preferred: Identifier,
}

#[derive(Default)]
struct Table
{
    m_settings: Option<Settings>,
    // a member's preferred id, per registry
    m_items: BTreeMap<Identifier, Identifier>,
    m_blocks: BTreeMap<Identifier, Identifier>,
    // a block's group tag, the key that dedupes ore features
    m_blockGroups: BTreeMap<Identifier, Identifier>,
    // numeric item id to its preferred numeric id, the runtime conversions read this
    m_numeric: HashMap<i32, i32>,
}

thread_local!
{
    static TABLE: RefCell<Table> = RefCell::new(Table::default());
}

fn settings(config: &Path) -> Settings
{
    let path: std::path::PathBuf = config.join(FILE);
    if !path.exists()
    {
        if let Err(error) = std::fs::create_dir_all(config).and_then(|()| std::fs::write(&path, DEFAULTS))
        {
            log::warn(UNIFY, &format!("could not write {}: {error}", path.display()));
        }

        return Settings::default();
    }

    let text: String = match std::fs::read_to_string(&path)
    {
        Ok(text) => text,

        Err(error) =>
        {
            log::warn(UNIFY, &format!("could not read {}, using defaults: {error}", path.display()));
            return Settings::default();
        }
    };

    match toml::from_str(&text)
    {
        Ok(settings) => settings,

        Err(error) =>
        {
            log::warn(UNIFY, &format!("{} is not valid, using defaults: {error}", path.display()));
            Settings::default()
        }
    }
}

// a shared tag's kind and material, c:ingots/copper reads as ingots and copper
fn parts(tag: &Identifier) -> (&str, &str) { tag.path().split_once('/').unwrap_or((tag.path(), "")) }

fn unifiable(settings: &Settings, tag: &Identifier) -> bool
{
    let (kind, _): (&str, &str) = parts(tag);
    tag.namespace() == COMMON && settings.prefixes.iter().any(|allowed| allowed == kind)
}

// how many kinds of each material every mod adds, item and block tags counted together
fn coverage(settings: &Settings) -> BTreeMap<String, BTreeMap<String, usize>>
{
    let mut found: BTreeSet<(String, String, String)> = BTreeSet::new();

    for registry in [RegistryKind::Item, RegistryKind::Block]
    {
        for tag in tags::names(registry).into_iter().filter(|tag| unifiable(settings, tag))
        {
            let (kind, material): (&str, &str) = parts(&tag);

            for member in tags::values(registry, &tag)
            {
                found.insert((material.to_owned(), member.namespace().to_owned(), kind.to_owned()));
            }
        }
    }

    let mut counts: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();

    for (material, namespace, _) in found
    {
        *counts.entry(material).or_default().entry(namespace).or_default() += 1;
    }

    counts
}

// one mod wins a whole material where it can, so ore ingot nugget and block never come from a mix of mods
struct Ranking<'a>
{
    m_settings: &'a Settings,
    m_order: Vec<String>,
    m_coverage: BTreeMap<String, BTreeMap<String, usize>>,
}

impl<'a> Ranking<'a>
{
    fn new(settings: &'a Settings) -> Self
    {
        let order: Vec<String> = state::mods().iter().map(|runtime| runtime.id().to_owned()).collect();
        Self { m_settings: settings, m_order: order, m_coverage: coverage(settings) }
    }

    fn checkPriority(&self)
    {
        let missing = |name: &&String| !self.m_order.contains(name) && *name != MINECRAFT;

        for name in self.m_settings.mod_priority.iter().filter(missing)
        {
            log::warn(UNIFY, &format!("mod_priority lists {name}, which is not loaded"));
        }
    }

    // vanilla wins then the listed mods then the mod adding the most kinds of the material then load order
    fn key(&self, material: &str, id: &Identifier) -> (usize, Reverse<usize>, usize, Identifier)
    {
        let namespace: &str = id.namespace();
        let listed: Option<usize> = self.m_settings.mod_priority.iter().position(|name| name == namespace);
        let unlisted: usize = self.m_settings.mod_priority.len();
        let tier: usize = if namespace == MINECRAFT
        {
            0
        }
        else
        {
            1 + listed.unwrap_or(unlisted)
        };
        let kinds: usize =
            self.m_coverage.get(material).and_then(|mods| mods.get(namespace)).copied().unwrap_or_default();
        let loaded: usize = self.m_order.iter().position(|name| name == namespace).unwrap_or(self.m_order.len());
        (tier, Reverse(kinds), loaded, id.clone())
    }

    // an item group follows its block group so the item placed is the block that generates
    fn preferred(
        &self,
        tag: &Identifier,
        members: &[Identifier],
        blocks: &BTreeMap<Identifier, Identifier>,
    ) -> Identifier
    {
        if let Some(chosen) = self.m_settings.preferred.get(&tag.to_string())
        {
            match Identifier::parse(chosen).ok().filter(|chosen| members.contains(chosen))
            {
                Some(chosen) => return chosen,
                None => log::warn(
                    UNIFY,
                    &format!("preferred {chosen} for {tag} is not one of its members, ranking instead"),
                ),
            }
        }

        if let Some(block) = blocks.get(tag)
            && let Some(item) = members.iter().find(|member| blockOf(member).as_ref() == Some(block))
        {
            return item.clone();
        }

        let (_, material): (&str, &str) = parts(tag);
        members.iter().min_by_key(|member| self.key(material, member)).cloned().expect("a group has members")
    }
}

fn blockOf(item: &Identifier) -> Option<Identifier>
{
    registry::with(|registries| registries.m_items.get(item).and_then(|row| row.m_value.m_block.clone()))
}

// the shared tags of a registry that two or more mods fill, blocks holds each block group's winner by tag
fn groups(ranking: &Ranking, registry: RegistryKind, blocks: &BTreeMap<Identifier, Identifier>) -> Vec<Group>
{
    let mut found: Vec<Group> = Vec::new();

    for tag in tags::names(registry).into_iter().filter(|tag| unifiable(ranking.m_settings, tag))
    {
        let members: Vec<Identifier> = tags::values(registry, &tag);
        let owners: BTreeSet<&str> = members.iter().map(Identifier::namespace).collect();
        if owners.len() < 2
        {
            continue;
        }

        let chosen: Identifier = ranking.preferred(&tag, &members, blocks);
        found.push(Group { m_tag: tag, m_members: members, m_preferred: chosen });
    }

    found
}

// a member of two groups that disagree is left alone, converting it either way would be a guess
fn mapping(registry: RegistryKind, groups: &[Group]) -> BTreeMap<Identifier, Identifier>
{
    let mut targets: BTreeMap<Identifier, (Identifier, Identifier)> = BTreeMap::new();
    let mut ambiguous: BTreeSet<Identifier> = BTreeSet::new();

    for group in groups
    {
        for member in &group.m_members
        {
            match targets.get(member)
            {
                Some((target, tag)) if *target != group.m_preferred =>
                {
                    log::warn(
                        UNIFY,
                        &format!(
                            "{registry:?} {member} is in {tag} and {} which prefer {target} and {}, it stays as it is",
                            group.m_tag, group.m_preferred
                        ),
                    );
                    ambiguous.insert(member.clone());
                }

                Some(_) =>
                {}
                None =>
                {
                    targets.insert(member.clone(), (group.m_preferred.clone(), group.m_tag.clone()));
                }
            }
        }
    }

    targets
        .into_iter()
        .filter(|(member, (target, _))| member != target && !ambiguous.contains(member))
        .map(|(member, (target, _))| (member, target))
        .collect()
}

fn report(registry: RegistryKind, groups: &[Group])
{
    for group in groups
    {
        let members: Vec<String> = group.m_members.iter().map(Identifier::to_string).collect();
        log::info(
            UNIFY,
            &format!("{registry:?} {} merges {} into {}", group.m_tag, members.join(", "), group.m_preferred),
        );
    }
}

// a stack size or tool difference between members changes play once they merge, so it is pointed out
fn checkItems(groups: &[Group])
{
    for group in groups
    {
        let sizes: BTreeSet<i32> = group
            .m_members
            .iter()
            .filter_map(|member| registry::with(|registries| registries.itemId(member)))
            .filter_map(game::item::Item::byId)
            .map(|item| item.getItemStackLimit())
            .collect();
        if sizes.len() > 1
        {
            log::warn(
                UNIFY,
                &format!("{} members stack to different sizes {sizes:?}, they take the preferred one's", group.m_tag),
            );
        }
    }
}

fn numeric(items: &BTreeMap<Identifier, Identifier>) -> HashMap<i32, i32>
{
    registry::with(|registries|
    {
        items
            .iter()
            .filter_map(|(member, target)| Some((registries.itemId(member)?, registries.itemId(target)?)))
            .collect()
    })
}

// startup, before tags reach the game: the shared tags gain what names say, so the groups below need no mod's help
pub fn infer()
{
    let Some(paths) = crate::paths()
    else
    {
        return;
    };
    let settings: Settings = settings(&paths.m_config);

    if settings.enabled && settings.infer_from_names
    {
        infer::run();
    }
}

// builds the groups once every tag is pushed, again whenever ids are remapped
pub fn build()
{
    let Some(paths) = crate::paths()
    else
    {
        return;
    };
    let settings: Settings = settings(&paths.m_config);
    if !settings.enabled
    {
        TABLE.with(|table| *table.borrow_mut() = Table { m_settings: Some(settings), ..Table::default() });
        return;
    }

    let ranking: Ranking = Ranking::new(&settings);
    ranking.checkPriority();

    let blockGroups: Vec<Group> = groups(&ranking, RegistryKind::Block, &BTreeMap::new());
    let winners: BTreeMap<Identifier, Identifier> =
        blockGroups.iter().map(|group| (group.m_tag.clone(), group.m_preferred.clone())).collect();
    let itemGroups: Vec<Group> = groups(&ranking, RegistryKind::Item, &winners);

    report(RegistryKind::Item, &itemGroups);
    report(RegistryKind::Block, &blockGroups);
    checkItems(&itemGroups);

    let items: BTreeMap<Identifier, Identifier> = mapping(RegistryKind::Item, &itemGroups);
    let blocks: BTreeMap<Identifier, Identifier> = mapping(RegistryKind::Block, &blockGroups);
    let blockGroups: BTreeMap<Identifier, Identifier> = blockGroups
        .iter()
        .flat_map(|group| group.m_members.iter().map(|member| (member.clone(), group.m_tag.clone())))
        .collect();

    TABLE.with(|table|
    {
        *table.borrow_mut() = Table
        {
            m_settings: Some(settings),
            m_numeric: numeric(&items),
            m_items: items,
            m_blocks: blocks,
            m_blockGroups: blockGroups,
        }
    });
}

pub fn item(id: &Identifier) -> Identifier
{
    TABLE.with(|table| table.borrow().m_items.get(id).cloned()).unwrap_or_else(|| id.clone())
}

pub fn numericItem(id: i32) -> i32 { TABLE.with(|table| table.borrow().m_numeric.get(&id).copied()).unwrap_or(id) }

fn stack(spec: &mut StackSpec) { spec.item = item(&spec.item); }

fn ingredient(spec: &mut IngredientSpec)
{
    if let Some(id) = spec.item.as_mut()
    {
        *id = item(id);
    }
}

// a mod recipe makes and takes the preferred members, since only those are left in play
pub fn recipe(mut spec: RecipeSpec) -> RecipeSpec
{
    match &mut spec
    {
        RecipeSpec::Shaped(shaped) =>
        {
            shaped.key.values_mut().for_each(stack);
            stack(&mut shaped.result);
        }

        RecipeSpec::Shapeless(shapeless) =>
        {
            shapeless.ingredients.iter_mut().for_each(stack);
            stack(&mut shapeless.result);
        }

        RecipeSpec::Smelting(smelting) =>
        {
            smelting.input = item(&smelting.input);
            stack(&mut smelting.result);
        }

        RecipeSpec::Fuel(fuel) => fuel.item = item(&fuel.item),

        RecipeSpec::ShapedTagged(shaped) =>
        {
            shaped.key.values_mut().for_each(ingredient);
            stack(&mut shaped.result);
        }

        RecipeSpec::ShapelessTagged(shapeless) =>
        {
            shapeless.ingredients.iter_mut().for_each(ingredient);
            stack(&mut shapeless.result);
        }
    }

    spec
}

// ore veins of one group generate once per dimension with the preferred block, so merged mods add no extra ore
pub fn worldgen(list: Vec<(String, Identifier, WorldgenSpec)>) -> Vec<(String, Identifier, WorldgenSpec)>
{
    let (dedupe, blocks, groups): (bool, BTreeMap<Identifier, Identifier>, BTreeMap<Identifier, Identifier>) = TABLE
        .with(|table|
        {
            let table = table.borrow();
            let dedupe: bool = table.m_settings.as_ref().is_some_and(|settings| settings.dedupe_ore_worldgen);
            (dedupe, table.m_blocks.clone(), table.m_blockGroups.clone())
        });

    // the feature already placing the preferred block keeps its slot, otherwise the first one listed
    let mut kept: BTreeMap<(Identifier, Identifier), Identifier> = BTreeMap::new();
    let mut sorted: Vec<&(String, Identifier, WorldgenSpec)> = list.iter().collect();
    sorted.sort_by_key(|(_, _, spec)| blocks.contains_key(&spec.block));

    for (_, id, spec) in &sorted
    {
        let Some(group) = groups.get(&spec.block).filter(|_| dedupe && spec.kind == WorldgenKind::Ore)
        else
        {
            continue;
        };

        kept.entry((group.clone(), spec.dimension.clone())).or_insert_with(|| id.clone());
    }

    list.into_iter()
        .filter_map(|(owner, id, mut spec)|
        {
            let group: Option<&Identifier> =
                groups.get(&spec.block).filter(|_| dedupe && spec.kind == WorldgenKind::Ore);
            if let Some(group) = group
                && let Some(winner) = kept.get(&(group.clone(), spec.dimension.clone()))
                && *winner != id
            {
                log::warn(
                    UNIFY,
                    &format!("ore feature {id} of {owner} is skipped, {winner} already generates {group}"),
                );
                return None;
            }

            if let Some(target) = blocks.get(&spec.block)
            {
                log::info(UNIFY, &format!("ore feature {id} places {target} instead of {}", spec.block));
                spec.block = target.clone();
            }

            Some((owner, id, spec))
        })
        .collect()
}

// turns a stack into its preferred member in place, true when it changed
pub fn convert(stack: &ItemStack) -> bool
{
    let id: i32 = stack.m_itemID.get();
    let target: i32 = numericItem(id);
    if target == id
    {
        return false;
    }

    stack.m_itemID.set(target);
    true
}

fn convertSlots(slots: &RefCell<Vec<Option<Rc<ItemStack>>>>) -> bool
{
    // every stack converts, so no short circuit
    slots.borrow().iter().flatten().filter(|stack| convert(stack)).count() > 0
}

thread_local!
{
    static TICKS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

// a sweep a second keeps up with chests emptied into inventories without costing a tick
pub fn tick()
{
    const SWEEP_TICKS: u32 = 20;

    let ticks: u32 = TICKS.with(|ticks| ticks.replace(ticks.get().wrapping_add(1)));
    if ticks.is_multiple_of(SWEEP_TICKS)
    {
        sweep();
    }
}

// what players already carry from before a merge, or got from a source no hook sees
pub fn sweep()
{
    let empty: bool = TABLE.with(|table| table.borrow().m_numeric.is_empty());
    if empty
    {
        return;
    }

    for world in context::worlds().iter().filter(|world| crate::side::isAuthority(world))
    {
        for player in world.m_playerEntities.toVec()
        {
            sweepPlayer(&player);
        }
    }
}

fn sweepPlayer(player: &Rc<Entity>)
{
    let inventory: Rc<game::inventory::InventoryPlayer> = game::entity::entity_player::inventory(player);
    let main: bool = convertSlots(&inventory.m_mainInventory);
    let armor: bool = convertSlots(&inventory.m_armorInventory);
    if main || armor
    {
        inventory.onInventoryChanged();
    }
}
