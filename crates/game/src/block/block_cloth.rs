use crate::block::{BLOCK, Block, BlockClass, Material};
use std::rc::Rc;

fn getBlockTextureFromSideAndMetadata(this: &Rc<Block>, _face: i32, metadata: i32) -> i32
{
    if metadata == 0
    {
        return this.m_blockIndexInTexture.get();
    }
    let metadata: i32 = !(metadata & 0xF);
    113 + ((metadata & 8) >> 3) + (metadata & 7) * 16
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockCloth",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSideAndMetadata,
    damageDropped: |_, metadata| metadata,
    ..BLOCK
};

pub fn create() -> Rc<Block> { Block::constructWithSprite(&CLASS, Box::new(()), 35, 64, Material::cloth()) }

pub fn getBlockMetadataForItemMetadata(metadata: i32) -> i32 { !metadata & 0xF }

pub fn getItemMetadataForBlockMetadata(metadata: i32) -> i32 { !metadata & 0xF }
