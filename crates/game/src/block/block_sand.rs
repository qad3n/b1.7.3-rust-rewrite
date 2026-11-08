use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::World;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    pub static FALL_INSTANTLY: Cell<bool> = const { Cell::new(false) };
}

fn tryToFall(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let mut y: i32 = y;
    let n: i32 = x;
    let n2: i32 = y;
    let n3: i32 = z;
    if !(canFallBelow(world, n, n2 - 1, n3) && n2 >= 0)
    {
        return;
    }

    let n4: i32 = 32;
    if FALL_INSTANTLY.with(Cell::get) || !world.checkChunksExist(x - n4, y - n4, z - n4, x + n4, y + n4, z + n4)
    {
        world.setBlockWithNotify(x, y, z, 0);
        while canFallBelow(world, x, y - 1, z) && y > 0
        {
            y -= 1;
        }
        if y > 0
        {
            world.setBlockWithNotify(x, y, z, this.m_blockID);
        }
    }
    else
    {
        let entity: Rc<crate::entity::Entity> = crate::entity::entity_falling_sand::create(
            world,
            (x as f32 + 0.5) as f64,
            (y as f32 + 0.5) as f64,
            (z as f32 + 0.5) as f64,
            this.m_blockID,
        );
        world.entityJoinedWorld(&entity);
    }
}

pub fn canFallBelow(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y, z);
    if n == 0 || n == Block::fire().m_blockID
    {
        return true;
    }
    let material: Rc<Material> = Rc::clone(&Block::byId(n).orNpe().m_blockMaterial);
    if Rc::ptr_eq(&material, &Material::water())
    {
        return true;
    }
    Rc::ptr_eq(&material, &Material::lava())
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockSand",
    m_parent: Some(&BLOCK),
    onBlockAdded: |this, world, x, y, z| world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate()),
    onNeighborBlockChange: |this, world, x, y, z, _| {
        world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate())
    },
    updateTick: |this, world, x, y, z, _: &Random| tryToFall(this, world, x, y, z),
    tickRate: |_| 3,
    ..BLOCK
};

pub fn construct(class: &'static BlockClass, id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(class, Box::new(()), id, sprite, Material::sand())
}

pub fn create(id: i32, sprite: i32) -> Rc<Block> { construct(&CLASS, id, sprite) }
