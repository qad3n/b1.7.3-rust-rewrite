use crate::attribute::{Attribute, Modifier, Operation};
use crate::damage::{DamageKind, DamageSource};
use crate::entity::Entity;
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList, Unread};
use jvm::string::JString;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

const NBT_KEY: &str = "ActiveEffects";

// one kind of status effect, beta has none so every row comes from a mod
pub struct EffectType
{
    pub m_name: String,
    pub m_color: i32,
    pub m_beneficial: bool,
    // a texture name the hud draws, empty draws only the colour
    pub m_icon: String,
    // each level adds the amount once more
    pub m_modifiers: Vec<(Attribute, f64, Operation)>,
    // every interval ticks heals a positive amount or hurts a negative one, 0 for none
    pub m_interval: i32,
    pub m_amount: i32,
    pub m_hooks: Option<Rc<dyn EffectHooks>>,
}

// what a mod runs when its effect starts, ticks and ends
pub trait EffectHooks
{
    fn added(&self, entity: &Rc<Entity>, effect: &EffectType, level: i32);
    fn tick(&self, entity: &Rc<Entity>, effect: &EffectType, level: i32, ticks: i32);
    fn removed(&self, entity: &Rc<Entity>, effect: &EffectType);
}

pub struct ActiveEffect
{
    pub m_effect: Rc<EffectType>,
    pub m_level: i32,
    pub m_ticks: i32,
}

thread_local!
{
    static TYPES: RefCell<Vec<Rc<EffectType>>> = const { RefCell::new(Vec::new()) };
}

pub fn register(effect: EffectType)
{
    TYPES.with(|types|
    {
        let mut types: RefMut<Vec<Rc<EffectType>>> = types.borrow_mut();
        types.retain(|old| old.m_name != effect.m_name);
        types.push(Rc::new(effect));
    });
}

pub fn byName(name: &str) -> Option<Rc<EffectType>>
{
    TYPES.with(|types| types.borrow().iter().find(|effect| effect.m_name == name).cloned())
}

pub fn clear() { TYPES.with(|types| types.borrow_mut().clear()); }

// the effects on one living entity, empty for everything in beta so nothing ticks, saves or draws
#[derive(Default)]
pub struct Effects
{
    m_active: RefCell<Vec<ActiveEffect>>,
    m_unread: Unread,
}

impl Effects
{
    pub fn empty(&self) -> bool { self.m_active.borrow().is_empty() }

    // name, level and ticks left of each effect, in the order they were applied
    pub fn list(&self) -> Vec<(Rc<EffectType>, i32, i32)>
    {
        self.m_active
            .borrow()
            .iter()
            .map(|active| (Rc::clone(&active.m_effect), active.m_level, active.m_ticks))
            .collect()
    }

    // written only when an effect is active, so a vanilla save is unchanged
    pub fn write(&self, nbt: &NBTTagCompound)
    {
        if self.empty() && self.m_unread.empty()
        {
            return;
        }

        let list: Rc<NBTBase> = NBTTagList::create();

        for active in self.m_active.borrow().iter()
        {
            let entry: Rc<NBTBase> = NBTTagCompound::create();
            entry.compound().setString("Name", Some(JString::from(active.m_effect.m_name.as_str())));
            entry.compound().setInteger("Level", active.m_level);
            entry.compound().setInteger("Ticks", active.m_ticks);
            list.list().setTag(entry);
        }

        self.m_unread.writeInto(&list);
        nbt.setTag(NBT_KEY, &list);
    }

    // effects whose type no mod registers any more are kept as saved, so reinstalling the mod restores them
    pub fn read(&self, nbt: &NBTTagCompound)
    {
        self.m_unread.clear();

        if !nbt.hasKey(NBT_KEY)
        {
            return;
        }

        let list: Rc<NBTBase> = nbt.getTagList(NBT_KEY);
        let mut active: Vec<ActiveEffect> = Vec::new();

        for i in 0..list.list().tagCount()
        {
            let tag: Rc<NBTBase> = list.list().tagAt(i);
            let entry: &NBTTagCompound = tag.compound();
            let name: String = entry.getString("Name").map(|name| name.to_string()).unwrap_or_default();
            let Some(effect) = byName(&name)
            else
            {
                self.m_unread.keep(tag);
                continue;
            };

            active.push(ActiveEffect
            {
                m_effect: effect,
                m_level: entry.getInteger("Level"),
                m_ticks: entry.getInteger("Ticks"),
            });
        }
        *self.m_active.borrow_mut() = active;
    }

    // a server's list for this client's own player
    pub fn replace(&self, active: Vec<ActiveEffect>) { *self.m_active.borrow_mut() = active; }
}

