use crate::resources::sprites::{ITEMS, Sprite, TERRAIN};
use crate::resources::{Location, Pack, legacy};
use jvm::image::{BufferedImage, ImageIO};
use jvm::io::ByteArrayInputStream;
use jvm::warning;
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
    m_image: BufferedImage,
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

// the top sprite of each slot from packs above the pack that supplied the whole sheet
fn find(sheet: &Sheet, above: &[Rc<Pack>]) -> Vec<Paint>
{
    let mut found: Vec<Paint> = Vec::new();

    for sprite in sheet.m_sprites
    {
        let location: Location = Location::minecraft(&format!("textures/{}/{}.png", sheet.m_folder, sprite.m_name));
        let Some(bytes) = above.iter().rev().find_map(|pack| pack.read(&location))
        else
        {
            continue;
        };

        if let Some(image) = decode(&location, bytes)
        {
            found.push(Paint { m_slot: sprite.m_slot, m_image: image });
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

fn paint(base: &BufferedImage, paints: &[Paint]) -> BufferedImage
{
    let baseTile: i32 = base.getWidth() / TILES;
    let tile: i32 = paints.iter().map(|paint| paint.m_image.getWidth()).fold(baseTile, i32::max);
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
        let slot: i32 = paint.m_slot as i32;
        let size: i32 = paint.m_image.getWidth();
        blit(&paint.m_image, (0, 0, size), &image, (slot % TILES * tile, slot / TILES * tile, tile));
    }

    image
}

// modern sprites painted over a beta sheet and the sheet comes back untouched when no pack above it has one
pub(crate) fn overlay(packs: &[Rc<Pack>], path: &str, image: Rc<BufferedImage>) -> Rc<BufferedImage>
{
    let path: &str = path.trim_start_matches('/');
    let Some(sheet) = SHEETS.iter().find(|sheet| sheet.m_beta == path)
    else
    {
        return image;
    };

    let whole: Location = legacy::location(path);
    let base: usize = packs.iter().rposition(|pack| pack.has(&whole)).unwrap_or(0);
    let paints: Vec<Paint> = find(sheet, &packs[base + 1..]);
    if paints.is_empty() || image.getWidth() < TILES
    {
        return image;
    }

    Rc::new(paint(&image, &paints))
}
