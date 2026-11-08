pub struct SoundPoolEntry
{
    pub m_soundName: String,
    pub m_soundUrl: String,
}

impl SoundPoolEntry
{
    pub fn new(path: String, url: String) -> SoundPoolEntry { SoundPoolEntry { m_soundName: path, m_soundUrl: url } }
}
