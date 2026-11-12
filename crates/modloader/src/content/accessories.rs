use crate::host::context::{stackData, stackFrom};
use game::entity::{Entity, entity_player};
use game::inventory::InventoryPlayer;
use mod_protocol::Identifier;
use mod_protocol::abi::{RegistryKind, StackData, Worn};
use mod_protocol::spec::AccessorySlotSpec;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

// the loader namespace is reserved, so no mod's synced bytes share this owner
const OWNER: &str = "loader:accessories";

struct Slot
{
    m_id: Identifier,
    m_spec: AccessorySlotSpec,
}

// saved by name so a remap or a slot added later never moves an item
#[derive(Serialize, Deserialize, Clone)]
struct Saved
{
    m_slot: Identifier,
    m_item: Identifier,
    m_count: i32,
    m_damage: i32,
}

thread_local!
{
    // in registration order, which panels show them in
    static SLOTS: RefCell<Vec<Slot>> = const { RefCell::new(Vec::new()) };
}

pub fn register(id: Identifier, spec: AccessorySlotSpec) -> Result<(), String>
{
    crate::registry::claim(RegistryKind::AccessorySlot, &id)?;
    SLOTS.with(|slots| slots.borrow_mut().push(Slot { m_id: id, m_spec: spec }));
    Ok(())
}

fn load(entity: &Entity) -> Vec<Saved>
{
    let bytes: Vec<u8> = crate::entities::synced(entity, OWNER);
    if bytes.is_empty()
    {
        return Vec::new();
    }

    let saved: Vec<Saved> = mod_protocol::wire::decode(&bytes).unwrap_or_else(|error|
    {
        crate::log::warn(
            crate::log::LOADER,
            &format!("accessories of entity {} unreadable: {error}", entity.m_entityId.get()),
        );
        Vec::new()
    });
    saved.into_iter().map(renamed).collect()
}

fn renamed(saved: Saved) -> Saved
{
    Saved { m_slot: crate::remap::rename(&saved.m_slot), m_item: crate::remap::rename(&saved.m_item), ..saved }
}

// a slot an uninstalled mod registered keeps its item until that mod comes back
fn store(entity: &Rc<Entity>, saved: &[Saved])
{
    crate::entities::publishSynced(entity, OWNER, mod_protocol::wire::encode(&saved));
}

// every registered slot with what the entity wears there
pub fn worn(entity: &Entity) -> Vec<Worn>
{
    let saved: Vec<Saved> = load(entity);
    SLOTS.with(|slots|
    {
        crate::registry::with(|registries|
        {
            slots
                .borrow()
                .iter()
                .map(|slot|
                {
                    let stack: Option<StackData> = saved.iter().find(|row| row.m_slot == slot.m_id).and_then(|row|
                    {
                        let id: i32 = registries.itemId(&row.m_item)?;
                        Some(StackData { id, count: row.m_count, damage: row.m_damage })
                    });
                    Worn { slot: slot.m_id.clone(), icon: slot.m_spec.icon.clone(), stack }
                })
                .collect()
        })
    })
}

// the slot's tag takes the stack, and a slot holds one item
fn accepts(slot: &Identifier, stack: &StackData) -> Result<Identifier, String>
{
    let tag: Identifier = SLOTS
        .with(|slots| slots.borrow().iter().find(|row| row.m_id == *slot).map(|row| row.m_spec.tag.clone()))
        .ok_or_else(|| format!("no accessory slot {slot}"))?;
    let item: Identifier =
        crate::registry::with(|registries| registries.m_items.byNumeric(stack.id).map(|row| row.m_id.clone()))
            .ok_or_else(|| format!("item {} is not registered", stack.id))?;

    if stack.count != 1 || !crate::content::tags::contains(RegistryKind::Item, &tag, &item)
    {
        return Err(format!("accessory slot {slot} does not take {} of {item}", stack.count));
    }

    Ok(item)
}

pub fn set(entity: &Rc<Entity>, slot: &Identifier, stack: Option<&StackData>) -> Result<(), String>
{
    let item: Option<Identifier> = stack.map(|stack| accepts(slot, stack)).transpose()?;
    if stack.is_none() && !SLOTS.with(|slots| slots.borrow().iter().any(|row| row.m_id == *slot))
    {
        return Err(format!("no accessory slot {slot}"));
    }

    let mut saved: Vec<Saved> = load(entity);
    saved.retain(|row| row.m_slot != *slot);

    if let (Some(item), Some(stack)) = (item, stack)
    {
        saved.push(Saved { m_slot: slot.clone(), m_item: item, m_count: stack.count, m_damage: stack.damage });
    }

    store(entity, &saved);
    Ok(())
}

// SlotAccessory: the cursor stack and the slot trade places when the slot takes the cursor stack or it is empty
pub fn swap(player: &Rc<Entity>, slot: &Identifier) -> Result<bool, String>
{
    let inventory: Rc<InventoryPlayer> = entity_player::inventory(player);
    let cursor: Option<StackData> = inventory.getItemStack().map(|stack| stackData(&stack));
    if let Some(stack) = &cursor
        && accepts(slot, stack).is_err()
    {
        return Ok(false);
    }

    let taken: Option<StackData> = worn(player).into_iter().find(|worn| worn.slot == *slot).and_then(|worn| worn.stack);
    set(player, slot, cursor.as_ref())?;
    inventory.setItemStack(taken.as_ref().and_then(stackFrom));
    crate::netplay::syncCursor(player);
    Ok(true)
}
