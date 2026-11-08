#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumSkyBlock
{
    Sky,
    Block,
}

impl EnumSkyBlock
{
    pub fn defaultValue(self) -> i32
    {
        match self
        {
            Self::Sky => 15,
            Self::Block => 0,
        }
    }
}
