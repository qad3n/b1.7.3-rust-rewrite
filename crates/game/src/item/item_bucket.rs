use crate::block::{Block, Material};
use crate::entity::Entity;
use crate::item::{ITEM, Item, ItemClass, ItemStack, item_boat};
use crate::math::{EnumMovingObjectType, MovingObjectPosition};
use crate::world::World;
use jvm::math;
use std::rc::Rc;

pub struct BucketExt
{
    m_isFull: i32,
}

fn onItemRightClick(
    this: &Rc<Item>,
    stack: &Rc<ItemStack>,
    world: &Rc<World>,
    player: &Rc<Entity>,
) -> Option<Rc<ItemStack>>
{
    let isFull: i32 = this.ext::<BucketExt>().m_isFull;
    let (movingobjectposition, d0, d1, d2): (Option<MovingObjectPosition>, f64, f64, f64) =
        item_boat::lookRay(world, player, isFull == 0);
    let Some(movingobjectposition) = movingobjectposition
    else
    {
        return Some(Rc::clone(stack));
    };
    if movingobjectposition.m_typeOfHit == EnumMovingObjectType::TILE
    {
        let mut i: i32 = movingobjectposition.m_blockX;
        let mut j: i32 = movingobjectposition.m_blockY;
        let mut k: i32 = movingobjectposition.m_blockZ;
        if !world.canModify(player, i, j, k)
        {
            return Some(Rc::clone(stack));
        }
        if isFull == 0
        {
            if Rc::ptr_eq(&world.getBlockMaterial(i, j, k), &Material::water()) && world.getBlockMetadata(i, j, k) == 0
            {
                world.setBlockWithNotify(i, j, k, 0);
                return Some(ItemStack::withItem(&Item::bucketWater()));
            }
            if Rc::ptr_eq(&world.getBlockMaterial(i, j, k), &Material::lava()) && world.getBlockMetadata(i, j, k) == 0
            {
                world.setBlockWithNotify(i, j, k, 0);
                return Some(ItemStack::withItem(&Item::bucketLava()));
            }
        }
        else
        {
            if isFull < 0
            {
                return Some(ItemStack::withItem(&Item::bucketEmpty()));
            }
            match movingobjectposition.m_sideHit
            {
                0 => j -= 1,
                1 => j += 1,
                2 => k -= 1,
                3 => k += 1,
                4 => i -= 1,
                5 => i += 1,
                _ =>
                {}
            }
            if world.isAirBlock(i, j, k) || !world.getBlockMaterial(i, j, k).isSolid()
            {
                if world.worldProvider().m_isHellWorld.get() && isFull == Block::waterMoving().m_blockID
                {
                    world.playSoundEffect(
                        d0 + 0.5,
                        d1 + 0.5,
                        d2 + 0.5,
                        "random.fizz",
                        0.5,
                        2.6 + (world.m_rand.nextFloat() - world.m_rand.nextFloat()) * 0.8,
                    );
                    for _ in 0..8
                    {
                        world.spawnParticle(
                            "largesmoke",
                            i as f64 + math::random(),
                            j as f64 + math::random(),
                            k as f64 + math::random(),
                            0.0,
                            0.0,
                            0.0,
                        );
                    }
                }
                else
                {
                    world.setBlockAndMetadataWithNotify(i, j, k, isFull, 0);
                }
                return Some(ItemStack::withItem(&Item::bucketEmpty()));
            }
        }
    }
    else if isFull == 0
        && movingobjectposition.m_entityHit.as_ref().is_some_and(|entity| entity.instanceOf("EntityCow"))
    {
        return Some(ItemStack::withItem(&Item::bucketMilk()));
    }
    Some(Rc::clone(stack))
}

pub const CLASS: ItemClass = ItemClass { m_name: "ItemBucket", m_parent: Some(&ITEM), onItemRightClick, ..ITEM };

pub fn create(id: i32, liquid: i32) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(BucketExt { m_isFull: liquid }), id);
    item.m_maxStackSize.set(1);
    item
}
