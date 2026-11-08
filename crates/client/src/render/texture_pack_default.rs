use crate::assets;
use crate::client::Minecraft;
use crate::render::{TexturePackBase, TexturePackFields};
use jvm::image::{BufferedImage, ImageIO};
use jvm::io::FileInputStream;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

pub struct TexturePackDefault
{
    m_pack: TexturePackFields,
    m_texturePackName: Cell<i32>,
    m_texturePackThumbnail: Option<Rc<BufferedImage>>,
}

pub(super) fn bindThumbnail(minecraft: &Minecraft, thumbnail: Option<&Rc<BufferedImage>>, texturePackName: &Cell<i32>)
{
    let renderEngine = minecraft.m_renderEngine.get().orNpe();
    match thumbnail
    {
        Some(image) =>
        {
            if texturePackName.get() < 0
            {
                texturePackName.set(renderEngine.allocateAndSetupTexture(Rc::clone(image)));
            }
            renderEngine.bindTexture(texturePackName.get());
        }
        None => GL11::glBindTexture(GL11::GL_TEXTURE_2D, renderEngine.getTexture("/gui/unknown_pack.png")),
    }
}

impl TexturePackDefault
{
    pub fn new() -> TexturePackDefault
    {
        let pack: TexturePackFields = TexturePackFields::default();
        pack.m_texturePackFileName.set(Some("Default".to_owned()));
        pack.m_firstDescriptionLine.set(Some("The default look of Minecraft".to_owned()));
        let mut stream: Option<FileInputStream> = assets::getResourceAsStream("/pack.png");
        let thumbnail: Option<Rc<BufferedImage>> = match ImageIO::read(stream.as_mut())
        {
            Ok(image) => image.map(Rc::new),
            Err(error) =>
            {
                error.printStackTrace();
                None
            }
        };
        TexturePackDefault { m_pack: pack, m_texturePackName: Cell::new(-1), m_texturePackThumbnail: thumbnail }
    }
}

impl TexturePackBase for TexturePackDefault
{
    fn pack(&self) -> &TexturePackFields { &self.m_pack }

    fn unload(&self, minecraft: &Rc<Minecraft>)
    {
        if self.m_texturePackThumbnail.is_some()
        {
            minecraft.m_renderEngine.get().orNpe().deleteTexture(self.m_texturePackName.get());
        }
    }

    fn bindThumbnailTexture(&self, minecraft: &Rc<Minecraft>)
    {
        bindThumbnail(minecraft, self.m_texturePackThumbnail.as_ref(), &self.m_texturePackName);
    }
}
