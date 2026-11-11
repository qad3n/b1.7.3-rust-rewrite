use crate::render::{GLAllocation, RenderEngine, TextureFX, TextureFXFields};
use game::resources::Frame;
use lwjgl::buffers::{Buffer, ByteBuffer};
use lwjgl::opengl::GL11;
use std::cell::{RefCell, RefMut};
use std::rc::{Rc, Weak};

thread_local!
{
    static PIXELS: RefCell<Option<ByteBuffer>> = const { RefCell::new(None) };
}

// the largest area one upload covers, a two tile sprite of 256 pixel tiles in rgba
const MAX_TILE_BYTES: i32 = 512 * 512 * 4;

const SHEETS: [&str; 2] = ["/terrain.png", "/gui/items.png"];

// copies rgba pixels into the bound texture through one shared direct buffer
pub(crate) fn uploadTile(x: i32, y: i32, size: i32, rgba: &[u8])
{
    PIXELS.with(|pixels|
    {
        let mut pixels: RefMut<Option<Buffer<i8>>> = pixels.borrow_mut();
        let buffer: &mut ByteBuffer =
            pixels.get_or_insert_with(|| GLAllocation::createDirectByteBuffer(MAX_TILE_BYTES));

        buffer.clear();
        for byte in rgba
        {
            buffer.put(*byte as i8);
        }
        buffer.setPosition(0).setLimit(rgba.len() as i32);
        GL11::glTexSubImage2D(GL11::GL_TEXTURE_2D, 0, x, y, size, size, GL11::GL_RGBA, GL11::GL_UNSIGNED_BYTE, buffer);
    });
}

// a texture effect in vanilla's own list that plays texture pack animations, it uploads nothing of its own
pub struct TexturePackSprites
{
    m_fx: TextureFXFields,
    m_engine: Weak<RenderEngine>,
}

impl TexturePackSprites
{
    pub fn new(engine: &Rc<RenderEngine>) -> Self
    {
        let fx: TextureFXFields = TextureFXFields::new(0);
        // a tile size of zero keeps vanilla from copying this effect's own pixels anywhere
        fx.m_tileSize.set(0);
        Self { m_fx: fx, m_engine: Rc::downgrade(engine) }
    }
}

impl TextureFX for TexturePackSprites
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        let Some(engine) = self.m_engine.upgrade()
        else
        {
            return;
        };

        for path in SHEETS
        {
            let mut bound: bool = false;
            game::resources::animate(path, &mut |frame: Frame<'_>|
            {
                if !bound
                {
                    GL11::glBindTexture(GL11::GL_TEXTURE_2D, engine.getTexture(path));
                    bound = true;
                }

                uploadTile(frame.m_x, frame.m_y, frame.m_size, frame.m_rgba);
            });
        }
    }

    fn bindImage(&self, _textureManager: &RenderEngine) {}
}
