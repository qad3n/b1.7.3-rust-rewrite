use crate::block::{Block, StepSound};
use crate::item::{EnumToolMaterial, ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemHoe",
    m_parent: Some(&ITEM),
    onItemUse: |_, stack, player, world, x, y, z, face| {
        let n: i32 = world.getBlockId(x, y, z);
        let n2: i32 = world.getBlockId(x, y + 1, z);
        if face != 0 && n2 == 0 && n == Block::grass().m_blockID || n == Block::dirt().m_blockID
        {
            let block: Rc<Block> = Block::tilledField();
            let sound: Rc<StepSound> = block.m_stepSound.get();
            world.playSoundEffect(
                (x as f32 + 0.5) as f64,
                (y as f32 + 0.5) as f64,
                (z as f32 + 0.5) as f64,
                &sound.getStepping(),
                (sound.getVolume() + 1.0) / 2.0,
                sound.getPitch() * 0.8,
            );
            if world.m_multiplayerWorld.get()
            {
                return true;
            }
            world.setBlockWithNotify(x, y, z, block.m_blockID);
            stack.damageItem(1, player);
            return true;
        }
        false
    },
    isFull3D: |_| true,
    ..ITEM
};

pub fn create(id: i32, tier: EnumToolMaterial) -> Rc<Item>
{
    let item: Rc<Item> = Item::construct(&CLASS, Box::new(()), id);
    item.m_maxStackSize.set(1);
    item.setMaxDamage(tier.getMaxUses())
}
