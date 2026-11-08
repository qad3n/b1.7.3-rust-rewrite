use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::entity::Entity;
use crate::math::AxisAlignedBB;
use crate::world::World;
use std::rc::Rc;

fn eatCakeSlice(world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>)
{
    if crate::entity::entity_living::fields(player).m_health.get() < 20
    {
        player.heal(3);
        let n: i32 = world.getBlockMetadata(x, y, z) + 1;
        if n >= 6
        {
            world.setBlockWithNotify(x, y, z, 0);
        }
        else
        {
            world.setBlockMetadataWithNotify(x, y, z, n);
            world.markBlockAsNeedsUpdate(x, y, z);
        }
    }
}

fn bounds(world: &Rc<World>, x: i32, y: i32, z: i32, top: f32) -> Rc<AxisAlignedBB>
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let f: f32 = 0.0625;
    let f2: f32 = (1 + n * 2) as f32 / 16.0;
    AxisAlignedBB::getBoundingBoxFromPool(
        (x as f32 + f2) as f64,
        y as f64,
        (z as f32 + f) as f64,
        ((x + 1) as f32 - f) as f64,
        top as f64,
        ((z + 1) as f32 - f) as f64,
    )
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockCake",
    m_parent: Some(&BLOCK),
    setBlockBoundsBasedOnState: |this, world, x, y, z| {
        let n: i32 = world.getBlockMetadata(x, y, z);
        let f: f32 = 0.0625;
        let f2: f32 = (1 + n * 2) as f32 / 16.0;
        let f3: f32 = 0.5;
        this.setBlockBounds(f2, 0.0, f, 1.0 - f, f3, 1.0 - f);
    },
    setBlockBoundsForItemRender: |this| {
        let f: f32 = 0.0625;
        let f2: f32 = 0.5;
        this.setBlockBounds(f, 0.0, f, 1.0 - f, f2, 1.0 - f);
    },
    getCollisionBoundingBoxFromPool: |_, world, x, y, z| {
        let f: f32 = 0.0625;
        let f3: f32 = 0.5;
        Some(bounds(world, x, y, z, y as f32 + f3 - f))
    },
    getSelectedBoundingBoxFromPool: |_, world, x, y, z| {
        let f3: f32 = 0.5;
        bounds(world, x, y, z, y as f32 + f3)
    },
    getBlockTextureFromSideAndMetadata: |this, face, metadata| {
        let sprite: i32 = this.m_blockIndexInTexture.get();
        if face == 1
        {
            return sprite;
        }
        if face == 0
        {
            return sprite + 3;
        }
        if metadata > 0 && face == 4
        {
            return sprite + 2;
        }
        sprite + 1
    },
    getBlockTextureFromSide: |this, face| {
        let sprite: i32 = this.m_blockIndexInTexture.get();
        match face
        {
            1 => sprite,
            0 => sprite + 3,
            _ => sprite + 1,
        }
    },
    renderAsNormalBlock: |_| false,
    isOpaqueCube: |_| false,
    blockActivated: |_, world, x, y, z, player| {
        eatCakeSlice(world, x, y, z, player);
        true
    },
    onBlockClicked: |_, world, x, y, z, player| eatCakeSlice(world, x, y, z, player),
    canPlaceBlockAt: |this, world, x, y, z| {
        (BLOCK.canPlaceBlockAt)(this, world, x, y, z) && this.canBlockStay(world, x, y, z)
    },
    onNeighborBlockChange: |this, world, x, y, z, _| {
        if !this.canBlockStay(world, x, y, z)
        {
            this.dropBlockAsItem(world, x, y, z, world.getBlockMetadata(x, y, z));
            world.setBlockWithNotify(x, y, z, 0);
        }
    },
    canBlockStay: |_, world, x, y, z| world.getBlockMaterial(x, y - 1, z).isSolid(),
    quantityDropped: |_, _| 0,
    idDropped: |_, _, _| 0,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32) -> Rc<Block>
{
    Block::constructWithSprite(&CLASS, Box::new(()), id, sprite, Material::cakeMaterial()).setTickOnLoad(true)
}
