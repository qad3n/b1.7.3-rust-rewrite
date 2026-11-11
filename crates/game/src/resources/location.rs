pub(crate) const MINECRAFT: &str = "minecraft";

pub(crate) struct Location
{
    pub(crate) m_namespace: String,
    pub(crate) m_path: String,
}

impl Location
{
    pub(crate) fn minecraft(path: &str) -> Self { Self { m_namespace: MINECRAFT.to_owned(), m_path: path.to_owned() } }

    // where a modern pack keeps the file
    pub(crate) fn file(&self) -> String { format!("assets/{}/{}", self.m_namespace, self.m_path) }
}
