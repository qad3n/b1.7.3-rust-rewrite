use crate::host::context;
use game::gamerule::{self, Category, GameRule, GameRules, RuleType, Value};
use mod_protocol::Identifier;
use mod_protocol::abi::{GameRuleValue, HostReply};
use mod_protocol::spec::{GameRuleCategory, GameRuleSpec};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

pub const GAMERULES: &str = "loader:gamerules";

// rule names with their values as /gamerule prints them, the client's copy of the server's rules
#[derive(Serialize, Deserialize)]
pub struct RuleSync
{
    pub rules: Vec<(String, String)>,
}

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

// a server passes every change on, a client mirrors the server so mods and beta rules read the same values
pub fn changed(name: &str, value: &str)
{
    if !crate::side::isServer() || crate::state::phase() != crate::state::Phase::Running
    {
        return;
    }

    let message: RuleSync = RuleSync { rules: vec![(name.to_owned(), value.to_owned())] };
    crate::network::broadcast(&crate::network::loader::id(GAMERULES), &mod_protocol::wire::encode(&message));
}

// a player's client gets every rule on login, respawn and dimension change
pub fn syncAll(player: &Rc<game::entity::Entity>)
{
    let Some(rules) = rules().ok().filter(|_| crate::side::isServer())
    else
    {
        return;
    };

    let values: Vec<(String, String)> =
        GameRule::all().into_iter().map(|rule| (rule.name(), rules.value(rule).to_string())).collect();
    crate::network::loader::send(GAMERULES, player, &RuleSync { rules: values });
}

pub fn received(message: RuleSync)
{
    let Some(rules) =
        crate::network::localPlayer().and_then(|player| player.m_worldObj.get()).map(|world| world.rules())
    else
    {
        return;
    };

    for (name, text) in message.rules
    {
        let applied: Result<(), String> = GameRule::byName(&name)
            .ok_or_else(|| format!("unknown rule {name}"))
            .and_then(|rule| rule.parse(&text).and_then(|value| rules.set(rule, value)));
        if let Err(reason) = applied
        {
            crate::log::warn(crate::log::LOADER, &format!("server game rule: {reason}"));
        }
    }
}