fn modifierId(effect: &EffectType) -> String { format!("effect/{}", effect.m_name) }

// a stronger or longer effect replaces the one already there, as later versions do
pub fn add(entity: &Rc<Entity>, effect: Rc<EffectType>, level: i32, ticks: i32)
{
    let Some(living) = entity.m_ext.living()
    else
    {
        return;
    };

    {
        let mut active: RefMut<Vec<ActiveEffect>> = living.m_effects.m_active.borrow_mut();
        if let Some(old) = active.iter_mut().find(|old| Rc::ptr_eq(&old.m_effect, &effect))
        {
            if old.m_level > level || (old.m_level == level && old.m_ticks >= ticks)
            {
                return;
            }

            old.m_level = level;
            old.m_ticks = ticks;
        }
        else
        {
            active.push(ActiveEffect { m_effect: Rc::clone(&effect), m_level: level, m_ticks: ticks });
        }
    }
    applyModifiers(entity, &effect, level);

    if let Some(hooks) = &effect.m_hooks
    {
        hooks.added(entity, &effect, level);
    }
}

pub fn remove(entity: &Rc<Entity>, name: &str)
{
    let Some(living) = entity.m_ext.living()
    else
    {
        return;
    };

    let removed: Vec<Rc<EffectType>> =
    {
        let mut active: RefMut<Vec<ActiveEffect>> = living.m_effects.m_active.borrow_mut();
        let gone: Vec<Rc<EffectType>> =
            active.iter().filter(|old| old.m_effect.m_name == name).map(|old| Rc::clone(&old.m_effect)).collect();
        active.retain(|old| old.m_effect.m_name != name);
        gone
    };
    for effect in removed
    {
        end(entity, &effect);
    }
}

fn end(entity: &Rc<Entity>, effect: &EffectType)
{
    removeModifiers(entity, effect);

    if let Some(hooks) = &effect.m_hooks
    {
        hooks.removed(entity, effect);
    }
}

fn applyModifiers(entity: &Entity, effect: &EffectType, level: i32)
{
    let Some(living) = entity.m_ext.living()
    else
    {
        return;
    };

    for (attribute, amount, operation) in &effect.m_modifiers
    {
        living.m_attributes.set(Modifier
        {
            m_attribute: *attribute,
            m_id: modifierId(effect),
            m_amount: amount * (level + 1) as f64,
            m_operation: *operation,
        });
    }
}

fn removeModifiers(entity: &Entity, effect: &EffectType)
{
    let Some(living) = entity.m_ext.living()
    else
    {
        return;
    };

    for (attribute, _, _) in &effect.m_modifiers
    {
        living.m_attributes.remove(*attribute, &modifierId(effect));
    }
}

// counts every effect down, applies its periodic heal or harm on the authoritative side, and ends expired ones
pub fn tick(entity: &Rc<Entity>)
{
    let Some(living) = entity.m_ext.living()
    else
    {
        return;
    };
    if living.m_effects.empty()
    {
        return;
    }

    let authority: bool = !entity.world().m_multiplayerWorld.get();
    let mut pulses: Vec<i32> = Vec::new();

    let mut expired: Vec<Rc<EffectType>> = Vec::new();
    let mut hooked: Vec<(Rc<EffectType>, i32, i32)> = Vec::new();
    {
        let mut active: RefMut<Vec<ActiveEffect>> = living.m_effects.m_active.borrow_mut();
        for effect in active.iter_mut()
        {
            effect.m_ticks -= 1;

            if effect.m_effect.m_hooks.is_some()
            {
                hooked.push((Rc::clone(&effect.m_effect), effect.m_level, effect.m_ticks));
            }

            let interval: i32 = effect.m_effect.m_interval;
            if authority && interval > 0 && effect.m_ticks % interval == 0
            {
                pulses.push(effect.m_effect.m_amount * (effect.m_level + 1));
            }

            if effect.m_ticks <= 0
            {
                expired.push(Rc::clone(&effect.m_effect));
            }
        }

        active.retain(|effect| effect.m_ticks > 0);
    }

    // the borrow ends first because hurting, healing and mod hooks may add or remove effects
    for amount in pulses
    {
        if amount > 0
        {
            entity.heal(amount);
        }
        else if amount < 0
        {
            entity.attackEntityFrom(&DamageSource::new(DamageKind::Magic), -amount);
        }
    }

    for (effect, level, ticks) in hooked
    {
        if let Some(hooks) = &effect.m_hooks
        {
            hooks.tick(entity, &effect, level, ticks);
        }
    }

    for effect in expired
    {
        end(entity, &effect);
    }
}
