use crate::render::RenderEngine;
use lwjgl::opengl::GL11;
use std::cell::{Cell, RefCell};

pub const TILE_BYTES: usize = 1024;
pub const TILE_PIXELS: usize = 256;

pub struct TextureFXFields
{
    pub m_imageData: RefCell<Vec<i8>>,
    pub m_iconIndex: Cell<i32>,
    pub m_anaglyphEnabled: Cell<bool>,
    pub m_textureId: Cell<i32>,
    pub m_tileSize: Cell<i32>,
    pub m_tileImage: Cell<i32>,
}

impl TextureFXFields
{
    pub fn new(sprite: i32) -> TextureFXFields
    {
        TextureFXFields {
            m_imageData: RefCell::new(vec![0; TILE_BYTES]),
            m_iconIndex: Cell::new(sprite),
            m_anaglyphEnabled: Cell::new(false),
            m_textureId: Cell::new(0),
            m_tileSize: Cell::new(1),
            m_tileImage: Cell::new(0),
        }
    }

    pub fn tint(&self, r: i32, g: i32, b: i32) -> (i32, i32, i32)
    {
        if !self.m_anaglyphEnabled.get()
        {
            return (r, g, b);
        }
        ((r * 30 + g * 59 + b * 11) / 100, (r * 30 + g * 70) / 100, (r * 30 + b * 70) / 100)
    }

    pub fn setPixel(&self, index: usize, r: i32, g: i32, b: i32, a: i32)
    {
        let mut data = self.m_imageData.borrow_mut();
        data[index * 4] = r as i8;
        data[index * 4 + 1] = g as i8;
        data[index * 4 + 2] = b as i8;
        data[index * 4 + 3] = a as i8;
    }
}

pub trait TextureFX
{
    fn fx(&self) -> &TextureFXFields;

    fn onTick(&self) {}

    fn bindImage(&self, textureManager: &RenderEngine)
    {
        match self.fx().m_tileImage.get()
        {
            0 => GL11::glBindTexture(GL11::GL_TEXTURE_2D, textureManager.getTexture("/terrain.png")),
            1 => GL11::glBindTexture(GL11::GL_TEXTURE_2D, textureManager.getTexture("/gui/items.png")),
            _ =>
            {}
        }
    }
}
