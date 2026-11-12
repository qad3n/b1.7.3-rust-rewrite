use crate::block::{Block, BlockClass};
use crate::damage::DamageSource;
use crate::entity::{Entity, EntityClass};
use crate::item::{ItemClass, ItemStack};
use crate::tileentity::{TileEntity, TileEntityClass};
use crate::world::World;
use common::nbt::NBTTagCompound;
use std::cell::Cell;
use std::rc::Rc;

// one game moment a mod can observe, cancel or change, the loader turns it into a catalog event
pub enum GameEvent<'a>
{
    WorldSave
    {
        world: &'a Rc<World>
    },

    ChunkLoad
    {
        world: &'a Rc<World>, x: i32, z: i32
    },

    ChunkUnload
    {
        world: &'a Rc<World>, x: i32, z: i32
    },

    ChunkGenerate
    {
        world: &'a Rc<World>, x: i32, z: i32
    },

    BlockPlace
    {
        world: &'a Rc<World>,
        x: i32,
        y: i32,
        z: i32,
        block: i32,
        meta: i32,
        player: Option<&'a Rc<Entity>>,
    },

    BlockBreak
    {
        world: &'a Rc<World>,
        x: i32,
        y: i32,
        z: i32,
        block: i32,
        meta: i32,
        player: Option<&'a Rc<Entity>>,
    },

    BlockActivate
    {
        world: &'a Rc<World>, x: i32, y: i32, z: i32, block: i32, player: &'a Rc<Entity>
    },

    NeighborChange
    {
        world: &'a Rc<World>, x: i32, y: i32, z: i32, block: i32, neighbor: i32
    },

    ExplosionAffect
    {
        world: &'a Rc<World>, x: i32, y: i32, z: i32, block: i32
    },

    HarvestDrops
    {
        world: &'a Rc<World>, x: i32, y: i32, z: i32, block: i32, meta: i32
    },

    ItemUse
    {
        world: &'a Rc<World>, player: &'a Rc<Entity>, stack: &'a Rc<ItemStack>
    },

    ItemUseOnBlock
    {
        world: &'a Rc<World>,
        player: &'a Rc<Entity>,
        stack: &'a Rc<ItemStack>,
        x: i32,
        y: i32,
        z: i32,
        face: i32,
    },

    ItemDestroyed
    {
        player: &'a Rc<Entity>, stack: &'a ItemStack
    },

    ItemPickup
    {
        player: &'a Rc<Entity>, item: &'a Rc<Entity>, stack: &'a Rc<ItemStack>
    },

    ItemToss
    {
        player: &'a Rc<Entity>, stack: &'a Rc<ItemStack>
    },

    EntityJoinWorld
    {
        world: &'a Rc<World>, entity: &'a Rc<Entity>
    },

    EntityTick
    {
        entity: &'a Rc<Entity>
    },

    EntityRemove
    {
        world: &'a Rc<World>, entity: &'a Rc<Entity>
    },

    LivingAttack
    {
        entity: &'a Rc<Entity>, damage: &'a DamageSource<'a>, amount: &'a mut i32
    },

    LivingDeath
    {
        entity: &'a Rc<Entity>, source: Option<&'a Rc<Entity>>
    },

    LivingDrops
    {
        entity: &'a Rc<Entity>, source: Option<&'a Rc<Entity>>
    },

    LivingSpawnCheck
    {
        entity: &'a Rc<Entity>, allowed: &'a mut bool
    },

    PlayerSleep
    {
        player: &'a Rc<Entity>, x: i32, y: i32, z: i32
    },

    PlayerBreakSpeed
    {
        player: &'a Rc<Entity>, block: &'a Rc<Block>, speed: &'a mut f32
    },

    PlayerCanHarvest
    {
        player: &'a Rc<Entity>, block: &'a Rc<Block>, allowed: &'a mut bool
    },

    ItemCrafted
    {
        player: &'a Rc<Entity>, stack: &'a Rc<ItemStack>
    },

    ItemSmelted
    {
        player: &'a Rc<Entity>, stack: &'a Rc<ItemStack>
    },

    BucketFill
    {
        world: &'a Rc<World>, player: &'a Rc<Entity>, stack: &'a Rc<ItemStack>
    },
}

// block properties a mod declares for its own blocks, vanilla blocks keep their hardcoded answers
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Component
{
    RedstoneConnect,
    Replaceable,
    SolidSides,
}

// the mod loader implements this, game code reaches it through the free functions below
pub trait LoaderHooks
{
    fn component(&self, _component: Component, _block: i32) -> Option<bool> { None }

