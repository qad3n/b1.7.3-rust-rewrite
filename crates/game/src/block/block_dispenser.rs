use crate::block::{Block, BlockClass, Material, block_container, block_furnace};
use crate::entity::{Entity, entity_arrow, entity_egg, entity_item, entity_snowball};
use crate::item::{Item, ItemStack};
use crate::port::{inventory, tile_entity};
use crate::tileentity::{TileEntity, tile_entity_dispenser};
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct DispenserExt
{
    m_random: Random,
}

fn setDispenserDefaultDirection(world: &Rc<World>, x: i32, y: i32, z: i32)
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
    index + 1
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
        return index + 1;
    }
    index
}

fn dispenseItem(world: &Rc<World>, x: i32, y: i32, z: i32, random: &Random)
{
    let n: i32 = world.getBlockMetadata(x, y, z);
    let mut n2: i32 = 0;
    let mut n3: i32 = 0;
    if n == 3
    {
        n3 = 1;
    }
    else if n == 2
    {
        n3 = -1;
    }
    else
    {
        n2 = if n == 5 { 1 } else { -1 };
    }
    // port cast is the java tile entity cast and throws ClassCastException on a wrong class
    let tileEntityDispenser: Rc<TileEntity> =
        tile_entity::cast(world.getBlockTileEntity(x, y, z), "TileEntityDispenser");
    let itemStack: Option<Rc<ItemStack>> = tile_entity_dispenser::getRandomStackFromInventory(&tileEntityDispenser);
    let d: f64 = x as f64 + n2 as f64 * 0.6 + 0.5;
    let d2: f64 = y as f64 + 0.5;
    let d3: f64 = z as f64 + n3 as f64 * 0.6 + 0.5;
    let Some(itemStack) = itemStack
    else
    {
        world.doEvent(1001, x, y, z, 0);
        return;
    };
    let itemID: i32 = itemStack.m_itemID.get();
    if itemID == Item::arrow().m_shiftedIndex
    {
        let entityArrow: Rc<Entity> = entity_arrow::create(world, d, d2, d3);
        entity_arrow::setArrowHeading(&entityArrow, n2 as f64, 0.1f32 as f64, n3 as f64, 1.1, 6.0);
        entity_arrow::fields(&entityArrow).m_doesArrowBelongToPlayer.set(true);
        world.entityJoinedWorld(&entityArrow);
        world.doEvent(1002, x, y, z, 0);
    }
    else if itemID == Item::egg().m_shiftedIndex
    {
        let entityEgg: Rc<Entity> = entity_egg::create(world, d, d2, d3);
        entity_egg::setEggHeading(&entityEgg, n2 as f64, 0.1f32 as f64, n3 as f64, 1.1, 6.0);
        world.entityJoinedWorld(&entityEgg);
        world.doEvent(1002, x, y, z, 0);
    }
    else if itemID == Item::snowball().m_shiftedIndex
    {
        let entitySnowball: Rc<Entity> = entity_snowball::create(world, d, d2, d3);
        entity_snowball::setSnowballHeading(&entitySnowball, n2 as f64, 0.1f32 as f64, n3 as f64, 1.1, 6.0);
        world.entityJoinedWorld(&entitySnowball);
        world.doEvent(1002, x, y, z, 0);
    }
    else
    {
        let entityItem: Rc<Entity> = entity_item::create(world, d, d2 - 0.3, d3, itemStack);
        let d4: f64 = random.nextDouble() * 0.1 + 0.2;
        entityItem.m_motionX.set(n2 as f64 * d4);
        entityItem.m_motionY.set(0.2f32 as f64);
        entityItem.m_motionZ.set(n3 as f64 * d4);
        let motionX: f64 = entityItem.m_motionX.get();
        entityItem.m_motionX.set(motionX + random.nextGaussian() * 0.0075f32 as f64 * 6.0);
        let motionY: f64 = entityItem.m_motionY.get();
        entityItem.m_motionY.set(motionY + random.nextGaussian() * 0.0075f32 as f64 * 6.0);
        let motionZ: f64 = entityItem.m_motionZ.get();
        entityItem.m_motionZ.set(motionZ + random.nextGaussian() * 0.0075f32 as f64 * 6.0);
        world.entityJoinedWorld(&entityItem);
        world.doEvent(1000, x, y, z, 0);
    }
    world.doEvent(2000, x, y, z, n2 + 1 + (n3 + 1) * 3);
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockDispenser",
    m_parent: Some(&block_container::CLASS),
    tickRate: |_| 4,
    idDropped: |_, _, _| Block::dispenser().m_blockID,
    onBlockAdded: |this, world, x, y, z| {
        (block_container::CLASS.onBlockAdded)(this, world, x, y, z);
        setDispenserDefaultDirection(world, x, y, z);
    },
    getBlockTexture,
    getBlockTextureFromSide,
    blockActivated: |_, world, x, y, z, player| {
        if world.m_multiplayerWorld.get()
        {
            return true;
        }
        // port castNullable is the java tile entity cast that lets null through
        let tileEntityDispenser: Option<Rc<TileEntity>> =
            tile_entity::castNullable(world.getBlockTileEntity(x, y, z), "TileEntityDispenser");
        player.displayGUIDispenser(tileEntityDispenser.as_ref());
        true
    },
    onNeighborBlockChange: |this, world, x, y, z, neighborBlock| {
        if neighborBlock > 0 && Block::byId(neighborBlock).orNpe().canProvidePower()
        {
            let flag: bool =
                world.isBlockIndirectlyGettingPowered(x, y, z) || world.isBlockIndirectlyGettingPowered(x, y + 1, z);
            if flag
            {
                world.scheduleBlockUpdate(x, y, z, this.m_blockID, this.tickRate());
            }
        }
    },
    updateTick: |_, world, x, y, z, random| {
        if world.isBlockIndirectlyGettingPowered(x, y, z) || world.isBlockIndirectlyGettingPowered(x, y + 1, z)
        {
            dispenseItem(world, x, y, z, random);
        }
    },
    getBlockEntity: |_| tile_entity_dispenser::create(),
    onBlockPlacedBy: block_furnace::onBlockPlacedBy,
    onBlockRemoval: |this, world, x, y, z| {
        // port cast is the java tile entity cast and throws ClassCastException on a wrong class
        let tileEntityDispenser: Rc<TileEntity> =
            tile_entity::cast(world.getBlockTileEntity(x, y, z), "TileEntityDispenser");
        // port spillInventory holds the slot spill java writes out in chest dispenser and furnace
        inventory::spillInventory(&this.ext::<DispenserExt>().m_random, world, x, y, z, &tileEntityDispenser);
        (block_container::CLASS.onBlockRemoval)(this, world, x, y, z);
    },
    ..block_container::CLASS
};

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> =
        block_container::construct(&CLASS, Box::new(DispenserExt { m_random: Random::new() }), id, Material::rock());
    block.m_blockIndexInTexture.set(45);
    block
}
