use crate::stats::achievement::{self, Achievement};
use crate::stats::{AchievementMap, IStatType, StatList, StatTypeDistance, StatTypeSimple, StatTypeTime};
use jvm::field::Field;
use std::cell::Cell;
use std::rc::Rc;

pub enum StatExt
{
    Basic,
    Crafting(i32),
    Achievement(Achievement),
}

pub struct StatBase
{
    pub m_statId: i32,
    pub m_statName: String,
    pub m_local: Cell<bool>,
    pub m_statGuid: Field<Option<String>>,
    m_formatter: &'static dyn IStatType,
    pub m_ext: StatExt,
}

impl StatBase
{
    pub const NUMBER_FORMATTER: &'static dyn IStatType = &StatTypeSimple;
    pub const TIME_FORMATTER: &'static dyn IStatType = &StatTypeTime;
    pub const DISTANCE_FORMATTER: &'static dyn IStatType = &StatTypeDistance;

    pub(super) fn new(id: i32, key: String, formatter: &'static dyn IStatType, ext: StatExt) -> Rc<StatBase>
    {
        Rc::new(StatBase {
            m_statId: id,
            m_statName: key,
            m_local: Cell::new(false),
            m_statGuid: Field::new(None),
            m_formatter: formatter,
            m_ext: ext,
        })
    }

    pub fn basic(id: i32, key: String, formatter: &'static dyn IStatType) -> Rc<StatBase>
    {
        StatBase::new(id, key, formatter, StatExt::Basic)
    }

    pub fn crafting(id: i32, name: String, item: i32) -> Rc<StatBase>
    {
        StatBase::new(id, name, StatBase::NUMBER_FORMATTER, StatExt::Crafting(item))
    }

    pub fn setLocal(self: &Rc<Self>) -> Rc<StatBase>
    {
        self.m_local.set(true);
        Rc::clone(self)
    }

    pub fn registerStat(self: &Rc<Self>) -> Rc<StatBase>
    {
        self.registerStatBase();
        match &self.m_ext
        {
            StatExt::Basic => StatList::general().add(Rc::clone(self)),
            StatExt::Crafting(_) => true,
            StatExt::Achievement(_) => achievement::register(self),
        };
        Rc::clone(self)
    }

    fn registerStatBase(self: &Rc<Self>)
    {
        if let Some(existing) = StatList::byKey(self.m_statId)
        {
            jvm::throwable::throw(
                &jvm::throwable::RUNTIME,
                &format!(
                    "Duplicate stat id: \"{}\" and \"{}\" at id {}",
                    existing.m_statName, self.m_statName, self.m_statId
                ),
            );
        }

        StatList::all().add(Rc::clone(self));
        StatList::putById(self);
        self.m_statGuid.set(AchievementMap::getGuid(self.m_statId));
    }

    pub fn isAchievement(&self) -> bool { matches!(self.m_ext, StatExt::Achievement(_)) }

    pub fn format(&self, value: i32) -> String { self.m_formatter.format(value) }

    pub fn toString(&self) -> String { self.m_statName.clone() }

    pub fn getItem(&self) -> i32
    {
        match self.m_ext
        {
            StatExt::Crafting(item) => item,
            _ => jvm::throwable::throwBare(&jvm::throwable::CLASS_CAST),
        }
    }
}
