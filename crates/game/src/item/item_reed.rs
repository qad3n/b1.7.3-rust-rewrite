use crate::block::{Block, StepSound};
use crate::entity::Entity;
use crate::item::{ITEM, Item, ItemClass, ItemStack};
use crate::world::World;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct ReedExt
{
    m_block: i32,
}

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
    let id: i32 = this.ext::<ReedExt>().m_block;
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
    if world.canBlockBePlacedAt(id, x, y, z, false, face)
    {
        let block: Rc<Block> = Block::byId(id).orNpe();
        if world.setBlockWithNotify(x, y, z, id)
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
    }
    true
}

pub const CLASS: ItemClass = ItemClass { m_name: "ItemReed", m_parent: Some(&ITEM), onItemUse, ..ITEM };

pub fn create(id: i32, block: &Block) -> Rc<Item>
{
    Item::construct(&CLASS, Box::new(ReedExt { m_block: block.m_blockID }), id)
}
