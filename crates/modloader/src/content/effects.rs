use crate::host::context::{entityInfo, withWorld};
use crate::network::loader::{self, EffectSync, SyncedEffect};
use crate::state;
use game::attribute::{Attribute, Operation};
use game::effect::{self, ActiveEffect, EffectHooks, EffectType};
use game::entity::Entity;
use mod_protocol::Identifier;
use mod_protocol::abi::{EffectCall, GuestCall, ModifierOperation};
use mod_protocol::spec::{Behavior, EffectMethod, EffectSpec};
use std::rc::Rc;

// a mod's effect behavior, only the methods it overrides cross into wasm
struct ModEffect
{
    m_owner: usize,
    m_behavior: Behavior<EffectMethod>,
}

impl ModEffect
{
    // the call is built only for a method the mod overrides, ticks run every tick
    fn call(&self, entity: &Rc<Entity>, method: EffectMethod, build: impl FnOnce() -> EffectCall)
    {
        if !self.m_behavior.methods.contains(&method)
        {
            return;
        }

        let Some(runtime) = state::modAt(self.m_owner)
        else
        {
            return;
        };

        let request: GuestCall = GuestCall::Effect { handle: self.m_behavior.handle, call: build() };
        withWorld(&entity.world(), || runtime.call(&request));
    }
}

impl EffectHooks for ModEffect
{
    fn added(&self, entity: &Rc<Entity>, effect: &EffectType, level: i32)
    {
        self.call(entity, EffectMethod::Added, || EffectCall::Added
        {
            entity: entityInfo(entity),
            effect: idOf(effect),
            level,
        });
    }

    fn tick(&self, entity: &Rc<Entity>, effect: &EffectType, level: i32, ticks: i32)
    {
        self.call(entity, EffectMethod::Tick, || EffectCall::Tick
        {
            entity: entityInfo(entity),
            effect: idOf(effect),
            level,
            ticks,
        });
    }

    fn removed(&self, entity: &Rc<Entity>, effect: &EffectType)
    {
        self.call(entity, EffectMethod::Removed, || EffectCall::Removed
        {
            entity: entityInfo(entity),
            effect: idOf(effect),
        });
    }
}

// effect names come from registered identifiers, so they always parse
fn idOf(effect: &EffectType) -> Identifier
{
    Identifier::parse(&effect.m_name).expect("effect names are registered identifiers")
}

pub fn registerWith(
    id: Identifier,
    spec: EffectSpec,
    owner: usize,
    behavior: Behavior<EffectMethod>,
) -> Result<(), String>
{
    let hooks: Rc<dyn EffectHooks> = Rc::new(ModEffect { m_owner: owner, m_behavior: behavior });
    build(id, spec, Some(hooks))
}

pub fn register(id: Identifier, spec: EffectSpec) -> Result<(), String> { build(id, spec, None) }

fn build(id: Identifier, spec: EffectSpec, hooks: Option<Rc<dyn EffectHooks>>) -> Result<(), String>
{
    let mut modifiers: Vec<(Attribute, f64, Operation)> = Vec::new();
    for modifier in &spec.modifiers
    {
        let attribute: Attribute = Attribute::byName(&modifier.attribute.to_string())
            .ok_or_else(|| format!("{} is not an attribute", modifier.attribute))?;
        modifiers.push((attribute, modifier.amount, operation(modifier.operation)));
    }

    effect::register(EffectType
    {
        m_name: id.to_string(),
        m_color: spec.color as i32,
        m_beneficial: spec.beneficial,
        m_icon: spec.icon.unwrap_or_default(),
        m_modifiers: modifiers,
        m_interval: spec.interval,
        m_amount: spec.amount,
        m_hooks: hooks,
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
        .and_then(|living|
        {
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
        .filter_map(|synced|
        {
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
