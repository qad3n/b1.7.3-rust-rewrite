use crate::content::BlockData;
use crate::{content, events};
use common::nbt::NBTTagCompound;
use game::block::Block;
use game::entity::Entity;
use game::hooks::LoaderHooks;
use game::item::ItemStack;
use game::tileentity::TileEntity;
use game::world::World;
use mod_protocol::abi::{EventData, EventKind};
use std::rc::Rc;

struct Hooks;

// the game reaches mods only through these, installed once and only when mods exist
impl LoaderHooks for Hooks
{
    fn fire(&self, event: game::hooks::GameEvent<'_>) -> bool { crate::gameevents::fire(event) }

    fn component(&self, component: game::hooks::Component, block: i32) -> Option<bool>
    {
        let data: Rc<BlockData> = content::blockData(block)?;
        let spec = &data.m_spec;

        match component
        {
            game::hooks::Component::RedstoneConnect => spec.redstone_connect,
            game::hooks::Component::Replaceable => spec.replaceable,
            game::hooks::Component::SolidSides => spec.solid_sides,
        }
    }

    fn fuel(&self, stack: &Rc<ItemStack>) -> Option<i32> { content::fuel(stack.m_itemID.get()) }

    fn canHarvest(&self, block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<bool>
    {
        content::harvest::canHarvest(block, stack)
    }

    fn strVsBlock(&self, block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<f32>
    {
        content::harvest::strVsBlock(block, stack)
    }

    fn seat(&self, mount: &Entity) -> Option<f64> { crate::entities::seat(mount) }

    fn leafDecayStride(&self, block: i32) -> Option<i32> { crate::states::decayStride(block) }

    fn lootCondition(&self, rule: u32, context: &game::loot::LootContext, random: &jvm::random::Random) -> bool
    {
        content::loot::check(rule, context, random)
    }

    fn lootCount(&self, rule: u32, context: &game::loot::LootContext, random: &jvm::random::Random) -> i32
    {
        content::loot::count(rule, context, random)
    }

    fn populate(&self, world: &Rc<World>, chunkX: i32, chunkZ: i32)
    {
        content::worldgen::populate(world, chunkX, chunkZ);
        content::features::populate(world, chunkX, chunkZ);
        content::retrogen::record(world, chunkX, chunkZ);
        crate::dimensions::populate(world, chunkX, chunkZ);

        if events::hasListeners(EventKind::ChunkPopulate)
        {
            let dimension: i32 = crate::host::context::dimension(world);

            crate::host::context::withWorld(world, ||
            {
                events::post(EventKind::ChunkPopulate, EventData::Chunk { world: dimension, x: chunkX, z: chunkZ })
            });
        }
    }

    fn structure(&self, world: &Rc<World>, key: &str, min: [i32; 3], max: [i32; 3])
    {
        crate::structures::addVanilla(world, key, min, max);
    }

    fn translate(&self, key: &str) -> Option<String> { content::lang::translate(key) }

    fn createTileEntity(&self, id: &str) -> Option<Rc<TileEntity>>
    {
        crate::tiles::createById(id).or_else(|| crate::remap::missingTile(id))
    }

    fn worldProvider(&self, dimension: i32) -> Option<Rc<game::world::WorldProvider>>
    {
        crate::worlds::provider(dimension)
    }

    fn createEntity(&self, key: &str, world: &Rc<World>) -> Option<Rc<Entity>>
    {
        crate::entities::createByKey(key, world).or_else(|| crate::remap::missingEntity(key, world))
    }

    fn ownsSpawns(&self, world: &World) -> bool
    {
        crate::dimensions::hasSpawnList(crate::host::context::dimension(world))
    }

    fn spawnList(
        &self,
        world: &Rc<World>,
        kind: game::worldgen::EnumCreatureType,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<Vec<game::worldgen::SpawnListEntry>>
    {
        let kind: mod_protocol::spec::CreatureKind = match kind
        {
            game::worldgen::EnumCreatureType::Monster => mod_protocol::spec::CreatureKind::Monster,
            game::worldgen::EnumCreatureType::Creature => mod_protocol::spec::CreatureKind::Creature,
            game::worldgen::EnumCreatureType::WaterCreature => mod_protocol::spec::CreatureKind::Water,
        };
        let entries: Vec<mod_protocol::abi::SpawnEntry> =
            crate::dimensions::spawnList(world, kind, mod_protocol::abi::BlockPos { x, y, z })?;
        Some(crate::modify::spawnEntries(&entries))
    }

    fn entityKey(&self, entity: &Entity) -> Option<(String, i32)>
    {
        if let Some(key) = crate::remap::missingEntityKey(entity)
        {
            return Some((key, 0));
        }

        Some((crate::entities::keyOf(entity)?, crate::entities::numericOf(entity)?))
    }

    fn writeEntityData(&self, entity: &Rc<Entity>, nbt: &NBTTagCompound) { crate::entities::writeData(entity, nbt); }

    fn readEntityData(&self, entity: &Rc<Entity>, nbt: &NBTTagCompound) { crate::entities::readData(entity, nbt); }

    fn savesNames(&self) -> bool { crate::play::active() }

    fn itemName(&self, item: i32) -> Option<String>
    {
        crate::registry::with(|registries| registries.m_items.byNumeric(item).map(|row| row.m_id.to_string()))
            .or_else(|| crate::remap::missingName("item", item).map(|id| id.to_string()))
    }

    fn itemByName(&self, name: &str) -> Option<i32>
    {
        let id: mod_protocol::Identifier = crate::remap::rename(&mod_protocol::Identifier::parse(name).ok()?);
        crate::registry::with(|registries| registries.itemId(&id))
    }

    fn blockName(&self, block: i32, meta: i32) -> Option<(String, Vec<(String, String)>)>
    {
        crate::palette::name(block, meta)
    }

    fn blockByName(&self, name: &str, properties: &[(String, String)], block: i32, meta: i32) -> Option<(i32, i32)>
    {
        crate::palette::resolve(name, properties, block, meta)
    }

    fn ruleChanged(&self, name: &str, value: &str) { crate::gamerules::changed(name, value); }

    fn statChanged(&self, player: &Rc<Entity>) { crate::content::progress::syncOwn(player); }

    fn overworldBiomes(&self) -> bool { crate::biomes::hasOverworld() }

    fn overworldBiome(&self, temperature: f64, downfall: f64) -> Option<Rc<game::worldgen::BiomeGenBase>>
    {
        crate::biomes::overworld(temperature, downfall)
    }

    fn damageTagged(&self, kind: &str, tag: &str) -> bool
    {
        let (Ok(kind), Ok(tag)) = (mod_protocol::Identifier::parse(kind), mod_protocol::Identifier::parse(tag))
        else
        {
            return false;
        };

        crate::content::tags::contains(mod_protocol::abi::RegistryKind::DamageType, &tag, &kind)
    }

    fn rename(&self, name: &str) -> Option<String>
    {
        let id: mod_protocol::Identifier = mod_protocol::Identifier::parse(name).ok()?;
        let renamed: mod_protocol::Identifier = crate::remap::rename(&id);
        (renamed != id).then(|| renamed.to_string())
    }
}

pub fn install()
{
    let hooks: &'static Hooks = Box::leak(Box::new(Hooks));
    game::hooks::install(hooks);
}
