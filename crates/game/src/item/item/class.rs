use crate::block::Block;
use crate::entity::Entity;
use crate::item::{Item, ItemStack};
use crate::network::packet::Packet;
use crate::world::World;
use std::rc::Rc;

crate::vtable! {
    ItemClass for Item {
        fn canHarvestBlock(block: &Rc<Block>) -> bool;
        fn getColorFromDamage(metadata: i32) -> i32;
        fn getDamageVsEntity(target: Option<&Rc<Entity>>) -> i32;
        fn getIconFromDamage(metadata: i32) -> i32;
        fn getItemName() -> Option<String>;
        fn getItemNameIS(stack: &Rc<ItemStack>) -> Option<String>;
        fn getPlacedBlockMetadata(metadata: i32) -> i32;
        fn getStrVsBlock(stack: &Rc<ItemStack>, block: &Rc<Block>) -> f32;
        fn hitEntity(stack: &Rc<ItemStack>, target: &Rc<Entity>, attacker: &Rc<Entity>) -> bool;
        fn isFull3D() -> bool;
        fn isNetworkSynced() -> bool;
        fn getUpdatePacket(stack: &Rc<ItemStack>, world: &Rc<World>, player: &Rc<Entity>) -> Option<Box<dyn Packet>>;
        fn onBlockDestroyed(stack: &Rc<ItemStack>, block: i32, x: i32, y: i32, z: i32, entity: &Rc<Entity>) -> bool;
        fn onCreated(stack: &Rc<ItemStack>, world: &Rc<World>, player: &Rc<Entity>);
        fn onItemRightClick(stack: &Rc<ItemStack>, world: &Rc<World>, player: &Rc<Entity>) -> Option<Rc<ItemStack>>;
        fn onItemUse(stack: &Rc<ItemStack>, player: &Rc<Entity>, world: &Rc<World>, x: i32, y: i32, z: i32, face: i32) -> bool;
        fn onUpdate(stack: &Rc<ItemStack>, world: &Rc<World>, entity: &Rc<Entity>, slot: i32, selected: bool);
        fn saddleEntity(stack: &Rc<ItemStack>, entity: &Rc<Entity>);
        fn shouldRotateAroundWhenRendering() -> bool;
    }
}

pub const ITEM: ItemClass = ItemClass {
    m_name: "Item",
    m_parent: None,
    m_interfaces: &[],
    canHarvestBlock: |_, _| false,
    getColorFromDamage: |_, _| 0xFFFFFF,
    getDamageVsEntity: |_, _| 1,
    getIconFromDamage: |this, _| this.m_iconIndex.get(),
    getItemName: |this| this.m_itemName.get(),
    getItemNameIS: |this, _| this.m_itemName.get(),
    getPlacedBlockMetadata: |_, _| 0,
    getStrVsBlock: |_, _, _| 1.0,
    hitEntity: |_, _, _, _| false,
    isFull3D: |this| this.m_bFull3D.get(),
    isNetworkSynced: |_| false,
    getUpdatePacket: |_, _, _, _| crate::port::abstractMethod("getUpdatePacket"),
    onBlockDestroyed: |_, _, _, _, _, _, _| false,
    onCreated: |_, _, _, _| {},
    onItemRightClick: |_, stack, _, _| Some(Rc::clone(stack)),
    onItemUse: |_, _, _, _, _, _, _, _| false,
    onUpdate: |_, _, _, _, _, _| {},
    saddleEntity: |_, _, _| {},
    shouldRotateAroundWhenRendering: |_| false,
};
