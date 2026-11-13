use super::{ok, unit};
use crate::content::loot::RuleKind;
use crate::content::{data, tags};
use crate::registry::{self, RegistryError};
use crate::state::{self, ModRuntime, Phase};
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    ContentRequest, HostCall, HostReply, InventoryRequest, MetaRequest, NetRequest, RegistryKind, RegistryRequest,
    StateData, StoredRequest, WorldRequest, WorldgenRequest,
};
use mod_protocol::identifier::MINECRAFT;
use std::rc::Rc;

const VANILLA_NAME: &str = "Minecraft";
const AIR: &str = "air";

fn registering() -> Result<(), String>
{
    match state::phase()
    {
        Phase::Register => Ok(()),
        phase => Err(format!("content registers during the register phase, the loader is in {phase:?}")),
    }
}

// data other content reads, such as tags, loot and machine recipes, may still be added while setup runs
fn loading() -> Result<(), String>
{
    match state::phase()
    {
        Phase::Register | Phase::Frozen => Ok(()),
        phase => Err(format!("content data is added while mods load, the loader is in {phase:?}")),
    }
}

fn resolve(kind: RegistryKind, id: &Identifier) -> Option<i32>
{
    registry::with(|registries| match kind
    {
        RegistryKind::Block if crate::states::isAir(id) => Some(crate::states::AIR_BLOCK),
        RegistryKind::Block => registries.blockId(id),
        RegistryKind::Item => registries.itemId(id),
        RegistryKind::Entity => registries.m_entities.numeric(id),
        RegistryKind::TileEntity => registries.m_tileEntities.numeric(id),
        RegistryKind::Recipe => registries.m_recipes.numeric(id),
        RegistryKind::Biome => registries.m_biomes.numeric(id),
        RegistryKind::Dimension => registries.m_dimensions.numeric(id),
        // the named registries have no numeric ids
        _ => None,
    })
}

fn nameOf(kind: RegistryKind, numeric: i32) -> Option<Identifier>
{
    registry::with(|registries| match kind
    {
        RegistryKind::Block if numeric == crate::states::AIR_BLOCK => Some(Identifier::minecraft(AIR)),
        // a hidden state page id answers with the block it extends
        RegistryKind::Block => registries
            .m_blocks
            .byNumeric(numeric)
            .map(|row| row.m_value.m_page.as_ref().map_or_else(|| row.m_id.clone(), |(base, _)| base.clone())),
        RegistryKind::Item => registries.m_items.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Entity => registries.m_entities.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::TileEntity => registries.m_tileEntities.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Recipe => registries.m_recipes.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Biome => registries.m_biomes.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Dimension => registries.m_dimensions.byNumeric(numeric).map(|row| row.m_id.clone()),
        _ => None,
    })
}

fn registryError(error: RegistryError) -> String { error.to_string() }

