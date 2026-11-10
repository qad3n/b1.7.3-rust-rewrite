use crate::network::loader::{self, AttributeSync, SyncedModifier};
use game::attribute::{Attribute, Modifier, Operation};
use game::entity::Entity;
use mod_protocol::Identifier;
use mod_protocol::abi::ModifierOperation;
use std::rc::Rc;

pub fn set(
    entity: &Rc<Entity>,
    attribute: &Identifier,
    modifier: &Identifier,
    amount: f64,
    operation: ModifierOperation,
) -> Result<(), String>
{
    let attribute: Attribute = attributeOf(attribute)?;
    let living = entity.m_ext.living().ok_or("only living entities have attributes")?;
    living.m_attributes.set(Modifier {
        m_attribute: attribute,
        m_id: modifier.to_string(),
        m_amount: amount,
        m_operation: operationOf(operation),
    });
    syncOwn(entity);
    Ok(())
}

pub fn remove(entity: &Rc<Entity>, attribute: &Identifier, modifier: &Identifier) -> Result<(), String>
{
    let attribute: Attribute = attributeOf(attribute)?;
    let living = entity.m_ext.living().ok_or("only living entities have attributes")?;
    living.m_attributes.remove(attribute, &modifier.to_string());
    syncOwn(entity);
    Ok(())
}

// a player's client moves the player and draws its hearts and armor, so it needs that player's modifiers
pub fn syncOwn(entity: &Rc<Entity>)
{
    if !crate::side::isServer() || !entity.instanceOf("EntityPlayer")
    {
        return;
    }

    let Some(living) = entity.m_ext.living()
    else
    {
        return;
    };
    let modifiers: Vec<SyncedModifier> = living
        .m_attributes
        .modifiers()
        .into_iter()
        .map(|modifier| SyncedModifier {
            attribute: modifier.m_attribute.name().to_owned(),
            id: modifier.m_id,
            amount: modifier.m_amount,
            operation: modifier.m_operation.name().to_owned(),
        })
        .collect();
    loader::send(loader::ATTRIBUTES, entity, &AttributeSync { id: entity.m_entityId.get(), modifiers });
}

pub fn received(message: AttributeSync)
{
    let Some(player) = crate::network::localPlayer().filter(|player| player.m_entityId.get() == message.id)
    else
    {
        return;
    };
    let Some(living) = player.m_ext.living()
    else
    {
        return;
    };
    let modifiers: Vec<Modifier> = message
        .modifiers
        .into_iter()
        .filter_map(|modifier| {
            Some(Modifier {
                m_attribute: Attribute::byName(&modifier.attribute)?,
                m_id: modifier.id,
                m_amount: modifier.amount,
                m_operation: Operation::byName(&modifier.operation)?,
            })
        })
        .collect();
    living.m_attributes.replace(modifiers);
}

fn attributeOf(id: &Identifier) -> Result<Attribute, String>
{
    id.isVanilla().then(|| Attribute::byName(id.path())).flatten().ok_or_else(|| format!("{id} is not an attribute"))
}

fn operationOf(operation: ModifierOperation) -> Operation
{
    match operation
    {
        ModifierOperation::Add => Operation::Add,
        ModifierOperation::MultiplyBase => Operation::MultiplyBase,
        ModifierOperation::MultiplyTotal => Operation::MultiplyTotal,
    }
}
