use crate::item::ItemStack;
use crate::locale::StatCollector;
use crate::stats::{AchievementList, IStatStringFormat, StatBase, StatExt};
use jvm::field::Field;
use std::cell::Cell;
use std::rc::Rc;

pub struct Achievement
{
    pub m_displayColumn: i32,
    pub m_displayRow: i32,
    pub m_parentAchievement: Option<Rc<StatBase>>,
    m_achievementDescription: String,
    m_statStringFormatter: Field<Option<Rc<dyn IStatStringFormat>>>,
    pub m_theItemStack: Rc<ItemStack>,
    m_isSpecial: Cell<bool>,
}

impl Achievement
{
    pub fn create(
        id: i32,
        name: &str,
        column: i32,
        row: i32,
        icon: Rc<ItemStack>,
        parent: Option<Rc<StatBase>>,
    ) -> Rc<StatBase>
    {
        let statName: String = StatCollector::translateToLocal(&format!("achievement.{name}"));

        let achievement: Self = Self
        {
            m_displayColumn: column,
            m_displayRow: row,
            m_parentAchievement: parent,
            m_achievementDescription: StatCollector::translateToLocal(&format!("achievement.{name}.desc")),
            m_statStringFormatter: Field::new(None),
            m_theItemStack: icon,
            m_isSpecial: Cell::new(false),
        };

        AchievementList::updateDisplayBounds(column, row);
        StatBase::new(0x500000 + id, statName, StatBase::NUMBER_FORMATTER, StatExt::Achievement(achievement))
    }

    // a stat that only carries a mod achievement's title and icon to beta's toast, never registered or saved
    pub fn display(title: String, description: String, icon: Rc<ItemStack>) -> Rc<StatBase>
    {
        Self::placed(title, description, icon, [0, 0], None)
    }

    // a mod achievement on the map, never registered so beta's list and display bounds never see it
    pub fn placed(
        title: String,
        description: String,
        icon: Rc<ItemStack>,
        [column, row]: [i32; 2],
        parent: Option<Rc<StatBase>>,
    ) -> Rc<StatBase>
    {
        let achievement: Self = Self
        {
            m_displayColumn: column,
            m_displayRow: row,
            m_parentAchievement: parent,
            m_achievementDescription: description,
            m_statStringFormatter: Field::new(None),
            m_theItemStack: icon,
            m_isSpecial: Cell::new(false),
        };
        StatBase::new(0, title, StatBase::NUMBER_FORMATTER, StatExt::Achievement(achievement))
    }

    pub fn of(stat: &StatBase) -> &Self
    {
        match &stat.m_ext
        {
            StatExt::Achievement(achievement) => achievement,
            _ => jvm::throwable::throwBare(&jvm::throwable::CLASS_CAST),
        }
    }

    pub fn setSpecial(this: &Rc<StatBase>) -> Rc<StatBase>
    {
        Self::of(this).m_isSpecial.set(true);
        Rc::clone(this)
    }

    pub fn getDescription(this: &StatBase) -> String
    {
        let achievement: &Self = Self::of(this);
        match achievement.m_statStringFormatter.get()
        {
            Some(formatter) => formatter.formatString(&achievement.m_achievementDescription),
            None => achievement.m_achievementDescription.clone(),
        }
    }

    pub fn setStatStringFormatter(this: &Rc<StatBase>, formatter: Rc<dyn IStatStringFormat>) -> Rc<StatBase>
    {
        Self::of(this).m_statStringFormatter.set(Some(formatter));
        Rc::clone(this)
    }

    pub fn getSpecial(this: &StatBase) -> bool { Self::of(this).m_isSpecial.get() }
}

pub(crate) fn register(this: &Rc<StatBase>) -> bool { AchievementList::achievementList().add(Rc::clone(this)) }
