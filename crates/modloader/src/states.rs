use crate::registry::{self, BlockEntry};
use mod_protocol::Identifier;
use mod_protocol::abi::StateData;
use mod_protocol::spec::{BlockStatesSpec, PropertySpec, StateVariantSpec};
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet, HashMap};

// a chunk stores 4 bits of metadata, so each block id carries 16 states and larger blocks span more ids
pub mod pages
{
    pub const STATES_PER_ID: usize = 16;
    pub const MAX_PAGES: usize = 16;
    pub const MARKER: &str = "/state_page_";
}

// a block's properties with each value list rotated to start at its default, so state 0 is the default state
#[derive(Clone)]
pub struct StateTable
{
    m_properties: Vec<PropertySpec>,
    m_variants: Vec<StateVariantSpec>,
}

impl StateTable
{
    fn new(spec: &BlockStatesSpec) -> Result<Self, String>
    {
        let mut names: BTreeSet<&str> = BTreeSet::new();
        let mut properties: Vec<PropertySpec> = Vec::new();
        for property in &spec.properties
        {
            if property.values.is_empty() || !names.insert(&property.name)
            {
                return Err(format!("property {} is empty or listed twice", property.name));
            }

            let mut values: Vec<String> = property.values.clone();

            if let Some(default) = spec.default.get(&property.name)
            {
                let at: usize = values
                    .iter()
                    .position(|value| value == default)
                    .ok_or_else(|| format!("default {default} is not a value of {}", property.name))?;
                values.rotate_left(at);
            }

            properties.push(PropertySpec { name: property.name.clone(), values });
        }

        if let Some(unknown) = spec.default.keys().find(|key| !names.contains(key.as_str()))
        {
            return Err(format!("default names unknown property {unknown}"));
        }

        let table: Self = Self { m_properties: properties, m_variants: spec.variants.clone() };
        if table.pages() > pages::MAX_PAGES
        {
            return Err(format!(
                "{} states exceed the limit of {}",
                table.count(),
                pages::MAX_PAGES * pages::STATES_PER_ID
            ));
        }

        Ok(table)
    }

    pub fn count(&self) -> usize { self.m_properties.iter().map(|property| property.values.len()).product() }

    pub fn pages(&self) -> usize { self.count().div_ceil(pages::STATES_PER_ID) }

    // the last property changes fastest, an absent or unknown value counts as the default
    pub fn index(&self, values: &BTreeMap<String, String>) -> usize
    {
        self.m_properties.iter().fold(0, |index, property|
        {
            let value: usize = values
                .get(&property.name)
                .and_then(|value| property.values.iter().position(|known| known == value))
                .unwrap_or(0);
            index * property.values.len() + value
        })
    }

    pub fn properties(&self, index: usize) -> BTreeMap<String, String>
    {
        let mut rest: usize = index;
        let mut values: BTreeMap<String, String> = BTreeMap::new();

        for property in self.m_properties.iter().rev()
        {
            values.insert(property.name.clone(), property.values[rest % property.values.len()].clone());
            rest /= property.values.len();
        }

        values
    }

    pub fn variants(&self) -> &[StateVariantSpec] { &self.m_variants }

    pub fn has(&self, property: &str) -> bool { self.m_properties.iter().any(|known| known.name == property) }
}

#[derive(Default)]
struct State
{
    m_tables: BTreeMap<Identifier, StateTable>,
    // numeric id to the block it carries states for and which page of 16 states it is
    m_pages: HashMap<i32, (Identifier, usize)>,
    m_ids: HashMap<Identifier, Vec<i32>>,
    // numeric id to the metadata step of its check_decay flag, read on first use
    m_decay: HashMap<i32, Option<i32>>,
}

// the named state leaves of any mod keep beta's check decay flag in, false then true
const CHECK_DECAY: &str = "check_decay";

