use crate::block::{BLOCK, Block, BlockClass, Material, StepSound, block_piston_moving};
use crate::entity::Entity;
use crate::math::math_helper;
use crate::tileentity::{TileEntity, tile_entity_piston};
use crate::world::{IBlockAccess, World};
use common::world::piston_block_textures::{OPPOSITE, X_OFFSET, Y_OFFSET, Z_OFFSET};
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct PistonExt
{
    m_isSticky: bool,
    m_deaf: Cell<bool>,
}

fn piston(this: &Block) -> &PistonExt { this.ext::<PistonExt>() }

pub fn getTopSprite(this: &Block) -> i32
{
    if piston(this).m_isSticky
    {
        return 106;
    }
    107
}

fn getBlockTextureFromSideAndMetadata(this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    let n: i32 = getFacing(metadata);
    if n > 5
    {
        return this.m_blockIndexInTexture.get();
    }
    if face == n
    {
        if isPowered(metadata)
            || this.m_minX.get() > 0.0
            || this.m_minY.get() > 0.0
            || this.m_minZ.get() > 0.0
            || this.m_maxX.get() < 1.0
            || this.m_maxY.get() < 1.0
            || this.m_maxZ.get() < 1.0
        {
            return 110;
        }
        return this.m_blockIndexInTexture.get();
    }
    if face == OPPOSITE[n as usize]
    {
        return 109;
    }
    108
}

fn onBlockPlacedBy(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>)
{
    if !entity.instanceOf("EntityPlayer")
    {
        jvm::throwable::throw(
            &jvm::throwable::CLASS_CAST,
            &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.EntityPlayer", entity.className()),
        );
    }
    let n: i32 = getFacingForPlacement(world, x, y, z, entity);
    world.setBlockMetadataWithNotify(x, y, z, n);
    if !world.m_multiplayerWorld.get()
    {
        checkExtended(this, world, x, y, z);
    }
}

fn checkExtended(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let n2: i32 = getFacing(n);
    let bl: bool = shouldExtend(this, world, x, y, z, n2);
    if n == 7
    {
        return;
    }
    if bl && !isPowered(n)
    {
        if canExtend(world, x, y, z, n2)
        {
            world.setBlockMetadata(x, y, z, n2 | 8);
            world.playNoteAt(x, y, z, 0, n2);
        }
    }
    else if !bl && isPowered(n)
    {
        world.setBlockMetadata(x, y, z, n2);
        world.playNoteAt(x, y, z, 1, n2);
    }
}

fn shouldExtend(_this: &Block, world: &Rc<World>, x: i32, y: i32, z: i32, facing: i32) -> bool
{
    if facing != 0 && world.isBlockIndirectlyProvidingPowerTo(x, y - 1, z, 0)
    {
        return true;
    }
    if facing != 1 && world.isBlockIndirectlyProvidingPowerTo(x, y + 1, z, 1)
    {
        return true;
    }
    if facing != 2 && world.isBlockIndirectlyProvidingPowerTo(x, y, z - 1, 2)
    {
        return true;
    }
    if facing != 3 && world.isBlockIndirectlyProvidingPowerTo(x, y, z + 1, 3)
    {
        return true;
    }
    if facing != 5 && world.isBlockIndirectlyProvidingPowerTo(x + 1, y, z, 5)
    {
        return true;
    }
    if facing != 4 && world.isBlockIndirectlyProvidingPowerTo(x - 1, y, z, 4)
    {
        return true;
    }
    if world.isBlockIndirectlyProvidingPowerTo(x, y, z, 0)
    {
        return true;
    }
    if world.isBlockIndirectlyProvidingPowerTo(x, y + 2, z, 1)
    {
        return true;
    }
    if world.isBlockIndirectlyProvidingPowerTo(x, y + 1, z - 1, 2)
    {
        return true;
    }
    if world.isBlockIndirectlyProvidingPowerTo(x, y + 1, z + 1, 3)
    {
        return true;
    }
    if world.isBlockIndirectlyProvidingPowerTo(x - 1, y + 1, z, 4)
    {
        return true;
    }
    world.isBlockIndirectlyProvidingPowerTo(x + 1, y + 1, z, 5)
}

