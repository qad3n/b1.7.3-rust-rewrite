use crate::block::{Block, BlockClass, Material, block_container};
use crate::entity::Entity;
use crate::inventory::{IInventory, InventoryLargeChest};
use crate::port::{inventory, tile_entity};
use crate::tileentity::{TileEntity, tile_entity_chest};
use crate::world::{IBlockAccess, World};
use jvm::random::Random;
use std::rc::Rc;

pub struct ChestExt
{
    m_random: Random,
}

fn getBlockTexture(this: &Rc<Block>, world: &dyn IBlockAccess, x: i32, y: i32, z: i32, face: i32) -> i32
{
    let index: i32 = this.m_blockIndexInTexture.get();
    let id: i32 = this.m_blockID;
    if face == 1
    {
        return index - 1;
    }
    if face == 0
    {
        return index - 1;
    }
    let n: i32 = world.getBlockId(x, y, z - 1);
    let n2: i32 = world.getBlockId(x, y, z + 1);
    let n3: i32 = world.getBlockId(x - 1, y, z);
    let n4: i32 = world.getBlockId(x + 1, y, z);
    let opaque = Block::opaqueCubeLookup;
    if n == id || n2 == id
    {
        if face == 2 || face == 3
        {
            return index;
        }
        let mut n5: i32 = 0;
        if n == id
        {
            n5 = -1;
        }
        let n6: i32 = world.getBlockId(x - 1, y, if n == id { z - 1 } else { z + 1 });
        let n7: i32 = world.getBlockId(x + 1, y, if n == id { z - 1 } else { z + 1 });
        if face == 4
        {
            n5 = -1 - n5;
        }
        let mut n8: i32 = 5;
        if (opaque(n3) || opaque(n6)) && !opaque(n4) && !opaque(n7)
        {
            n8 = 5;
        }
        if (opaque(n4) || opaque(n7)) && !opaque(n3) && !opaque(n6)
        {
            n8 = 4;
        }
        return (if face == n8 { index + 16 } else { index + 32 }) + n5;
    }
    if n3 == id || n4 == id
    {
        if face == 4 || face == 5
        {
            return index;
        }
        let mut n9: i32 = 0;
        if n3 == id
        {
            n9 = -1;
        }
        let n10: i32 = world.getBlockId(if n3 == id { x - 1 } else { x + 1 }, y, z - 1);
        let n11: i32 = world.getBlockId(if n3 == id { x - 1 } else { x + 1 }, y, z + 1);
        if face == 3
        {
            n9 = -1 - n9;
        }
        let mut n12: i32 = 3;
        if (opaque(n) || opaque(n10)) && !opaque(n2) && !opaque(n11)
        {
            n12 = 3;
        }
        if (opaque(n2) || opaque(n11)) && !opaque(n) && !opaque(n10)
        {
            n12 = 2;
        }
        return (if face == n12 { index + 16 } else { index + 32 }) + n9;
    }
    let mut n13: i32 = 3;
    if opaque(n) && !opaque(n2)
    {
        n13 = 3;
    }
    if opaque(n2) && !opaque(n)
    {
        n13 = 2;
    }
    if opaque(n3) && !opaque(n4)
    {
        n13 = 5;
    }
    if opaque(n4) && !opaque(n3)
    {
        n13 = 4;
    }
    if face == n13 { index + 1 } else { index }
}

fn getBlockTextureFromSide(this: &Rc<Block>, face: i32) -> i32
{
    let index: i32 = this.m_blockIndexInTexture.get();
    if face == 1
    {
        return index - 1;
    }
    if face == 0
    {
        return index - 1;
    }
    if face == 3
    {
        return index + 1;
    }
    index
}

fn canPlaceBlockAt(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    let mut n: i32 = 0;
    if world.getBlockId(x - 1, y, z) == this.m_blockID
    {
        n += 1;
    }
    if world.getBlockId(x + 1, y, z) == this.m_blockID
    {
        n += 1;
    }
    if world.getBlockId(x, y, z - 1) == this.m_blockID
    {
        n += 1;
    }
    if world.getBlockId(x, y, z + 1) == this.m_blockID
    {
        n += 1;
    }
    if n > 1
    {
        return false;
    }
    if isThereANeighborChest(this, world, x - 1, y, z)
    {
        return false;
    }
    if isThereANeighborChest(this, world, x + 1, y, z)
    {
        return false;
    }
    if isThereANeighborChest(this, world, x, y, z - 1)
    {
        return false;
    }
    !isThereANeighborChest(this, world, x, y, z + 1)
}