thread_local!
{
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn register(block: Identifier, spec: &BlockStatesSpec) -> Result<(), String>
{
    let table: StateTable = StateTable::new(spec).map_err(|reason| format!("states of {block}: {reason}"))?;

    for texture in spec.variants.iter().flat_map(variantTextures)
    {
        crate::atlas::request(crate::atlas::Atlas::Terrain, &texture);
    }

    STATE.with(|state| match state.borrow_mut().m_tables.entry(block.clone())
    {
        std::collections::btree_map::Entry::Occupied(_) => Err(format!("states of {block} are registered twice")),

        std::collections::btree_map::Entry::Vacant(slot) =>
        {
            slot.insert(table);
            Ok(())
        }
    })
}

fn variantTextures(variant: &StateVariantSpec) -> Vec<String>
{
    let mut textures: Vec<String> = variant.texture.iter().cloned().collect();
    if let Some(faces) = &variant.textures
    {
        textures.extend([&faces.top, &faces.bottom, &faces.side, &faces.front].into_iter().flatten().cloned());
    }

    textures
}

pub fn table(block: &Identifier) -> Option<StateTable>
{
    STATE.with(|state| state.borrow().m_tables.get(block).cloned())
}

fn pageId(block: &Identifier, page: usize) -> Option<Identifier>
{
    Identifier::new(block.namespace(), &format!("{}{}{page}", block.path(), pages::MARKER)).ok()
}

// blocks past 16 states take hidden rows for their extra ids before the registries freeze
pub fn registerPages() -> Vec<(String, String)>
{
    let tables: Vec<(Identifier, usize)> =
        STATE.with(|state| state.borrow().m_tables.iter().map(|(id, table)| (id.clone(), table.pages())).collect());
    let mut problems: Vec<(String, String)> = Vec::new();

    for (block, pages) in tables
    {
        let result: Result<(), String> = registry::with(|registries|
        {
            let base: &registry::Row<BlockEntry> =
                registries.m_blocks.get(&block).ok_or_else(|| format!("states name unknown block {block}"))?;
            let (owner, spec): (String, Option<mod_protocol::spec::BlockSpec>) =
                (base.m_owner.clone(), base.m_value.m_spec.clone());

            for page in 1..pages
            {
                let id: Identifier = pageId(&block, page).ok_or("a page id is a valid identifier")?;
                let entry: BlockEntry = BlockEntry { m_spec: spec.clone(), m_page: Some((block.clone(), page)) };
                registries.m_blocks.register(id, &owner, entry).map_err(|error| error.to_string())?;
            }

            Ok(())
        });
        if let Err(reason) = result
        {
            problems.push((block.namespace().to_owned(), reason));
        }
    }

    problems
}

pub fn clearBindings()
{
    STATE.with(|state|
    {
        let mut state: RefMut<crate::states::State> = state.borrow_mut();
        state.m_pages.clear();
        state.m_ids.clear();
        state.m_decay.clear();
    });
}

// how far apart a block's states with check_decay false and true lie in metadata, none without the property
pub fn decayStride(numeric: i32) -> Option<i32>
{
    if let Some(stride) = STATE.with(|state| state.borrow().m_decay.get(&numeric).copied())
    {
        return stride;
    }

    let stride: Option<i32> = measureDecay(numeric);
    STATE.with(|state| state.borrow_mut().m_decay.insert(numeric, stride));
    stride
}

fn measureDecay(numeric: i32) -> Option<i32>
{
    let state: StateData = describe(numeric, 0)?;
    state.properties.get(CHECK_DECAY)?;

    let at = |flag: &str|
    {
        let mut properties: BTreeMap<String, String> = state.properties.clone();
        properties.insert(CHECK_DECAY.to_owned(), flag.to_owned());
        resolve(&StateData { block: state.block.clone(), properties })
    };
    let ((idle, idleMeta), (checking, checkingMeta)) = (at("false")?, at("true")?);
    (idle == numeric && checking == numeric && checkingMeta > idleMeta).then_some(checkingMeta - idleMeta)
}

// every numeric id a block spans, its base id first and then its hidden pages
pub fn pageIds(block: &Identifier) -> Vec<i32>
{
    // unbound pages hold 0, which is air
    STATE.with(|state|
    {
        state
            .borrow()
            .m_ids
            .get(block)
            .map(|ids| ids.iter().copied().filter(|id| *id > 0).collect())
            .unwrap_or_default()
    })
}

pub fn bindPage(numeric: i32, block: &Identifier, page: usize)
{
    STATE.with(|state|
    {
        let mut state: RefMut<crate::states::State> = state.borrow_mut();
        state.m_pages.insert(numeric, (block.clone(), page));

        let ids: &mut Vec<i32> = state.m_ids.entry(block.clone()).or_default();
        if ids.len() <= page
        {
            ids.resize(page + 1, 0);
        }

        ids[page] = numeric;
    });
}

// the block and state index a numeric id and metadata hold, for blocks with named states
pub fn stateAt(numeric: i32, meta: i32) -> Option<(Identifier, usize)>
{
    STATE.with(|state|
    {
        let state: std::cell::Ref<crate::states::State> = state.borrow();
        let (block, page): (Identifier, usize) = state.m_pages.get(&numeric).cloned()?;
        let index: usize = page * pages::STATES_PER_ID + (meta as usize & (pages::STATES_PER_ID - 1));
        // metadata past the named states, as an older version of the mod wrote, stays raw metadata
        (index < state.m_tables.get(&block)?.count()).then_some((block, index))
    })
}

pub fn page(numeric: i32) -> Option<usize> { STATE.with(|state| state.borrow().m_pages.get(&numeric).map(|row| row.1)) }

// the numeric id and metadata a state of a block is stored as
pub fn locate(block: &Identifier, index: usize) -> Option<(i32, i32)>
{
    STATE.with(|state|
    {
        let numeric: i32 = *state.borrow().m_ids.get(block)?.get(index / pages::STATES_PER_ID)?;
        Some((numeric, (index % pages::STATES_PER_ID) as i32))
    })
}

// the snapshot entry naming each block's state count, peers with different state layouts must not play together
pub const SNAPSHOT_KEY: &str = "block_state";

pub fn counts() -> BTreeMap<Identifier, i32>
{
    STATE.with(|state| state.borrow().m_tables.iter().map(|(id, table)| (id.clone(), table.count() as i32)).collect())
}

// blocks whose state count differs from the server's, named for the refusal message
pub fn mismatches(server: Option<&BTreeMap<Identifier, i32>>) -> Vec<String>
{
    let empty: BTreeMap<Identifier, i32> = BTreeMap::new();
    let server: &BTreeMap<Identifier, i32> = server.unwrap_or(&empty);
    let local: BTreeMap<Identifier, i32> = counts();
    let mut ids: BTreeSet<&Identifier> = server.keys().collect();
    ids.extend(local.keys());
    ids.into_iter()
        .filter(|id| server.get(*id) != local.get(*id))
        .map(|id| format!("{id} has different block states than the server"))
        .collect()
}

// blocks without named states, vanilla ones included, report their metadata under this property
pub const META_PROPERTY: &str = "meta";

pub fn describe(numeric: i32, meta: i32) -> Option<StateData>
{
    if let Some((block, index)) = stateAt(numeric, meta)
    {
        let properties: BTreeMap<String, String> = table(&block)?.properties(index);
        return Some(StateData { block, properties });
    }

    let block: Identifier = STATE
        .with(|state| state.borrow().m_pages.get(&numeric).map(|(block, _)| block.clone()))
        .or_else(|| registry::with(|registries| registries.m_blocks.byNumeric(numeric).map(|row| row.m_id.clone())))?;
    Some(StateData { block, properties: BTreeMap::from([(META_PROPERTY.to_owned(), meta.to_string())]) })
}

// air has no registry row, block 0 by name in both directions
pub const AIR_BLOCK: i32 = 0;

pub fn isAir(id: &Identifier) -> bool { id.isVanilla() && id.path() == "air" }

pub fn resolve(state: &StateData) -> Option<(i32, i32)>
{
    if isAir(&state.block)
    {
        return Some((AIR_BLOCK, 0));
    }

    if let Some(table) = table(&state.block)
        && !state.properties.contains_key(META_PROPERTY)
    {
        return locate(&state.block, table.index(&state.properties));
    }
    if let Some(meta) = state.properties.get(META_PROPERTY).and_then(|meta| meta.parse::<i32>().ok())
        && let Some((base, _)) = locate(&state.block, 0)
    {
        return Some((base, meta & 15));
    }

    let numeric: i32 = registry::with(|registries| registries.blockId(&state.block))?;
    let meta: i32 = state.properties.get(META_PROPERTY).and_then(|meta| meta.parse().ok()).unwrap_or(0);
    Some((numeric, meta & 15))
}
