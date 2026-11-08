use jvm::image::BufferedImage;
use mod_protocol::Identifier;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Atlas
{
    Terrain,
    Items,
}

// slots the vanilla sheets fill with the purple placeholder or leave transparent, taken from the b1.7.3 pngs
const TERRAIN_FREE: [i32; 85] = [
    30, 54, 76, 77, 78, 85, 100, 101, 111, 125, 126, 127, 136, 137, 138, 139, 141, 142, 143, 148, 153, 154, 155, 156,
    157, 158, 159, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189,
    190, 191, 196, 197, 198, 199, 200, 201, 202, 203, 204, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 224,
    226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 250, 251, 252, 253,
];

const ITEMS_FREE: [i32; 120] = [
    46, 61, 62, 91, 101, 102, 105, 106, 107, 108, 117, 118, 119, 120, 121, 122, 123, 124, 133, 134, 137, 138, 139, 140,
    144, 145, 146, 147, 148, 149, 150, 152, 153, 154, 155, 156, 160, 161, 162, 163, 164, 165, 166, 168, 169, 170, 171,
    172, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 192, 193, 194, 195, 196, 197, 198, 199, 200,
    201, 202, 203, 204, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226,
    227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251,
    252, 253, 254, 255,
];

pub const TERRAIN_PATH: &str = "/terrain.png";
pub const ITEMS_PATH: &str = "/gui/items.png";
const SHEET_TILES: i32 = 16;
const VANILLA_TILES: i32 = 256;

struct Sprite
{
    m_size: u32,
    m_argb: Vec<i32>,
}

#[derive(Default)]
struct State
{
    m_requested: BTreeSet<(Atlas, Identifier)>,
    m_slots: BTreeMap<(Atlas, Identifier), i32>,
    m_sprites: BTreeMap<(Atlas, Identifier), Sprite>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn request(atlas: Atlas, texture: &str)
{
    if let Ok(id) = Identifier::parse(texture)
        && !id.isVanilla()
    {
        STATE.with(|state| state.borrow_mut().m_requested.insert((atlas, id)));
    }
}

// slots go out in sorted identifier order, so every side and every run agrees without syncing them
// the vanilla placeholder slots fill first, then whole rows grow below the 256 vanilla tiles
pub fn assign()
{
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let requested: Vec<(Atlas, Identifier)> = state.m_requested.iter().cloned().collect();
        let mut terrain = TERRAIN_FREE.iter().copied().chain(VANILLA_TILES..);
        let mut items = ITEMS_FREE.iter().copied().chain(VANILLA_TILES..);
        for key in requested
        {
            let slot: Option<i32> = match key.0
            {
                Atlas::Terrain => terrain.next(),
                Atlas::Items => items.next(),
            };
            if let Some(slot) = slot
            {
                state.m_slots.insert(key, slot);
            }
        }
    });
}

// tile rows of a sheet, 16 for vanilla and more once mod sprites spill past slot 255
pub fn rows(atlas: Atlas) -> i32
{
    STATE.with(|state| {
        let last: i32 = state
            .borrow()
            .m_slots
            .iter()
            .filter(|((sheet, _), _)| *sheet == atlas)
            .map(|(_, slot)| *slot)
            .max()
            .unwrap_or(0);
        (last / SHEET_TILES + 1).max(SHEET_TILES)
    })
}

fn atlasOf(path: &str) -> Option<Atlas>
{
    match path
    {
        TERRAIN_PATH => Some(Atlas::Terrain),
        ITEMS_PATH => Some(Atlas::Items),
        _ => None,
    }
}

// v scale the texture matrix applies to a sheet, 1 for every texture that is not a grown sheet
pub fn vScale(path: &str) -> f32 { atlasOf(path).map_or(1.0, |atlas| SHEET_TILES as f32 / rows(atlas) as f32) }

pub fn slotOf(atlas: Atlas, id: &Identifier) -> Option<i32>
{
    STATE.with(|state| state.borrow().m_slots.get(&(atlas, id.clone())).copied())
}

