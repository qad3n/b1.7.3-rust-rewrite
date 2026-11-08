use crate::block::{BLOCK, Block, BlockClass, Material};
use crate::world::{IBlockAccess, World};
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn setBlockBoundsForBlockRender(this: &Block, metadata: i32)
{
    let f: f32 = 0.1875;
    this.setBlockBounds(0.0, 0.0, 0.0, 1.0, f, 1.0);
    if isTrapdoorOpen(metadata)
    {
        if (metadata & 3) == 0
        {
            this.setBlockBounds(0.0, 0.0, 1.0 - f, 1.0, 1.0, 1.0);
        }
        if (metadata & 3) == 1
        {
            this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, f);
        }
        if (metadata & 3) == 2
        {
            this.setBlockBounds(1.0 - f, 0.0, 0.0, 1.0, 1.0, 1.0);
        }
        if (metadata & 3) == 3
        {
            this.setBlockBounds(0.0, 0.0, 0.0, f, 1.0, 1.0);
        }
    }
}

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    setBlockBoundsForBlockRender(this, world.getBlockMetadata(x, y, z));
}

fn isIron(this: &Block) -> bool { Rc::ptr_eq(&this.m_blockMaterial, &Material::iron()) }

fn onNeighborBlockChange(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, neighborBlock: i32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    let mut n2: i32 = x;
    let mut n3: i32 = z;
    if (n & 3) == 0
    {
        n3 += 1;
    }
    if (n & 3) == 1
    {
        n3 -= 1;
    }
    if (n & 3) == 2
    {
        n2 += 1;
    }
    if (n & 3) == 3
    {
        n2 -= 1;
    }
    if !world.isBlockNormalCube(n2, y, n3)
    {
        world.setBlockWithNotify(x, y, z, 0);
        this.dropBlockAsItem(world, x, y, z, n);
    }
    if neighborBlock > 0 && Block::byId(neighborBlock).orNpe().canProvidePower()
    {
        let bl: bool = world.isBlockIndirectlyGettingPowered(x, y, z);
        this.onPoweredBlockChange(world, x, y, z, bl);
    }
}

fn onBlockPlaced(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, face: i32)
{
    let mut n: i32 = 0;
    if face == 2
    {
        n = 0;
    }
    if face == 3
    {
        n = 1;
    }
    if face == 4
    {
        n = 2;
    }
    if face == 5
    {
        n = 3;
    }
    world.setBlockMetadataWithNotify(x, y, z, n);
}

fn canPlaceBlockOnSide(_this: &Rc<Block>, world: &Rc<World>, mut x: i32, y: i32, mut z: i32, face: i32) -> bool
{
    if face == 0
    {
        return false;
    }
    if face == 1
    {
        return false;
    }
    if face == 2
    {
        z += 1;
    }
    if face == 3
    {
        z -= 1;
    }
    if face == 4
    {
        x += 1;
    }
    if face == 5
    {
        x -= 1;
    }
    world.isBlockNormalCube(x, y, z)
}

pub fn isTrapdoorOpen(metadata: i32) -> bool { (metadata & 4) != 0 }

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockTrapDoor",
    m_parent: Some(&BLOCK),
    isOpaqueCube: |_| false,
    renderAsNormalBlock: |_| false,
    getRenderType: |_| 0,
    getSelectedBoundingBoxFromPool: |this, world, x, y, z| {
        this.setBlockBoundsBasedOnState(&**world, x, y, z);
        (BLOCK.getSelectedBoundingBoxFromPool)(this, world, x, y, z)
    },
    getCollisionBoundingBoxFromPool: |this, world, x, y, z| {
        this.setBlockBoundsBasedOnState(&**world, x, y, z);
        (BLOCK.getCollisionBoundingBoxFromPool)(this, world, x, y, z)
    },
    setBlockBoundsBasedOnState,
    setBlockBoundsForItemRender: |this| {
        let f: f32 = 0.1875;
        this.setBlockBounds(0.0, 0.5 - f / 2.0, 0.0, 1.0, 0.5 + f / 2.0, 1.0);
    },
    onBlockClicked: |this, world, x, y, z, player| {
        this.blockActivated(world, x, y, z, player);
    },
    blockActivated: |this, world, x, y, z, player| {
        if isIron(this)
        {
            return true;
        }
        let n: i32 = world.getBlockMetadata(x, y, z);
        world.setBlockMetadataWithNotify(x, y, z, n ^ 4);
        world.doEventFrom(Some(player), 1003, x, y, z, 0);
        true
    },
    onPoweredBlockChange: |_, world, x, y, z, open| {
        let n: i32 = world.getBlockMetadata(x, y, z);
        let bl: bool = (n & 4) > 0;
        if bl == open
        {
            return;
        }
        world.setBlockMetadataWithNotify(x, y, z, n ^ 4);
        world.doEventFrom(None, 1003, x, y, z, 0);
    },
    onNeighborBlockChange,
    collisionRayTrace: |this, world, x, y, z, from, to| {
        this.setBlockBoundsBasedOnState(&**world, x, y, z);
        (BLOCK.collisionRayTrace)(this, world, x, y, z, from, to)
    },
    onBlockPlaced,
    canPlaceBlockOnSide,
    ..BLOCK
};

pub fn create(id: i32, material: Rc<Material>) -> Rc<Block>
{
    let iron: bool = Rc::ptr_eq(&material, &Material::iron());
    let block: Rc<Block> = Block::construct(&CLASS, Box::new(()), id, material);
    block.m_blockIndexInTexture.set(84);
    if iron
    {
        block.m_blockIndexInTexture.set(block.m_blockIndexInTexture.get() + 1);
    }
    let f: f32 = 0.5;
    let f2: f32 = 1.0;
    block.setBlockBounds(0.5 - f, 0.0, 0.5 - f, 0.5 + f, f2, 0.5 + f);
    block
}
