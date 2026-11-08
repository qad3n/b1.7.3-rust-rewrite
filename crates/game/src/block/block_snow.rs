use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::entity::Entity;
use crate::item::{Item, ItemStack};
use crate::math::AxisAlignedBB;
use crate::world::{EnumSkyBlock, World};
use jvm::throwable::OrNpe;
use std::rc::Rc;

fn canPlaceBlockAt(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let n: i32 = world.getBlockId(x, y - 1, z);
    if n == 0 || !Block::byId(n).orNpe().isOpaqueCube()
    {
        return false;
    }
    world.getBlockMaterial(x, y - 1, z).getIsSolid()
}

fn harvestBlock(this: &Rc<Block>, world: &Rc<World>, player: &Rc<Entity>, x: i32, y: i32, z: i32, _metadata: i32)
{
    let n: i32 = Item::snowball().m_shiftedIndex;
    let f: f32 = 0.7;
    let d: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.5;
    let d2: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.5;
    let d3: f64 = (world.m_rand.nextFloat() * f) as f64 + (1.0 - f) as f64 * 0.5;
    let entityItem: Rc<Entity> = crate::entity::entity_item::create(
        world,
        x as f64 + d,
        y as f64 + d2,
        z as f64 + d3,
        ItemStack::withDamage(n, 1, 0),
    );
    crate::entity::entity_item::fields(&entityItem).m_delayBeforeCanPickup.set(10);
    world.entityJoinedWorld(&entityItem);
    world.setBlockWithNotify(x, y, z, 0);
    player.addStat(crate::stats::StatList::mineBlockStat(this.m_blockID).as_ref(), 1);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockSnow",
    m_parent: Some(&BLOCK),
    getCollisionBoundingBoxFromPool: |this, world, x, y, z| {
        let n: i32 = world.getBlockMetadata(x, y, z) & 7;
        if n >= 3
        {
            return Some(AxisAlignedBB::getBoundingBoxFromPool(
                x as f64 + this.m_minX.get(),
                y as f64 + this.m_minY.get(),
                z as f64 + this.m_minZ.get(),
                x as f64 + this.m_maxX.get(),
                (y as f32 + 0.5) as f64,
                z as f64 + this.m_maxZ.get(),
            ));
        }
        None
    },
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    setBlockBoundsBasedOnState: |this, world, x, y, z| {
        let n: i32 = world.getBlockMetadata(x, y, z) & 7;
        let f: f32 = (2 * (1 + n)) as f32 / 16.0;
        this.setBlockBounds(0.0, 0.0, 0.0, 1.0, f, 1.0);
    },
    canPlaceBlockAt,
    onNeighborBlockChange: |this, world, x, y, z, _| {
        if !this.canPlaceBlockAt(world, x, y, z)
        {
            this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
            world.setBlockWithNotify(x, y, z, 0);
        }
    },
    harvestBlock,
    idDropped: |_, _, _| Item::snowball().m_shiftedIndex,
    quantityDropped: |_, _| 0,
    updateTick: |this, world, x, y, z, _| {
        if world.getSavedLightValue(EnumSkyBlock::Block, x, y, z) > 11
        {
            this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
            world.setBlockWithNotify(x, y, z, 0);
        }
    },
    shouldSideBeRendered: |this, world, x, y, z, face| {
        face == 1 || (BLOCK.shouldSideBeRendered)(this, world, x, y, z, face)
    },
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::snow());
    block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.125, 1.0);
    block.setTickOnLoad(true)
}
