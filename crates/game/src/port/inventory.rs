use crate::entity::{Entity, entity_item};
use crate::item::ItemStack;
use crate::tileentity::TileEntity;
use crate::world::World;
use jvm::random::Random;
use std::rc::Rc;

pub fn spillInventory(random: &Random, world: &Rc<World>, x: i32, y: i32, z: i32, inventory: &Rc<TileEntity>)
{
    let mut i: i32 = 0;
    while i < inventory.getSizeInventory()
    {
        if let Some(itemStack) = inventory.getStackInSlot(i)
        {
            let f: f32 = random.nextFloat() * 0.8 + 0.1;
            let f2: f32 = random.nextFloat() * 0.8 + 0.1;
            let f3: f32 = random.nextFloat() * 0.8 + 0.1;
            while itemStack.m_stackSize.get() > 0
            {
                let mut n: i32 = random.nextIntBounded(21) + 10;
                if n > itemStack.m_stackSize.get()
                {
                    n = itemStack.m_stackSize.get();
                }
                itemStack.m_stackSize.set(itemStack.m_stackSize.get() - n);
                let entityItem: Rc<Entity> = entity_item::create(
                    world,
                    (x as f32 + f) as f64,
                    (y as f32 + f2) as f64,
                    (z as f32 + f3) as f64,
                    ItemStack::withDamage(itemStack.m_itemID.get(), n, itemStack.getItemDamage()),
                );
                let f4: f32 = 0.05;
                entityItem.m_motionX.set((random.nextGaussian() as f32 * f4) as f64);
                entityItem.m_motionY.set((random.nextGaussian() as f32 * f4 + 0.2) as f64);
                entityItem.m_motionZ.set((random.nextGaussian() as f32 * f4) as f64);
                world.entityJoinedWorld(&entityItem);
            }
        }
        i += 1;
    }
}
