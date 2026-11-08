use crate::assets;
use crate::client::Minecraft;
use jvm::field::Field;
use jvm::io::{InputStream, IoResult};
use std::rc::Rc;

#[derive(Default)]
pub struct TexturePackFields
{
    pub m_texturePackFileName: Field<Option<String>>,
    pub m_firstDescriptionLine: Field<Option<String>>,
    pub m_secondDescriptionLine: Field<Option<String>>,
    pub m_key: Field<Option<String>>,
}

pub trait TexturePackBase
{
    fn pack(&self) -> &TexturePackFields;

    fn open(&self) {}

    fn closeTexturePackFile(&self) {}

    fn load(&self, _minecraft: &Rc<Minecraft>) -> IoResult<()> { Ok(()) }

    fn unload(&self, _minecraft: &Rc<Minecraft>) {}

    fn bindThumbnailTexture(&self, _minecraft: &Rc<Minecraft>) {}

    fn getResourceAsStream(&self, path: &str) -> Option<Box<dyn InputStream>>
    {
        assets::getResourceAsStream(path).map(|stream| Box::new(stream) as Box<dyn InputStream>)
    }
}
