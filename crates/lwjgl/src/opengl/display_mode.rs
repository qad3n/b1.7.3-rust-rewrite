#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DisplayMode
{
    m_width: i32,
    m_height: i32,
    m_bpp: i32,
    m_freq: i32,
}

impl DisplayMode
{
    pub fn new(width: i32, height: i32) -> DisplayMode
    {
        DisplayMode { m_width: width, m_height: height, m_bpp: 0, m_freq: 0 }
    }

    pub(crate) fn withFormat(width: i32, height: i32, bpp: i32, freq: i32) -> DisplayMode
    {
        DisplayMode { m_width: width, m_height: height, m_bpp: bpp, m_freq: freq }
    }

    pub fn getWidth(&self) -> i32 { self.m_width }

    pub fn getHeight(&self) -> i32 { self.m_height }

    pub fn getBitsPerPixel(&self) -> i32 { self.m_bpp }

    pub fn getFrequency(&self) -> i32 { self.m_freq }
}

impl std::fmt::Display for DisplayMode
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(formatter, "{} x {} x {} @{}Hz", self.m_width, self.m_height, self.m_bpp, self.m_freq)
    }
}
