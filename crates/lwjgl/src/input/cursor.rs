use crate::LWJGLException;
use crate::buffers::IntBuffer;

// MouseHelper builds an empty cursor and never installs it
pub struct Cursor
{
    m_width: i32,
    m_height: i32,
    m_xHotspot: i32,
    m_yHotspot: i32,
}

impl Cursor
{
    pub fn new(
        width: i32,
        height: i32,
        xHotspot: i32,
        yHotspot: i32,
        numImages: i32,
        images: &IntBuffer,
        _delays: Option<&IntBuffer>,
    ) -> Result<Cursor, LWJGLException>
    {
        if images.remaining() < width * height * numImages
        {
            return Err(LWJGLException::new("images buffer is smaller than width * height * numImages"));
        }
        Ok(Cursor { m_width: width, m_height: height, m_xHotspot: xHotspot, m_yHotspot: yHotspot })
    }

    pub fn getWidth(&self) -> i32 { self.m_width }

    pub fn getHeight(&self) -> i32 { self.m_height }

    pub fn getXHotspot(&self) -> i32 { self.m_xHotspot }

    pub fn getYHotspot(&self) -> i32 { self.m_yHotspot }

    pub fn destroy(&self) {}
}
