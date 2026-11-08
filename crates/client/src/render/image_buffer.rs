use jvm::image::BufferedImage;

pub trait ImageBuffer: Send
{
    fn parseUserSkin(&mut self, image: Option<BufferedImage>) -> Option<BufferedImage>;
}