fn asPiston(tileEntity: Option<Rc<TileEntity>>) -> Option<Rc<TileEntity>>
{
    tileEntity.filter(|entity| entity.instanceOf("TileEntityPiston"))
}

fn playBlock(this: &Rc<Block>, world: &Rc<World>, mut x: i32, mut y: i32, mut z: i32, kind: i32, data: i32)
{
    let deaf: &Cell<bool> = &piston(this).m_deaf;
    deaf.set(true);
    let n: i32 = data;
    let dir: usize = n as usize;
    if kind == 0
    {
        if push(this, world, x, y, z, n)
        {
            world.setBlockMetadataWithNotify(x, y, z, n | 8);
            world.playSoundEffect(
                x as f64 + 0.5,
                y as f64 + 0.5,
                z as f64 + 0.5,
                "tile.piston.out",
                0.5,
                world.m_rand.nextFloat() * 0.25 + 0.6,
            );
        }
    }
    else if kind == 1
    {
        if let Some(tileEntity) =
            asPiston(world.getBlockTileEntity(x + X_OFFSET[dir], y + Y_OFFSET[dir], z + Z_OFFSET[dir]))
        {
            tile_entity_piston::finish(&tileEntity);
        }
        world.setBlockAndMetadata(x, y, z, Block::pistonMoving().m_blockID, n);
        world.setBlockTileEntity(
            x,
            y,
            z,
            &block_piston_moving::createMovingBlockEntity(this.m_blockID, n, n, false, true),
        );
        if piston(this).m_isSticky
        {
            let n2: i32 = x + X_OFFSET[dir] * 2;
            let n3: i32 = y + Y_OFFSET[dir] * 2;
            let n4: i32 = z + Z_OFFSET[dir] * 2;
            let mut n5: i32 = world.getBlockId(n2, n3, n4);
            let mut n6: i32 = world.getBlockMetadata(n2, n3, n4);
            let mut bl: bool = false;
            if n5 == Block::pistonMoving().m_blockID
                && let Some(tileEntityPiston) = asPiston(world.getBlockTileEntity(n2, n3, n4))
                && tile_entity_piston::getFacing(&tileEntityPiston) == n
                && tile_entity_piston::isExtending(&tileEntityPiston)
            {
                tile_entity_piston::finish(&tileEntityPiston);
                n5 = tile_entity_piston::getStoredBlockID(&tileEntityPiston);
                n6 = tileEntityPiston.getBlockMetadata();
                bl = true;
            }
            if !bl
                && n5 > 0
                && canPushBlock(n5, world, n2, n3, n4, false)
                && (Block::byId(n5).orNpe().getMobilityFlag() == 0
                    || n5 == Block::pistonBase().m_blockID
                    || n5 == Block::pistonStickyBase().m_blockID)
            {
                deaf.set(false);
                world.setBlockWithNotify(n2, n3, n4, 0);
                deaf.set(true);
                x += X_OFFSET[dir];
                y += Y_OFFSET[dir];
                z += Z_OFFSET[dir];
                world.setBlockAndMetadata(x, y, z, Block::pistonMoving().m_blockID, n6);
                world.setBlockTileEntity(
                    x,
                    y,
                    z,
                    &block_piston_moving::createMovingBlockEntity(n5, n6, n, false, false),
                );
            }
            else if !bl
            {
                deaf.set(false);
                world.setBlockWithNotify(x + X_OFFSET[dir], y + Y_OFFSET[dir], z + Z_OFFSET[dir], 0);
                deaf.set(true);
            }
        }
        else
        {
            deaf.set(false);
            world.setBlockWithNotify(x + X_OFFSET[dir], y + Y_OFFSET[dir], z + Z_OFFSET[dir], 0);
            deaf.set(true);
        }
        world.playSoundEffect(
            x as f64 + 0.5,
            y as f64 + 0.5,
            z as f64 + 0.5,
            "tile.piston.in",
            0.5,
            world.m_rand.nextFloat() * 0.15 + 0.6,
        );
    }
    deaf.set(false);
}

