use crate::render::ImageBuffer;
use jvm::image::{BufferedImage, Graphics};

const SKIN_WIDTH: i32 = 64;
const SKIN_HEIGHT: i32 = 32;
const OPAQUE: i32 = 0xFF000000u32 as i32;
const RGB_MASK: i32 = 0xFFFFFF;

#[derive(Default)]
pub struct ImageBufferDownload
{
    m_imageWidth: i32,
    m_imageHeight: i32,
}

impl ImageBufferDownload
{
    pub fn new() -> ImageBufferDownload { ImageBufferDownload::default() }

    fn setTransparent(&self, imageData: &mut [i32], u1: i32, v1: i32, u2: i32, v2: i32)
    {
        if self.hasTransperancy(imageData, u1, v1, u2, v2)
        {
            return;
        }
        for i in u1..u2
        {
            for j in v1..v2
            {
                let index: usize = (i + j * self.m_imageWidth) as usize;
                imageData[index] &= RGB_MASK;
            }
        }
    }

    fn setOpaque(&self, imageData: &mut [i32], u1: i32, v1: i32, u2: i32, v2: i32)
    {
        for i in u1..u2
        {
            for j in v1..v2
            {
                let index: usize = (i + j * self.m_imageWidth) as usize;
                imageData[index] |= OPAQUE;
            }
        }
    }

    fn hasTransperancy(&self, imageData: &[i32], u1: i32, v1: i32, u2: i32, v2: i32) -> bool
    {
        (u1..u2).any(|i| (v1..v2).any(|j| (imageData[(i + j * self.m_imageWidth) as usize] >> 24 & 0xFF) < 128))
    }
}

impl ImageBuffer for ImageBufferDownload
{
    fn parseUserSkin(&mut self, image: Option<BufferedImage>) -> Option<BufferedImage>
    {
        let image: BufferedImage = image?;
        self.m_imageWidth = SKIN_WIDTH;
        self.m_imageHeight = SKIN_HEIGHT;
        let bufferedimage: BufferedImage =
            BufferedImage::new(self.m_imageWidth, self.m_imageHeight, BufferedImage::TYPE_INT_ARGB);
        let graphics: Graphics<'_> = bufferedimage.getGraphics();
        graphics.drawImage(&image, 0, 0);
        graphics.dispose();
        {
            let mut imageData = bufferedimage.rasterData();
            self.setOpaque(&mut imageData, 0, 0, 32, 16);
            self.setTransparent(&mut imageData, 32, 0, 64, 32);
            self.setOpaque(&mut imageData, 0, 16, 64, 32);
        }
        Some(bufferedimage)
    }
}