pub fn texturePath(id: &Identifier) -> String { format!("assets/{}/textures/{}.png", id.namespace(), id.path()) }

fn decode(bytes: &[u8]) -> Result<Sprite, String>
{
    let mut decoder: png::Decoder<&[u8]> = png::Decoder::new(bytes);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader: png::Reader<&[u8]> = decoder.read_info().map_err(|error| error.to_string())?;
    let mut buffer: Vec<u8> = vec![0; reader.output_buffer_size()];
    let info: png::OutputInfo = reader.next_frame(&mut buffer).map_err(|error| error.to_string())?;
    if info.width != info.height || info.width == 0
    {
        return Err(format!("texture must be square, found {}x{}", info.width, info.height));
    }
    let channels: usize = info.color_type.samples();
    let argb: Vec<i32> = buffer[..info.buffer_size()]
        .chunks(channels)
        .map(|pixel| {
            let (red, green, blue, alpha): (u8, u8, u8, u8) = match channels
            {
                4 => (pixel[0], pixel[1], pixel[2], pixel[3]),
                3 => (pixel[0], pixel[1], pixel[2], 255),
                2 => (pixel[0], pixel[0], pixel[0], pixel[1]),
                _ => (pixel[0], pixel[0], pixel[0], 255),
            };
            ((alpha as u32) << 24 | (red as u32) << 16 | (green as u32) << 8 | blue as u32) as i32
        })
        .collect();
    Ok(Sprite { m_size: info.width, m_argb: argb })
}

// the client reads the png of every assigned texture from the mod that owns its namespace
pub fn loadSprites(read: &dyn Fn(&Identifier) -> Option<Vec<u8>>) -> Vec<String>
{
    let keys: Vec<(Atlas, Identifier)> = STATE.with(|state| state.borrow().m_slots.keys().cloned().collect());
    let mut problems: Vec<String> = Vec::new();
    for key in keys
    {
        let sprite: Result<Sprite, String> = match read(&key.1)
        {
            Some(bytes) => decode(&bytes),
            None => Err(format!("{} is missing", texturePath(&key.1))),
        };
        match sprite
        {
            Ok(sprite) => STATE.with(|state| state.borrow_mut().m_sprites.insert(key, sprite)).map_or((), |_| ()),
            Err(reason) => problems.push(format!("texture {}: {reason}", key.1)),
        }
    }
    problems
}

// paints mod sprites into a freshly loaded vanilla sheet, scaled to the tile size of the active texture pack
// a sheet that needs more rows is copied into a taller image first
pub fn stitch(path: &str, image: Rc<BufferedImage>) -> Rc<BufferedImage>
{
    let Some(atlas) = atlasOf(path)
    else
    {
        return image;
    };
    let tile: i32 = image.getWidth() / SHEET_TILES;
    if tile <= 0
    {
        return image;
    }
    let rows: i32 = rows(atlas);
    let image: Rc<BufferedImage> = if rows * tile > image.getHeight()
    {
        let grown: BufferedImage = BufferedImage::new(image.getWidth(), rows * tile, image.getType());
        let source = image.rasterData();
        grown.rasterData()[..source.len()].copy_from_slice(&source);
        drop(source);
        Rc::new(grown)
    }
    else
    {
        image
    };
    STATE.with(|state| {
        let state = state.borrow();
        for ((sheet, id), sprite) in &state.m_sprites
        {
            if *sheet != atlas
            {
                continue;
            }
            let Some(slot) = state.m_slots.get(&(*sheet, id.clone()))
            else
            {
                continue;
            };
            let originX: i32 = (slot % SHEET_TILES) * tile;
            let originY: i32 = (slot / SHEET_TILES) * tile;
            for y in 0..tile
            {
                for x in 0..tile
                {
                    let sourceX: u32 = (x as u32 * sprite.m_size) / tile as u32;
                    let sourceY: u32 = (y as u32 * sprite.m_size) / tile as u32;
                    let pixel: i32 = sprite.m_argb[(sourceY * sprite.m_size + sourceX) as usize];
                    image.setRGB(originX + x, originY + y, pixel);
                }
            }
        }
    });
    image
}
