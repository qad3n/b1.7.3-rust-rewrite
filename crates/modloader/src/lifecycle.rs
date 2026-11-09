use crate::content::{self, data};
use crate::discovery::{self, Discovered};
use crate::manifest::{SideSpec, WASM_FILE};
use crate::paths::LoaderPaths;
use crate::registry::{self, RegistryError};
use crate::resolver::{self, Candidate, Problem, Resolution};
use crate::state::{self, ModRuntime, Phase};
use crate::wasm::ModInstance;
use crate::{atlas, dimensions, events, log, services, side, tiles};
use jvm::logging::took;
use mod_protocol::Identifier;
use mod_protocol::abi::{EventData, EventKind, GuestCall, Handle, Lifecycle};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

pub mod ids
{
    // the first 256 ids fit vanilla chunk bytes, the rest use the Add nibbles of 12-bit chunks
    pub const BLOCK_FIRST_FREE: i32 = 97;
    pub const BLOCK_LIMIT: i32 = 4096;
    pub const ITEM_FIRST: i32 = 4096;
    pub const ITEM_LIMIT: i32 = 32000;
    pub const ENTITY_FIRST: i32 = 1000;
    pub const TILE_ENTITY_FIRST: i32 = 100;
    pub const RECIPE_FIRST: i32 = 10_000;
    pub const DIMENSION_FIRST: i32 = 2;
    pub const BIOME_FIRST: i32 = 13;
}

thread_local! {
    static RECIPE_REMOVALS: RefCell<Vec<Identifier>> = const { RefCell::new(Vec::new()) };
    static PENDING_LOOKUPS: RefCell<Vec<(usize, Identifier, Identifier, Handle)>> = const { RefCell::new(Vec::new()) };
    static BLOCK_LIMIT: std::cell::Cell<i32> = const { std::cell::Cell::new(ids::BLOCK_LIMIT) };
    static WORLDGEN: RefCell<Vec<(String, Identifier, mod_protocol::spec::WorldgenSpec)>> = const { RefCell::new(Vec::new()) };
}

pub fn queueRecipeRemoval(id: &Identifier) -> Result<(), String>
{
    RECIPE_REMOVALS.with(|list| list.borrow_mut().push(id.clone()));
    Ok(())
}

pub fn queueLookup(modIndex: usize, api: Identifier, block: Identifier, handler: Handle)
{
    PENDING_LOOKUPS.with(|list| list.borrow_mut().push((modIndex, api, block, handler)));
}

// the chunk format decides how many block ids exist, Phase 3 raises it to 4096
pub fn setBlockLimit(limit: i32) { BLOCK_LIMIT.with(|cell| cell.set(limit)); }

pub fn blockLimit() -> i32 { BLOCK_LIMIT.with(std::cell::Cell::get) }

fn physicalSide() -> SideSpec
{
    if side::isClient()
    {
        SideSpec::Client
    }
    else
    {
        SideSpec::Server
    }
}

