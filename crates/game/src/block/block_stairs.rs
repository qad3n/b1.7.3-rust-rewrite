use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::math::math_helper;
use std::rc::Rc;

pub struct StairsExt
{
    m_modelBlock: Rc<Block>,
}

fn model(this: &Block) -> Rc<Block> { Rc::clone(&this.ext::<StairsExt>().m_modelBlock) }

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockStairs",
    m_parent: Some(&BLOCK),
    setBlockBoundsBasedOnState: |this, _, _, _, _| this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
    getCollisionBoundingBoxFromPool: |this, world, x, y, z| {
        (BLOCK.getCollisionBoundingBoxFromPool)(this, world, x, y, z)
    },
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 10,
    shouldSideBeRendered: |this, world, x, y, z, face| (BLOCK.shouldSideBeRendered)(this, world, x, y, z, face),
    getCollidingBoundingBoxes: |this, world, x, y, z, shape, collisions| {
        let n: i32 = world.getBlockMetadata(x, y, z);
        let parts: Option<([f32; 6], [f32; 6])> = match n
        {
            0 => Some(([0.0, 0.0, 0.0, 0.5, 0.5, 1.0], [0.5, 0.0, 0.0, 1.0, 1.0, 1.0])),
            1 => Some(([0.0, 0.0, 0.0, 0.5, 1.0, 1.0], [0.5, 0.0, 0.0, 1.0, 0.5, 1.0])),
            2 => Some(([0.0, 0.0, 0.0, 1.0, 0.5, 0.5], [0.0, 0.0, 0.5, 1.0, 1.0, 1.0])),
            3 => Some(([0.0, 0.0, 0.0, 1.0, 1.0, 0.5], [0.0, 0.0, 0.5, 1.0, 0.5, 1.0])),
            _ => None,
        };
        if let Some((first, second)) = parts
        {
            for b in [first, second]
            {
                this.setBlockBounds(b[0], b[1], b[2], b[3], b[4], b[5]);
                (BLOCK.getCollidingBoundingBoxes)(this, world, x, y, z, shape, collisions);
            }
        }
        this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    },
    randomDisplayTick: |this, world, x, y, z, random| model(this).randomDisplayTick(world, x, y, z, random),
    onBlockClicked: |this, world, x, y, z, player| model(this).onBlockClicked(world, x, y, z, player),
    onBlockDestroyedByPlayer: |this, world, x, y, z, metadata| {
        model(this).onBlockDestroyedByPlayer(world, x, y, z, metadata)
    },
    getBlockBrightness: |this, world, x, y, z| model(this).getBlockBrightness(world, x, y, z),
    getExplosionResistance: |this, entity| model(this).getExplosionResistance(entity),
    getRenderBlockPass: |this| model(this).getRenderBlockPass(),
    idDropped: |this, metadata, random| model(this).idDropped(metadata, random),
    quantityDropped: |this, random| model(this).quantityDropped(random),
    getBlockTextureFromSideAndMetadata: |this, face, metadata| {
        model(this).getBlockTextureFromSideAndMetadata(face, metadata)
    },
    getBlockTextureFromSide: |this, face| model(this).getBlockTextureFromSide(face),
    getBlockTexture: |this, world, x, y, z, face| model(this).getBlockTexture(world, x, y, z, face),
    tickRate: |this| model(this).tickRate(),
    getSelectedBoundingBoxFromPool: |this, world, x, y, z| model(this).getSelectedBoundingBoxFromPool(world, x, y, z),
    velocityToAddToEntity: |this, world, x, y, z, entity, velocity| {
        model(this).velocityToAddToEntity(world, x, y, z, entity, velocity)
    },
    isCollidable: |this| model(this).isCollidable(),
    canCollideCheck: |this, metadata, allowLiquids| model(this).canCollideCheck(metadata, allowLiquids),
    canPlaceBlockAt: |this, world, x, y, z| model(this).canPlaceBlockAt(world, x, y, z),
    onBlockAdded: |this, world, x, y, z| {
        this.onNeighborBlockChange(world, x, y, z, 0);
        model(this).onBlockAdded(world, x, y, z);
    },
    onBlockRemoval: |this, world, x, y, z| model(this).onBlockRemoval(world, x, y, z),
    dropBlockAsItemWithChance: |this, world, x, y, z, metadata, luck| {
        model(this).dropBlockAsItemWithChance(world, x, y, z, metadata, luck)
    },
    onEntityWalking: |this, world, x, y, z, entity| model(this).onEntityWalking(world, x, y, z, entity),
    updateTick: |this, world, x, y, z, random| model(this).updateTick(world, x, y, z, random),
    blockActivated: |this, world, x, y, z, player| model(this).blockActivated(world, x, y, z, player),
    onBlockDestroyedByExplosion: |this, world, x, y, z| model(this).onBlockDestroyedByExplosion(world, x, y, z),
    onBlockPlacedBy: |_, world, x, y, z, entity| {
        let n: i32 = math_helper::floor_double((entity.m_rotationYaw.get() * 4.0 / 360.0) as f64 + 0.5) & 3;
        if n == 0
        {
            world.setBlockMetadataWithNotify(x, y, z, 2);
        }
        if n == 1
        {
            world.setBlockMetadataWithNotify(x, y, z, 1);
        }
        if n == 2
        {
            world.setBlockMetadataWithNotify(x, y, z, 3);
        }
        if n == 3
        {
            world.setBlockMetadataWithNotify(x, y, z, 0);
        }
    },
    ..BLOCK
};

pub fn create(id: i32, baseBlock: Rc<Block>) -> Rc<Block>
{
    let sprite: i32 = baseBlock.m_blockIndexInTexture.get();
    let material: Rc<Material> = Rc::clone(&baseBlock.m_blockMaterial);
    let block: Rc<Block> = Block::constructWithSprite(
        &CLASS,
        Box::new(StairsExt { m_modelBlock: Rc::clone(&baseBlock) }),
        id,
        sprite,
        material,
    );
    block
        .setHardness(baseBlock.m_blockHardness.get())
        .setResistance(baseBlock.m_blockResistance.get() / 3.0)
        .setStepSound(baseBlock.m_stepSound.get())
        .setLightOpacity(255)
}
