use crate::entity::{Entity, entity_player};
use crate::item::ItemStack;
use crate::stats::{Achievement, StatBase, StatList};
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList, NBTTagString};
use jvm::string::JString;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

const STATS_KEY: &str = "ModStats";
const ACHIEVEMENTS_KEY: &str = "ModAchievements";

// what happened to a block or item, beta counts these per numeric id in its own stats file
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContentStat
{
    Mined,
    Crafted,
    Used,
    Broken,
}

pub struct ContentStatRow
{
    pub m_kind: ContentStat,
    pub m_name: &'static str,
}

pub const CONTENT_STATS: [ContentStatRow; 4] = [
    ContentStatRow { m_kind: ContentStat::Mined, m_name: "mined" },
    ContentStatRow { m_kind: ContentStat::Crafted, m_name: "crafted" },
    ContentStatRow { m_kind: ContentStat::Used, m_name: "used" },
    ContentStatRow { m_kind: ContentStat::Broken, m_name: "broken" },
];

impl ContentStat
{
    pub fn name(self) -> &'static str
    {
        CONTENT_STATS.iter().find(|row| row.m_kind == self).map_or("", |row| row.m_name)
    }
}

// a mod achievement, kept apart from beta's list so its stats file and checksum never see it
pub struct ModAchievement
{
    pub m_name: String,
    pub m_title: String,
    pub m_description: String,
    pub m_icon: std::cell::Cell<i32>,
    pub m_parent: Option<String>,
}

// stats and achievements a player earned with mod content or from mods, saved in the player file
#[derive(Default)]
pub struct Progress
{
    m_stats: RefCell<BTreeMap<String, i32>>,
    m_achievements: RefCell<BTreeSet<String>>,
}

thread_local!
{
    static ACHIEVEMENTS: RefCell<Vec<Rc<ModAchievement>>> = const { RefCell::new(Vec::new()) };
    // achievements this client saw unlock, the client shows each once as beta's toast
    static TOASTS: RefCell<Vec<Rc<ModAchievement>>> = const { RefCell::new(Vec::new()) };
}

// beta's stat when the block or item has one, otherwise a named stat for mod content only
pub fn count(player: &Rc<Entity>, kind: ContentStat, id: i32, amount: i32)
{
    let stat: Option<Rc<StatBase>> = match kind
    {
        ContentStat::Mined => StatList::mineBlockStat(id),
        ContentStat::Crafted => StatList::itemsCrafted(id),
        ContentStat::Used => StatList::itemsUsed(id),
        ContentStat::Broken => StatList::itemsBroken(id),
    };
    if stat.is_some()
    {
        player.addStat(stat.as_ref(), amount);
        return;
    }

    player.addStat(None, amount);

    let Some(name) = modName(kind, id)
    else
    {
        return;
    };

    add(player, &format!("{}/{name}", kind.name()), amount);
}

// only mods name content outside minecraft, so vanilla blocks without a beta stat stay uncounted
fn modName(kind: ContentStat, id: i32) -> Option<String>
{
    let name: String = match kind
    {
        ContentStat::Mined => crate::hooks::blockName(id, 0).map(|(name, _)| name)?,
        _ => crate::hooks::itemName(id)?,
    };
    (!name.starts_with("minecraft:")).then_some(name)
}

pub fn add(player: &Entity, name: &str, amount: i32)
{
    let progress: &Progress = &entity_player::fields(player).m_progress;
    *progress.m_stats.borrow_mut().entry(name.to_owned()).or_insert(0) += amount;
}

pub fn value(player: &Entity, name: &str) -> i32
{
    entity_player::fields(player).m_progress.m_stats.borrow().get(name).copied().unwrap_or(0)
}

pub fn registerAchievement(achievement: ModAchievement)
{
    ACHIEVEMENTS.with(|list|
    {
        let mut list: RefMut<Vec<Rc<ModAchievement>>> = list.borrow_mut();
        list.retain(|old| old.m_name != achievement.m_name);
        list.push(Rc::new(achievement));
    });
}

pub fn achievement(name: &str) -> Option<Rc<ModAchievement>>
{
    ACHIEVEMENTS.with(|list| list.borrow().iter().find(|achievement| achievement.m_name == name).cloned())
}

pub fn achievements() -> Vec<Rc<ModAchievement>> { ACHIEVEMENTS.with(|list| list.borrow().clone()) }

