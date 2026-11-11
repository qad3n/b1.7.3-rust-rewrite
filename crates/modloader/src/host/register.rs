use super::{ok, unit};
use crate::content::{data, tags};
use crate::registry::{self, RegistryError, TileEntityEntry};
use crate::state::{self, ModRuntime, Phase};
use mod_protocol::Identifier;
use mod_protocol::abi::{HostCall, HostReply, RegistryKind};
use mod_protocol::identifier::MINECRAFT;
use std::rc::Rc;

const VANILLA_NAME: &str = "Minecraft";

fn registering() -> Result<(), String>
{
    match state::phase()
    {
        Phase::Register => Ok(()),
        phase => Err(format!("content registers during the register phase, the loader is in {phase:?}")),
    }
}

fn resolve(kind: RegistryKind, id: &Identifier) -> Option<i32>
{
    registry::with(|registries| match kind
    {
        RegistryKind::Block => registries.blockId(id),
        RegistryKind::Item => registries.itemId(id),
        RegistryKind::Entity => registries.m_entities.numeric(id),
        RegistryKind::TileEntity => registries.m_tileEntities.numeric(id),
        RegistryKind::Recipe => registries.m_recipes.numeric(id),
        RegistryKind::Biome => registries.m_biomes.numeric(id),
        RegistryKind::Dimension => registries.m_dimensions.numeric(id),
    })
}

fn nameOf(kind: RegistryKind, numeric: i32) -> Option<Identifier>
{
    registry::with(|registries| match kind
    {
        RegistryKind::Block => registries.m_blocks.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Item => registries.m_items.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Entity => registries.m_entities.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::TileEntity => registries.m_tileEntities.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Recipe => registries.m_recipes.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Biome => registries.m_biomes.byNumeric(numeric).map(|row| row.m_id.clone()),
        RegistryKind::Dimension => registries.m_dimensions.byNumeric(numeric).map(|row| row.m_id.clone()),
    })
}

fn registryError(error: RegistryError) -> String { error.to_string() }

// registration, lookups and loader queries, anything else goes on to the world handler
pub(crate) fn handle(runtime: &Rc<ModRuntime>, call: HostCall) -> Result<HostReply, Box<HostCall>>
{
    let owner: String = runtime.id().to_owned();

    let reply: HostReply = match call
    {
        HostCall::RegisterBlock { name, spec } =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| data::registerBlock(id, &owner, spec)))
        }

        HostCall::RegisterItem { name, spec } =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| data::registerItem(id, &owner, spec)))
        }

        HostCall::RegisterRecipe { name, spec } =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| data::registerRecipe(id, &owner, spec)))
        }

        HostCall::TileModel { tile, model } =>
        {
            unit(registering().and_then(|()| crate::tiles::registerModel(runtime.m_index, tile, model)))
        }

        HostCall::RegisterTileEntity { name, spec } =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id|
            {
                crate::tiles::define(id.clone(), runtime.m_index, spec);

                registry::with(|registries|
                {
                    let entry: TileEntityEntry = registry::TileEntityEntry { m_saveId: id.to_string(), m_spec: None };
                    registries.m_tileEntities.register(id, &owner, entry).map_err(registryError)
                })
            }))
        }

        HostCall::RegisterEntity { name, spec } => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::entities::register(id, runtime.m_index, spec)),
        ),
        HostCall::RemoveRecipe { id } => unit(registering().and_then(|()| crate::lifecycle::queueRecipeRemoval(&id))),
        HostCall::ReplaceBlock { id, spec } => unit(registering().and_then(|()|
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
        HostCall::ModifyBlock { spec } => unit(registering().map(|()| crate::modify::queueBlock(&owner, spec))),
        HostCall::RegisterBlockStates { block, spec } => unit(
            registering()
                .and_then(|()| runtime.own(&block.to_string()))
                .and_then(|block| crate::states::register(block, &spec)),
        ),
        HostCall::ModifySpawns { spec } => unit(registering().map(|()| crate::modify::queueSpawn(&owner, spec))),

        HostCall::AddTag { registry, tag, values } =>
        {
            tags::add(registry, tag, values, false);
            tags::push();
            HostReply::Unit
        }

        HostCall::AddTagEntries { registry, tag, entries, replace } =>
        {
            tags::addEntries(registry, tag, entries, replace);
            tags::push();
            HostReply::Unit
        }

        HostCall::RegisterAchievement { name, spec } =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).map(|id| crate::content::progress::register(id, spec)))
        }

        HostCall::RegisterParticle { name, spec } => unit(
            registering().and_then(|()| runtime.own(&name)).map(|id| crate::content::particles::register(id, spec)),
        ),
        HostCall::RegisterEffect { name, spec } => unit(
            registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::content::effects::register(id, spec)),
        ),
        HostCall::RegisterEffectBehavior { name, spec, behavior } => unit(
            registering()
                .and_then(|()| runtime.own(&name))
                .and_then(|id| crate::content::effects::registerWith(id, spec, runtime.m_index, behavior)),
        ),
        HostCall::RegisterAttribute { name, spec } =>
        {
            unit(registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::attributes::register(id, spec)))
        }
        HostCall::RegisterFluid { name, spec } => unit(
            registering().and_then(|()| runtime.own(&name)).and_then(|id| crate::content::fluids::register(id, spec)),
        ),

        HostCall::SetLoot { registry, target, spec } =>
        {
            let added: Result<(), String> = crate::content::loot::add(registry, target, spec);

            crate::content::loot::push(false);
            unit(added)
        }

        HostCall::TagValues { registry, tag } => HostReply::Ids(tags::values(registry, &tag)),

        HostCall::RegisterWorldgen { name, spec } =>
        {
            // abi 2 never checked the name, one outside the namespace still registers under the block's path
            let fallback: Result<Identifier, String> =
                Identifier::new(&owner, &format!("worldgen/{}", spec.block.path())).map_err(|error| error.to_string());
            unit(registering().and(runtime.own(&name).or(fallback)).map(|id| data::registerWorldgen(&owner, id, spec)))
        }

        HostCall::DefineConfig { entries } => unit(runtime.m_config.borrow_mut().define(entries)),
        HostCall::DefineConfigOptions { options } => unit(runtime.m_config.borrow_mut().defineOptions(options)),
        HostCall::SetConfigValue { key, value } => unit(runtime.m_config.borrow_mut().set(&key, &value).map(|_| ())),
        HostCall::ConfigValue { key } => HostReply::Text(runtime.m_config.borrow().value(&key).unwrap_or_default()),
        HostCall::Translate { key } => HostReply::Text(game::locale::StatCollector::translateToLocal(&key)),

        HostCall::Resolve { registry, id } =>
        {
            ok(resolve(registry, &id).ok_or_else(|| format!("{id} is not registered")), HostReply::Int)
        }

        HostCall::NameOf { registry, numeric } => HostReply::Id(nameOf(registry, numeric)),
        HostCall::IsClient => HostReply::Bool(crate::side::isClient()),
        HostCall::IsLoaded { modid } => HostReply::Bool(state::byId(&modid).is_some_and(|other| other.isActive())),
        HostCall::ModName { modid } => HostReply::Text(modName(&modid)),
        other => return Err(Box::new(other)),
    };
    Ok(reply)
}

fn modName(modid: &str) -> String
{
    match modid
    {
        MINECRAFT => VANILLA_NAME.to_owned(),
        _ => state::byId(modid).map(|other| other.m_manifest.displayName().to_owned()).unwrap_or_default(),
    }
}
