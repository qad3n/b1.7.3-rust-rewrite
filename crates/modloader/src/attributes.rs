use crate::network::loader::{self, AttributeSync, SyncedModifier};
use game::attribute::{self, Attribute, AttributeType, Modifier, Operation};
use game::entity::Entity;
use game::entity::entity_living::LivingFields;
use mod_protocol::Identifier;
use mod_protocol::abi::ModifierOperation;
use mod_protocol::spec::AttributeSpec;
use std::rc::Rc;

pub fn register(id: Identifier, spec: AttributeSpec) -> Result<(), String>
{
    if spec.min > spec.max || !(spec.min..=spec.max).contains(&spec.default)
    {
        return Err(format!("{id} needs min <= default <= max"));
    }

    attribute::register(AttributeType
    {
        m_name: id.to_string(),
        m_default: spec.default,
        m_min: spec.min,
        m_max: spec.max,
    });
    Ok(())
}

pub fn value(entity: &Entity, attribute: &Identifier) -> Result<f64, String>
{
    let attribute: Attribute = attributeOf(attribute)?;
    entity.m_ext.living().ok_or("only living entities have attributes")?;
    Ok(attribute::value(entity, attribute, attribute.default()))
}

pub fn set(
    entity: &Rc<Entity>,
    attribute: &Identifier,
    modifier: &Identifier,
    amount: f64,
    operation: ModifierOperation,
) -> Result<(), String>
{
    let attribute: Attribute = attributeOf(attribute)?;
    let living: &LivingFields = entity.m_ext.living().ok_or("only living entities have attributes")?;
    living.m_attributes.set(Modifier
    {
        m_attribute: attribute,
        m_id: modifier.to_string(),
        m_amount: amount,
        m_operation: operationOf(operation),
    });

    syncOwn(entity);
    Ok(())
}

// writing the max health field from before api 1.27 becomes an added modifier on the beta 20
pub fn setMaxHealth(entity: &Rc<Entity>, health: i32)
{
    let Some(living) = entity.m_ext.living()
    else
    {
        return;
    };

    let id: &str = "loader:max_health";
    let extra: i32 = health - attribute::VANILLA_MAX_HEALTH;
    match extra
    {
        0 => living.m_attributes.remove(Attribute::MAX_HEALTH, id),
        _ => living.m_attributes.set(Modifier
        {
            m_attribute: Attribute::MAX_HEALTH,
            m_id: id.to_owned(),
            m_amount: extra as f64,
            m_operation: Operation::Add,
        }),
    }

    syncOwn(entity);
}

pub fn remove(entity: &Rc<Entity>, attribute: &Identifier, modifier: &Identifier) -> Result<(), String>
{
    let attribute: Attribute = attributeOf(attribute)?;
    let living: &LivingFields = entity.m_ext.living().ok_or("only living entities have attributes")?;
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
        .map(|modifier| SyncedModifier
        {
            attribute: modifier.m_attribute.name(),
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
        .filter_map(|modifier|
        {
            Some(Modifier
            {
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
    Attribute::byName(&id.to_string()).ok_or_else(|| format!("{id} is not an attribute"))
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
