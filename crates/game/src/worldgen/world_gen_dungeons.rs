use crate::block::Block;
use crate::item::{Item, ItemStack};
use crate::tileentity::{TileEntity, tile_entity_mob_spawner};
use crate::world::World;
use crate::worldgen::WorldGenerator;
use jvm::random::Random;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct WorldGenDungeons;

impl WorldGenDungeons
{
    fn pickCheckLootItem(&self, random: &Random) -> Option<Rc<ItemStack>>
    {
        let n: i32 = random.nextIntBounded(11);
        match n
        {
            0 => Some(ItemStack::withItem(&Item::saddle())),
            1 => Some(ItemStack::withItemCount(&Item::ingotIron(), random.nextIntBounded(4) + 1)),
            2 => Some(ItemStack::withItem(&Item::bread())),
            3 => Some(ItemStack::withItemCount(&Item::wheat(), random.nextIntBounded(4) + 1)),
            4 => Some(ItemStack::withItemCount(&Item::gunpowder(), random.nextIntBounded(4) + 1)),
            5 => Some(ItemStack::withItemCount(&Item::silk(), random.nextIntBounded(4) + 1)),
            6 => Some(ItemStack::withItem(&Item::bucketEmpty())),
            7 if random.nextIntBounded(100) == 0 => Some(ItemStack::withItem(&Item::appleGold())),
            8 if random.nextIntBounded(2) == 0 =>
            {
                Some(ItemStack::withItemCount(&Item::redstone(), random.nextIntBounded(4) + 1))
            }
            9 if random.nextIntBounded(10) == 0 =>
            {
                Some(ItemStack::withItem(&Item::byIdOrNpe(Item::record13().m_shiftedIndex + random.nextIntBounded(2))))
            }
            10 => Some(ItemStack::withItemDamage(&Item::dyePowder(), 1, 3)),
            _ => None,
        }
    }

    fn pickMobSpawner(&self, random: &Random) -> &'static str
    {
        match random.nextIntBounded(4)
        {
            0 => "Skeleton",
            1 | 2 => "Zombie",
            3 => "Spider",
            _ => "",
        }
    }
}

impl WorldGenerator for WorldGenDungeons
{
    fn generate(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let n4: i32 = 3;
        let n5: i32 = random.nextIntBounded(2) + 2;
        let n6: i32 = random.nextIntBounded(2) + 2;
        let mut n7: i32 = 0;
        for n3 in x - n5 - 1..=x + n5 + 1
        {
            for n2 in y - 1..=y + n4 + 1
            {
                for n in z - n6 - 1..=z + n6 + 1
                {
                    let solid: bool = world.getBlockMaterial(n3, n2, n).isSolid();
                    if n2 == y - 1 && !solid
                    {
                        return false;
                    }
                    if n2 == y + n4 + 1 && !solid
                    {
                        return false;
                    }
                    if (n3 == x - n5 - 1 || n3 == x + n5 + 1 || n == z - n6 - 1 || n == z + n6 + 1)
                        && n2 == y
                        && world.isAirBlock(n3, n2, n)
                        && world.isAirBlock(n3, n2 + 1, n)
                    {
                        n7 += 1;
                    }
                }
            }
        }
        if !(1..=5).contains(&n7)
        {
            return false;
        }
        for n3 in x - n5 - 1..=x + n5 + 1
        {
            for n2 in (y - 1..=y + n4).rev()
            {
                for n in z - n6 - 1..=z + n6 + 1
                {
                    if n3 == x - n5 - 1
                        || n2 == y - 1
                        || n == z - n6 - 1
                        || n3 == x + n5 + 1
                        || n2 == y + n4 + 1
                        || n == z + n6 + 1
                    {
                        if n2 >= 0 && !world.getBlockMaterial(n3, n2 - 1, n).isSolid()
                        {
                            world.setBlockWithNotify(n3, n2, n, 0);
                            continue;
                        }
                        if !world.getBlockMaterial(n3, n2, n).isSolid()
                        {
                            continue;
                        }
                        if n2 == y - 1 && random.nextIntBounded(4) != 0
                        {
                            world.setBlockWithNotify(n3, n2, n, Block::cobblestoneMossy().m_blockID);
                            continue;
                        }
                        world.setBlockWithNotify(n3, n2, n, Block::cobblestone().m_blockID);
                        continue;
                    }
                    world.setBlockWithNotify(n3, n2, n, 0);
                }
            }
        }
        'chests: for _ in 0..2
        {
            for _ in 0..3
            {
                let n: i32 = x + random.nextIntBounded(n5 * 2 + 1) - n5;
                let n9: i32 = y;
                let n8: i32 = z + random.nextIntBounded(n6 * 2 + 1) - n6;
                if !world.isAirBlock(n, n9, n8)
                {
                    continue;
                }
                let mut n10: i32 = 0;
                if world.getBlockMaterial(n - 1, n9, n8).isSolid()
                {
                    n10 += 1;
                }
                if world.getBlockMaterial(n + 1, n9, n8).isSolid()
                {
                    n10 += 1;
                }
                if world.getBlockMaterial(n, n9, n8 - 1).isSolid()
                {
                    n10 += 1;
                }
                if world.getBlockMaterial(n, n9, n8 + 1).isSolid()
                {
                    n10 += 1;
                }
                if n10 != 1
                {
                    continue;
                }
                world.setBlockWithNotify(n, n9, n8, Block::chest().m_blockID);
                let chest: Rc<TileEntity> = world.getBlockTileEntity(n, n9, n8).orNpe();
                for _ in 0..8
                {
                    let itemStack: Option<Rc<ItemStack>> = self.pickCheckLootItem(random);
                    if itemStack.is_none()
                    {
                        continue;
                    }
                    chest.setInventorySlotContents(random.nextIntBounded(chest.getSizeInventory()), itemStack);
                }
                continue 'chests;
            }
        }
        world.setBlockWithNotify(x, y, z, Block::mobSpawner().m_blockID);
        let spawner: Rc<TileEntity> = world.getBlockTileEntity(x, y, z).orNpe();
        tile_entity_mob_spawner::setMobID(&spawner, Some(JString::from(self.pickMobSpawner(random))));
        true
    }
}
