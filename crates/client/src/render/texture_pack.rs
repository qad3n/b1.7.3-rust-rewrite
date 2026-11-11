use crate::client::Minecraft;
use crate::render::RenderEngine;
use game::resources::Pack;
use jvm::image::{BufferedImage, ImageIO};
use jvm::io::ByteArrayInputStream;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

const UNKNOWN_ICON: &str = "/gui/unknown_pack.png";

pub struct TexturePack
{
    m_pack: Rc<Pack>,
    m_key: Option<String>,
    m_thumbnail: Option<Rc<BufferedImage>>,
    m_texture: Cell<i32>,
}

fn readIcon(pack: &Pack) -> Option<Rc<BufferedImage>>
{
    let mut input: ByteArrayInputStream = ByteArrayInputStream::new(pack.icon()?);
    match ImageIO::read(Some(&mut input))
    {
        Ok(image) => image.map(Rc::new),

        Err(error) =>
        {
            error.printStackTrace();
            None
        }
    }
}

impl TexturePack
{
    pub fn new(pack: Rc<Pack>, key: Option<String>) -> Self
    {
        let thumbnail: Option<Rc<BufferedImage>> = readIcon(&pack);
        Self { m_pack: pack, m_key: key, m_thumbnail: thumbnail, m_texture: Cell::new(-1) }
    }

    pub fn pack(&self) -> &Rc<Pack> { &self.m_pack }

    pub fn key(&self) -> Option<&str> { self.m_key.as_deref() }

    pub fn unload(&self, minecraft: &Minecraft)
    {
        if self.m_thumbnail.is_some()
        {
            minecraft.m_renderEngine.get().orNpe().deleteTexture(self.m_texture.get());
        }
    }

    pub fn bindThumbnail(&self, minecraft: &Minecraft)
    {
        let engine: Rc<RenderEngine> = minecraft.m_renderEngine.get().orNpe();

        match &self.m_thumbnail
        {
            Some(image) =>
            {
                if self.m_texture.get() < 0
                {
                    self.m_texture.set(engine.allocateAndSetupTexture(Rc::clone(image)));
                }

                engine.bindTexture(self.m_texture.get());
            }

            None => GL11::glBindTexture(GL11::GL_TEXTURE_2D, engine.getTexture(UNKNOWN_ICON)),
        }
    }
}
