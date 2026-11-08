use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::math::AxisAlignedBB;
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

fn updateTick(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if random.nextIntBounded(5) != 0
    {
        return;
    }
    if isWaterNearby(world, x, y, z) || world.canBlockBeRainedOn(x, y + 1, z)
    {
        world.setBlockMetadataWithNotify(x, y, z, 7);
        return;
    }
    let blockMetadata: i32 = world.getBlockMetadata(x, y, z);
    if blockMetadata > 0
    {
        world.setBlockMetadataWithNotify(x, y, z, blockMetadata - 1);
    }
    else if !isCropsNearby(world, x, y, z)
    {
        world.setBlockWithNotify(x, y, z, Block::dirt().m_blockID);
    }
}

fn isCropsNearby(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let n: i32 = 0;
    for i in (x - n)..=(x + n)
    {
        for j in (z - n)..=(z + n)
        {
            if world.getBlockId(i, y + 1, j) == Block::crops().m_blockID
            {
                return true;
            }
        }
    }
    false
}

fn isWaterNearby(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let water: Rc<Material> = Material::water();
    for i in (x - 4)..=(x + 4)
    {
        for j in y..=(y + 1)
        {
            for k in (z - 4)..=(z + 4)
            {
                if Rc::ptr_eq(&world.getBlockMaterial(i, j, k), &water)
                {
                    return true;
                }
            }
        }
    }
    false
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockFarmland",
    m_parent: Some(&BLOCK),
    getCollisionBoundingBoxFromPool: |_, _, x, y, z| {
        Some(AxisAlignedBB::getBoundingBoxFromPool(
            x as f64,
            y as f64,
            z as f64,
            (x + 1) as f64,
            (y + 1) as f64,
            (z + 1) as f64,
        ))
    },
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getBlockTextureFromSideAndMetadata: |this, face, metadata| {
        if face == 1 && metadata > 0
        {
            return this.m_blockIndexInTexture.get() - 1;
        }
        if face == 1
        {
            return this.m_blockIndexInTexture.get();
        }
        2
    },
    updateTick,
    onEntityWalking: |_, world, x, y, z, _| {
        if world.m_rand.nextIntBounded(4) == 0
        {
            world.setBlockWithNotify(x, y, z, Block::dirt().m_blockID);
        }
    },
    onNeighborBlockChange: |this, world, x, y, z, neighborBlock| {
        (BLOCK.onNeighborBlockChange)(this, world, x, y, z, neighborBlock);
        if world.getBlockMaterial(x, y + 1, z).isSolid()
        {
            world.setBlockWithNotify(x, y, z, Block::dirt().m_blockID);
        }
    },
    idDropped: |_, _, random| Block::dirt().idDropped(0, random),
    ..BLOCK
};

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, Material::ground());
    block.m_blockIndexInTexture.set(87);
    let block: Rc<Block> = block.setTickOnLoad(true);
    block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.9375, 1.0);
    block.setLightOpacity(255)
}
