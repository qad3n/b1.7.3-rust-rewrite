use crate::resources::animation::{self, Animation};
use crate::resources::sprites::{ITEMS, Sprite, TERRAIN};
use crate::resources::{Location, Stack, legacy};
use jvm::image::{BufferedImage, ImageIO};
use jvm::io::ByteArrayInputStream;
use jvm::{debug, warning};
use std::rc::Rc;

const TILES: i32 = 16;

struct Sheet
{
    m_beta: &'static str,
    m_folder: &'static str,
    m_sprites: &'static [Sprite],
}

const SHEETS: &[Sheet] = &[
    Sheet { m_beta: "terrain.png", m_folder: "block", m_sprites: TERRAIN },
    Sheet { m_beta: "gui/items.png", m_folder: "item", m_sprites: ITEMS },
];

struct Paint
{
    m_slot: usize,
    m_span: usize,
    m_image: BufferedImage,
    m_meta: Option<Vec<u8>>,
}

impl Paint
{
    // the tile size this sprite would need on its own
    fn tile(&self) -> i32 { self.m_image.getWidth() / self.m_span as i32 }

    // the covered slots from the top left across and down
    fn slots(&self) -> impl Iterator<Item = usize> + '_
    {
        (0..self.m_span)
            .flat_map(move |down| (0..self.m_span).map(move |across| self.m_slot + down * TILES as usize + across))
    }
}

fn decode(location: &Location, bytes: Vec<u8>) -> Option<BufferedImage>
{
    let mut input: ByteArrayInputStream = ByteArrayInputStream::new(bytes);
    match ImageIO::read(Some(&mut input))
    {
        Ok(image) => image,

        Err(error) =>
        {
            warning!("Texture {location} could not be decoded: {error}");
            None
        }
    }
}

// the top sprite of each slot when it comes from a pack above the pack that supplied the whole sheet
fn find(sheet: &Sheet, stack: &Stack, base: usize) -> Vec<Paint>
{
    let mut found: Vec<Paint> = Vec::new();

    for sprite in sheet.m_sprites
    {
        let location: Location = Location::minecraft(&format!("textures/{}/{}.png", sheet.m_folder, sprite.m_name));
        let owner: Option<usize> = stack.owner(&location).filter(|owner| *owner > base);
        let Some(bytes) = owner.and_then(|owner| stack.pack(owner).read(&location))
        else
        {
            continue;
        };

        // a .mcmeta only counts from the pack serving the png or one above it
        let meta: Location = Location::new(&location.m_namespace, &format!("{}.mcmeta", location.m_path));
        let metaOwner: Option<usize> = stack.owner(&meta).filter(|metaOwner| Some(*metaOwner) >= owner);
        let metaBytes: Option<Vec<u8>> = metaOwner.and_then(|metaOwner| stack.pack(metaOwner).read(&meta));

        if let Some(image) = decode(&location, bytes)
        {
            found.push(Paint { m_slot: sprite.m_slot, m_span: sprite.m_span, m_image: image, m_meta: metaBytes });
        }
    }

    found
}

// nearest neighbour copy of a square from the source into a square of the target
fn blit(source: &BufferedImage, from: (i32, i32, i32), target: &BufferedImage, to: (i32, i32, i32))
{
    let (sourceX, sourceY, sourceSize): (i32, i32, i32) = from;
    let (targetX, targetY, targetSize): (i32, i32, i32) = to;

    for y in 0..targetSize
    {
        for x in 0..targetSize
        {
            let rgb: i32 = source.getRGB(sourceX + x * sourceSize / targetSize, sourceY + y * sourceSize / targetSize);
            target.setRGB(targetX + x, targetY + y, rgb);
        }
    }
}

fn paint(base: &BufferedImage, paints: &[Paint], tile: i32) -> BufferedImage
{
    let baseTile: i32 = base.getWidth() / TILES;
    let rows: i32 = base.getHeight() / baseTile;
    let image: BufferedImage = BufferedImage::new(TILES * tile, rows * tile, BufferedImage::TYPE_INT_ARGB);

    for slot in 0..TILES * rows
    {
        let (x, y): (i32, i32) = (slot % TILES, slot / TILES);
        blit(base, (x * baseTile, y * baseTile, baseTile), &image, (x * tile, y * tile, tile));
    }

    // a sprite taller than wide is an animation strip and its first frame is shown
    for paint in paints
    {
        let (x, y, size): (i32, i32, i32) = place(paint, tile);
        let width: i32 = paint.m_image.getWidth();
        blit(&paint.m_image, (0, 0, width), &image, (x, y, size));
    }

    image
}

// where a sprite sits on the sheet in pixels and its size there
fn place(paint: &Paint, tile: i32) -> (i32, i32, i32)
{
    let slot: i32 = paint.m_slot as i32;
    (slot % TILES * tile, slot / TILES * tile, tile * paint.m_span as i32)
}

// modern sprites painted over a beta sheet and the sheet comes back untouched when no pack above it has one
pub(crate) fn overlay(stack: &Stack, path: &str, image: Rc<BufferedImage>) -> Rc<BufferedImage>
{
    let path: &str = path.trim_start_matches('/');
    let Some(sheet) = SHEETS.iter().find(|sheet| sheet.m_beta == path)
    else
    {
        return image;
    };

    let whole: Location = legacy::location(path);
    let base: usize = stack.owner(&whole).unwrap_or(0);
    let paints: Vec<Paint> = find(sheet, stack, base);
    if paints.is_empty() || image.getWidth() < TILES
    {
        animation::set(path, Vec::new(), Vec::new());
        return image;
    }

    debug!("Painting {} texture pack sprites onto {path}", paints.len());

    let tile: i32 = paints.iter().map(Paint::tile).fold(image.getWidth() / TILES, i32::max);
    let painted: Vec<usize> = paints.iter().flat_map(Paint::slots).collect();
    let animations: Vec<Animation> = paints
        .iter()
        .filter_map(|paint| Animation::new(&paint.m_image, paint.m_meta.as_deref(), place(paint, tile)))
        .collect();
    animation::set(path, painted, animations);

    Rc::new(paint(&image, &paints, tile))
}