// registration, lookups and loader queries, anything else goes on to the world handler
pub(crate) fn handle(runtime: &Rc<ModRuntime>, call: HostCall) -> Result<HostReply, Box<HostCall>>
{
    let owner: String = runtime.id().to_owned();

    let reply: HostReply = match call
    {
        HostCall::Content(ContentRequest::RegisterBlock { name, spec }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| data::registerBlock(id, &owner, spec)))
        }

        HostCall::Content(ContentRequest::RegisterItem { name, spec }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| data::registerItem(id, &owner, spec)))
        }

        HostCall::Content(ContentRequest::RegisterRecipe { name, spec }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| data::registerRecipe(id, &owner, spec)))
        }

        HostCall::Content(ContentRequest::TileModel { tile, model }) =>
        {
            unit(registering().and_then(|()| crate::tiles::registerModel(runtime.m_index, tile, model)))
        }

        HostCall::Content(ContentRequest::RegisterTileEntity { name, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::tiles::register(id, &owner, runtime.m_index, spec)),
        ),

        HostCall::Content(ContentRequest::RegisterEntity { name, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::entities::register(id, runtime.m_index, spec)),
        ),
        HostCall::Content(ContentRequest::RemoveRecipe { id }) =>
        {
            unit(registering().and_then(|()| crate::lifecycle::queueRecipeRemoval(&id)))
        }
        HostCall::Content(ContentRequest::ReplaceBlock { id, spec }) => unit(registering().and_then(|()|
        {
            crate::content::blocks::validate(&spec)?;

            registry::with(|registries|
            {
                registries
                    .m_blocks
                    .replace(&id, &owner, registry::BlockEntry { m_spec: Some(spec), m_page: None })
                    .map_err(registryError)
            })
        })),
        HostCall::Content(ContentRequest::ModifyBlock { spec }) =>
        {
            unit(registering().map(|()| crate::modify::queueBlock(&owner, spec)))
        }
        HostCall::Content(ContentRequest::ModifyItem { spec }) =>
        {
            unit(registering().map(|()| crate::modify::queueItem(&owner, spec)))
        }
        HostCall::Content(ContentRequest::ModifyMachine { spec }) =>
        {
            unit(loading().and_then(|()| crate::content::machines::removeOutputs(&spec.machine, &spec.remove_outputs)))
        }
        HostCall::Content(ContentRequest::RegisterBlockStates { block, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&block.to_string()))
                .and_then(|block| crate::states::register(block, &spec)),
        ),
        HostCall::Content(ContentRequest::ModifySpawns { spec }) =>
        {
            unit(registering().map(|()| crate::modify::queueSpawn(&owner, spec)))
        }

        HostCall::Registry(RegistryRequest::AddTagEntries { registry, tag, entries, replace }) =>
        {
            unit(loading().map(|()|
            {
                tags::addEntries(registry, tag, entries, replace);
                tags::push();
            }))
        }

        HostCall::Content(ContentRequest::RegisterAchievement { name, spec }) => unit(
            registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::content::progress::register(id, spec)),
        ),

        HostCall::Content(ContentRequest::RegisterStat { name, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::content::progress::registerStat(id, spec)),
        ),

        HostCall::Content(ContentRequest::RegisterParticle { name, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::content::particles::register(id, spec)),
        ),
        HostCall::Content(ContentRequest::RegisterEffect { name, spec }) => unit(
            registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::content::effects::register(id, spec)),
        ),
        HostCall::Content(ContentRequest::RegisterEffectBehavior { name, spec, behavior }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::content::effects::registerWith(id, spec, runtime.m_index, behavior)),
        ),
        HostCall::Content(ContentRequest::RegisterAttribute { name, spec }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::attributes::register(id, spec)))
        }
        HostCall::Content(ContentRequest::RegisterGameRule { name, spec }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::gamerules::register(id, spec)))
        }
        HostCall::Content(ContentRequest::RegisterEnergy { name, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::energy::register(runtime.id(), id, spec)),
        ),
        HostCall::World(WorldRequest::GameRule { rule }) => ok(crate::gamerules::value(&rule), |reply| reply),
        HostCall::World(WorldRequest::SetGameRule { rule, value }) => unit(crate::gamerules::set(&rule, value)),
        HostCall::Content(ContentRequest::RegisterFluid { name, spec }) => unit(
            registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::content::fluids::register(id, spec)),
        ),

        HostCall::Content(ContentRequest::SetLoot { registry, target, spec }) =>
        {
            let added: Result<(), String> = loading().and_then(|()| crate::content::loot::add(registry, target, spec));

            crate::content::loot::push(false);
            unit(added)
        }

        HostCall::Content(ContentRequest::SetChestLoot { table, spec }) =>
        {
            unit(loading().and_then(|()| crate::content::loot::addChest(table, spec)))
        }

        HostCall::Content(ContentRequest::AddConditionalPool { target, id, pool }) =>
        {
            let added: Result<(), String> = loading().and_then(|()| crate::content::loot::addPool(target, id, pool));

            crate::content::loot::push(false);
            unit(added)
        }

        HostCall::Content(ContentRequest::RegisterLootCondition { name, label, handler }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id|
            {
                crate::content::loot::registerRule(runtime.m_index, id, label, RuleKind::Condition, handler)
            }))
        }
        HostCall::Content(ContentRequest::RegisterLootCount { name, label, handler }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id|
            {
                crate::content::loot::registerRule(runtime.m_index, id, label, RuleKind::Count, handler)
            }))
        }

        HostCall::Content(ContentRequest::RegisterMachine { name, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&name.to_string()))
                .and_then(|id| crate::content::machines::register(id, spec)),
        ),
        HostCall::Content(ContentRequest::AddMachineRecipe { machine, recipe }) =>
        {
            unit(loading().and_then(|()| crate::content::machines::addRecipe(&machine, recipe)))
        }
        HostCall::Content(ContentRequest::AddWorkstation { machine, item }) =>
        {
            unit(loading().and_then(|()| crate::content::machines::addWorkstation(&machine, item)))
        }
        HostCall::Content(ContentRequest::RegisterAccessorySlot { name, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&name.to_string()))
                .and_then(|id| crate::content::accessories::register(id, spec)),
        ),
        HostCall::Worldgen(WorldgenRequest::RegisterFeature { name, spec, handler }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id|
            {
                crate::content::features::register(runtime.id(), Some(runtime.m_index), id, spec, handler)
            }))
        }
        HostCall::Worldgen(WorldgenRequest::Features) => HostReply::Features(crate::content::features::list()),
        HostCall::Worldgen(WorldgenRequest::BiomeSpawns { biome, kind }) =>
        {
            ok(crate::biomes::spawns(&biome, kind), HostReply::SpawnEntries)
        }
        HostCall::Content(ContentRequest::AddMachineJobRules { machine, job, rules }) =>
        {
            unit(loading().and_then(|()| crate::content::machines::addJob(&machine, job, rules)))
        }
        HostCall::Content(ContentRequest::RegisterFluidContainer { spec }) =>
        {
            unit(registering().and_then(|()| crate::content::fluids::addContainer(spec)))
        }
        HostCall::Inventory(InventoryRequest::ContainedFluid { stack }) =>
        {
            HostReply::Fluid(crate::content::fluids::contained(&stack))
        }
        HostCall::Inventory(InventoryRequest::FillContainer { stack, fluid }) =>
        {
            HostReply::Swap(crate::content::fluids::fill(&stack, &fluid))
        }
        HostCall::Inventory(InventoryRequest::DrainContainer { stack }) =>
        {
            HostReply::Swap(crate::content::fluids::drain(&stack))
        }
        HostCall::Inventory(InventoryRequest::FluidContainers) =>
        {
            HostReply::FluidContainers(crate::content::fluids::containers())
        }
        HostCall::Content(ContentRequest::AddMachineFuel { machine, fuel }) =>
        {
            unit(loading().and_then(|()| crate::content::machines::addFuel(&machine, fuel)))
        }
        HostCall::Inventory(InventoryRequest::MatchMachineJob { machine, inputs, fluids }) =>
        {
            HostReply::Job(crate::content::machines::matchJob(&machine, &inputs, &fluids))
        }
        HostCall::Inventory(InventoryRequest::MachineFuel { machine, stack, furnace }) =>
        {
            HostReply::Int(crate::content::machines::fuel(&machine, &stack, furnace))
        }

        HostCall::Registry(RegistryRequest::TagValues { registry, tag }) =>
        {
            HostReply::Ids(tags::values(registry, &tag))
        }
        HostCall::Registry(RegistryRequest::TagsOf { registry, id }) => HostReply::Ids(tags::of(registry, &id)),

        HostCall::Worldgen(WorldgenRequest::RegisterWorldgen { name, spec }) =>
        {
            // abi 2 never checked the name, one outside the namespace still registers under the block's path
            let fallback: Result<Identifier, String> =
                Identifier::new(&owner, &format!("worldgen/{}", spec.block.path())).map_err(|error| error.to_string());
            unit(registering().and(runtime.own(&name).or(fallback)).map(|id| data::registerWorldgen(&owner, id, spec)))
        }

        HostCall::Meta(MetaRequest::DefineConfigOptions { options }) =>
        {
            unit(runtime.m_config.borrow_mut().defineOptions(options))
        }
        HostCall::Meta(MetaRequest::SetConfigValue { key, value }) =>
        {
            unit(runtime.m_config.borrow_mut().set(&key, &value).map(|_| ()))
        }
        HostCall::Meta(MetaRequest::ConfigValue { key }) =>
        {
            HostReply::Text(runtime.m_config.borrow().value(&key).unwrap_or_default())
        }
        HostCall::Meta(MetaRequest::Translate { key }) =>
        {
            HostReply::Text(game::locale::StatCollector::translateToLocal(&key))
        }

        HostCall::Registry(RegistryRequest::Resolve { registry, id }) =>
        {
            ok(resolve(registry, &id).ok_or_else(|| format!("{id} is not registered")), HostReply::Int)
        }

        HostCall::Registry(RegistryRequest::NameOf { registry, numeric }) => HostReply::Id(nameOf(registry, numeric)),
        HostCall::Worldgen(WorldgenRequest::RegisterTemplate { name, spec }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::content::structure_templates::register(id, spec)),
        ),
        HostCall::Net(NetRequest::RegisterConfigTask { name, handler }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::configtasks::register(runtime.m_index, id, handler)),
        ),
        HostCall::Content(ContentRequest::RegisterComponent { name, synced }) => unit(
            registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::network::stacks::register(id, synced)),
        ),
        HostCall::Stored(StoredRequest::Declare { name, spec }) => unit(
            registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::attach::declare(&owner, id, spec)),
        ),
        HostCall::Registry(RegistryRequest::RegistryList { registry }) =>
        {
            HostReply::Ids(registry::with(|registries| registries.list(registry)))
        }
        HostCall::Registry(RegistryRequest::CreateRegistry { name, synced }) => unit(
            registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::content::custom::create(id, synced)),
        ),
        HostCall::Registry(RegistryRequest::AddRegistryEntry { registry, name, value }) => unit(
            loading().and_then(|()| runtime.own(&name)).and_then(|id| crate::content::custom::add(registry, id, value)),
        ),
        HostCall::Registry(RegistryRequest::RegistryEntries { registry }) =>
        {
            HostReply::Entries(crate::content::custom::entries(&registry))
        }
        HostCall::Registry(RegistryRequest::AddRegistryTag { registry, tag, entries, replace }) =>
        {
            unit(loading().map(|()|
            {
                tags::addEntries(tags::Scope::Custom(registry), tag, entries, replace);
                tags::push();
            }))
        }
        HostCall::Registry(RegistryRequest::RegistryTagValues { registry, tag }) =>
        {
            HostReply::Ids(tags::values(tags::Scope::Custom(registry), &tag))
        }
        HostCall::Registry(RegistryRequest::CreateDataMap { name, target, synced }) => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::content::custom::createMap(id, target, synced)),
        ),
        HostCall::Registry(RegistryRequest::SetDataMapValues { map, values, remove, replace }) =>
        {
            unit(loading().map(|()|
            {
                crate::content::custom::setValues(map, None, values, remove, replace);
            }))
        }
        HostCall::Registry(RegistryRequest::DataMapValue { map, id }) =>
        {
            HostReply::Entries(crate::content::custom::value(&map, &id).map(|value| (id, value)).into_iter().collect())
        }
        HostCall::Registry(RegistryRequest::DataMapEntries { map }) =>
        {
            HostReply::Entries(crate::content::custom::mapEntries(&map))
        }
        HostCall::Content(ContentRequest::RegisterDamageType { name, spec }) =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::damage::register(id, spec)))
        }
        HostCall::Registry(RegistryRequest::ResolveState { state }) =>
        {
            ok(resolveState(&state), |(block, meta)| HostReply::Size(block, meta))
        }
        HostCall::Registry(RegistryRequest::DescribeState { block, meta }) =>
        {
            HostReply::State(crate::states::describe(block, meta))
        }
        HostCall::Meta(MetaRequest::IsClient) => HostReply::Bool(crate::side::isClient()),
        HostCall::Meta(MetaRequest::IsLoaded { modid }) =>
        {
            HostReply::Bool(state::byId(&modid).is_some_and(|other| other.isActive()))
        }
        HostCall::Meta(MetaRequest::ModName { modid }) => HostReply::Text(modName(&modid)),
        other => return Err(Box::new(other)),
    };
    Ok(reply)
}

fn resolveState(state: &StateData) -> Result<(i32, i32), String>
{
    crate::states::resolve(state).ok_or_else(|| format!("{} is not a registered block", state.block))
}

fn modName(modid: &str) -> String
{
    match modid
    {
        MINECRAFT => VANILLA_NAME.to_owned(),
        _ => state::byId(modid).map(|other| other.m_manifest.displayName().to_owned()).unwrap_or_default(),
    }
}
