use jvm::image::BufferedImage;
use mod_protocol::Identifier;
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet};
use std::iter::Copied;
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
pub(crate) const VANILLA_TILES: i32 = 256;

struct Sprite
{
    m_size: u32,
    m_argb: Vec<i32>,
    // a png taller than wide is a strip of square frames, played in order like a resource pack .mcmeta says
    m_animation: Option<Animation>,
}

struct Animation
{
    m_frames: Vec<Vec<i32>>,
    m_order: Vec<usize>,
    m_frameTime: u32,
}

// a frame of an animated sprite due on the sheet, its pixels already scaled to the sheet tile
pub struct AnimatedTile
{
    pub m_x: i32,
    pub m_y: i32,
    pub m_size: i32,
    pub m_rgba: Vec<u8>,
}

// one tile of a texture cut into a grid, blown up to a whole sprite
#[derive(Clone)]
struct Slice
{
    m_source: Identifier,
    m_columns: i32,
    m_rows: i32,
    m_column: i32,
    m_row: i32,
}

mod tiles
{
    pub const NAMESPACE: &str = "loader";
    pub const MOST: i32 = 16;
}

#[derive(Default)]
struct State
{
    m_requested: BTreeSet<(Atlas, Identifier)>,
    m_slots: BTreeMap<(Atlas, Identifier), i32>,
    m_sprites: BTreeMap<(Atlas, Identifier), Sprite>,
    // tile size of each sheet as last stitched, animation frames are scaled to it
    m_tiles: BTreeMap<Atlas, i32>,
    m_ticks: u32,
    // the tile sprites cut from other textures, painted from the sheet once their source is in it
    m_slices: BTreeMap<(Atlas, Identifier), Slice>,
    // sprites mods drew in code since the last upload, as TextureFX effects draw theirs
    m_painted: Vec<(Atlas, i32, u32, Vec<i32>)>,
}

