use crate::block::block_leaves_base::{self, LeavesExt};
use crate::block::{Block, BlockClass, Material};
use crate::entity::Entity;
use crate::item::{Item, ItemStack};
use crate::world::{IBlockAccess, World};
use crate::worldgen::ColorizerFoliage;
use jvm::random::Random;
use std::rc::Rc;

fn ext(this: &Block) -> &LeavesExt { this.ext::<LeavesExt>() }

fn getRenderColor(_this: &Rc<Block>, metadata: i32) -> i32
{
    if (metadata & 1) == 1
    {
        return ColorizerFoliage::getFoliageColorPine();
    }
    if (metadata & 2) == 2
    {
        return ColorizerFoliage::getFoliageColorBirch();
    }
    ColorizerFoliage::getDefaultColor()
}

fn colorMultiplier(_this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> i32
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 1) == 1
    {
        return ColorizerFoliage::getFoliageColorPine();
    }
    if (n & 2) == 2
    {
        return ColorizerFoliage::getFoliageColorBirch();
    }
    world.getWorldChunkManager().getBiomes(x, z, 1, 1);
    let d: f64 = world.getWorldChunkManager().temperature(0);
    let d2: f64 = world.getWorldChunkManager().humidity(0);
    ColorizerFoliage::getFoliageColor(d, d2)
}

fn onBlockRemoval(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let b0: i32 = 1;
    let i: i32 = b0 + 1;
    if !world.checkChunksExist(x - i, y - i, z - i, x + i, y + i, z + i)
    {
        return;
    }
    for j in -b0..=b0
    {
        for k in -b0..=b0
        {
            for l in -b0..=b0
            {
                let i1: i32 = world.getBlockId(x + j, y + k, z + l);
                if i1 == Block::leaves().m_blockID
                {
                    let j1: i32 = world.getBlockMetadata(x + j, y + k, z + l);
                    world.setBlockMetadata(x + j, y + k, z + l, j1 | 8);
                }
            }
        }
    }
}

fn updateTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _random: &Random)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 8) == 0
    {
        return;
    }

    let n3: i32 = 4;
    let n4: i32 = n3 + 1;
    let n5: i32 = 32;
    let n6: i32 = n5 * n5;
    let n7: i32 = n5 / 2;
    let mut cells = ext(this).m_adjacentTreeBlocks.borrow_mut();
    let blocks: &mut Vec<i32> = cells.get_or_insert_with(|| vec![0; (n5 * n5 * n5) as usize]);
    let at = |a: i32, b: i32, c: i32| -> usize { ((a + n7) * n6 + (b + n7) * n5 + (c + n7)) as usize };
    if world.checkChunksExist(x - n4, y - n4, z - n4, x + n4, y + n4, z + n4)
    {
        for n2 in -n3..=n3
        {
            for n10 in -n3..=n3
            {
                for n9 in -n3..=n3
                {
                    let n8: i32 = world.getBlockId(x + n2, y + n10, z + n9);
                    blocks[at(n2, n10, n9)] = if n8 == Block::wood().m_blockID
                    {
                        0
                    }
                    else if n8 == Block::leaves().m_blockID
                    {
                        -2
                    }
                    else
                    {
                        -1
                    };
                }
            }
        }
        for n2 in 1..=4
        {
            for n10 in -n3..=n3
            {
                for n9 in -n3..=n3
                {
                    for n8 in -n3..=n3
                    {
                        if blocks[at(n10, n9, n8)] != n2 - 1
                        {
                            continue;
                        }
                        for (a, b, c) in [
                            (n10 - 1, n9, n8),
                            (n10 + 1, n9, n8),
                            (n10, n9 - 1, n8),
                            (n10, n9 + 1, n8),
                            (n10, n9, n8 - 1),
                            (n10, n9, n8 + 1),
                        ]
                        {
                            if blocks[at(a, b, c)] == -2
                            {
                                blocks[at(a, b, c)] = n2;
                            }
                        }
                    }
                }
            }
        }
    }

    let n2: i32 = blocks[(n7 * n6 + n7 * n5 + n7) as usize];
    drop(cells);
    if n2 >= 0
    {
        world.setBlockMetadata(x, y, z, n & 0xFFFFFFF7u32 as i32);
    }
    else
    {
        this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
        world.setBlockWithNotify(x, y, z, 0);
    }
}

fn harvestBlock(this: &Rc<Block>, world: &Rc<World>, player: &Rc<Entity>, x: i32, y: i32, z: i32, metadata: i32)
{
    let shears: bool = crate::entity::entity_player::getCurrentEquippedItem(player)
        .is_some_and(|stack| stack.m_itemID.get() == Item::shears().m_shiftedIndex);
    if !world.m_multiplayerWorld.get() && shears
    {
        player.addStat(crate::stats::StatList::mineBlockStat(this.m_blockID).as_ref(), 1);
        this.dropBlockAsItem_do(world, x, y, z, ItemStack::withDamage(Block::leaves().m_blockID, 1, metadata & 3));
    }
    else
    {
        (block_leaves_base::CLASS.harvestBlock)(this, world, player, x, y, z, metadata);
    }
}

pub fn setGraphicsLevel(this: &Block, culling: bool)
{
    ext(this).m_graphicsLevel.set(culling);
    this.m_blockIndexInTexture.set(ext(this).m_baseIndexInPNG.get() + if culling { 0 } else { 1 });
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockLeaves",
    m_parent: Some(&block_leaves_base::CLASS),
    getRenderColor,
    colorMultiplier,
    onBlockRemoval,
    updateTick,
    quantityDropped: |_, random| if random.nextIntBounded(20) == 0 { 1 } else { 0 },
    idDropped: |_, _, _| Block::sapling().m_blockID,
    harvestBlock,
    damageDropped: |_, metadata| metadata & 3,
    isOpaqueCube: |this| !ext(this).m_graphicsLevel.get(),
    getBlockTextureFromSideAndMetadata: |this, _, metadata| {
        if (metadata & 3) == 1
        {
            this.m_blockIndexInTexture.get() + 80
        }
        else
        {
            this.m_blockIndexInTexture.get()
        }
    },
    onEntityWalking: |this, world, x, y, z, entity| {
        (block_leaves_base::CLASS.onEntityWalking)(this, world, x, y, z, entity)
    },
    ..block_leaves_base::CLASS
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    let block: Rc<Block> = block_leaves_base::construct(&CLASS, id, sprite, Material::leaves(), false);
    ext(&block).m_baseIndexInPNG.set(sprite);
    block.setTickOnLoad(true)
}
