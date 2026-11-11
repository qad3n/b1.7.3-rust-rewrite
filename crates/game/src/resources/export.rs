use crate::resources::sheet::{SHEETS, TILES};
use crate::resources::sprites::EFFECTS;
use crate::resources::{legacy, vanilla};
use jvm::image::{BufferedImage, ImageIO};
use jvm::info;
use jvm::io::ByteArrayInputStream;
use std::path::{Path, PathBuf};

const ATLAS: &str = "textures/atlas/";
const ASSETS: &str = "assets/minecraft/";
const META: &str = "pack.mcmeta";
const ICON: &str = "pack.png";
const META_TEXT: &str =
    "{\n  \"pack\": {\n    \"pack_format\": 1,\n    \"description\": \"Beta 1.7.3 files from your own jar\"\n  }\n}\n";
const SKIPPED: &str = "_list.json";

struct SoundFolder
{
    m_beta: &'static str,
    m_modern: &'static str,
}

// the resources folder in pool order where a later folder wins a shared name
const SOUND_FOLDERS: &[SoundFolder] = &[
    SoundFolder { m_beta: "sound", m_modern: "sounds/" },
    SoundFolder { m_beta: "newsound", m_modern: "sounds/" },
    SoundFolder { m_beta: "music", m_modern: "sounds/music/" },
    SoundFolder { m_beta: "newmusic", m_modern: "sounds/music/" },
    SoundFolder { m_beta: "streaming", m_modern: "sounds/records/" },
];

fn write(target: &Path, name: &str, bytes: &[u8]) -> Result<(), String>
{
    let path: PathBuf = target.join(name);
    path.parent()
        .map_or(Ok(()), std::fs::create_dir_all)
        .and_then(|()| std::fs::write(&path, bytes))
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn jar(path: &str) -> Option<Vec<u8>> { vanilla().read(&legacy::location(path)) }

fn image(path: &str) -> Result<BufferedImage, String>
{
    let bytes: Vec<u8> = jar(path).ok_or_else(|| format!("{path} is missing from the extracted jar"))?;
    ImageIO::read(Some(&mut ByteArrayInputStream::new(bytes)))
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("{path} is not an image"))
}

// one sprite square cut out of a beta sheet
fn cut(sheet: &BufferedImage, slot: usize, span: usize) -> BufferedImage
{
    let tile: i32 = sheet.getWidth() / TILES;
    let size: i32 = tile * span as i32;
    let (left, top): (i32, i32) = (slot as i32 % TILES * tile, slot as i32 / TILES * tile);
    let sprite: BufferedImage = BufferedImage::new(size, size, BufferedImage::TYPE_INT_ARGB);

    for y in 0..size
    {
        for x in 0..size
        {
            sprite.setRGB(x, y, sheet.getRGB(left + x, top + y));
        }
    }

    sprite
}

fn sheets(target: &Path) -> Result<usize, String>
{
    let mut count: usize = 0;

    for sheet in SHEETS
    {
        let image: BufferedImage = image(sheet.m_beta)?;
        for sprite in sheet.m_sprites.iter().filter(|sprite| !EFFECTS.contains(&sprite.m_name))
        {
            let path: PathBuf = target.join(format!("{ASSETS}textures/{}/{}.png", sheet.m_folder, sprite.m_name));
            path.parent().map_or(Ok(()), std::fs::create_dir_all).map_err(|error| error.to_string())?;
            ImageIO::write(&cut(&image, sprite.m_slot, sprite.m_span), "png", &path)
                .map_err(|error| error.to_string())?;
            count += 1;
        }
    }

    Ok(count)
}

fn copyTree(from: &Path, root: &Path, target: &Path, prefix: &str) -> Result<usize, String>
{
    let Ok(entries) = std::fs::read_dir(from)
    else
    {
        return Ok(0);
    };

    let mut count: usize = 0;

    for entry in entries.flatten()
    {
        let path: PathBuf = entry.path();
        if path.is_dir()
        {
            count += copyTree(&path, root, target, prefix)?;
            continue;
        }

        let Ok(relative) = path.strip_prefix(root)
        else
        {
            continue;
        };

        let relative: String = relative.to_string_lossy().replace('\\', "/");
        if relative.ends_with(SKIPPED)
        {
            continue;
        }

        let bytes: Vec<u8> = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        write(target, &format!("{ASSETS}{prefix}{relative}"), &bytes)?;
        count += 1;
    }

    Ok(count)
}

// the user's own jar and sounds written as a modern texture pack folder to start a pack from
pub fn export(target: &Path) -> Result<usize, String>
{
    info!("Exporting the vanilla texture pack template to {}, this may take a moment", target.display());

    write(target, META, META_TEXT.as_bytes())?;
    if let Some(icon) = jar(ICON)
    {
        write(target, ICON, &icon)?;
    }

    let mut count: usize = 0;

    for (beta, modern) in legacy::files().filter(|(_, modern)| !modern.starts_with(ATLAS))
    {
        if let Some(bytes) = jar(beta)
        {
            write(target, &format!("{ASSETS}{modern}"), &bytes)?;
            count += 1;
        }
    }

    count += sheets(target)?;

    let resources: PathBuf = jvm::data_dir::root().join("resources");
    for folder in SOUND_FOLDERS
    {
        let root: PathBuf = resources.join(folder.m_beta);
        count += copyTree(&root, &root, target, folder.m_modern)?;
    }

    info!("Wrote {count} files to the vanilla texture pack template");
    Ok(count)
}
