use std::cmp::Ordering;

pub struct SaveFormatComparator
{
    m_fileName: String,
    m_displayName: String,
    m_lastPlayed: i64,
    m_size: i64,
    m_sameVersion: bool,
}

impl SaveFormatComparator
{
    pub fn new(
        fileName: String,
        worldName: String,
        lastPlayed: i64,
        size: i64,
        sameVersion: bool,
    ) -> SaveFormatComparator
    {
        SaveFormatComparator {
            m_fileName: fileName,
            m_displayName: worldName,
            m_lastPlayed: lastPlayed,
            m_size: size,
            m_sameVersion: sameVersion,
        }
    }

    pub fn getFileName(&self) -> &str { &self.m_fileName }

    pub fn getDisplayName(&self) -> &str { &self.m_displayName }

    pub fn getSize(&self) -> i64 { self.m_size }

    pub fn isSameVersion(&self) -> bool { self.m_sameVersion }

    pub fn getLastPlayed(&self) -> i64 { self.m_lastPlayed }

    pub fn compareTo(&self, other: &SaveFormatComparator) -> Ordering
    {
        if self.m_lastPlayed < other.m_lastPlayed
        {
            return Ordering::Greater;
        }
        if self.m_lastPlayed > other.m_lastPlayed
        {
            return Ordering::Less;
        }
        self.m_fileName.encode_utf16().cmp(other.m_fileName.encode_utf16())
    }
}