pub fn unlocked(player: &Entity, name: &str) -> bool
{
    entity_player::fields(player).m_progress.m_achievements.borrow().contains(name)
}

// as in beta an achievement needs its parent first, true when it was newly earned
pub fn unlock(player: &Entity, name: &str) -> bool
{
    let Some(achievement) = achievement(name)
    else
    {
        return false;
    };
    if achievement.m_parent.as_deref().is_some_and(|parent| !unlocked(player, parent))
    {
        return false;
    }

    let newly: bool = entity_player::fields(player).m_progress.m_achievements.borrow_mut().insert(name.to_owned());
    // only the client's own player shows a toast, a server player's client hears through the sync
    if newly && player.instanceOf("EntityPlayerSP")
    {
        TOASTS.with(|toasts| toasts.borrow_mut().push(achievement));
    }

    newly
}

// the next mod achievement to announce as a beta style toast, built as a display only stat
pub fn nextToast() -> Option<Rc<StatBase>>
{
    let achievement: Rc<ModAchievement> = TOASTS.with(|toasts|
    {
        let mut toasts: RefMut<Vec<Rc<ModAchievement>>> = toasts.borrow_mut();
        (!toasts.is_empty()).then(|| toasts.remove(0))
    })?;
    let icon: Rc<ItemStack> = ItemStack::withDamage(achievement.m_icon.get(), 1, 0);
    Some(Achievement::display(achievement.m_title.clone(), achievement.m_description.clone(), icon))
}

impl Progress
{
    pub fn stats(&self) -> BTreeMap<String, i32> { self.m_stats.borrow().clone() }

    // a respawned player keeps what the dead one earned, beta makes a new player object on respawn
    pub fn copyFrom(&self, other: &Self)
    {
        *self.m_stats.borrow_mut() = other.m_stats.borrow().clone();
        *self.m_achievements.borrow_mut() = other.m_achievements.borrow().clone();
    }

    pub fn earned(&self) -> BTreeSet<String> { self.m_achievements.borrow().clone() }

    // a server's copy for this client's own player, newly earned achievements still get their toast
    pub fn replace(&self, stats: BTreeMap<String, i32>, earned: BTreeSet<String>)
    {
        let fresh: Vec<String> = earned.difference(&self.m_achievements.borrow()).cloned().collect();

        *self.m_stats.borrow_mut() = stats;
        *self.m_achievements.borrow_mut() = earned;

        for name in fresh
        {
            if let Some(achievement) = achievement(&name)
            {
                TOASTS.with(|toasts| toasts.borrow_mut().push(achievement));
            }
        }
    }

    // written only when something was earned, so a vanilla player file is unchanged
    pub fn write(&self, nbt: &NBTTagCompound)
    {
        let stats: Ref<BTreeMap<String, i32>> = self.m_stats.borrow();
        if !stats.is_empty()
        {
            let compound: Rc<NBTBase> = NBTTagCompound::create();

            for (name, value) in stats.iter()
            {
                compound.compound().setInteger(name, *value);
            }

            nbt.setTag(STATS_KEY, &compound);
        }

        let achievements: Ref<BTreeSet<String>> = self.m_achievements.borrow();
        if !achievements.is_empty()
        {
            let list: Rc<NBTBase> = NBTTagList::create();

            for name in achievements.iter()
            {
                list.list().setTag(NBTTagString::create(Some(JString::from(name.as_str()))));
            }

            nbt.setTag(ACHIEVEMENTS_KEY, &list);
        }
    }

    pub fn read(&self, nbt: &NBTTagCompound)
    {
        if nbt.hasKey(STATS_KEY)
        {
            let tag: Rc<NBTBase> = nbt.getCompoundTag(STATS_KEY);
            let mut stats: RefMut<BTreeMap<String, i32>> = self.m_stats.borrow_mut();

            for entry in tag.compound().getValues()
            {
                stats.insert(entry.getKey().to_string(), tag.compound().getInteger(&entry.getKey().to_string()));
            }
        }

        if nbt.hasKey(ACHIEVEMENTS_KEY)
        {
            let list: Rc<NBTBase> = nbt.getTagList(ACHIEVEMENTS_KEY);
            let mut achievements: RefMut<BTreeSet<String>> = self.m_achievements.borrow_mut();

            for i in 0..list.list().tagCount()
            {
                if let Some(name) = list.list().tagAt(i).stringTag().m_stringValue.get()
                {
                    achievements.insert(name.to_string());
                }
            }
        }
    }
}
