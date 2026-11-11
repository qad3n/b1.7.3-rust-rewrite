pub const MINECRAFT: &str = "minecraft";

pub struct Location
{
    pub m_namespace: String,
    pub m_path: String,
}

impl Location
{
    pub fn new(namespace: &str, path: &str) -> Self
    {
        Self { m_namespace: namespace.to_owned(), m_path: path.to_owned() }
    }

    pub fn minecraft(path: &str) -> Self { Self::new(MINECRAFT, path) }

    // where a modern pack keeps the file
    pub(crate) fn file(&self) -> String { format!("assets/{}/{}", self.m_namespace, self.m_path) }
}

impl std::fmt::Display for Location
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(formatter, "{}:{}", self.m_namespace, self.m_path)
    }
}