    // true when a listener canceled the event
    fn fire(&self, _event: GameEvent<'_>) -> bool { false }

    fn fuel(&self, _stack: &Rc<ItemStack>) -> Option<i32> { None }

    fn canHarvest(&self, _block: &Rc<Block>, _stack: Option<&Rc<ItemStack>>) -> Option<bool> { None }

    fn strVsBlock(&self, _block: &Rc<Block>, _stack: Option<&Rc<ItemStack>>) -> Option<f32> { None }

    // a mod's own seat height for riders of this mount
    fn seat(&self, _mount: &Entity) -> Option<f64> { None }

    fn populate(&self, _world: &Rc<World>, _chunkX: i32, _chunkZ: i32) {}

    // a vanilla structure finished building, named by key with its inclusive corners
    fn structure(&self, _world: &Rc<World>, _key: &str, _min: [i32; 3], _max: [i32; 3]) {}

    fn translate(&self, _key: &str) -> Option<String> { None }

    fn createTileEntity(&self, _id: &str) -> Option<Rc<TileEntity>> { None }

    fn worldProvider(&self, _dimension: i32) -> Option<Rc<crate::world::WorldProvider>> { None }

    fn createEntity(&self, _key: &str, _world: &Rc<World>) -> Option<Rc<Entity>> { None }

    // a mod dimension may name what can spawn at a spot instead of its biome, as Forge's getPossibleCreatures
    fn ownsSpawns(&self, _world: &World) -> bool { false }

    fn leafDecayStride(&self, _block: i32) -> Option<i32> { None }

    fn lootCondition(&self, _rule: u32, _context: &crate::loot::LootContext, _random: &jvm::random::Random) -> bool
    {
        false
    }

    fn lootCount(&self, _rule: u32, _context: &crate::loot::LootContext, _random: &jvm::random::Random) -> i32 { 0 }

    fn spawnList(
        &self,
        _world: &Rc<World>,
        _kind: crate::worldgen::EnumCreatureType,
        _x: i32,
        _y: i32,
        _z: i32,
    ) -> Option<Vec<crate::worldgen::SpawnListEntry>>
    {
        None
    }

    fn entityKey(&self, _entity: &Entity) -> Option<(String, i32)> { None }

    fn writeEntityData(&self, _entity: &Rc<Entity>, _nbt: &NBTTagCompound) {}

    fn readEntityData(&self, _entity: &Rc<Entity>, _nbt: &NBTTagCompound) {}

    fn itemName(&self, _item: i32) -> Option<String> { None }

    // false while the loader is installed but the open world is vanilla
    fn savesNames(&self) -> bool { false }

    fn itemByName(&self, _name: &str) -> Option<i32> { None }

    // the registry name and named properties a stored block id and metadata save as
    fn blockName(&self, _block: i32, _meta: i32) -> Option<(String, Vec<(String, String)>)> { None }

    // the id and metadata a saved name loads as, given what was stored with it
    fn blockByName(&self, _name: &str, _properties: &[(String, String)], _block: i32, _meta: i32)
    -> Option<(i32, i32)>
    {
        None
    }

    // the current name of a saved registry name a mod has since renamed
    fn rename(&self, _name: &str) -> Option<String> { None }

    // a game rule took a new value, a server passes it on to modded clients
    fn ruleChanged(&self, _name: &str, _value: &str) {}

    // whether a damage type is in a damage type tag
    fn damageTagged(&self, _kind: &str, _tag: &str) -> bool { false }

    // whether any mod biome joins overworld generation
    fn overworldBiomes(&self) -> bool { false }

    // a mod biome that takes this climate of the overworld instead of beta's biome
    fn overworldBiome(&self, _temperature: f64, _downfall: f64) -> Option<Rc<crate::worldgen::BiomeGenBase>> { None }
}

thread_local!
{
    static HOOKS: Cell<Option<&'static dyn LoaderHooks>> = const { Cell::new(None) };
}

pub fn install(hooks: &'static dyn LoaderHooks) { HOOKS.with(|slot| slot.set(Some(hooks))); }

pub fn active() -> Option<&'static dyn LoaderHooks> { HOOKS.with(Cell::get) }

pub fn component(component: Component, block: i32) -> Option<bool> { active()?.component(component, block) }

pub fn fire(event: GameEvent<'_>) -> bool { active().is_some_and(|hooks| hooks.fire(event)) }

pub fn spawnAllowed(entity: &Rc<Entity>, allowed: bool) -> bool
{
    let mut allowed: bool = allowed;
    fire(GameEvent::LivingSpawnCheck { entity, allowed: &mut allowed });
    allowed
}

pub fn fuel(stack: &Rc<ItemStack>) -> Option<i32> { active()?.fuel(stack) }

pub fn canHarvest(block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<bool>
{
    active()?.canHarvest(block, stack)
}

pub fn strVsBlock(block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<f32>
{
    active()?.strVsBlock(block, stack)
}

pub fn seat(mount: &Entity) -> Option<f64> { active()?.seat(mount) }

pub fn populate(world: &Rc<World>, chunkX: i32, chunkZ: i32)
{
    if let Some(hooks) = active()
    {
        hooks.populate(world, chunkX, chunkZ);
    }
}

pub fn structure(world: &Rc<World>, key: &str, min: [i32; 3], max: [i32; 3])
{
    if let Some(hooks) = active()
    {
        hooks.structure(world, key, min, max);
    }
}

pub fn translate(key: &str) -> Option<String> { active()?.translate(key) }

pub fn createTileEntity(id: &str) -> Option<Rc<TileEntity>> { active()?.createTileEntity(id) }

pub fn worldProvider(dimension: i32) -> Option<Rc<crate::world::WorldProvider>> { active()?.worldProvider(dimension) }

pub fn createEntity(key: &str, world: &Rc<World>) -> Option<Rc<Entity>> { active()?.createEntity(key, world) }

pub fn ownsSpawns(world: &World) -> bool { active().is_some_and(|hooks| hooks.ownsSpawns(world)) }

pub fn leafDecayStride(block: i32) -> Option<i32> { active()?.leafDecayStride(block) }

// only mod loot tables carry rules, so without the loader no entry asks
pub fn lootCondition(rule: u32, context: &crate::loot::LootContext, random: &jvm::random::Random) -> bool
{
    active().is_some_and(|hooks| hooks.lootCondition(rule, context, random))
}

pub fn lootCount(rule: u32, context: &crate::loot::LootContext, random: &jvm::random::Random) -> i32
{
    active().map_or(0, |hooks| hooks.lootCount(rule, context, random))
}

pub fn spawnList(
    world: &Rc<World>,
    kind: crate::worldgen::EnumCreatureType,
    x: i32,
    y: i32,
    z: i32,
) -> Option<Vec<crate::worldgen::SpawnListEntry>>
{
    active()?.spawnList(world, kind, x, y, z)
}

pub fn entityKey(entity: &Entity) -> Option<(String, i32)> { active()?.entityKey(entity) }

pub fn writeEntityData(entity: &Rc<Entity>, nbt: &NBTTagCompound)
{
    if let Some(hooks) = active()
    {
        hooks.writeEntityData(entity, nbt);
    }
}

pub fn readEntityData(entity: &Rc<Entity>, nbt: &NBTTagCompound)
{
    if let Some(hooks) = active()
    {
        hooks.readEntityData(entity, nbt);
    }
}

pub fn blockName(block: i32, meta: i32) -> Option<(String, Vec<(String, String)>)> { active()?.blockName(block, meta) }

pub fn blockByName(name: &str, properties: &[(String, String)], block: i32, meta: i32) -> Option<(i32, i32)>
{
    active()?.blockByName(name, properties, block, meta)
}

pub fn overworldBiomes() -> bool { active().is_some_and(|hooks| hooks.overworldBiomes()) }

pub fn overworldBiome(temperature: f64, downfall: f64) -> Option<Rc<crate::worldgen::BiomeGenBase>>
{
    active()?.overworldBiome(temperature, downfall)
}

pub fn damageTagged(kind: &str, tag: &str) -> bool { active().is_some_and(|hooks| hooks.damageTagged(kind, tag)) }

pub fn ruleChanged(name: &str, value: &str)
{
    if let Some(hooks) = active()
    {
        hooks.ruleChanged(name, value);
    }
}

// every saved registry name passes here before lookup so renamed content keeps loading
pub fn renamed(name: &str) -> String
{
    active().and_then(|hooks| hooks.rename(name)).unwrap_or_else(|| name.to_owned())
}

const ITEM_NAME: &str = "loader:id";

pub fn writeItemName(item: i32, nbt: &NBTTagCompound)
{
    if let Some(name) = active().filter(|hooks| hooks.savesNames()).and_then(|hooks| hooks.itemName(item))
    {
        nbt.setString(ITEM_NAME, Some(jvm::string::JString::from(name.as_str())));
    }
}

pub fn itemName(item: i32) -> Option<String> { active()?.itemName(item) }

pub fn itemByName(nbt: &NBTTagCompound) -> Option<i32>
{
    let name: String = nbt.getString(ITEM_NAME)?.to_string();
    active()?.itemByName(&name)
}

// with mods loaded chunks save blocks by name, a vanilla game keeps the McRegion arrays
pub fn savesBlockNames() -> bool { active().is_some_and(|hooks| hooks.savesNames()) }

// mod content reuses a vanilla constructor while the next constructed object takes a derived class
macro_rules! class_override
{
    ($slot:ident, $with:ident, $take:ident, $class:ty) =>
    {
        thread_local!
        {
            static $slot: Cell<Option<&'static $class>> = const { Cell::new(None) };
        }

        pub fn $with<R>(class: &'static $class, build: impl FnOnce() -> R) -> R
        {
            $slot.with(|slot| slot.set(Some(class)));

            let result: R = build();

            $slot.with(|slot| slot.set(None));
            result
        }

        pub fn $take(class: &'static $class) -> &'static $class { $slot.with(Cell::take).unwrap_or(class) }
    };
}

class_override!(NEXT_BLOCK_CLASS, withBlockClass, blockClass, BlockClass);
class_override!(NEXT_ITEM_CLASS, withItemClass, itemClass, ItemClass);
class_override!(NEXT_ENTITY_CLASS, withEntityClass, entityClass, EntityClass);
class_override!(NEXT_TILE_ENTITY_CLASS, withTileEntityClass, tileEntityClass, TileEntityClass);
