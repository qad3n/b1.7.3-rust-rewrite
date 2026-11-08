use crate::client::Minecraft;
use crate::render::texture_pack_default::bindThumbnail;
use crate::render::{TexturePackBase, TexturePackFields};
use jvm::field::Field;
use jvm::image::{BufferedImage, ImageIO};
use jvm::io::{ByteArrayInputStream, InputStream, IoResult};
use jvm::throwable::{OrNpe, Throwable};
use jvm::zip::ZIP;
use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use zip::ZipArchive;

const DESCRIPTION_LIMIT: usize = 34;

type ZipFile = ZipArchive<File>;

pub struct TexturePackCustom
{
    m_pack: TexturePackFields,
    m_texturePackZipFile: RefCell<Option<ZipFile>>,
    m_texturePackName: Cell<i32>,
    m_texturePackThumbnail: Field<Option<Rc<BufferedImage>>>,
    m_texturePackFile: PathBuf,
}

fn openZip(path: &Path) -> Result<ZipFile, Throwable>
{
    let file: File = File::open(path).map_err(|error| jvm::io::fromStd(&error))?;
    ZipArchive::new(file).map_err(|_| Throwable::new(&ZIP, Some("error in opening zip file".to_owned())))
}

fn readEntry(zip: &mut ZipFile, name: &str) -> Option<Vec<u8>>
{
    let mut entry = zip.by_name(name).ok()?;
    let mut bytes: Vec<u8> = Vec::new();
    entry.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

fn truncateString(line: Option<String>) -> Option<String>
{
    line.map(|line| {
        let units: Vec<u16> = line.encode_utf16().collect();
        if units.len() > DESCRIPTION_LIMIT
        {
            String::from_utf16_lossy(&units[..DESCRIPTION_LIMIT])
        }
        else
        {
            line
        }
    })
}

fn readLines(text: &str) -> Vec<String>
{
    let text: String = text.replace("\r\n", "\n");
    let mut lines: Vec<String> = text.split(['\n', '\r']).map(str::to_owned).collect();
    if text.is_empty() || text.ends_with(['\n', '\r'])
    {
        lines.pop();
    }
    lines
}

impl TexturePackCustom
{
    pub fn new(file: &Path) -> TexturePackCustom
    {
        let pack: TexturePackFields = TexturePackFields::default();
        pack.m_texturePackFileName.set(file.file_name().map(|name| name.to_string_lossy().into_owned()));
        TexturePackCustom {
            m_pack: pack,
            m_texturePackZipFile: RefCell::new(None),
            m_texturePackName: Cell::new(-1),
            m_texturePackThumbnail: Field::new(None),
            m_texturePackFile: file.to_path_buf(),
        }
    }
}

impl TexturePackBase for TexturePackCustom
{
    fn pack(&self) -> &TexturePackFields { &self.m_pack }

    fn load(&self, _minecraft: &Rc<Minecraft>) -> IoResult<()>
    {
        let mut zipFile: ZipFile = match openZip(&self.m_texturePackFile)
        {
            Ok(zipFile) => zipFile,
            Err(exception) =>
            {
                exception.printStackTrace();
                return Ok(());
            }
        };
        if let Some(text) = readEntry(&mut zipFile, "pack.txt")
        {
            let mut lines = readLines(&String::from_utf8_lossy(&text)).into_iter();
            self.m_pack.m_firstDescriptionLine.set(truncateString(lines.next()));
            self.m_pack.m_secondDescriptionLine.set(truncateString(lines.next()));
        }
        if let Some(bytes) = readEntry(&mut zipFile, "pack.png")
        {
            let mut input: ByteArrayInputStream = ByteArrayInputStream::new(bytes);
            if let Ok(image) = ImageIO::read(Some(&mut input))
            {
                self.m_texturePackThumbnail.set(image.map(Rc::new));
            }
        }
        Ok(())
    }

    fn unload(&self, minecraft: &Rc<Minecraft>)
    {
        if self.m_texturePackThumbnail.with(Option::is_some)
        {
            minecraft.m_renderEngine.get().orNpe().deleteTexture(self.m_texturePackName.get());
        }
        self.closeTexturePackFile();
    }

    fn bindThumbnailTexture(&self, minecraft: &Rc<Minecraft>)
    {
        bindThumbnail(minecraft, self.m_texturePackThumbnail.get().as_ref(), &self.m_texturePackName);
    }

    fn open(&self)
    {
        if let Ok(zipFile) = openZip(&self.m_texturePackFile)
        {
            *self.m_texturePackZipFile.borrow_mut() = Some(zipFile);
        }
    }

    fn closeTexturePackFile(&self) { *self.m_texturePackZipFile.borrow_mut() = None; }

    fn getResourceAsStream(&self, path: &str) -> Option<Box<dyn InputStream>>
    {
        let bytes: Option<Vec<u8>> =
            self.m_texturePackZipFile.borrow_mut().as_mut().and_then(|zip| readEntry(zip, path.get(1..)?));
        match bytes
        {
            Some(bytes) => Some(Box::new(ByteArrayInputStream::new(bytes))),
            None => crate::assets::getResourceAsStream(path).map(|stream| Box::new(stream) as Box<dyn InputStream>),
        }
    }
}