fn setBlockBoundsBasedOnState(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    if isPowered(n)
    {
        match getFacing(n)
        {
            0 => this.setBlockBounds(0.0, 0.25, 0.0, 1.0, 1.0, 1.0),
            1 => this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 0.75, 1.0),
            2 => this.setBlockBounds(0.0, 0.0, 0.25, 1.0, 1.0, 1.0),
            3 => this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 0.75),
            4 => this.setBlockBounds(0.25, 0.0, 0.0, 1.0, 1.0, 1.0),
            5 => this.setBlockBounds(0.0, 0.0, 0.0, 0.75, 1.0, 1.0),
            _ =>
            {}
        }
    }
    else
    {
        this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    }
}

pub fn getFacing(metadata: i32) -> i32 { metadata & 7 }

pub fn isPowered(metadata: i32) -> bool { (metadata & 8) != 0 }

fn getFacingForPlacement(_world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>) -> i32
{
    if math_helper::abs(player.m_posX.get() as f32 - x as f32) < 2.0
        && math_helper::abs(player.m_posZ.get() as f32 - z as f32) < 2.0
    {
        let n: f64 = player.m_posY.get() + 1.82 - player.m_yOffset.get() as f64;
        if n - y as f64 > 2.0
        {
            return 1;
        }
        if y as f64 - n > 0.0
        {
            return 0;
        }
    }
    let n2: i32 = math_helper::floor_double((player.m_rotationYaw.get() * 4.0 / 360.0) as f64 + 0.5) & 0x3;
    if n2 == 0
    {
        return 2;
    }
    if n2 == 1
    {
        return 5;
    }
    if n2 == 2
    {
        return 3;
    }
    if n2 == 3
    {
        return 4;
    }
    0
}

fn canPushBlock(block: i32, world: &Rc<World>, x: i32, y: i32, z: i32, allowBreaking: bool) -> bool
{
    if block == Block::obsidian().m_blockID
    {
        return false;
    }
    if block == Block::pistonBase().m_blockID || block == Block::pistonStickyBase().m_blockID
    {
        if isPowered(world.getBlockMetadata(x, y, z))
        {
            return false;
        }
    }
    else
    {
        if Block::byId(block).orNpe().getHardness() == -1.0
        {
            return false;
        }
        if Block::byId(block).orNpe().getMobilityFlag() == 2
        {
            return false;
        }
        if !allowBreaking && Block::byId(block).orNpe().getMobilityFlag() == 1
        {
            return false;
        }
    }
    world.getBlockTileEntity(x, y, z).is_none()
}

fn canExtend(world: &Rc<World>, x: i32, y: i32, z: i32, dir: i32) -> bool
{
    let dir: usize = dir as usize;
    let mut n: i32 = x + X_OFFSET[dir];
    let mut n2: i32 = y + Y_OFFSET[dir];
    let mut n3: i32 = z + Z_OFFSET[dir];
    for i in 0..13
    {
        if n2 <= 0 || n2 >= 127
        {
            return false;
        }
        let n4: i32 = world.getBlockId(n, n2, n3);
        if n4 == 0
        {
            break;
        }
        if !canPushBlock(n4, world, n, n2, n3, true)
        {
            return false;
        }
        if Block::byId(n4).orNpe().getMobilityFlag() == 1
        {
            break;
        }
        if i == 12
        {
            return false;
        }
        n += X_OFFSET[dir];
        n2 += Y_OFFSET[dir];
        n3 += Z_OFFSET[dir];
    }
    true
}

