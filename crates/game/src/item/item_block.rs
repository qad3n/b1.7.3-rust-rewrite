use crate::block::{Block, StepSound};
use crate::entity::Entity;
use crate::item::{ITEM, Item, ItemClass, ItemStack};
use crate::world::World;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct BlockItemExt
{
    m_blockID: i32,
}

pub fn blockID(this: &Item) -> i32 { this.ext::<BlockItemExt>().m_blockID }

fn onItemUse(
    this: &Rc<Item>,
    stack: &Rc<ItemStack>,
    player: &Rc<Entity>,
    world: &Rc<World>,
    mut x: i32,
    mut y: i32,
    mut z: i32,
    mut face: i32,
) -> bool
{
    let id: i32 = blockID(this);
    if world.getBlockId(x, y, z) == Block::snow().m_blockID
    {
        face = 0;
    }
    else
    {
        if face == 0
        {
            y -= 1;
        }
        if face == 1
        {
            y += 1;
        }
        if face == 2
        {
            z -= 1;
        }
        if face == 3
        {
            z += 1;
        }
        if face == 4
        {
            x -= 1;
        }
        if face == 5
        {
            x += 1;
        }
    }
    if stack.m_stackSize.get() == 0
    {
        return false;
    }
    if y == 127 && Block::byId(id).orNpe().m_blockMaterial.isSolid()
    {
        return false;
    }
    if world.canBlockBePlacedAt(id, x, y, z, false, face)
    {
        let block: Rc<Block> = Block::byId(id).orNpe();
        if world.setBlockAndMetadataWithNotify(x, y, z, id, this.getPlacedBlockMetadata(stack.getItemDamage()))
        {
            Block::byId(id).orNpe().onBlockPlaced(world, x, y, z, face);
            Block::byId(id).orNpe().onBlockPlacedBy(world, x, y, z, player);
            let sound: Rc<StepSound> = block.m_stepSound.get();
            world.playSoundEffect(
                (x as f32 + 0.5) as f64,
                (y as f32 + 0.5) as f64,
                (z as f32 + 0.5) as f64,
                &sound.getStepping(),
                (sound.getVolume() + 1.0) / 2.0,
                sound.getPitch() * 0.8,
            );
            stack.m_stackSize.set(stack.m_stackSize.get() - 1);
        }
        return true;
    }
    false
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemBlock",
    m_parent: Some(&ITEM),
    onItemUse,
    getItemNameIS: |this, _| Block::byId(blockID(this)).orNpe().getBlockName(),
    getItemName: |this| Block::byId(blockID(this)).orNpe().getBlockName(),
    ..ITEM
};

pub fn construct(class: &'static ItemClass, n: i32) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(class, Box::new(BlockItemExt { m_blockID: n + 256 }), n);
    item.setIconIndex(Block::byId(n + 256).orNpe().getBlockTextureFromSide(2))
}

pub fn create(n: i32) -> Rc<Item> { construct(&CLASS, n) }
