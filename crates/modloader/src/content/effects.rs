use crate::network::loader::{self, EffectSync, SyncedEffect};
use game::attribute::{Attribute, Operation};
use game::effect::{self, ActiveEffect, EffectType};
use game::entity::Entity;
use mod_protocol::Identifier;
use mod_protocol::abi::ModifierOperation;
use mod_protocol::spec::EffectSpec;
use std::rc::Rc;

pub fn register(id: Identifier, spec: EffectSpec) -> Result<(), String>
{
    let mut modifiers: Vec<(Attribute, f64, Operation)> = Vec::new();
    for modifier in &spec.modifiers
    {
        let attribute: Attribute = modifier
            .attribute
            .isVanilla()
            .then(|| Attribute::byName(modifier.attribute.path()))
            .flatten()
            .ok_or_else(|| format!("{} is not an attribute", modifier.attribute))?;
        modifiers.push((attribute, modifier.amount, operation(modifier.operation)));
    }
    effect::register(EffectType {
        m_name: id.to_string(),
        m_color: spec.color as i32,
        m_beneficial: spec.beneficial,
        m_icon: spec.icon.unwrap_or_default(),
        m_modifiers: modifiers,
        m_interval: spec.interval,
        m_amount: spec.amount,
    });
    Ok(())
}

pub fn add(entity: &Rc<Entity>, id: &Identifier, level: i32, ticks: i32) -> Result<(), String>
{
    let kind: Rc<EffectType> = effect::byName(&id.to_string()).ok_or_else(|| format!("{id} is not an effect"))?;
    entity.m_ext.living().ok_or("only living entities have effects")?;
    effect::add(entity, kind, level, ticks);
    syncOwn(entity);
    crate::attributes::syncOwn(entity);
    Ok(())
}

pub fn remove(entity: &Rc<Entity>, id: &Identifier)
{
    effect::remove(entity, &id.to_string());
    syncOwn(entity);
    crate::attributes::syncOwn(entity);
}

pub fn level(entity: &Entity, id: &Identifier) -> i32
{
    let name: String = id.to_string();
    entity
        .m_ext
        .living()
        .and_then(|living| {
            living.m_effects.list().into_iter().find(|(kind, _, _)| kind.m_name == name).map(|(_, level, _)| level)
        })
        .unwrap_or(-1)
}

// a player's client draws its effects and counts them down, so it gets the list on login and on every change
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
    let effects: Vec<SyncedEffect> = living
        .m_effects
        .list()
        .into_iter()
        .map(|(kind, level, ticks)| SyncedEffect { name: kind.m_name.clone(), level, ticks })
        .collect();
    loader::send(loader::EFFECTS, entity, &EffectSync { id: entity.m_entityId.get(), effects });
}

pub fn received(message: EffectSync)
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
    let active: Vec<ActiveEffect> = message
        .effects
        .into_iter()
        .filter_map(|synced| {
            Some(ActiveEffect { m_effect: effect::byName(&synced.name)?, m_level: synced.level, m_ticks: synced.ticks })
        })
        .collect();
    living.m_effects.replace(active);
}

fn operation(operation: ModifierOperation) -> Operation
{
    match operation
    {
        ModifierOperation::Add => Operation::Add,
        ModifierOperation::MultiplyBase => Operation::MultiplyBase,
        ModifierOperation::MultiplyTotal => Operation::MultiplyTotal,
    }
}
