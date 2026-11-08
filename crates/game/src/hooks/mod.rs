use crate::block::{Block, BlockClass};
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
    LivingAttack
    {
        entity: &'a Rc<Entity>, source: Option<&'a Rc<Entity>>, amount: &'a mut i32
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
    Ladder,
    SolidSides,
    Leaves,
    SustainsLeaves,
}

// the mod loader implements this, game code reaches it through the free functions below
pub trait LoaderHooks
{
    fn component(&self, _component: Component, _block: i32) -> Option<bool> { None }

    // true when a listener canceled the event
    fn fire(&self, _event: GameEvent<'_>) -> bool { false }

    fn fuel(&self, _stack: &Rc<ItemStack>) -> Option<i32> { None }

    fn maxHealth(&self, _entity: &Entity) -> Option<i32> { None }

    fn invisible(&self, _entity: &Entity) -> bool { false }

    fn canHarvest(&self, _block: &Rc<Block>, _stack: Option<&Rc<ItemStack>>) -> Option<bool> { None }

    fn strVsBlock(&self, _block: &Rc<Block>, _stack: Option<&Rc<ItemStack>>) -> Option<f32> { None }

    fn populate(&self, _world: &Rc<World>, _chunkX: i32, _chunkZ: i32) {}

    fn translate(&self, _key: &str) -> Option<String> { None }

    fn createTileEntity(&self, _id: &str) -> Option<Rc<TileEntity>> { None }

    fn worldProvider(&self, _dimension: i32) -> Option<Rc<crate::world::WorldProvider>> { None }

    fn createEntity(&self, _key: &str, _world: &Rc<World>) -> Option<Rc<Entity>> { None }

    fn entityKey(&self, _entity: &Entity) -> Option<(String, i32)> { None }

    fn writeEntityData(&self, _entity: &Rc<Entity>, _nbt: &NBTTagCompound) {}

    fn readEntityData(&self, _entity: &Rc<Entity>, _nbt: &NBTTagCompound) {}
}

thread_local! {
    static HOOKS: Cell<Option<&'static dyn LoaderHooks>> = const { Cell::new(None) };
}

pub fn install(hooks: &'static dyn LoaderHooks) { HOOKS.with(|slot| slot.set(Some(hooks))); }

pub fn active() -> Option<&'static dyn LoaderHooks> { HOOKS.with(Cell::get) }

pub fn component(component: Component, block: i32) -> Option<bool> { active()?.component(component, block) }

pub fn fire(event: GameEvent<'_>) -> bool { active().is_some_and(|hooks| hooks.fire(event)) }

// vanilla leaves plus every block a mod marks as leaves, for decay checks
pub fn isLeaves(block: i32) -> bool
{
    block == Block::leaves().m_blockID || component(Component::Leaves, block) == Some(true)
}

// vanilla logs plus every block a mod marks as keeping leaves alive
pub fn sustainsLeaves(block: i32) -> bool
{
    block == Block::wood().m_blockID || component(Component::SustainsLeaves, block) == Some(true)
}

pub fn spawnAllowed(entity: &Rc<Entity>, allowed: bool) -> bool
{
    let mut allowed: bool = allowed;
    fire(GameEvent::LivingSpawnCheck { entity, allowed: &mut allowed });
    allowed
}

pub fn fuel(stack: &Rc<ItemStack>) -> Option<i32> { active()?.fuel(stack) }

// the health a mod lets an entity heal to, vanilla caps every living entity at 20
pub fn maxHealth(entity: &Entity) -> i32
{
    active().and_then(|hooks| hooks.maxHealth(entity)).unwrap_or(VANILLA_MAX_HEALTH)
}

pub const VANILLA_MAX_HEALTH: i32 = 20;

pub fn invisible(entity: &Entity) -> bool { active().is_some_and(|hooks| hooks.invisible(entity)) }

pub fn canHarvest(block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<bool>
{
    active()?.canHarvest(block, stack)
}

pub fn strVsBlock(block: &Rc<Block>, stack: Option<&Rc<ItemStack>>) -> Option<f32>
{
    active()?.strVsBlock(block, stack)
}

pub fn populate(world: &Rc<World>, chunkX: i32, chunkZ: i32)
{
    if let Some(hooks) = active()
    {
        hooks.populate(world, chunkX, chunkZ);
    }
}

pub fn translate(key: &str) -> Option<String> { active()?.translate(key) }

pub fn createTileEntity(id: &str) -> Option<Rc<TileEntity>> { active()?.createTileEntity(id) }

pub fn worldProvider(dimension: i32) -> Option<Rc<crate::world::WorldProvider>> { active()?.worldProvider(dimension) }

pub fn createEntity(key: &str, world: &Rc<World>) -> Option<Rc<Entity>> { active()?.createEntity(key, world) }

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

// mod content reuses a vanilla constructor while the next constructed object takes a derived class
macro_rules! class_override {
    ($slot:ident, $with:ident, $take:ident, $class:ty) => {
        thread_local! {
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
