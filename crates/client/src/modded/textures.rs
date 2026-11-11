use crate::render::{GLAllocation, RenderEngine, TextureFX, TextureFXFields};
use jvm::image::BufferedImage;
use lwjgl::buffers::{Buffer, ByteBuffer};
use lwjgl::opengl::GL11;
use modloader::atlas::{self, AnimatedTile, Atlas};
use std::cell::{Cell, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

thread_local!
{
    static PIXELS: RefCell<Option<ByteBuffer>> = const { RefCell::new(None) };
    static REGISTERED: Cell<bool> = const { Cell::new(false) };
    // a dynamic texture's GL name and the version last uploaded to it
    static UPLOADED: RefCell<HashMap<String, (i32, u32)>> = RefCell::new(HashMap::new());
    // a mod texture name's file path, resolved once since names never change meaning
    static PATHS: RefCell<HashMap<String, Rc<str>>> = RefCell::new(HashMap::new());
}

// RenderEngine.allocateAndSetupTexture's image kind for argb
const ARGB_IMAGE: i32 = 2;

// binds a texture a mod named, its runtime pixels when it sent some, its file otherwise
pub fn bind(engine: &RenderEngine, texture: &str)
{
    let Some(dynamic) = modloader::textures::get(texture)
    else
    {
        bindFile(engine, texture);
        return;
    };
    let (width, height): (i32, i32) = (dynamic.m_width as i32, dynamic.m_height as i32);

    let id: i32 = UPLOADED.with(|uploaded|
    {
        let mut uploaded: RefMut<HashMap<String, (i32, u32)>> = uploaded.borrow_mut();
        if !uploaded.contains_key(texture)
        {
            let image: Rc<BufferedImage> = Rc::new(BufferedImage::new(width, height, ARGB_IMAGE));
            uploaded.insert(texture.to_owned(), (engine.allocateAndSetupTexture(image), 0));
        }

        let entry: &mut (i32, u32) = uploaded.get_mut(texture).expect("texture was just inserted");
        if entry.1 != dynamic.m_version
        {
            let pixels: Vec<i32> = dynamic.m_pixels.iter().map(|pixel| *pixel as i32).collect();
            engine.bind(&pixels, width, height, entry.0);
            entry.1 = dynamic.m_version;
        }

        entry.0
    });
    GL11::glBindTexture(GL11::GL_TEXTURE_2D, id);
}

// binds a mod texture's file through the engine's own cache
pub fn bindFile(engine: &RenderEngine, texture: &str)
{
    let path: Rc<str> = PATHS.with(|paths|
    {
        if let Some(path) = paths.borrow().get(texture)
        {
            return Rc::clone(path);
        }

        let path: Rc<str> = modloader::entities::modTexturePath(texture).map_or_else(|| texture.into(), Into::into);
        paths.borrow_mut().insert(texture.to_owned(), Rc::clone(&path));
        path
    });
    engine.bindTexture(engine.getTexture(&path));
}

// the largest tile a texture pack uses, 256 pixel tiles in rgba
const MAX_TILE_BYTES: i32 = 256 * 256 * 4;

fn upload(tile: &AnimatedTile)
{
    PIXELS.with(|pixels|
    {
        let mut pixels: RefMut<Option<Buffer<i8>>> = pixels.borrow_mut();
        let buffer: &mut ByteBuffer =
            pixels.get_or_insert_with(|| GLAllocation::createDirectByteBuffer(MAX_TILE_BYTES));
        let bytes: Vec<i8> = tile.m_rgba.iter().map(|byte| *byte as i8).collect();

        buffer.clear();
        buffer.putSlice(&bytes);
        buffer.setPosition(0).setLimit(bytes.len() as i32);
        GL11::glTexSubImage2D(
            GL11::GL_TEXTURE_2D,
            0,
            tile.m_x,
            tile.m_y,
            tile.m_size,
            tile.m_size,
            GL11::GL_RGBA,
            GL11::GL_UNSIGNED_BYTE,
            buffer,
        );
    });
}

// a texture effect in vanilla's own list that steps every mod frame strip, it uploads nothing of its own
struct ModSprites
{
    m_fx: TextureFXFields,
    m_engine: Weak<RenderEngine>,
}

impl TextureFX for ModSprites
{
    fn fx(&self) -> &TextureFXFields { &self.m_fx }

    fn onTick(&self)
    {
        if let Some(engine) = self.m_engine.upgrade()
        {
            animate(&engine);
        }
    }

    fn bindImage(&self, _textureManager: &RenderEngine) {}
}

// registers the mod sprite effect with the engine once, through RenderEngine.registerTextureFX
pub fn register(engine: &Rc<RenderEngine>)
{
    if REGISTERED.get()
    {
        return;
    }

    REGISTERED.set(true);

    let fx: TextureFXFields = TextureFXFields::new(0);
    // a tile size of zero keeps vanilla from copying this effect's own pixels anywhere
    fx.m_tileSize.set(0);
    engine.registerTextureFX(Rc::new(ModSprites { m_fx: fx, m_engine: Rc::downgrade(engine) }));
}

// mod sprites drawn as frame strips step through their frames once a tick like the vanilla texture effects
fn animate(engine: &RenderEngine)
{
    for (sheet, path) in [(Atlas::Terrain, atlas::TERRAIN_PATH), (Atlas::Items, atlas::ITEMS_PATH)]
    {
        let due: Vec<AnimatedTile> = atlas::animate(sheet);
        if due.is_empty()
        {
            continue;
        }

        GL11::glBindTexture(GL11::GL_TEXTURE_2D, engine.getTexture(path));

        for tile in &due
        {
            upload(tile);
        }
    }
}
