use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::math::math_helper;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct PumpkinExt
{
    m_blockType: Cell<bool>,
}

fn getBlockTextureFromSideAndMetadata(this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    let sprite: i32 = this.m_blockIndexInTexture.get();
    if face == 1 || face == 0
    {
        return sprite;
    }
    let mut n: i32 = sprite + 1 + 16;
    if this.ext::<PumpkinExt>().m_blockType.get()
    {
        n += 1;
    }
    if metadata == 2 && face == 2
        || metadata == 3 && face == 5
        || metadata == 0 && face == 3
        || metadata == 1 && face == 4
    {
        return n;
    }
    sprite + 16
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockPumpkin",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSideAndMetadata,
    getBlockTextureFromSide: |this, face| {
        let sprite: i32 = this.m_blockIndexInTexture.get();
        match face
        {
            1 | 0 => sprite,
            3 => sprite + 1 + 16,
            _ => sprite + 16,
        }
    },
    onBlockAdded: |this, world, x, y, z| (BLOCK.onBlockAdded)(this, world, x, y, z),
    canPlaceBlockAt: |_, world, x, y, z| {
        let n: i32 = world.getBlockId(x, y, z);
        (n == 0 || Block::byId(n).orNpe().m_blockMaterial.getIsGroundCover()) && world.isBlockNormalCube(x, y - 1, z)
    },
    onBlockPlacedBy: |_, world, x, y, z, entity| {
        let n: i32 = math_helper::floor_double((entity.m_rotationYaw.get() * 4.0 / 360.0) as f64 + 2.5) & 3;
        world.setBlockMetadataWithNotify(x, y, z, n);
    },
    ..BLOCK
};

pub fn create(id: i32, sprite: i32, lit: bool) -> Rc<Block>
{
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(PumpkinExt::default()), id, Material::pumpkin());
    block.m_blockIndexInTexture.set(sprite);
    let block: Rc<Block> = block.setTickOnLoad(true);
    block.ext::<PumpkinExt>().m_blockType.set(lit);
    block
}