thread_local!
{
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

// the sprite name of one tile, matches b173_api::world::tileName
fn tileId(source: &Identifier, column: i32, row: i32) -> Result<Identifier, String>
{
    let path: String = format!("tiles/{}/{}/{column}_{row}", source.namespace(), source.path());
    Identifier::new(tiles::NAMESPACE, &path).map_err(|error| format!("tile name {path}: {error:?}"))
}

// cuts a texture into a grid whose tiles each get a slot, a vanilla terrain/<index> tile or a mod texture
pub fn requestTiles(atlas: Atlas, texture: &str, columns: i32, rows: i32) -> Result<(), String>
{
    if !(1..=tiles::MOST).contains(&columns) || !(1..=tiles::MOST).contains(&rows)
    {
        return Err(format!("a texture cuts into 1 to {} tiles each way, asked {columns}x{rows}", tiles::MOST));
    }

    let source: Identifier = Identifier::parse(texture).map_err(|error| format!("texture {texture}: {error:?}"))?;

    request(atlas, texture);

    for column in 0..columns
    {
        for row in 0..rows
        {
            let id: Identifier = tileId(&source, column, row)?;
            let slice: Slice =
                Slice { m_source: source.clone(), m_columns: columns, m_rows: rows, m_column: column, m_row: row };

            STATE.with(|state|
            {
                let mut state: RefMut<crate::atlas::State> = state.borrow_mut();
                state.m_requested.insert((atlas, id.clone()));
                state.m_slices.insert((atlas, id), slice);
            });
        }
    }

    Ok(())
}

// slots go out in sorted identifier order, so every side and every run agrees without syncing them
// the vanilla placeholder slots fill first, then whole rows grow below the 256 vanilla tiles
pub fn assign()
{
    STATE.with(|state|
    {
        let mut state: RefMut<crate::atlas::State> = state.borrow_mut();
        let requested: Vec<(Atlas, Identifier)> = state.m_requested.iter().cloned().collect();
        let mut terrain: std::iter::Chain<Copied<std::slice::Iter<i32>>, std::ops::RangeFrom<i32>> =
            TERRAIN_FREE.iter().copied().chain(VANILLA_TILES..);
        let mut items: std::iter::Chain<Copied<std::slice::Iter<i32>>, std::ops::RangeFrom<i32>> =
            ITEMS_FREE.iter().copied().chain(VANILLA_TILES..);

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
    STATE.with(|state|
    {
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

pub fn metaPath(id: &Identifier) -> String { format!("{}.mcmeta", texturePath(id)) }

// the two .mcmeta fields an animation needs, frametime and a frames list of plain indices
fn parseMeta(text: &str) -> (u32, Vec<usize>)
{
    let number = |key: &str| -> Option<u32>
    {
        let rest: &str = &text[text.find(key)? + key.len()..];
        let digits: String = rest
            .trim_start_matches(|c: char| c == '"' || c == ':' || c.is_whitespace())
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        digits.parse().ok()
    };
    let frameTime: u32 = number("\"frametime\"").unwrap_or(1).max(1);
    let order: Vec<usize> = text
        .find("\"frames\"")
        .and_then(|start|
        {
            let rest: &str = &text[start..];
            let open: usize = rest.find('[')?;
            let close: usize = rest.find(']')?;
            Some(rest[open + 1..close].split(',').filter_map(|value| value.trim().parse().ok()).collect())
        })
        .unwrap_or_default();
    (frameTime, order)
}

fn decode(bytes: &[u8], meta: Option<&str>) -> Result<Sprite, String>
{
    let mut decoder: png::Decoder<&[u8]> = png::Decoder::new(bytes);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);

    let mut reader: png::Reader<&[u8]> = decoder.read_info().map_err(|error| error.to_string())?;
    let mut buffer: Vec<u8> = vec![0; reader.output_buffer_size()];
    let info: png::OutputInfo = reader.next_frame(&mut buffer).map_err(|error| error.to_string())?;
    if info.width == 0 || !info.height.is_multiple_of(info.width)
    {
        return Err(format!(
            "texture must be square or a strip of square frames, found {}x{}",
            info.width, info.height
        ));
    }

    let channels: usize = info.color_type.samples();
    let argb: Vec<i32> = buffer[..info.buffer_size()]
        .chunks(channels)
        .map(|pixel|
        {
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
    let area: usize = (info.width * info.width) as usize;
    let frames: Vec<Vec<i32>> = argb.chunks(area).map(<[i32]>::to_vec).collect();
    let first: Vec<i32> = frames[0].clone();

    let animation: Option<Animation> = (frames.len() > 1).then(||
    {
        let (frameTime, mut order): (u32, Vec<usize>) = meta.map(parseMeta).unwrap_or((1, Vec::new()));
        order.retain(|frame| *frame < frames.len());

        if order.is_empty()
        {
            order = (0..frames.len()).collect();
        }

        Animation { m_frames: frames, m_order: order, m_frameTime: frameTime }
    });
    Ok(Sprite { m_size: info.width, m_argb: first, m_animation: animation })
}

// the client reads the png of every assigned texture from the mod that owns its namespace
pub fn loadSprites(
    read: &dyn Fn(&Identifier) -> Option<Vec<u8>>,
    readMeta: &dyn Fn(&Identifier) -> Option<String>,
) -> Vec<String>
{
    let keys: Vec<(Atlas, Identifier)> = STATE.with(|state|
    {
        let state: std::cell::Ref<crate::atlas::State> = state.borrow();
        state.m_slots.keys().filter(|key| !state.m_slices.contains_key(*key)).cloned().collect()
    });

    let mut problems: Vec<String> = Vec::new();

    for key in keys
    {
        let sprite: Result<Sprite, String> = match read(&key.1)
        {
            Some(bytes) => decode(&bytes, readMeta(&key.1).as_deref()),
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

    STATE.with(|state| state.borrow_mut().m_tiles.insert(atlas, tile));

    let rows: i32 = rows(atlas);
    let image: Rc<BufferedImage> = if rows * tile > image.getHeight()
    {
        let grown: BufferedImage = BufferedImage::new(image.getWidth(), rows * tile, image.getType());
        let source: RefMut<Vec<i32>> = image.rasterData();
        grown.rasterData()[..source.len()].copy_from_slice(&source);
        drop(source);
        Rc::new(grown)
    }
    else
    {
        image
    };

    STATE.with(|state|
    {
        let state: std::cell::Ref<crate::atlas::State> = state.borrow();
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

    paintSlices(atlas, &image, tile);
    image
}

// the sheet slot of a texture, a vanilla texture names its own tile
fn sourceSlot(state: &State, atlas: Atlas, source: &Identifier) -> Option<i32>
{
    if source.isVanilla()
    {
        return crate::content::vanillaSlot(atlas, source.path());
    }

    state.m_slots.get(&(atlas, source.clone())).copied()
}

// each tile is read from its source already in the sheet and blown up pixel by pixel to a whole slot
fn paintSlices(atlas: Atlas, image: &BufferedImage, tile: i32)
{
    STATE.with(|state|
    {
        let state: std::cell::Ref<crate::atlas::State> = state.borrow();
        for ((sheet, id), slice) in &state.m_slices
        {
            if *sheet != atlas
            {
                continue;
            }
            let (Some(slot), Some(from)) =
                (state.m_slots.get(&(*sheet, id.clone())), sourceSlot(&state, atlas, &slice.m_source))
            else
            {
                continue;
            };

            let (width, height): (i32, i32) = (tile / slice.m_columns, tile / slice.m_rows);
            let sourceX: i32 = (from % SHEET_TILES) * tile + slice.m_column * width;
            let sourceY: i32 = (from / SHEET_TILES) * tile + slice.m_row * height;
            let originX: i32 = (slot % SHEET_TILES) * tile;
            let originY: i32 = (slot / SHEET_TILES) * tile;

            for y in 0..tile
            {
                for x in 0..tile
                {
                    let pixel: i32 = image.getRGB(sourceX + x * width / tile, sourceY + y * height / tile);
                    image.setRGB(originX + x, originY + y, pixel);
                }
            }
        }
    });
}

fn scaled(frame: &[i32], size: u32, tile: i32) -> Vec<u8>
{
    let mut rgba: Vec<u8> = Vec::with_capacity((tile * tile * 4) as usize);

    for y in 0..tile
    {
        for x in 0..tile
        {
            let pixel: u32 =
                frame[((y as u32 * size) / tile as u32 * size + (x as u32 * size) / tile as u32) as usize] as u32;
            rgba.extend_from_slice(&[(pixel >> 16) as u8, (pixel >> 8) as u8, pixel as u8, (pixel >> 24) as u8]);
        }
    }

    rgba
}

// advances every animated sprite one tick and hands back the tiles whose frame changed
// a mod's sprite drawn in code, square argb pixels, uploaded with the next animation step
pub fn paint(id: &Identifier, size: u32, argb: Vec<i32>) -> Result<(), String>
{
    if size == 0 || argb.len() != (size * size) as usize
    {
        return Err(format!("a painted sprite needs {size} by {size} pixels, got {}", argb.len()));
    }

    STATE.with(|state|
    {
        let mut state: RefMut<crate::atlas::State> = state.borrow_mut();
        let found: Option<(Atlas, i32)> = [Atlas::Terrain, Atlas::Items]
            .into_iter()
            .find_map(|sheet| state.m_slots.get(&(sheet, id.clone())).map(|slot| (sheet, *slot)));
        let (sheet, slot) = found.ok_or_else(|| format!("{id} is no sprite on a sheet"))?;

        state.m_painted.retain(|(painted, at, _, _)| !(*painted == sheet && *at == slot));
        state.m_painted.push((sheet, slot, size, argb));
        Ok(())
    })
}

// a sheet sprite's first frame as the texture packs gave it, for mods that redraw it in code
pub fn pixels(id: &Identifier) -> Option<(u32, Vec<u32>)>
{
    STATE.with(|state|
    {
        let state: std::cell::Ref<crate::atlas::State> = state.borrow();
        let sprite: &Sprite = [Atlas::Terrain, Atlas::Items]
            .into_iter()
            .find_map(|sheet| state.m_sprites.get(&(sheet, id.clone())))?;
        Some((sprite.m_size, sprite.m_argb.iter().map(|pixel| *pixel as u32).collect()))
    })
}

pub fn animate(atlas: Atlas) -> Vec<AnimatedTile>
{
    STATE.with(|state|
    {
        let mut state: RefMut<crate::atlas::State> = state.borrow_mut();

        if atlas == Atlas::Terrain
        {
            state.m_ticks = state.m_ticks.wrapping_add(1);
        }

        let ticks: u32 = state.m_ticks;

        let Some(tile) = state.m_tiles.get(&atlas).copied()
        else
        {
            return Vec::new();
        };

        let mut due: Vec<AnimatedTile> = Vec::new();
        let painted: Vec<(Atlas, i32, u32, Vec<i32>)> = std::mem::take(&mut state.m_painted);

        for (sheet, slot, size, argb) in painted
        {
            if sheet != atlas
            {
                state.m_painted.push((sheet, slot, size, argb));
                continue;
            }

            due.push(AnimatedTile
            {
                m_x: (slot % SHEET_TILES) * tile,
                m_y: (slot / SHEET_TILES) * tile,
                m_size: tile,
                m_rgba: scaled(&argb, size, tile),
            });
        }

        for ((sheet, id), sprite) in &state.m_sprites
        {
            let Some(animation) = sprite.m_animation.as_ref().filter(|_| *sheet == atlas)
            else
            {
                continue;
            };
            if !ticks.is_multiple_of(animation.m_frameTime)
            {
                continue;
            }
            let Some(slot) = state.m_slots.get(&(*sheet, id.clone()))
            else
            {
                continue;
            };

            let step: usize = (ticks / animation.m_frameTime) as usize % animation.m_order.len();
            let frame: &[i32] = &animation.m_frames[animation.m_order[step]];

            due.push(AnimatedTile
            {
                m_x: (slot % SHEET_TILES) * tile,
                m_y: (slot / SHEET_TILES) * tile,
                m_size: tile,
                m_rgba: scaled(frame, sprite.m_size, tile),
            });
        }

        due
    })
}
