use crate::block::{Block, BlockClass, block_flower};
use crate::item::{Item, ItemStack};
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    (block_flower::CLASS.updateTick)(this, world, x, y, z, random);
    if world.getBlockLightValue(x, y + 1, z) < 9
    {
        return;
    }
    let mut n: i32 = world.getBlockMetadata(x, y, z);
    if n < 7
    {
        let f: f32 = getGrowthRate(this, world, x, y, z);
        if random.nextIntBounded((100.0 / f) as i32) == 0
        {
            n += 1;
            world.setBlockMetadataWithNotify(x, y, z, n);
        }
    }
}

pub fn fertilize(world: &Rc<World>, x: i32, y: i32, z: i32) { world.setBlockMetadataWithNotify(x, y, z, 7); }

fn getGrowthRate(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> f32
{
    let mut f: f32 = 1.0;
    let id: i32 = this.m_blockID;
    let n: i32 = world.getBlockId(x, y, z - 1);
    let n2: i32 = world.getBlockId(x, y, z + 1);
    let n3: i32 = world.getBlockId(x - 1, y, z);
    let n4: i32 = world.getBlockId(x + 1, y, z);
    let n5: i32 = world.getBlockId(x - 1, y, z - 1);
    let n6: i32 = world.getBlockId(x + 1, y, z - 1);
    let n7: i32 = world.getBlockId(x + 1, y, z + 1);
    let n8: i32 = world.getBlockId(x - 1, y, z + 1);
    let bl: bool = n3 == id || n4 == id;
    let bl2: bool = n == id || n2 == id;
    let bl3: bool = n5 == id || n6 == id || n7 == id || n8 == id;
    for i in (x - 1)..=(x + 1)
    {
        for j in (z - 1)..=(z + 1)
        {
            let n9: i32 = world.getBlockId(i, y - 1, j);
            let mut f2: f32 = 0.0;
            if n9 == Block::tilledField().m_blockID
            {
                f2 = 1.0;
                if world.getBlockMetadata(i, y - 1, j) > 0
                {
                    f2 = 3.0;
                }
            }
            if i != x || j != z
            {
                f2 /= 4.0;
            }
            f += f2;
        }
    }
    if bl3 || bl && bl2
    {
        f /= 2.0;
    }
    f
}

fn dropBlockAsItemWithChance(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32, luck: f32)
{
    (block_flower::CLASS.dropBlockAsItemWithChance)(this, world, x, y, z, metadata, luck);
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    for _ in 0..3
    {
        if world.m_rand.nextIntBounded(15) > metadata
        {
            continue;
        }
        let f: f32 = 0.7;
        let f2: f32 = world.m_rand.nextFloat() * f + (1.0 - f) * 0.5;
        let f3: f32 = world.m_rand.nextFloat() * f + (1.0 - f) * 0.5;
        let f4: f32 = world.m_rand.nextFloat() * f + (1.0 - f) * 0.5;
        let entityItem: Rc<crate::entity::Entity> = crate::entity::entity_item::create(
            world,
            (x as f32 + f2) as f64,
            (y as f32 + f3) as f64,
            (z as f32 + f4) as f64,
            ItemStack::withItem(&Item::seeds()),
        );
        crate::entity::entity_item::fields(&entityItem).m_delayBeforeCanPickup.set(10);
        world.entityJoinedWorld(&entityItem);
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockCrops",
    m_parent: Some(&block_flower::CLASS),
    canThisPlantGrowOnThisBlockID: |_, block| block == Block::tilledField().m_blockID,
    updateTick,
    getBlockTextureFromSideAndMetadata: |this, _, metadata| {
        let metadata: i32 = if metadata < 0 { 7 } else { metadata };
        this.m_blockIndexInTexture.get() + metadata
    },
    getRenderType: |_| 6,
    dropBlockAsItemWithChance,
    idDropped: |_, metadata, _| {
        if metadata == 7
        {
            Item::wheat().m_shiftedIndex
        }
        else
        {
            -1
        }
    },
    quantityDropped: |_, _| 1,
    ..block_flower::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = block_flower::construct(&CLASS, Box::new(()), id, sprite);
    block.m_blockIndexInTexture.set(sprite);
    let block: Rc<Block> = block.setTickOnLoad(true);
    let f: f32 = 0.5;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, 0.25, 0.5 + f);
    block
}
