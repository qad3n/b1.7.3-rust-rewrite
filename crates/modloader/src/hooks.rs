use crate::{content, events};
use common::nbt::NBTTagCompound;
use game::block::Block;
use game::entity::Entity;
use game::hooks::LoaderHooks;
use game::item::ItemStack;
use game::tileentity::TileEntity;
use game::world::World;
use mod_protocol::Identifier;
use mod_protocol::abi::{EventData, EventKind};
use std::rc::Rc;

struct Hooks;

// the game reaches mods only through these, installed once and only when mods exist
impl LoaderHooks for Hooks
{
    fn fire(&self, event: game::hooks::GameEvent<'_>) -> bool { crate::gameevents::fire(event) }

    fn component(&self, component: game::hooks::Component, block: i32) -> Option<bool>
    {
        let data = content::blockData(block)?;
        let spec = &data.m_spec;
        match component
        {
            game::hooks::Component::RedstoneConnect => spec.redstone_connect,
            game::hooks::Component::Replaceable => spec.replaceable,
            game::hooks::Component::Ladder => spec.ladder,
            game::hooks::Component::SolidSides => spec.solid_sides,
            game::hooks::Component::Leaves => spec.leaves.or_else(|| {
                Some(spec.template.as_ref().is_some_and(|template| *template == Identifier::minecraft("leaves")))
            }),
            game::hooks::Component::SustainsLeaves => spec.sustains_leaves,
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

    fn populate(&self, world: &Rc<World>, chunkX: i32, chunkZ: i32)
    {
        content::worldgen::populate(world, chunkX, chunkZ);
        crate::dimensions::populate(world, chunkX, chunkZ);
        if events::hasListeners(EventKind::ChunkPopulate)
        {
            let dimension: i32 = crate::host::context::dimension(world);
            crate::host::context::withWorld(world, || {
                events::post(EventKind::ChunkPopulate, EventData::Chunk { world: dimension, x: chunkX, z: chunkZ })
            });
        }
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
}

pub fn install()
{
    let hooks: &'static Hooks = Box::leak(Box::new(Hooks));
    game::hooks::install(hooks);
}
