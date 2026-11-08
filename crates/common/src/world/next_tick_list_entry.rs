use jvm::object::JavaObject;
use std::cmp::Ordering;
use std::sync::atomic::{AtomicI64, Ordering as AtomicOrdering};

static NEXT_TICK_ENTRY_ID: AtomicI64 = AtomicI64::new(0);

#[derive(Clone, Debug)]
pub struct NextTickListEntry
{
    pub m_xCoord: i32,
    pub m_yCoord: i32,
    pub m_zCoord: i32,
    pub m_blockID: i32,
    pub m_scheduledTime: i64,
    m_tickEntryID: i64,
}

impl NextTickListEntry
{
    pub fn new(x: i32, y: i32, z: i32, block: i32) -> Self
    {
        Self {
            m_xCoord: x,
            m_yCoord: y,
            m_zCoord: z,
            m_blockID: block,
            m_scheduledTime: 0,
            m_tickEntryID: NEXT_TICK_ENTRY_ID.fetch_add(1, AtomicOrdering::Relaxed),
        }
    }

    pub fn equals(&self, other: &Self) -> bool
    {
        self.m_xCoord == other.m_xCoord
            && self.m_yCoord == other.m_yCoord
            && self.m_zCoord == other.m_zCoord
            && self.m_blockID == other.m_blockID
    }

    pub fn hashCode(&self) -> i32
    {
        (self
            .m_xCoord
            .wrapping_mul(128)
            .wrapping_mul(1024)
            .wrapping_add(self.m_zCoord.wrapping_mul(128))
            .wrapping_add(self.m_yCoord))
        .wrapping_mul(256)
        .wrapping_add(self.m_blockID)
    }

    pub fn setScheduledTime(mut self, time: i64) -> Self
    {
        self.m_scheduledTime = time;
        self
    }

    pub fn compareTo(&self, other: &Self) -> i32
    {
        if self.m_scheduledTime < other.m_scheduledTime
        {
            return -1;
        }
        if self.m_scheduledTime > other.m_scheduledTime
        {
            return 1;
        }
        if self.m_tickEntryID < other.m_tickEntryID
        {
            return -1;
        }
        if self.m_tickEntryID > other.m_tickEntryID
        {
            return 1;
        }
        0
    }
}

impl JavaObject for NextTickListEntry
{
    fn javaEquals(&self, other: &Self) -> bool { self.equals(other) }

    fn javaHash(&self) -> i32 { self.hashCode() }
}

impl Ord for NextTickListEntry
{
    fn cmp(&self, other: &Self) -> Ordering { self.compareTo(other).cmp(&0) }
}

impl PartialOrd for NextTickListEntry
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl PartialEq for NextTickListEntry
{
    fn eq(&self, other: &Self) -> bool { self.compareTo(other) == 0 }
}

impl Eq for NextTickListEntry {}
