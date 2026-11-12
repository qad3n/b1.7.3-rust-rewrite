use crate::network::loader::{self, ProgressSync};
use game::entity::{Entity, entity_player};
use game::stats::named::{self, ModAchievement, ModStat};
use game::stats::{IStatType, StatBase};
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::{AchievementSpec, StatFormat, StatSpec};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

const FORMATS: [(StatFormat, &dyn IStatType); 3] = [
    (StatFormat::Number, StatBase::NUMBER_FORMATTER),
    (StatFormat::Time, StatBase::TIME_FORMATTER),
    (StatFormat::Distance, StatBase::DISTANCE_FORMATTER),
];

thread_local!
{
    // mod achievement icons by achievement, resolved to item numbers on every content build
    static ICONS: RefCell<BTreeMap<String, Identifier>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn register(id: Identifier, spec: AchievementSpec) -> Result<(), String>
{
    crate::registry::claim(RegistryKind::Achievement, &id)?;

    let name: String = id.to_string();

    named::registerAchievement(ModAchievement
    {
        m_name: name.clone(),
        m_title: text(&spec.title),
        m_description: text(&spec.description),
        m_icon: Cell::new(0),
        m_parent: spec.parent.map(|parent| parent.to_string()),
    });

    ICONS.with(|icons| icons.borrow_mut().insert(name, spec.icon));
    Ok(())
}

pub fn registerStat(id: Identifier, spec: StatSpec) -> Result<(), String>
{
    crate::registry::claim(RegistryKind::Stat, &id)?;

    let formatter: &'static dyn IStatType =
        FORMATS.iter().find(|(format, _)| *format == spec.format).map_or(StatBase::NUMBER_FORMATTER, |row| row.1);
    named::registerStat(ModStat { m_name: id.to_string(), m_title: text(&spec.title), m_formatter: formatter });
    Ok(())
}

// a custom stat counted without registration still lists, titled by its lang key stat.<modid>.<name> or its name
fn implicit(name: &str) -> bool
{
    if !named::custom(name) || named::stat(name).is_some()
    {
        return false;
    }

    let Ok(id) = Identifier::parse(name)
    else
    {
        return false;
    };

    let title: String = super::lang::translate(&format!("stat.{}.{}", id.namespace(), id.path()))
        .unwrap_or_else(|| id.path().to_owned());
    crate::registry::with(|registries| registries.seedName(RegistryKind::Stat, id));
    // keyed by the name as counted so a bare name still finds its value
    named::registerStat(ModStat { m_name: name.to_owned(), m_title: title, m_formatter: StatBase::NUMBER_FORMATTER });
    true
}

fn text(key: &str) -> String { super::lang::translate(key).unwrap_or_else(|| key.to_owned()) }

pub fn push()
{
    let icons: Vec<(String, Identifier)> =
        ICONS.with(|icons| icons.borrow().iter().map(|(name, icon)| (name.clone(), icon.clone())).collect());

    for (name, icon) in icons
    {
        let item: Option<i32> = crate::registry::with(|registries| registries.itemId(&icon));
        match (named::achievement(&name), item)
        {
            (Some(achievement), Some(item)) => achievement.m_icon.set(item),
            _ => crate::log::warn(crate::log::LOADER, &format!("achievement {name} has no registered icon {icon}")),
        }
    }
}

pub fn unlock(player: &Rc<Entity>, achievement: &Identifier) -> bool
{
    let newly: bool = named::unlock(player, &achievement.to_string());
    if newly
    {
        syncOwn(player);
    }

    newly
}

pub fn addStat(player: &Rc<Entity>, stat: &str, amount: i32)
{
    if implicit(stat)
    {
        crate::log::warn(crate::log::LOADER, &format!("stat {stat} was counted before it was registered"));
    }

    named::add(player, stat, amount);
    syncOwn(player);
}

// a player's client shows the toasts and answers stat reads, so it gets the player's copy on login and changes
pub fn syncOwn(player: &Rc<Entity>)
{
    if !crate::side::isServer() || !player.instanceOf("EntityPlayer")
    {
        return;
    }

    let progress: &named::Progress = &entity_player::fields(player).m_progress;

    let message: ProgressSync = ProgressSync
    {
        id: player.m_entityId.get(),
        stats: progress.stats().into_iter().collect(),
        achievements: progress.earned().into_iter().collect(),
    };
    loader::send(loader::PROGRESS, player, &message);
}

pub fn received(message: ProgressSync)
{
    let Some(player) = crate::network::localPlayer().filter(|player| player.m_entityId.get() == message.id)
    else
    {
        return;
    };

    for (name, _) in &message.stats
    {
        implicit(name);
    }

    entity_player::fields(&player)
        .m_progress
        .replace(message.stats.into_iter().collect(), message.achievements.into_iter().collect());
}
