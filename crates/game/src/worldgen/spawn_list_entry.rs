#[derive(Clone, Debug)]
pub struct SpawnListEntry
{
    pub m_entityClass: &'static str,
    pub m_spawnRarityRate: i32,
}

impl SpawnListEntry
{
    pub fn new(entityClass: &'static str, weight: i32) -> SpawnListEntry
    {
        SpawnListEntry { m_entityClass: entityClass, m_spawnRarityRate: weight }
    }
}
