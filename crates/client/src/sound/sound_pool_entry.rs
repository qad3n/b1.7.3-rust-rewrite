pub struct SoundPoolEntry
{
    pub m_soundName: String,
    pub m_soundUrl: String,
}

impl SoundPoolEntry
{
    pub fn new(path: String, url: String) -> Self { Self { m_soundName: path, m_soundUrl: url } }
}
