use common::nbt::{NBTBase, NBTTag, NBTTagCompound};
use jvm::string::JString;
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

const NAMESPACE: &str = "minecraft:";
const NBT_KEY: &str = "GameRules";
// a server chat line longer than 119 characters disconnects a beta client
const LINE_WIDTH: usize = 100;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Value
{
    Bool(bool),
    Int(i32),
}

impl fmt::Display for Value
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Value::Bool(value) => write!(formatter, "{value}"),
            Value::Int(value) => write!(formatter, "{value}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Category
{
    Player,
    Mobs,
    Spawning,
    Drops,
    Updates,
    Chat,
    Misc,
}

struct CategoryRow
{
    m_category: Category,
    m_name: &'static str,
}

// display order of the rule list
const CATEGORIES: [CategoryRow; 7] = [
    CategoryRow { m_category: Category::Player, m_name: "player" },
    CategoryRow { m_category: Category::Mobs, m_name: "mobs" },
    CategoryRow { m_category: Category::Spawning, m_name: "spawning" },
    CategoryRow { m_category: Category::Drops, m_name: "drops" },
    CategoryRow { m_category: Category::Updates, m_name: "updates" },
    CategoryRow { m_category: Category::Chat, m_name: "chat" },
    CategoryRow { m_category: Category::Misc, m_name: "misc" },
];

impl Category
{
    pub fn name(self) -> &'static str
    {
        CATEGORIES.iter().find(|row| row.m_category == self).map_or("misc", |row| row.m_name)
    }

    pub fn byName(name: &str) -> Option<Self>
    {
        CATEGORIES.iter().find(|row| row.m_name == name).map(|row| row.m_category)
    }
}

// a world setting, an index into the rule registry where vanilla rows come first
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GameRule(usize);

#[derive(Clone)]
pub struct RuleType
{
    pub m_name: String,
    pub m_category: Category,
    // the vanilla rows default to how beta plays
    pub m_default: Value,
    // whole number rules stay within min and max
    pub m_min: i32,
    pub m_max: i32,
}

struct VanillaRow
{
    m_name: &'static str,
    m_category: Category,
    m_default: Value,
    m_min: i32,
    m_max: i32,
}

const fn flag(name: &'static str, category: Category, default: bool) -> VanillaRow
{
    VanillaRow { m_name: name, m_category: category, m_default: Value::Bool(default), m_min: 0, m_max: 0 }
}

const fn number(name: &'static str, category: Category, default: i32, min: i32) -> VanillaRow
{
    VanillaRow { m_name: name, m_category: category, m_default: Value::Int(default), m_min: min, m_max: i32::MAX }
}

// order matches the GameRule consts, names follow the modern rules
const VANILLA: [VanillaRow; 21] = [
    flag("minecraft:advance_time", Category::Updates, true),
    flag("minecraft:advance_weather", Category::Updates, true),
    // beta ticks 80 random blocks per chunk where later versions tick 3 per section
    number("minecraft:random_tick_speed", Category::Updates, 80, 0),
    // -1 spreads fire everywhere as beta does and 0 stops fire ticking
    number("minecraft:fire_spread_radius_around_player", Category::Updates, -1, -1),
    flag("minecraft:water_source_conversion", Category::Updates, true),
    flag("minecraft:lava_source_conversion", Category::Updates, false),
    flag("minecraft:block_drops", Category::Drops, true),
    flag("minecraft:entity_drops", Category::Drops, true),
    flag("minecraft:mob_drops", Category::Drops, true),
    flag("minecraft:mob_griefing", Category::Mobs, true),
    flag("minecraft:spawn_mobs", Category::Spawning, true),
    flag("minecraft:spawner_blocks_work", Category::Misc, true),
    flag("minecraft:keep_inventory", Category::Player, false),
    flag("minecraft:natural_health_regeneration", Category::Player, true),
    flag("minecraft:fall_damage", Category::Player, true),
    flag("minecraft:fire_damage", Category::Player, true),
    flag("minecraft:drowning_damage", Category::Player, true),
    number("minecraft:players_sleeping_percentage", Category::Player, 100, 0),
    flag("minecraft:allow_entering_nether_using_portals", Category::Misc, true),
    flag("minecraft:tnt_explodes", Category::Misc, true),
    flag("minecraft:log_admin_commands", Category::Chat, true),
];

thread_local!
{
    static TYPES: RefCell<Vec<RuleType>> = RefCell::new(
        VANILLA
            .iter()
            .map(|row| RuleType
            {
                m_name: row.m_name.to_owned(),
                m_category: row.m_category,
                m_default: row.m_default,
                m_min: row.m_min,
                m_max: row.m_max,
            })
            .collect(),
    );
}

impl GameRule
{
    pub const ADVANCE_TIME: Self = Self(0);
    pub const ADVANCE_WEATHER: Self = Self(1);
    pub const RANDOM_TICK_SPEED: Self = Self(2);
    pub const FIRE_SPREAD_RADIUS: Self = Self(3);
    pub const WATER_SOURCE_CONVERSION: Self = Self(4);
    pub const LAVA_SOURCE_CONVERSION: Self = Self(5);
    pub const BLOCK_DROPS: Self = Self(6);
    pub const ENTITY_DROPS: Self = Self(7);
    pub const MOB_DROPS: Self = Self(8);
    pub const MOB_GRIEFING: Self = Self(9);
    pub const SPAWN_MOBS: Self = Self(10);
    pub const SPAWNER_BLOCKS_WORK: Self = Self(11);
    pub const KEEP_INVENTORY: Self = Self(12);
    pub const NATURAL_REGENERATION: Self = Self(13);
    pub const FALL_DAMAGE: Self = Self(14);
    pub const FIRE_DAMAGE: Self = Self(15);
    pub const DROWNING_DAMAGE: Self = Self(16);
    pub const SLEEPING_PERCENTAGE: Self = Self(17);
    pub const NETHER_PORTALS: Self = Self(18);
    pub const TNT_EXPLODES: Self = Self(19);
    pub const LOG_ADMIN_COMMANDS: Self = Self(20);

    pub fn kind(self) -> RuleType { TYPES.with(|types| types.borrow()[self.0].clone()) }

    pub fn name(self) -> String { TYPES.with(|types| types.borrow()[self.0].m_name.clone()) }

    pub fn default(self) -> Value { TYPES.with(|types| types.borrow()[self.0].m_default) }

    // vanilla rules answer to their name without the namespace as the command takes them
    pub fn byName(name: &str) -> Option<Self>
    {
        let full: String = if name.contains(':')
        {
            name.to_owned()
        }
        else
        {
            format!("{NAMESPACE}{name}")
        };
        TYPES.with(|types| types.borrow().iter().position(|kind| kind.m_name == full).map(Self))
    }

    // the short name for vanilla rules, the full name for mod rules
    pub fn label(self) -> String
    {
        let name: String = self.name();
        name.strip_prefix(NAMESPACE).map_or_else(|| name.clone(), str::to_owned)
    }

    pub fn all() -> Vec<Self> { TYPES.with(|types| (0..types.borrow().len()).map(Self).collect()) }

    // a value of the rule's type within its range
    pub fn check(self, value: Value) -> Result<Value, String>
    {
        let kind: RuleType = self.kind();
        match (kind.m_default, value)
        {
            (Value::Bool(_), Value::Bool(_)) => Ok(value),
            (Value::Int(_), Value::Int(number)) if (kind.m_min..=kind.m_max).contains(&number) => Ok(value),
            (Value::Int(_), Value::Int(number)) =>
            {
                Err(format!("{number} is outside {} to {} for {}", kind.m_min, kind.m_max, self.label()))
            }
            _ => Err(format!("{} takes {}", self.label(), expected(kind.m_default))),
        }
    }

    pub fn parse(self, text: &str) -> Result<Value, String>
    {
        let value: Option<Value> = match self.default()
        {
            Value::Bool(_) => text.parse().ok().map(Value::Bool),
            Value::Int(_) => text.parse().ok().map(Value::Int),
        };
        let value: Value =
            value.ok_or_else(|| format!("{text} is not {} for {}", expected(self.default()), self.label()))?;
        self.check(value)
    }
}

fn expected(default: Value) -> &'static str
{
    match default
    {
        Value::Bool(_) => "true or false",
        Value::Int(_) => "a whole number",
    }
}

// registering a name again replaces its row and keeps its index, so values already set stay on it
pub fn register(kind: RuleType) -> Result<GameRule, String>
{
    let range: bool = match kind.m_default
    {
        Value::Bool(_) => true,
        Value::Int(default) => kind.m_min <= kind.m_max && (kind.m_min..=kind.m_max).contains(&default),
    };
    if !range
    {
        return Err(format!("{} needs min <= default <= max", kind.m_name));
    }

    Ok(TYPES.with(|types|
    {
        let mut types: RefMut<Vec<RuleType>> = types.borrow_mut();
        match types.iter().position(|old| old.m_name == kind.m_name)
        {
            Some(index) =>
            {
                types[index] = kind;
                GameRule(index)
            }

            None =>
            {
                types.push(kind);
                GameRule(types.len() - 1)
            }
        }
    }))
}

// the rule values of one save, every dimension of it shares one set
#[derive(Default)]
pub struct GameRules
{
    // by rule index, None keeps the registered default
    m_values: RefCell<Vec<Option<Value>>>,
    // saved rules no registered rule claims keep their text, so removing a mod for a while loses nothing
    m_unknown: RefCell<BTreeMap<String, String>>,
}

impl GameRules
{
    pub fn value(&self, rule: GameRule) -> Value
    {
        let set: Option<Value> = self.m_values.borrow().get(rule.0).copied().flatten();
        set.unwrap_or_else(|| rule.default())
    }

    pub fn bool(&self, rule: GameRule) -> bool { matches!(self.value(rule), Value::Bool(true)) }

    pub fn int(&self, rule: GameRule) -> i32
    {
        match self.value(rule)
        {
            Value::Int(value) => value,
            Value::Bool(value) => value as i32,
        }
    }

    pub fn set(&self, rule: GameRule, value: Value) -> Result<(), String>
    {
        let value: Value = rule.check(value)?;
        let mut values: RefMut<Vec<Option<Value>>> = self.m_values.borrow_mut();
        if values.len() <= rule.0
        {
            values.resize(rule.0 + 1, None);
        }

        values[rule.0] = (value != rule.default()).then_some(value);
        Ok(())
    }

    // written only when a rule differs from its default, so a vanilla level.dat is unchanged
    pub fn write(&self, nbt: &NBTTagCompound)
    {
        let tag: Rc<NBTBase> = NBTTagCompound::create();
        let mut empty: bool = true;

        for (name, text) in self.m_unknown.borrow().iter()
        {
            tag.compound().setString(name, Some(JString::from(text.as_str())));
            empty = false;
        }

        for (index, value) in self.m_values.borrow().iter().enumerate()
        {
            let Some(value) = value
            else
            {
                continue;
            };

            tag.compound().setString(&GameRule(index).name(), Some(JString::from(value.to_string().as_str())));
            empty = false;
        }

        if empty
        {
            return;
        }

        nbt.setCompoundTag(NBT_KEY, &tag);
    }

    pub fn read(&self, nbt: &NBTTagCompound)
    {
        if !nbt.hasKey(NBT_KEY)
        {
            return;
        }

        let tag: Rc<NBTBase> = nbt.getCompoundTag(NBT_KEY);
        for entry in tag.compound().getValues()
        {
            let NBTTag::String(text) = &entry.m_tag
            else
            {
                continue;
            };

            let name: String = entry.getKey().to_string();
            let text: String = text.m_stringValue.get().map(|text| text.to_string()).unwrap_or_default();
            self.load(name, text);
        }
    }

    fn load(&self, name: String, text: String)
    {
        let Some(rule) = GameRule::byName(&name)
        else
        {
            self.m_unknown.borrow_mut().insert(name, text);
            return;
        };

        match rule.parse(&text).and_then(|value| self.set(rule, value))
        {
            Ok(()) =>
            {}
            Err(error) => jvm::warning!("Ignoring saved game rule: {error}"),
        }
    }
}

// the /gamerule command, the list with no arguments, the value with one and a change with two
pub fn command(rules: &GameRules, args: &[&str]) -> Result<String, String>
{
    match args
    {
        [] => Ok(list(rules)),

        [name] =>
        {
            let rule: GameRule = find(name)?;
            Ok(format!("Gamerule {} is currently set to: {}", rule.label(), rules.value(rule)))
        }

        [name, text] =>
        {
            let rule: GameRule = find(name)?;
            let value: Value = rule.parse(text)?;
            rules.set(rule, value)?;
            Ok(format!("Gamerule {} is now set to: {value}", rule.label()))
        }

        _ => Err("Usage: /gamerule <rule> [value]".to_owned()),
    }
}

fn find(name: &str) -> Result<GameRule, String>
{
    GameRule::byName(name).ok_or_else(|| format!("Unknown game rule {name}"))
}

// one category per line, wrapped so no line passes the chat limit
fn list(rules: &GameRules) -> String
{
    let all: Vec<GameRule> = GameRule::all();
    let mut lines: Vec<String> = vec!["Usage: /gamerule <rule> [value]".to_owned()];

    for row in &CATEGORIES
    {
        let entries: Vec<String> = all
            .iter()
            .filter(|rule| rule.kind().m_category == row.m_category)
            .map(|rule| format!("{}={}", rule.label(), rules.value(*rule)))
            .collect();
        if entries.is_empty()
        {
            continue;
        }

        let mut line: String = format!("{}:", row.m_name);
        for entry in entries
        {
            if line.len() + entry.len() + 1 > LINE_WIDTH
            {
                lines.push(std::mem::take(&mut line));
            }

            if !line.is_empty()
            {
                line.push(' ');
            }

            line.push_str(&entry);
        }

        lines.push(line);
    }

    lines.join("\n")
}
