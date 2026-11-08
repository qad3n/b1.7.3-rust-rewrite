use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::entity::Entity;
use crate::item::Item;
use crate::world::{IBlockAccess, World};
use jvm::throwable::OrNpe;
use std::rc::Rc;

fn getBlockTextureFromSideAndMetadata(this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    if face == 0 || face == 1
    {
        return this.m_blockIndexInTexture.get();
    }
    let n: i32 = getState(metadata);
    if (n == 0 || n == 2) ^ (face <= 3)
    {
        return this.m_blockIndexInTexture.get();
    }
    let mut n2: i32 = n / 2 + ((face & 1) ^ n);
    let mut n3: i32 = this.m_blockIndexInTexture.get() - (metadata & 8) * 2;
    n2 += (metadata & 4) / 4;
    if (n2 & 1) != 0
    {
        n3 = -n3;
    }
    n3
}

pub fn setDoorRotation(this: &Block, facing: i32)
{
    let f: f32 = 0.1875;
    this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 2.0, 1.0);
    if facing == 0
    {
        this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, f);
    }
    if facing == 1
    {
        this.setBlockBounds(1.0 - f, 0.0, 0.0, 1.0, 1.0, 1.0);
    }
    if facing == 2
    {
        this.setBlockBounds(0.0, 0.0, 1.0 - f, 1.0, 1.0, 1.0);
    }
    if facing == 3
    {
        this.setBlockBounds(0.0, 0.0, 0.0, f, 1.0, 1.0);
    }
}

fn isIron(this: &Block) -> bool { Rc::ptr_eq(&this.m_blockMaterial, &Material::iron()) }

fn blockActivated(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>) -> bool
{
    if isIron(this)
    {
        return true;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 8) != 0
    {
        if world.getBlockId(x, y - 1, z) == this.m_blockID
        {
            this.blockActivated(world, x, y - 1, z, player);
        }
        return true;
    }
    if world.getBlockId(x, y + 1, z) == this.m_blockID
    {
        world.setBlockMetadataWithNotify(x, y + 1, z, (n ^ 4) + 8);
    }
    world.setBlockMetadataWithNotify(x, y, z, n ^ 4);
    world.markBlocksDirty(x, y - 1, z, x, y, z);
    world.doEventFrom(Some(player), 1003, x, y, z, 0);
    true
}

fn onPoweredBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, open: bool)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 8) != 0
    {
        if world.getBlockId(x, y - 1, z) == this.m_blockID
        {
            this.onPoweredBlockChange(world, x, y - 1, z, open);
        }
        return;
    }
    let bl: bool = (world.getBlockMetadata(x, y, z) & 4) > 0;
    if bl == open
    {
        return;
    }
    if world.getBlockId(x, y + 1, z) == this.m_blockID
    {
        world.setBlockMetadataWithNotify(x, y + 1, z, (n ^ 4) + 8);
    }
    world.setBlockMetadataWithNotify(x, y, z, n ^ 4);
    world.markBlocksDirty(x, y - 1, z, x, y, z);
    world.doEventFrom(None, 1003, x, y, z, 0);
}

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, neighborBlock: i32)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if (n & 8) != 0
    {
        if world.getBlockId(x, y - 1, z) != this.m_blockID
        {
            world.setBlockWithNotify(x, y, z, 0);
        }
        if neighborBlock > 0 && Block::byId(neighborBlock).orNpe().canProvidePower()
        {
            this.onNeighborBlockChange(world, x, y - 1, z, neighborBlock);
        }
    }
    else
    {
        let mut bl: bool = false;
        if world.getBlockId(x, y + 1, z) != this.m_blockID
        {
            world.setBlockWithNotify(x, y, z, 0);
            bl = true;
        }
        if !world.isBlockNormalCube(x, y - 1, z)
        {
            world.setBlockWithNotify(x, y, z, 0);
            bl = true;
            if world.getBlockId(x, y + 1, z) == this.m_blockID
            {
                world.setBlockWithNotify(x, y + 1, z, 0);
            }
        }
        if bl
        {
            if !world.m_multiplayerWorld.get()
            {
                this.dropBlockAsItem(world, x, y, z, n);
            }
        }
        else if neighborBlock > 0 && Block::byId(neighborBlock).orNpe().canProvidePower()
        {
            let bl2: bool =
                world.isBlockIndirectlyGettingPowered(x, y, z) || world.isBlockIndirectlyGettingPowered(x, y + 1, z);
            this.onPoweredBlockChange(world, x, y, z, bl2);
        }
    }
}

pub fn getState(metadata: i32) -> i32
{
    if (metadata & 4) == 0
    {
        return (metadata - 1) & 3;
    }
    metadata & 3
}

pub fn isOpen(metadata: i32) -> bool { (metadata & 4) != 0 }

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    setDoorRotation(this, getState(world.getBlockMetadata(x, y, z)));
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockDoor",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSideAndMetadata,
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 7,
    getSelectedBoundingBoxFromPool: |this, world, x, y, z| {
        this.setBlockBoundsBasedOnState(&**world, x, y, z);
        (BLOCK.getSelectedBoundingBoxFromPool)(this, world, x, y, z)
    },
    getCollisionBoundingBoxFromPool: |this, world, x, y, z| {
        this.setBlockBoundsBasedOnState(&**world, x, y, z);
        (BLOCK.getCollisionBoundingBoxFromPool)(this, world, x, y, z)
    },
    setBlockBoundsBasedOnState,
    onBlockClicked: |this, world, x, y, z, player| {
        this.blockActivated(world, x, y, z, player);
    },
    blockActivated,
    onPoweredBlockChange,
    onNeighborBlockChange,
    idDropped: |this, metadata, _| {
        if (metadata & 8) != 0
        {
            return 0;
        }
        if isIron(this)
        {
            return Item::doorSteel().m_shiftedIndex;
        }
        Item::doorWood().m_shiftedIndex
    },
    collisionRayTrace: |this, world, x, y, z, from, to| {
        this.setBlockBoundsBasedOnState(&**world, x, y, z);
        (BLOCK.collisionRayTrace)(this, world, x, y, z, from, to)
    },
    canPlaceBlockAt: |this, world, x, y, z| {
        if y >= 127
        {
            return false;
        }
        world.isBlockNormalCube(x, y - 1, z)
            && (BLOCK.canPlaceBlockAt)(this, world, x, y, z)
            && (BLOCK.canPlaceBlockAt)(this, world, x, y + 1, z)
    },
    getMobilityFlag: |_| 1,
    ..BLOCK
};

pub fn create(id: i32, material: Rc<Material>) -> Rc<Block>
{
    let iron: bool = Rc::ptr_eq(&material, &Material::iron());
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, material);
    block.m_blockIndexInTexture.set(97);
    if iron
    {
        block.m_blockIndexInTexture.set(block.m_blockIndexInTexture.get() + 1);
    }
    let f: f32 = 0.5;
    let f2: f32 = 1.0;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, f2, 0.5 + f);
    block
}
