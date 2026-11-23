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
    // a status icon index on a 256 square sheet laid out like later inventory.png, none draws the whole texture
    pub m_index: Option<i32>,
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

// the status icon sheet the launcher installs from the 1.0 client jar's inventory.png
pub const ICONS: &str = "/gui/effects.png";

// pulse of Potion.regeneration at its first level
mod pulse
{
    pub const REGENERATION_TICKS: i32 = 25;
    pub const REGENERATION_HEAL: i32 = 1;
}

struct VanillaRow
{
    m_name: &'static str,
    m_color: i32,
    m_beneficial: bool,
    m_index: i32,
    m_modifier: Option<(Attribute, f64, Operation)>,
    m_interval: i32,
    m_amount: i32,
}

// the b1.9 potions whose effect beta gameplay has an attribute or pulse for, with their colours and icon indices
const VANILLA: [VanillaRow; 7] = [
    VanillaRow
    {
        m_name: "minecraft:speed",
        m_color: 0x7CAFC6,
        m_beneficial: true,
        m_index: 0,
        m_modifier: Some((Attribute::MOVE_SPEED, 0.2, Operation::MultiplyTotal)),
        m_interval: 0,
        m_amount: 0,
    },
    VanillaRow
    {
        m_name: "minecraft:slowness",
        m_color: 0x5A6C81,
        m_beneficial: false,
        m_index: 1,
        m_modifier: Some((Attribute::MOVE_SPEED, -0.15, Operation::MultiplyTotal)),
        m_interval: 0,
        m_amount: 0,
    },
    VanillaRow
    {
        m_name: "minecraft:haste",
        m_color: 0xD9C043,
        m_beneficial: true,
        m_index: 2,
        m_modifier: Some((Attribute::BREAK_SPEED, 0.2, Operation::MultiplyTotal)),
        m_interval: 0,
        m_amount: 0,
    },
    VanillaRow
    {
        m_name: "minecraft:mining_fatigue",
        m_color: 0x4A4217,
        m_beneficial: false,
        m_index: 3,
        m_modifier: Some((Attribute::BREAK_SPEED, -0.2, Operation::MultiplyTotal)),
        m_interval: 0,
        m_amount: 0,
    },
    VanillaRow
    {
        m_name: "minecraft:strength",
        m_color: 0x932423,
        m_beneficial: true,
        m_index: 4,
        m_modifier: Some((Attribute::ATTACK_DAMAGE, 3.0, Operation::Add)),
        m_interval: 0,
        m_amount: 0,
    },
    VanillaRow
    {
        m_name: "minecraft:weakness",
        m_color: 0x484D48,
        m_beneficial: false,
        m_index: 5,
        m_modifier: Some((Attribute::ATTACK_DAMAGE, -2.0, Operation::Add)),
        m_interval: 0,
        m_amount: 0,
    },
    VanillaRow
    {
        m_name: "minecraft:regeneration",
        m_color: 0xCD5CAB,
        m_beneficial: true,
        m_index: 7,
        m_modifier: None,
        m_interval: pulse::REGENERATION_TICKS,
        m_amount: pulse::REGENERATION_HEAL,
    },
];

thread_local!
{
    static TYPES: RefCell<Vec<Rc<EffectType>>> = RefCell::new(VANILLA.iter().map(vanilla).collect());
}

fn vanilla(row: &VanillaRow) -> Rc<EffectType>
{
    Rc::new(EffectType
    {
        m_name: row.m_name.to_owned(),
        m_color: row.m_color,
        m_beneficial: row.m_beneficial,
        m_icon: ICONS.to_owned(),
        m_index: Some(row.m_index),
        m_modifiers: row.m_modifier.into_iter().collect(),
        m_interval: row.m_interval,
        m_amount: row.m_amount,
        m_hooks: None,
    })
}

pub fn names() -> Vec<String> { VANILLA.iter().map(|row| row.m_name.to_owned()).collect() }

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

pub fn clear() { TYPES.with(|types| *types.borrow_mut() = VANILLA.iter().map(vanilla).collect()); }

// the effects on one living entity, empty unless a mod applies one so a vanilla game never ticks saves or draws them
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
            let Some(effect) = byName(&crate::hooks::renamed(&name))
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