fn push(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, dir: i32) -> bool
{
    let side: usize = dir as usize;
    let mut n: i32;
    let mut n2: i32 = 0;
    let mut n3: i32 = x + X_OFFSET[side];
    let mut n4: i32 = y + Y_OFFSET[side];
    let mut n5: i32 = z + Z_OFFSET[side];
    while n2 < 13
    {
        if n4 <= 0 || n4 >= 127
        {
            return false;
        }
        n = world.getBlockId(n3, n4, n5);
        if n == 0
        {
            break;
        }
        if !canPushBlock(n, world, n3, n4, n5, true)
        {
            return false;
        }
        if Block::byId(n).orNpe().getMobilityFlag() == 1
        {
            Block::byId(n).orNpe().dropBlockAsItem(world, n3, n4, n5, world.getBlockMetadata(n3, n4, n5));
            world.setBlockWithNotify(n3, n4, n5, 0);
            break;
        }
        if n2 == 12
        {
            return false;
        }
        n3 += X_OFFSET[side];
        n4 += Y_OFFSET[side];
        n5 += Z_OFFSET[side];
        n2 += 1;
    }
    let sticky: i32 = if piston(this).m_isSticky { 8 } else { 0 };
    while n3 != x || n4 != y || n5 != z
    {
        n2 = n3 - X_OFFSET[side];
        n = n4 - Y_OFFSET[side];
        let n6: i32 = n5 - Z_OFFSET[side];
        let n7: i32 = world.getBlockId(n2, n, n6);
        let n8: i32 = world.getBlockMetadata(n2, n, n6);
        if n7 == this.m_blockID && n2 == x && n == y && n6 == z
        {
            world.setBlockAndMetadata(n3, n4, n5, Block::pistonMoving().m_blockID, dir | sticky);
            world.setBlockTileEntity(
                n3,
                n4,
                n5,
                &block_piston_moving::createMovingBlockEntity(
                    Block::pistonExtension().m_blockID,
                    dir | sticky,
                    dir,
                    true,
                    false,
                ),
            );
        }
        else
        {
            world.setBlockAndMetadata(n3, n4, n5, Block::pistonMoving().m_blockID, n8);
            world.setBlockTileEntity(
                n3,
                n4,
                n5,
                &block_piston_moving::createMovingBlockEntity(n7, n8, dir, true, false),
            );
        }
        n3 = n2;
        n4 = n;
        n5 = n6;
    }
    true
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockPistonBase",
    m_parent: Some(&BLOCK),
    getBlockTextureFromSideAndMetadata,
    getRenderType: |_| 16,
    isOpaqueCube: |_| false,
    blockActivated: |_, _, _, _, _, _| false,
    onBlockPlacedBy,
    onNeighborBlockChange: |this, world, x, y, z, _| {
        if !world.m_multiplayerWorld.get() && !piston(this).m_deaf.get()
        {
            checkExtended(this, world, x, y, z);
        }
    },
    onBlockAdded: |this, world, x, y, z| {
        if !world.m_multiplayerWorld.get() && world.getBlockTileEntity(x, y, z).is_none()
        {
            checkExtended(this, world, x, y, z);
        }
    },
    playBlock,
    setBlockBoundsBasedOnState,
    setBlockBoundsForItemRender: |this| this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
    getCollidingBoundingBoxes: |this, world, x, y, z, shape, collisions| {
        this.setBlockBounds(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
        (BLOCK.getCollidingBoundingBoxes)(this, world, x, y, z, shape, collisions);
    },
    renderAsNormalBlock: |_| false,
    ..BLOCK
};

pub fn create(id: i32, sprite: i32, sticky: bool) -> Rc<Block>
{
    let ext: PistonExt = PistonExt { m_isSticky: sticky, m_deaf: Cell::new(false) };
    let block: Rc<Block> = Block::constructWithSprite(&CLASS, Box::new(ext), id, sprite, Material::PISTON());
    block.setStepSound(StepSound::sounds(|s| Rc::clone(&s.m_stone))).setHardness(0.5)
}
