use crate::block::{Block, BlockClass, Material, block_container};
use crate::entity::Entity;
use crate::math::math_helper;
use crate::port::{inventory, tile_entity};
use crate::tileentity::{TileEntity, tile_entity_furnace};
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static KEEP_FURNACE_INVENTORY: Cell<bool> = const { Cell::new(false) };
}

pub struct FurnaceExt
{
    m_furnaceRand: Random,
    m_isActive: bool,
}

fn furnace(this: &Block) -> &FurnaceExt { this.ext::<FurnaceExt>() }

fn setDefaultDirection(world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let n: i32 = world.getBlockId(x, y, z - 1);
    let n2: i32 = world.getBlockId(x, y, z + 1);
    let n3: i32 = world.getBlockId(x - 1, y, z);
    let n4: i32 = world.getBlockId(x + 1, y, z);
    let opaque = Block::opaqueCubeLookup;
    let mut n5: i32 = 3;
    if opaque(n) && !opaque(n2)
    {
        n5 = 3;
    }
    if opaque(n2) && !opaque(n)
    {
        n5 = 2;
    }
    if opaque(n3) && !opaque(n4)
    {
        n5 = 5;
    }
    if opaque(n4) && !opaque(n3)
    {
        n5 = 4;
    }
    world.setBlockMetadataWithNotify(x, y, z, n5);
}

fn getBlockTexture(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> i32
{
    let index: i32 = this.m_blockIndexInTexture.get();
    if face == 1
    {
        return index + 17;
    }
    if face == 0
    {
        return index + 17;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    if face != n
    {
        return index;
    }
    if furnace(this).m_isActive
    {
        return index + 16;
    }
    index - 1
}

fn randomDisplayTick(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    if !furnace(this).m_isActive
    {
        return;
    }
    let n: i32 = world.getBlockMetadata(x, y, z);
    let f: f32 = x as f32 + 0.5;
    let f2: f32 = y as f32 + 0.0 + random.nextFloat() * 6.0 / 16.0;
    let f3: f32 = z as f32 + 0.5;
    let f4: f32 = 0.52;
    let f5: f32 = random.nextFloat() * 0.6 - 0.3;
    let at: Option<(f32, f32)> = match n
    {
        4 => Some((f - f4, f3 + f5)),
        5 => Some((f + f4, f3 + f5)),
        2 => Some((f + f5, f3 - f4)),
        3 => Some((f + f5, f3 + f4)),
        _ => None,
    };
    if let Some((px, pz)) = at
    {
        world.spawnParticle("smoke", px as f64, f2 as f64, pz as f64, 0.0, 0.0, 0.0);
        world.spawnParticle("flame", px as f64, f2 as f64, pz as f64, 0.0, 0.0, 0.0);
    }
}

fn getBlockTextureFromSide(this: &Rc<Block>, face: i32) -> i32
{
    let index: i32 = this.m_blockIndexInTexture.get();
    if face == 1
    {
        return index + 17;
    }
    if face == 0
    {
        return index + 17;
    }
    if face == 3
    {
        return index - 1;
    }
    index
}

pub fn updateFurnaceBlockState(lit: bool, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let tileEntity: Option<Rc<TileEntity>> = world.getBlockTileEntity(x, y, z);
    KEEP_FURNACE_INVENTORY.with(|keep| keep.set(true));
    if lit
    {
        world.setBlockWithNotify(x, y, z, Block::stoneOvenActive().m_blockID);
    }
    else
    {
        world.setBlockWithNotify(x, y, z, Block::stoneOvenIdle().m_blockID);
    }
    KEEP_FURNACE_INVENTORY.with(|keep| keep.set(false));
    world.setBlockMetadataWithNotify(x, y, z, n);
    let tileEntity: Rc<TileEntity> = tileEntity.orNpe();
    tileEntity.cancelRemoval();
    world.setBlockTileEntity(x, y, z, &tileEntity);
}

pub fn onBlockPlacedBy(_this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, entity: &Rc<Entity>)
{
    let n: i32 = math_helper::floor_double((entity.m_rotationYaw.get() * 4.0 / 360.0) as f64 + 0.5) & 3;
    if n == 0
    {
        world.setBlockMetadataWithNotify(x, y, z, 2);
    }
    if n == 1
    {
        world.setBlockMetadataWithNotify(x, y, z, 5);
    }
    if n == 2
    {
        world.setBlockMetadataWithNotify(x, y, z, 3);
    }
    if n == 3
    {
        world.setBlockMetadataWithNotify(x, y, z, 4);
    }
}

fn onBlockRemoval(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    if !KEEP_FURNACE_INVENTORY.with(Cell::get)
    {
        // port cast is the java tile entity cast and throws ClassCastException on a wrong class
        let tileEntityFurnace: Rc<TileEntity> =
            tile_entity::cast(world.getBlockTileEntity(x, y, z), "TileEntityFurnace");
        // port spillInventory holds the slot spill java writes out in chest dispenser and furnace
        inventory::spillInventory(&furnace(this).m_furnaceRand, world, x, y, z, &tileEntityFurnace);
    }
    (block_container::CLASS.onBlockRemoval)(this, world, x, y, z);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockFurnace",
    m_parent: Some(&block_container::CLASS),
    idDropped: |_, _, _| Block::stoneOvenIdle().m_blockID,
    onBlockAdded: |this, world, x, y, z| {
        (block_container::CLASS.onBlockAdded)(this, world, x, y, z);
        setDefaultDirection(world, x, y, z);
    },
    getBlockTexture,
    randomDisplayTick,
    getBlockTextureFromSide,
    blockActivated: |_, world, x, y, z, player| {
        if world.m_multiplayerWorld.get()
        {
            return true;
        }
        // port castNullable is the java tile entity cast that lets null through
        let tileEntityFurnace: Option<Rc<TileEntity>> =
            tile_entity::castNullable(world.getBlockTileEntity(x, y, z), "TileEntityFurnace");
        player.displayGUIFurnace(tileEntityFurnace.as_ref());
        true
    },
    getBlockEntity: |_| tile_entity_furnace::create(),
    onBlockPlacedBy,
    onBlockRemoval,
    ..block_container::CLASS
};

pub fn create(id: i32, lit: bool) -> Rc<Block>
{
    let ext: FurnaceExt = FurnaceExt { m_furnaceRand: Random::new(), m_isActive: lit };
    let block: Rc<Block> = block_container::construct(&CLASS, Box::new(ext), id, Material::rock());
    block.m_blockIndexInTexture.set(45);
    block
}
