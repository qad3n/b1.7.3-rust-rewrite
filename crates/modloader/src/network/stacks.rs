use common::nbt::attachments::Attachments;
use game::container::Container;
use game::entity::{Entity, entity_player};
use game::item::ItemStack;
use game::network::packet::Packet;
use game::network::packets::{Packet5PlayerInventory, Packet21PickupSpawn, Packet103SetSlot, Packet104WindowItems};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

pub const STACK_COMPONENTS: &str = "loader:stack_components";

// the vanilla packets carry only id, count and damage, so synced components follow in their own message
#[derive(Serialize, Deserialize)]
pub enum Holder
{
    Slot
    {
        window: i32, slot: i32
    },
    Equipment
    {
        entity: i32, slot: i32
    },
    Dropped
    {
        entity: i32
    },
}

#[derive(Serialize, Deserialize)]
pub struct Stack
{
    pub holder: Holder,
    pub components: Attachments,
}

#[derive(Serialize, Deserialize)]
pub struct StackBatch
{
    pub stacks: Vec<Stack>,
}

thread_local!
{
    // component types whose values clients need, such as a charge a tooltip or a durability bar shows
    static SYNCED: RefCell<BTreeSet<String>> = const { RefCell::new(BTreeSet::new()) };
}

pub fn register(id: Identifier, synced: bool) -> Result<(), String>
{
    crate::registry::claim(RegistryKind::Component, &id)?;

    if synced
    {
        SYNCED.with(|types| types.borrow_mut().insert(id.to_string()));
    }

    Ok(())
}

fn synced(stack: &ItemStack) -> Attachments
{
    SYNCED.with(|types|
    {
        let types = types.borrow();
        stack
            .m_components
            .borrow()
            .iter()
            .filter(|(key, _)| types.contains(*key))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    })
}

// the stack a window slot holds, window -1 is the cursor and 0 the player's own inventory
fn slotStack(player: &Rc<Entity>, window: i32, slot: i32) -> Option<Rc<ItemStack>>
{
    let fields: &entity_player::PlayerFields = entity_player::fields(player);
    let container: Option<Rc<Container>> = match window
    {
        -1 => return entity_player::inventory(player).getItemStack(),
        0 => fields.m_inventorySlots.get(),
        _ => fields.m_craftingInventory.get().filter(|container| container.m_windowId.get() == window),
    };
    container.and_then(|container| container.getSlot(slot).getStack())
}

fn equipment(entity: &Rc<Entity>, slot: i32) -> Option<Rc<ItemStack>>
{
    entity.getInventory()?.get(usize::try_from(slot).ok()?).cloned().flatten()
}

fn dropped(entity: &Rc<Entity>) -> Option<Rc<ItemStack>>
{
    let fields: Option<&game::entity::entity_item::ItemFields> = entity.m_ext.asAny().downcast_ref();
    fields.and_then(|fields| fields.m_item.get())
}

fn holderStack(player: &Rc<Entity>, holder: &Holder) -> Option<Rc<ItemStack>>
{
    let entity = |id: i32| player.m_worldObj.get().and_then(|world| crate::host::context::entityById(&world, id));

    match *holder
    {
        Holder::Slot { window, slot } => slotStack(player, window, slot),
        Holder::Equipment { entity: id, slot } => equipment(&entity(id)?, slot),
        Holder::Dropped { entity: id } => dropped(&entity(id)?),
    }
}

// the holders a vanilla packet fills, read on the server right as the packet leaves
fn holders(packet: &dyn Any) -> Vec<Holder>
{
    if let Some(set) = packet.downcast_ref::<Packet103SetSlot>()
    {
        return vec![Holder::Slot { window: set.m_windowId, slot: set.m_itemSlot }];
    }

    if let Some(items) = packet.downcast_ref::<Packet104WindowItems>()
    {
        let count: usize = items.m_itemStack.as_ref().map_or(0, Vec::len);
        return (0..count)
            .filter_map(|slot| i32::try_from(slot).ok())
            .map(|slot| Holder::Slot { window: items.m_windowId, slot })
            .collect();
    }

    if let Some(equipment) = packet.downcast_ref::<Packet5PlayerInventory>()
    {
        return vec![Holder::Equipment { entity: equipment.m_entityID, slot: equipment.m_slot }];
    }

    if let Some(spawn) = packet.downcast_ref::<Packet21PickupSpawn>()
    {
        return vec![Holder::Dropped { entity: spawn.m_entityId }];
    }

    Vec::new()
}

// the synced components of every stack a vanilla packet sends, none when no stack carries any
pub fn outgoing(player: &Rc<Entity>, packet: &dyn Packet) -> Option<StackBatch>
{
    if SYNCED.with(|types| types.borrow().is_empty())
    {
        return None;
    }

    let any: &dyn Any = packet;
    let stacks: Vec<Stack> = holders(any)
        .into_iter()
        .filter_map(|holder|
        {
            let stack: Rc<ItemStack> = holderStack(player, &holder)?;
            let components: Attachments = synced(&stack);
            (!components.is_empty()).then_some(Stack { holder, components })
        })
        .collect();
    (!stacks.is_empty()).then_some(StackBatch { stacks })
}

pub fn send(player: &Rc<Entity>, batch: &StackBatch) { super::loader::send(STACK_COMPONENTS, player, batch); }

// the vanilla packet already built the stack, its synced components join it
pub fn received(batch: StackBatch)
{
    let Some(player) = crate::network::localPlayer()
    else
    {
        return;
    };

    for Stack { holder, components } in batch.stacks
    {
        match holderStack(&player, &holder)
        {
            Some(stack) => stack.m_components.borrow_mut().extend(components),
            None => crate::log::debug(crate::log::LOADER, "synced components arrived for a stack the client lacks"),
        }
    }
}
