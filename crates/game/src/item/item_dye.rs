use crate::block::{Block, block_cloth, block_crops, block_sapling};
use crate::entity::{Entity, entity_sheep};
use crate::item::{ITEM, Item, ItemClass, ItemStack};
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

pub const DYE_COLORS: [&str; 16] = [
    "black",
    "red",
    "green",
    "brown",
    "blue",
    "purple",
    "cyan",
    "silver",
    "gray",
    "pink",
    "lime",
    "yellow",
    "lightBlue",
    "magenta",
    "orange",
    "white",
];

pub const COLORS: [i32; 16] = [
    1973019, 11743532, 3887386, 5320730, 2437522, 8073150, 2651799, 2651799, 4408131, 14188952, 4312372, 14602026,
    6719955, 12801229, 15435844, 15790320,
];

fn onItemUse(
    _this: &Rc<Item>,
    stack: &Rc<ItemStack>,
    _player: &Rc<Entity>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    _face: i32,
) -> bool
{
    if stack.getItemDamage() != 15
    {
        return false;
    }
    let i: i32 = world.getBlockId(x, y, z);
    if i == Block::sapling().m_blockID
    {
        if !world.m_multiplayerWorld.get()
        {
            block_sapling::growTree(&Block::sapling(), world, x, y, z, &world.m_rand);
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        }
        return true;
    }
    if i == Block::crops().m_blockID
    {
        if !world.m_multiplayerWorld.get()
        {
            block_crops::fertilize(world, x, y, z);
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        }
        return true;
    }
    if i == Block::grass().m_blockID
    {
        if !world.m_multiplayerWorld.get()
        {
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
            let itemRand: Rc<Random> = Item::itemRand();
            'label53: for j in 0..128
            {
                let mut k: i32 = x;
                let mut l: i32 = y + 1;
                let mut i1: i32 = z;
                for _ in 0..j / 16
                {
                    k += itemRand.nextIntBounded(3) - 1;
                    l += (itemRand.nextIntBounded(3) - 1) * itemRand.nextIntBounded(3) / 2;
                    i1 += itemRand.nextIntBounded(3) - 1;
                    if world.getBlockId(k, l - 1, i1) != Block::grass().m_blockID || world.isBlockNormalCube(k, l, i1)
                    {
                        continue 'label53;
                    }
                }
                if world.getBlockId(k, l, i1) == 0
                {
                    if itemRand.nextIntBounded(10) != 0
                    {
                        world.setBlockAndMetadataWithNotify(k, l, i1, Block::tallGrass().m_blockID, 1);
                    }
                    else if itemRand.nextIntBounded(3) != 0
                    {
                        world.setBlockWithNotify(k, l, i1, Block::plantYellow().m_blockID);
                    }
                    else
                    {
                        world.setBlockWithNotify(k, l, i1, Block::plantRed().m_blockID);
                    }
                }
            }
        }
        return true;
    }
    false
}

fn saddleEntity(_this: &Rc<Item>, stack: &Rc<ItemStack>, entity: &Rc<Entity>)
{
    if entity.instanceOf("EntitySheep")
    {
        let i: i32 = block_cloth::getBlockMetadataForItemMetadata(stack.getItemDamage());
        if !entity_sheep::getSheared(entity) && entity_sheep::getFleeceColor(entity) != i
        {
            entity_sheep::setFleeceColor(entity, i);
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        }
    }
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemDye",
    m_parent: Some(&ITEM),
    getIconFromDamage: |this, metadata| {
        let i: i32 = metadata;
        this.m_iconIndex.get() + i % 8 * 16 + i / 8
    },
    getItemNameIS: |this, stack| {
        let name: String = (ITEM.getItemName)(this).unwrap_or_else(|| "null".to_string());
        Some(format!("{name}.{}", DYE_COLORS[stack.getItemDamage() as usize]))
    },
    onItemUse,
    saddleEntity,
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item> { Item::construct(&CLASS, Box::new(()), id).setHasSubtypes(true).setMaxDamage(0) }
