use super::base;
use crate::block::Block;
use crate::entity::Entity;
use crate::math::{AxisAlignedBB, MovingObjectPosition, Vec3D};
use crate::tileentity::TileEntity;
use crate::world::{IBlockAccess, World};
use jvm::array_list::JArrayList;
use jvm::random::Random;
use std::rc::Rc;

crate::vtable! {
    BlockClass for Block {
        fn blockActivated(world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>) -> bool;
        fn canBlockStay(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool;
        fn canCollideCheck(metadata: i32, allowLiquids: bool) -> bool;
        fn canPlaceBlockAt(world: &Rc<World>, x: i32, y: i32, z: i32) -> bool;
        fn canPlaceBlockOnSide(world: &Rc<World>, x: i32, y: i32, z: i32, face: i32) -> bool;
        fn canProvidePower() -> bool;
        fn canThisPlantGrowOnThisBlockID(id: i32) -> bool;
        fn collisionRayTrace(world: &Rc<World>, x: i32, y: i32, z: i32, from: &Vec3D, to: &Vec3D) -> Option<MovingObjectPosition>;
        fn colorMultiplier(world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> i32;
        fn damageDropped(metadata: i32) -> i32;
        fn dropBlockAsItemWithChance(world: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32, luck: f32);
        fn getBlockBrightness(world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> f32;
        fn getBlockEntity() -> Rc<TileEntity>;
        fn getBlockTexture(world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> i32;
        fn getBlockTextureFromSide(face: i32) -> i32;
        fn getBlockTextureFromSideAndMetadata(face: i32, metadata: i32) -> i32;
        fn getCollidingBoundingBoxes(world: &Rc<World>, x: i32, y: i32, z: i32, shape: &AxisAlignedBB, collisions: &JArrayList<Rc<AxisAlignedBB>>);
        fn getCollisionBoundingBoxFromPool(world: &Rc<World>, x: i32, y: i32, z: i32) -> Option<Rc<AxisAlignedBB>>;
        fn getExplosionResistance(entity: Option<&Rc<Entity>>) -> f32;
        fn getIsBlockSolid(world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> bool;
        fn getMobilityFlag() -> i32;
        fn getRenderBlockPass() -> i32;
        fn getRenderColor(metadata: i32) -> i32;
        fn getRenderType() -> i32;
        fn getSelectedBoundingBoxFromPool(world: &Rc<World>, x: i32, y: i32, z: i32) -> Rc<AxisAlignedBB>;
        fn harvestBlock(world: &Rc<World>, player: &Rc<Entity>, x: i32, y: i32, z: i32, metadata: i32);
        fn idDropped(metadata: i32, random: &Random) -> i32;
        fn initializeBlock();
        fn isCollidable() -> bool;
        fn isIndirectlyPoweringTo(world: &Rc<World>, x: i32, y: i32, z: i32, dir: i32) -> bool;
        fn isOpaqueCube() -> bool;
        fn isPoweringTo(world: &dyn IBlockAccess, x: i32, y: i32, z: i32, dir: i32) -> bool;
        fn onBlockAdded(world: &Rc<World>, x: i32, y: i32, z: i32);
        fn onBlockClicked(world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>);
        fn onBlockDestroyedByExplosion(world: &Rc<World>, x: i32, y: i32, z: i32);
        fn onBlockDestroyedByPlayer(world: &Rc<World>, x: i32, y: i32, z: i32, metadata: i32);
        fn onBlockPlaced(world: &Rc<World>, x: i32, y: i32, z: i32, face: i32);
        fn onBlockPlacedBy(world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>);
        fn onBlockRemoval(world: &Rc<World>, x: i32, y: i32, z: i32);
        fn onEntityCollidedWithBlock(world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>);
        fn onEntityWalking(world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>);
        fn onNeighborBlockChange(world: &Rc<World>, x: i32, y: i32, z: i32, neighborBlock: i32);
        fn onPoweredBlockChange(world: &Rc<World>, x: i32, y: i32, z: i32, powered: bool);
        fn playBlock(world: &Rc<World>, x: i32, y: i32, z: i32, kind: i32, data: i32);
        fn quantityDropped(random: &Random) -> i32;
        fn randomDisplayTick(world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random);
        fn renderAsNormalBlock() -> bool;
        fn setBlockBoundsBasedOnState(world: &dyn IBlockAccess, x: i32, y: i32, z: i32);
        fn setBlockBoundsForItemRender();
        fn shouldSideBeRendered(world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> bool;
        fn tickRate() -> i32;
        fn updateTick(world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random);
        fn velocityToAddToEntity(world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>, velocity: &Vec3D);
    }
}

fn abstractMethod(name: &str) -> !
{
    jvm::throwable::throw(&jvm::throwable::ERROR, &format!("java.lang.AbstractMethodError: {name}"))
}

pub const BLOCK: BlockClass = BlockClass {
    m_name: "Block",
    m_parent: None,
    m_interfaces: &[],
    blockActivated: |_, _, _, _, _, _| false,
    canBlockStay: |_, _, _, _, _| true,
    canCollideCheck: |this, _, _| this.isCollidable(),
    canPlaceBlockAt: base::canPlaceBlockAt,
    canPlaceBlockOnSide: |this, world, x, y, z, _| this.canPlaceBlockAt(world, x, y, z),
    canProvidePower: |_| false,
    canThisPlantGrowOnThisBlockID: |_, _| abstractMethod("canThisPlantGrowOnThisBlockID"),
    collisionRayTrace: base::collisionRayTrace,
    colorMultiplier: |_, _, _, _, _| 0xFFFFFF,
    damageDropped: |_, _| 0,
    dropBlockAsItemWithChance: base::dropBlockAsItemWithChance,
    getBlockBrightness: |this, world, x, y, z| world.getBrightness(x, y, z, Block::lightValue(this.m_blockID)),
    getBlockEntity: |_| abstractMethod("getBlockEntity"),
    getBlockTexture: |this, world, x, y, z, face| {
        this.getBlockTextureFromSideAndMetadata(face, world.getBlockMetadata(x, y, z))
    },
    getBlockTextureFromSide: |this, _| this.m_blockIndexInTexture.get(),
    getBlockTextureFromSideAndMetadata: |this, face, _| this.getBlockTextureFromSide(face),
    getCollidingBoundingBoxes: base::getCollidingBoundingBoxes,
    getCollisionBoundingBoxFromPool: |this, _, x, y, z| Some(base::boundsAt(this, x, y, z)),
    getExplosionResistance: |this, _| this.m_blockResistance.get() / 5.0,
    getIsBlockSolid: |_, world, x, y, z, _| world.getBlockMaterial(x, y, z).isSolid(),
    getMobilityFlag: |this| this.m_blockMaterial.getMaterialMobility(),
    getRenderBlockPass: |_| 0,
    getRenderColor: |_, _| 0xFFFFFF,
    getRenderType: |_| 0,
    getSelectedBoundingBoxFromPool: |this, _, x, y, z| base::boundsAt(this, x, y, z),
    harvestBlock: base::harvestBlock,
    idDropped: |this, _, _| this.m_blockID,
    initializeBlock: |_| {},
    isCollidable: |_| true,
    isIndirectlyPoweringTo: |_, _, _, _, _, _| false,
    isOpaqueCube: |_| true,
    isPoweringTo: |_, _, _, _, _, _| false,
    onBlockAdded: |_, _, _, _, _| {},
    onBlockClicked: |_, _, _, _, _, _| {},
    onBlockDestroyedByExplosion: |_, _, _, _, _| {},
    onBlockDestroyedByPlayer: |_, _, _, _, _, _| {},
    onBlockPlaced: |_, _, _, _, _, _| {},
    onBlockPlacedBy: |_, _, _, _, _, _| {},
    onBlockRemoval: |_, _, _, _, _| {},
    onEntityCollidedWithBlock: |_, _, _, _, _, _| {},
    onEntityWalking: |_, _, _, _, _, _| {},
    onNeighborBlockChange: |_, _, _, _, _, _| {},
    onPoweredBlockChange: |_, _, _, _, _, _| abstractMethod("onPoweredBlockChange"),
    playBlock: |_, _, _, _, _, _, _| {},
    quantityDropped: |_, _| 1,
    randomDisplayTick: |_, _, _, _, _, _| {},
    renderAsNormalBlock: |_| true,
    setBlockBoundsBasedOnState: |_, _, _, _, _| {},
    setBlockBoundsForItemRender: |_| {},
    shouldSideBeRendered: base::shouldSideBeRendered,
    tickRate: |_| 10,
    updateTick: |_, _, _, _, _, _| {},
    velocityToAddToEntity: |_, _, _, _, _, _, _| {},
};
