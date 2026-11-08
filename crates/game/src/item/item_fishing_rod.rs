use crate::entity::{Entity, entity_fish, entity_player};
use crate::item::{ITEM, Item, ItemClass};
use std::rc::Rc;

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemFishingRod",
    m_parent: Some(&ITEM),
    isFull3D: |_| true,
    shouldRotateAroundWhenRendering: |_| true,
    onItemRightClick: |_, stack, world, player| {
        let fishEntity: Option<Rc<Entity>> = entity_player::fields(player).m_fishEntity.get();
        if let Some(fish) = fishEntity
        {
            let n: i32 = entity_fish::catchFish(&fish);
            stack.damageItem(n, player);
            player.swingItem();
        }
        else
        {
            world.playSoundAtEntity(player, "random.bow", 0.5, 0.4 / (Item::itemRand().nextFloat() * 0.4 + 0.8));
            if !world.m_multiplayerWorld.get()
            {
                world.entityJoinedWorld(&entity_fish::createFromAngler(world, player));
            }
            player.swingItem();
        }
        Some(Rc::clone(stack))
    },
    ..ITEM
};

pub fn create(id: i32) -> Rc<Item> { Item::construct(&CLASS, Box::new(()), id).setMaxDamage(64).setMaxStackSize(1) }