fn fileName(discovered: &Discovered) -> String
{
    discovered.m_source.path().file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

fn logReport(problems: &[Problem], skipped: &[(String, String)])
{
    for (id, reason) in skipped
    {
        log::info(log::LOADER, &format!("skipped {id}: {reason}"));
    }
    if problems.is_empty()
    {
        return;
    }
    let mut report: String = format!("{} mod loading problems:", problems.len());
    for problem in problems
    {
        report.push_str(&format!("\n  {} ({}): {}", problem.m_mod, problem.m_file, problem.m_reason));
    }
    log::error(log::LOADER, &report);
}

fn instantiate(runtime: &mut ModRuntime, paths: &LoaderPaths) -> Result<(), String>
{
    let Some(wasm) = runtime.m_source.borrow_mut().read(WASM_FILE)
    else
    {
        log::debug(runtime.id(), "data only mod, no mod.wasm");
        return Ok(());
    };

    let started: Instant = Instant::now();
    runtime.m_instance = Some(ModInstance::load(runtime.m_index, &wasm, &paths.m_cache)?);
    log::debug(runtime.id(), &format!("mod.wasm ({} KiB) instantiated in {}", wasm.len() / 1024, took(started)));
    Ok(())
}

fn allocate(
    first: i32,
    limit: i32,
    skip: &dyn Fn(i32) -> bool,
) -> impl FnMut(&Identifier, &dyn Fn(i32) -> bool) -> Option<i32>
{
    let skip: Vec<bool> = (first..limit).map(skip).collect();
    move |_, taken| (first..limit).find(|candidate| !skip[(candidate - first) as usize] && !taken(*candidate))
}

fn freezeRegistries() -> Result<(), RegistryError>
{
    let blockLimit: i32 = blockLimit();
    registry::with(|registries| {
        let vanillaItems: std::collections::HashSet<i32> =
            registries.m_items.rows().filter(|row| row.m_id.isVanilla()).map(|row| row.m_numeric).collect();
        // block ids above 255 must not land on a vanilla item id, the item form shares the number
        registries.m_blocks.freeze(allocate(ids::BLOCK_FIRST_FREE, blockLimit, &|id| vanillaItems.contains(&id)))?;
        let blocks: std::collections::BTreeMap<Identifier, i32> = registries.m_blocks.snapshot();
        let mut items = allocate(ids::ITEM_FIRST, ids::ITEM_LIMIT, &|_| false);
        registries.m_items.freeze(|id, taken| blocks.get(id).copied().or_else(|| items(id, taken)))?;
        registries.m_entities.freeze(allocate(ids::ENTITY_FIRST, i32::MAX, &|_| false))?;
        registries.m_tileEntities.freeze(allocate(ids::TILE_ENTITY_FIRST, i32::MAX, &|_| false))?;
        registries.m_recipes.freeze(allocate(ids::RECIPE_FIRST, i32::MAX, &|_| false))?;
        registries.m_smelting.freeze(allocate(ids::RECIPE_FIRST, i32::MAX, &|_| false))?;
        registries.m_dimensions.freeze(allocate(ids::DIMENSION_FIRST, i8::MAX as i32, &|_| false))?;
        registries.m_biomes.freeze(allocate(ids::BIOME_FIRST, i32::MAX, &|_| false))?;
        registries.m_fuel.freeze(|_, _| None)
    })
}

fn ownerIndex(owner: &str) -> usize { state::byId(owner).map_or(usize::MAX, |runtime| runtime.m_index) }

fn fail(owner: &str, reason: String)
{
    match state::byId(owner)
    {
        Some(runtime) => runtime.fail(reason),
        None => state::problem(owner, "", &reason),
    }
}

fn rebuildBlock(id: &Identifier, owner: &str)
{
    let row = registry::with(|registries| clone(registries.m_blocks.get(id).expect("the id was just listed")));
    let tile: Option<Identifier> = row.m_value.m_spec.as_ref().and_then(|spec| spec.tile_entity.clone());
    let bound: Result<(), String> = match &tile
    {
        Some(tile) => tiles::bindBlock(row.m_numeric, tile),
        None => Ok(()),
    };
    if row.m_id.isVanilla()
    {
        game::block::Block::unregister(row.m_numeric);
        game::item::Item::unregister(row.m_numeric);
    }
    if let Err(reason) = bound.and_then(|()| content::blocks::build(&row, ownerIndex(owner)))
    {
        fail(owner, format!("block {id}: {reason}"));
    }
}

// modded rows and vanilla rows a mod replaced are built from their specs, in registry order
pub fn buildContent()
{
    buildRegistered();
    crate::modify::apply();
}

fn buildRegistered()
{
    let blocks: Vec<(Identifier, String)> = registry::with(|registries| {
        registries
            .m_blocks
            .rows()
            .filter(|row| !row.m_id.isVanilla() || row.m_replacedBy.is_some())
            .map(|row| (row.m_id.clone(), row.m_replacedBy.clone().unwrap_or_else(|| row.m_owner.clone())))
            .collect()
    });
    for (id, owner) in blocks
    {
        rebuildBlock(&id, &owner);
    }
    let itemIds: Vec<(Identifier, String)> = registry::with(|registries| {
        registries.m_items.modded().map(|row| (row.m_id.clone(), row.m_owner.clone())).collect()
    });
    for (id, owner) in itemIds
    {
        let row = registry::with(|registries| cloneItem(registries.m_items.get(&id).expect("listed")));
        if let Err(reason) = content::items::build(&row, ownerIndex(&owner))
        {
            fail(&owner, format!("item {id}: {reason}"));
        }
    }
}

fn clone(row: &registry::Row<registry::BlockEntry>) -> registry::Row<registry::BlockEntry>
{
    registry::Row {
        m_id: row.m_id.clone(),
        m_numeric: row.m_numeric,
        m_owner: row.m_owner.clone(),
        m_replacedBy: row.m_replacedBy.clone(),
        m_value: registry::BlockEntry { m_spec: row.m_value.m_spec.clone(), m_page: row.m_value.m_page.clone() },
    }
}

fn cloneItem(row: &registry::Row<registry::ItemEntry>) -> registry::Row<registry::ItemEntry>
{
    registry::Row {
        m_id: row.m_id.clone(),
        m_numeric: row.m_numeric,
        m_owner: row.m_owner.clone(),
        m_replacedBy: row.m_replacedBy.clone(),
        m_value: registry::ItemEntry { m_spec: row.m_value.m_spec.clone(), m_block: row.m_value.m_block.clone() },
    }
}

// abi 2 named vanilla recipes by list index, crafting/<index> still resolves
fn legacyRecipe(id: &Identifier) -> Option<i32>
{
    id.path().strip_prefix("crafting/")?.parse().ok().filter(|_| id.isVanilla())
}

fn removeVanillaRecipes()
{
    let mut removals: Vec<i32> = RECIPE_REMOVALS
        .with(|list| list.borrow_mut().drain(..).collect::<Vec<Identifier>>())
        .iter()
        .filter_map(|id| {
            registry::with(|registries| {
                let row = registries
                    .m_recipes
                    .get(id)
                    .or_else(|| legacyRecipe(id).and_then(|n| registries.m_recipes.byNumeric(n)));
                row.filter(|row| row.m_id.isVanilla()).map(|row| row.m_numeric)
            })
        })
        .collect();
    content::recipes::removeVanilla(&mut removals);
}

fn applyRecipes()
{
    let recipes: Vec<(String, Identifier, mod_protocol::spec::RecipeSpec)> = registry::with(|registries| {
        registries
            .m_recipes
            .modded()
            .chain(registries.m_smelting.modded())
            .filter_map(|row| row.m_value.m_spec.clone().map(|spec| (row.m_owner.clone(), row.m_id.clone(), spec)))
            .collect()
    });
    for (owner, id, spec) in recipes
    {
        if let Err(reason) = content::recipes::apply(&id, &spec)
        {
            fail(&owner, format!("recipe {id}: {reason}"));
        }
    }
    content::recipes::finish();
}

fn applyWorldgen()
{
    for (owner, id, spec) in WORLDGEN.with(|list| list.borrow().clone())
    {
        if let Err(reason) = content::worldgen::add(&id, &spec)
        {
            fail(&owner, format!("worldgen for {}: {reason}", spec.block));
        }
    }
}

fn applyLate()
{
    WORLDGEN.with(|list| *list.borrow_mut() = data::takeWorldgen());
    applyWorldgen();
    for (modIndex, api, block, handler) in PENDING_LOOKUPS.with(|list| list.borrow_mut().drain(..).collect::<Vec<_>>())
    {
        match registry::with(|registries| registries.blockId(&block))
        {
            Some(numeric) => services::registerLookup(modIndex, api, numeric, handler),
            None => fail(&state::modId(modIndex), format!("lookup {api} names unknown block {block}")),
        }
    }
    dimensions::finish();
    crate::entities::registerSpawns();
    crate::modify::applySpawns();
}

fn loadSprites()
{
    if !side::isClient()
    {
        return;
    }
    let mods: Vec<Rc<ModRuntime>> = state::mods();
    let problems: Vec<String> = atlas::loadSprites(&|id: &Identifier| {
        let owner: Rc<ModRuntime> = mods.iter().find(|runtime| runtime.id() == id.namespace())?.clone();
        let bytes: Option<Vec<u8>> = owner.m_source.borrow_mut().read(&atlas::texturePath(id));
        bytes
    });
    for problem in problems
    {
        log::warn(log::LOADER, &problem);
    }
}

fn callAll(call: GuestCall)
{
    for runtime in state::mods()
    {
        runtime.call(&call);
    }
}

pub fn post(kind: EventKind) { events::post(kind, EventData::Empty); }

// discovery to load_complete, the steps of design section 3.4
pub fn load(paths: &LoaderPaths)
{
    if !discovery::hasMods(&paths.m_mods)
    {
        log::info(log::LOADER, &format!("no mods in {}, playing vanilla", paths.m_mods.display()));
        return;
    }

    start(paths);
}

fn registryCounts() -> String
{
    registry::with(|registries| {
        format!(
            "{} blocks, {} items, {} entities, {} tile entities, {} recipes, {} dimensions, {} biomes",
            registries.m_blocks.modded().count(),
            registries.m_items.modded().count(),
            registries.m_entities.modded().count(),
            registries.m_tileEntities.modded().count(),
            registries.m_recipes.modded().count() + registries.m_smelting.modded().count(),
            registries.m_dimensions.modded().count(),
            registries.m_biomes.modded().count()
        )
    })
}

// a world saved with mods opens in a game without any, the loader starts with no mods so placeholders keep that
// world's mod blocks, items, tile entities and entities instead of the vanilla code dropping them
pub fn keep()
{
    let Some(paths) = crate::paths()
    else
    {
        return;
    };
    log::warn(log::LOADER, "this world was saved with mods that are not installed, keeping their content");
    let started: bool = state::phase() != Phase::Bootstrap;
    state::setKeeper(true);
    if !started
    {
        start(&paths);
    }
}

fn start(paths: &LoaderPaths)
{
    let started: Instant = Instant::now();
    log::info(log::LOADER, &format!("loading mods from {}", paths.m_mods.display()));
    let (found, errors) = discovery::discover(&paths.m_mods);
    for discovered in &found
    {
        log::debug(
            log::LOADER,
            &format!(
                "found {} {} in {}",
                discovered.m_manifest.id,
                discovered.m_manifest.version,
                fileName(discovered)
            ),
        );
    }

    for error in errors
    {
        state::problem(state::UNKNOWN_MOD, &error.m_path.display().to_string(), &error.m_reason);
    }
    let candidates: Vec<Candidate> = found
        .iter()
        .map(|discovered| Candidate { m_manifest: discovered.m_manifest.clone(), m_file: fileName(discovered) })
        .collect();
    let resolution: Resolution = resolver::resolve(&candidates, physicalSide());
    let order: Vec<&str> = resolution.m_order.iter().map(|index| candidates[*index].m_manifest.id.as_str()).collect();
    log::debug(log::LOADER, &format!("load order: {}", order.join(", ")));

    state::addProblems(resolution.m_problems.clone());
    state::setSkipped(resolution.m_skipped.clone());
    let mut found: Vec<Option<Discovered>> = found.into_iter().map(Some).collect();
    let mut runtimes: Vec<ModRuntime> = Vec::new();
    for (index, candidateIndex) in resolution.m_order.iter().enumerate()
    {
        let discovered: Discovered = found[*candidateIndex].take().expect("each candidate is ordered once");
        let file: String = fileName(&discovered);
        let config = crate::config::ModConfig::new(&paths.m_config, &discovered.m_manifest.id);
        runtimes.push(ModRuntime::new(
            index,
            discovered.m_manifest,
            file,
            discovered.m_sha256,
            discovered.m_source,
            config,
        ));
    }
    crate::hooks::install();
    crate::network::loader::register();
    crate::network::blocks::register();
    game::block::Block::byId(1);
    game::item::Item::byId(256);
    registry::with(|_| ());
    state::setPhase(Phase::Register);
    let stage: Instant = Instant::now();
    content::harvest::seedTags();
    for runtime in &mut runtimes
    {
        let id: String = runtime.id().to_owned();
        let problems: Vec<String> = data::load(&id, &mut runtime.m_source.borrow_mut());
        for problem in problems
        {
            state::problem(&id, &runtime.m_file, &problem);
        }
        if let Err(reason) = instantiate(runtime, paths)
        {
            state::problem(&id, &runtime.m_file, &reason);
        }
    }
    state::setMods(runtimes.into_iter().map(Rc::new).collect());
    for runtime in state::mods()
    {
        runtime.call(&GuestCall::Lifecycle(Lifecycle::Register(runtime.id().to_owned())));
    }
    for (owner, reason) in crate::states::registerPages()
    {
        state::problem(&owner, "", &reason);
    }
    atlas::assign();
    log::debug(log::LOADER, &format!("register phase took {}", took(stage)));
    if let Err(error) = freezeRegistries()
    {
        state::problem(log::LOADER, "", &error.to_string());
    }
    state::setPhase(Phase::Frozen);
    log::debug(log::LOADER, &format!("registries frozen with {}", registryCounts()));

    let stage: Instant = Instant::now();
    crate::netsync::checkSides();
    buildContent();
    removeVanillaRecipes();
    applyRecipes();
    applyLate();
    loadSprites();
    log::debug(log::LOADER, &format!("content, recipes and sprites built in {}", took(stage)));
    post(EventKind::Setup);
    post(
        if side::isClient()
        {
            EventKind::ClientSetup
        }
        else
        {
            EventKind::ServerSetup
        },
    );
    services::deliverInterMod();
    callAll(GuestCall::Lifecycle(Lifecycle::InterMod));
    post(EventKind::LoadComplete);
    state::setPhase(Phase::Running);
    crate::netsync::rememberLocalSnapshot();
    for runtime in state::mods().iter().filter(|runtime| runtime.isActive())
    {
        log::info(log::LOADER, &format!("loaded {} {} ({})", runtime.id(), runtime.m_manifest.version, runtime.m_file));
    }
    logReport(&state::problems(), &resolution.m_skipped);
    let active: usize = state::mods().iter().filter(|runtime| runtime.isActive()).count();
    log::info(log::LOADER, &format!("{active} of {} mods loaded in {}", found.len(), took(started)));
}

// a new id map from a world or a server, every mod object is built again at its new id
pub fn rebuild()
{
    let started: Instant = Instant::now();
    tiles::clearBindings();
    buildContent();
    applyRecipes();
    applyWorldgen();
    dimensions::finish();
    callAll(GuestCall::Lifecycle(Lifecycle::Remapped));
    log::info(log::LOADER, &format!("rebuilt mod content for the new id map in {}", took(started)));
}

pub fn shutdown()
{
    if state::hasMods()
    {
        log::debug(log::LOADER, "posting shutdown to mods");
        post(EventKind::Shutdown);
    }
}
