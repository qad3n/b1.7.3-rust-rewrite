use crate::host::context;
use game::gamerule::{self, Category, GameRule, GameRules, RuleType, Value};
use mod_protocol::Identifier;
use mod_protocol::abi::{GameRuleValue, HostReply};
use mod_protocol::spec::{GameRuleCategory, GameRuleSpec};
use std::rc::Rc;

pub fn register(id: Identifier, spec: GameRuleSpec) -> Result<(), String>
{
    crate::registry::claim(mod_protocol::abi::RegistryKind::GameRule, &id)?;
    gamerule::register(RuleType
    {
        m_name: id.to_string(),
        m_category: categoryOf(spec.category),
        m_default: valueOf(spec.default),
        m_min: spec.min,
        m_max: spec.max,
    })
    .map(drop)
}

pub fn value(rule: &Identifier) -> Result<HostReply, String>
{
    let reply: HostReply = match rules()?.value(ruleOf(rule)?)
    {
        Value::Bool(value) => HostReply::Bool(value),
        Value::Int(value) => HostReply::Int(value),
    };
    Ok(reply)
}

pub fn set(rule: &Identifier, value: GameRuleValue) -> Result<(), String>
{
    rules()?.set(ruleOf(rule)?, valueOf(value))
}

// every dimension of a save shares its rules, so any loaded world answers
fn rules() -> Result<Rc<GameRules>, String>
{
    let world: Rc<game::world::World> = context::worlds().into_iter().next().ok_or("no world is loaded")?;
    Ok(world.rules())
}

fn ruleOf(id: &Identifier) -> Result<GameRule, String>
{
    GameRule::byName(&id.to_string()).ok_or_else(|| format!("{id} is not a game rule"))
}

fn valueOf(value: GameRuleValue) -> Value
{
    match value
    {
        GameRuleValue::Bool(value) => Value::Bool(value),
        GameRuleValue::Int(value) => Value::Int(value),
    }
}

fn categoryOf(category: GameRuleCategory) -> Category
{
    match category
    {
        GameRuleCategory::Player => Category::Player,
        GameRuleCategory::Mobs => Category::Mobs,
        GameRuleCategory::Spawning => Category::Spawning,
        GameRuleCategory::Drops => Category::Drops,
        GameRuleCategory::Updates => Category::Updates,
        GameRuleCategory::Chat => Category::Chat,
        GameRuleCategory::Misc => Category::Misc,
    }
}
