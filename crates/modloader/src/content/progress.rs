use crate::network::loader::{self, ProgressSync};
use game::entity::{Entity, entity_player};
use game::stats::named::{self, ModAchievement};
use mod_protocol::Identifier;
use mod_protocol::spec::AchievementSpec;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

thread_local!
{
    // mod achievement icons by achievement, resolved to item numbers on every content build
    static ICONS: RefCell<BTreeMap<String, Identifier>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn register(id: Identifier, spec: AchievementSpec) -> Result<(), String>
{
    crate::registry::claim(mod_protocol::abi::RegistryKind::Achievement, &id)?;

    let name: String = id.to_string();
    let text = |key: &str| super::lang::translate(key).unwrap_or_else(|| key.to_owned());

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

    entity_player::fields(&player)
        .m_progress
        .replace(message.stats.into_iter().collect(), message.achievements.into_iter().collect());
}
