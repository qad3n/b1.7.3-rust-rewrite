use crate::block::{BLOCK, Block, BlockClass, Material};
use std::rc::Rc;

fn applyBounds(this: &Block, n: i32)
{
    let f: f32 = 0.125;
    if n == 2
    {
        this.setBlockBounds(0.0, 0.0, 1.0 - f, 1.0, 1.0, 1.0);
    }
    if n == 3
    {
        this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, f);
    }
    if n == 4
    {
        this.setBlockBounds(1.0 - f, 0.0, 0.0, 1.0, 1.0, 1.0);
    }
    if n == 5
    {
        this.setBlockBounds(0.0, 0.0, 0.0, f, 1.0, 1.0);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockLadder",
    m_parent: Some(&BLOCK),
    getCollisionBoundingBoxFromPool: |this, world, x, y, z| {
        applyBounds(this, world.getBlockMetadata(x, y, z));
        (BLOCK.getCollisionBoundingBoxFromPool)(this, world, x, y, z)
    },
    getSelectedBoundingBoxFromPool: |this, world, x, y, z| {
        applyBounds(this, world.getBlockMetadata(x, y, z));
        (BLOCK.getSelectedBoundingBoxFromPool)(this, world, x, y, z)
    },
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 8,
    canPlaceBlockAt: |_, world, x, y, z| {
        world.isBlockNormalCube(x - 1, y, z)
            || world.isBlockNormalCube(x + 1, y, z)
            || world.isBlockNormalCube(x, y, z - 1)
            || world.isBlockNormalCube(x, y, z + 1)
    },
    onBlockPlaced: |_, world, x, y, z, face| {
        let mut n: i32 = world.getBlockMetadata(x, y, z);
        if (n == 0 || face == 2) && world.isBlockNormalCube(x, y, z + 1)
        {
            n = 2;
        }
        if (n == 0 || face == 3) && world.isBlockNormalCube(x, y, z - 1)
        {
            n = 3;
        }
        if (n == 0 || face == 4) && world.isBlockNormalCube(x + 1, y, z)
        {
            n = 4;
        }
        if (n == 0 || face == 5) && world.isBlockNormalCube(x - 1, y, z)
        {
            n = 5;
        }
        world.setBlockMetadataWithNotify(x, y, z, n);
    },
    onNeighborBlockChange: |this, world, x, y, z, neighborBlock| {
        let n: i32 = world.getBlockMetadata(x, y, z);
        let mut bl: bool = false;
        if n == 2 && world.isBlockNormalCube(x, y, z + 1)
        {
            bl = true;
        }
        if n == 3 && world.isBlockNormalCube(x, y, z - 1)
        {
            bl = true;
        }
        if n == 4 && world.isBlockNormalCube(x + 1, y, z)
        {
            bl = true;
        }
        if n == 5 && world.isBlockNormalCube(x - 1, y, z)
        {
            bl = true;
        }
        if !bl
        {
            this.dropBlockAsItem(world, x, y, z, n);
            world.setBlockWithNotify(x, y, z, 0);
        }
        (BLOCK.onNeighborBlockChange)(this, world, x, y, z, neighborBlock);
    },
    quantityDropped: |_, _| 1,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::circuits())
}
