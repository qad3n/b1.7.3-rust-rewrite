use crate::block::{BLOCK, Block, BlockClass, Material};
use std::cell::Cell;
use std::rc::Rc;

pub const VARIANTS: [&str; 4] = ["stone", "sand", "wood", "cobble"];

#[derive(Default)]
pub struct StepExt
{
    m_blockType: Cell<bool>,
}

fn blockType(this: &Block) -> bool { this.ext::<StepExt>().m_blockType.get() }

fn getBlockTextureFromSideAndMetadata(_this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    match metadata
    {
        0 =>
        {
            if face <= 1
            {
                6
            }
            else
            {
                5
            }
        }
        1 => match face
        {
            0 => 208,
            1 => 176,
            _ => 192,
        },
        2 => 4,
        3 => 16,
        _ => 6,
    }
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockStep",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSideAndMetadata,
    getBlockTextureFromSide: |this, face| this.getBlockTextureFromSideAndMetadata(face, 0),
    isOpaqueCube: |this| blockType(this),
    onBlockAdded: |this, world, x, y, z| {
        if !Rc::ptr_eq(this, &Block::stairSingle())
        {
            (BLOCK.onBlockAdded)(this, world, x, y, z);
        }
        let n2: i32 = world.getBlockId(x, y - 1, z);
        let n3: i32 = world.getBlockMetadata(x, y, z);
        let n: i32 = world.getBlockMetadata(x, y - 1, z);
        if n3 != n
        {
            return;
        }
        if n2 == Block::stairSingle().m_blockID
        {
            world.setBlockWithNotify(x, y, z, 0);
            world.setBlockAndMetadataWithNotify(x, y - 1, z, Block::stairDouble().m_blockID, n3);
        }
    },
    idDropped: |_, _, _| Block::stairSingle().m_blockID,
    quantityDropped: |this, _| if blockType(this) { 2 } else { 1 },
    damageDropped: |_, metadata| metadata,
    renderAsNormalBlock: |this| blockType(this),
    shouldSideBeRendered: |this, world, x, y, z, face| {
        if !Rc::ptr_eq(this, &Block::stairSingle())
        {
            (BLOCK.shouldSideBeRendered)(this, world, x, y, z, face);
        }
        if face == 1
        {
            return true;
        }
        if !(BLOCK.shouldSideBeRendered)(this, world, x, y, z, face)
        {
            return false;
        }
        if face == 0
        {
            return true;
        }
        world.getBlockId(x, y, z) != this.m_blockID
    },
    ..BLOCK
};

pub fn create(id: i32, doubleSlab: bool) -> Rc<Block>
{
    let block: Rc<Block> = Block::constructWithSprite(&CLASS, Box::new(StepExt::default()), id, 6, Material::rock());
    block.ext::<StepExt>().m_blockType.set(doubleSlab);
    if !doubleSlab
    {
        block.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.5, 1.0);
    }
    block.setLightOpacity(255)
}
