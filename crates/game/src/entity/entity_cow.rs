use crate::entity::{Entity, EntityClass, EntityExt, entity_animal, entity_creature, entity_living, entity_player};
use crate::inventory::{IInventory, InventoryPlayer};
use crate::item::{Item, ItemStack};
use crate::world::World;
use jvm::string::JString;
use std::rc::Rc;

#[derive(Default)]
pub struct AnimalExt
{
    pub m_living: entity_living::LivingFields,
    pub m_creature: entity_creature::CreatureFields,
}

impl EntityExt for AnimalExt
{
    fn asAny(&self) -> &dyn std::any::Any { self }

    fn living(&self) -> Option<&entity_living::LivingFields> { Some(&self.m_living) }

    fn creature(&self) -> Option<&entity_creature::CreatureFields> { Some(&self.m_creature) }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityCow",
    m_parent: Some(&entity_animal::CLASS),
    writeEntityToNBT: |this, nbt| (entity_animal::CLASS.writeEntityToNBT)(this, nbt),
    readEntityFromNBT: |this, nbt| (entity_animal::CLASS.readEntityFromNBT)(this, nbt),
    getLivingSound: |_| Some("mob.cow".to_string()),
    getHurtSound: |_| Some("mob.cowhurt".to_string()),
    getDeathSound: |_| Some("mob.cowhurt".to_string()),
    getSoundVolume: |_| 0.4,
    getDropItemId: |_| Item::leather().m_shiftedIndex,
    interact: |_, player| {
        let inventory: Rc<InventoryPlayer> = entity_player::inventory(player);
        let itemStack: Option<Rc<ItemStack>> = inventory.getCurrentItem();
        if itemStack.is_some_and(|stack| stack.m_itemID.get() == Item::bucketEmpty().m_shiftedIndex)
        {
            inventory.setInventorySlotContents(
                inventory.m_currentItem.get(),
                Some(ItemStack::withItem(&Item::bucketMilk())),
            );
            return true;
        }
        false
    },
    ..entity_animal::CLASS
};

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(AnimalExt::default()));
    entity_animal::init(&entity, world);
    entity_living::fields(&entity).m_texture.set(Some(JString::from("/mob/cow.png")));
    entity.setSize(0.9, 1.3);
    entity
}