fn isThereANeighborChest(this: &Block, world: &Rc<World>, x: i32, y: i32, z: i32) -> bool
{
    if world.getBlockId(x, y, z) != this.m_blockID
    {
        return false;
    }
    if world.getBlockId(x - 1, y, z) == this.m_blockID
    {
        return true;
    }
    if world.getBlockId(x + 1, y, z) == this.m_blockID
    {
        return true;
    }
    if world.getBlockId(x, y, z - 1) == this.m_blockID
    {
        return true;
    }
    world.getBlockId(x, y, z + 1) == this.m_blockID
}

fn onBlockRemoval(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32)
{
    // port cast is the java tile entity cast and throws ClassCastException on a wrong class
    let tileEntityChest: Rc<TileEntity> = tile_entity::cast(world.getBlockTileEntity(x, y, z), "TileEntityChest");
    // port spillInventory holds the slot spill java writes out in chest dispenser and furnace
    inventory::spillInventory(&this.ext::<ChestExt>().m_random, world, x, y, z, &tileEntityChest);
    (block_container::CLASS.onBlockRemoval)(this, world, x, y, z);
}

fn chestAt(world: &Rc<World>, x: i32, y: i32, z: i32) -> Option<Rc<dyn IInventory>>
{
    // port castNullable is the java tile entity cast that lets null through
    tile_entity::castNullable(world.getBlockTileEntity(x, y, z), "TileEntityChest")
        .map(|chest| chest as Rc<dyn IInventory>)
}

fn blockActivated(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, player: &Rc<Entity>) -> bool
{
    let mut iInventory: Option<Rc<dyn IInventory>> = chestAt(world, x, y, z);
    if world.isBlockNormalCube(x, y + 1, z)
    {
        return true;
    }
    if world.getBlockId(x - 1, y, z) == this.m_blockID && world.isBlockNormalCube(x - 1, y + 1, z)
    {
        return true;
    }
    if world.getBlockId(x + 1, y, z) == this.m_blockID && world.isBlockNormalCube(x + 1, y + 1, z)
    {
        return true;
    }
    if world.getBlockId(x, y, z - 1) == this.m_blockID && world.isBlockNormalCube(x, y + 1, z - 1)
    {
        return true;
    }
    if world.getBlockId(x, y, z + 1) == this.m_blockID && world.isBlockNormalCube(x, y + 1, z + 1)
    {
        return true;
    }
    if world.getBlockId(x - 1, y, z) == this.m_blockID
    {
        iInventory = Some(
            InventoryLargeChest::new("Large chest", chestAt(world, x - 1, y, z), iInventory) as Rc<dyn IInventory>
        );
    }
    if world.getBlockId(x + 1, y, z) == this.m_blockID
    {
        iInventory = Some(
            InventoryLargeChest::new("Large chest", iInventory, chestAt(world, x + 1, y, z)) as Rc<dyn IInventory>
        );
    }
    if world.getBlockId(x, y, z - 1) == this.m_blockID
    {
        iInventory = Some(
            InventoryLargeChest::new("Large chest", chestAt(world, x, y, z - 1), iInventory) as Rc<dyn IInventory>
        );
    }
    if world.getBlockId(x, y, z + 1) == this.m_blockID
    {
        iInventory = Some(
            InventoryLargeChest::new("Large chest", iInventory, chestAt(world, x, y, z + 1)) as Rc<dyn IInventory>
        );
    }
    if world.m_multiplayerWorld.get()
    {
        return true;
    }
    player.displayGUIChest(iInventory.as_ref());
    true
}

pub const CLASS: BlockClass = BlockClass {
    m_name: "BlockChest",
    m_parent: Some(&block_container::CLASS),
    getBlockTexture,
    getBlockTextureFromSide,
    canPlaceBlockAt,
    onBlockRemoval,
    blockActivated,
    getBlockEntity: |_| tile_entity_chest::create(),
    ..block_container::CLASS
};

pub fn create(id: i32) -> Rc<Block>
{
    let block: Rc<Block> =
        block_container::construct(&CLASS, Box::new(ChestExt { m_random: Random::new() }), id, Material::wood());
    block.m_blockIndexInTexture.set(26);
    block
}
