use crate::render::{GLAllocation, ImageBuffer, TextureFX, TexturePackBase, TexturePackList, ThreadDownloadImageData};
use crate::settings::GameSettings;
use jvm::hash_map::JHashMap;
use jvm::image::{BufferedImage, Color, Graphics, ImageIO};
use jvm::io::{InputStream, IoResult};
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::buffers::{ByteBuffer, IntBuffer};
use lwjgl::opengl::GL11;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const IMAGE_DATA_BYTES: i32 = 1048576;
const MISSING_TEXTURE_SIZE: i32 = 64;
const MIPMAP_LEVELS: i32 = 4;
const TILE: i32 = 16;

thread_local! {
    static USE_MIPMAPS: Cell<bool> = const { Cell::new(false) };
}

enum TextureKind<'a>
{
    Columns(&'a str),
    Clamp(&'a str),
    Blur(&'a str),
    Plain(&'a str),
}

fn textureKind(path: &str) -> TextureKind<'_>
{
    if let Some(rest) = path.strip_prefix("##")
    {
        TextureKind::Columns(rest)
    }
    else if let Some(rest) = path.strip_prefix("%clamp%")
    {
        TextureKind::Clamp(rest)
    }
    else if let Some(rest) = path.strip_prefix("%blur%")
    {
        TextureKind::Blur(rest)
    }
    else
    {
        TextureKind::Plain(path)
    }
}

fn averageColor(color1: i32, color2: i32) -> i32
{
    let i: i32 = (color1 & 0xFF000000u32 as i32) >> 24 & 0xFF;
    let j: i32 = (color2 & 0xFF000000u32 as i32) >> 24 & 0xFF;
    (i + j >> 1 << 24) + ((color1 & 16711422) + (color2 & 16711422) >> 1)
}

fn weightedAverageColor(color1: i32, color2: i32) -> i32
{
    let mut i: i32 = (color1 & 0xFF000000u32 as i32) >> 24 & 0xFF;
    let mut j: i32 = (color2 & 0xFF000000u32 as i32) >> 24 & 0xFF;
    let mut alpha: i32 = 255;
    if i + j == 0
    {
        i = 1;
        j = 1;
        alpha = 0;
    }
    let k: i32 = (color1 >> 16 & 0xFF) * i;
    let l: i32 = (color1 >> 8 & 0xFF) * i;
    let i1: i32 = (color1 & 0xFF) * i;
    let j1: i32 = (color2 >> 16 & 0xFF) * j;
    let k1: i32 = (color2 >> 8 & 0xFF) * j;
    let l1: i32 = (color2 & 0xFF) * j;
    let i2: i32 = (k + j1) / (i + j);
    let j2: i32 = (l + k1) / (i + j);
    let k2: i32 = (i1 + l1) / (i + j);
    alpha << 24 | i2 << 16 | j2 << 8 | k2
}

fn readColors(image: &BufferedImage) -> Vec<i32>
{
    let i: i32 = image.getWidth();
    let j: i32 = image.getHeight();
    let mut colors: Vec<i32> = vec![0; (i * j) as usize];
    image.getRGBRegion(0, 0, i, j, &mut colors, 0, i);
    colors
}

fn readTextureImage(stream: Option<Box<dyn InputStream>>) -> IoResult<Rc<BufferedImage>>
{
    let mut stream: Option<Box<dyn InputStream>> = stream;
    let image: Option<BufferedImage> = ImageIO::read(stream.as_mut())?;
    stream.orNpe().close()?;
    Ok(Rc::new(image.orNpe()))
}

fn unwrapImageByColumns(image: &BufferedImage) -> Rc<BufferedImage>
{
    let i: i32 = image.getWidth() / TILE;
    let bufferedimage: BufferedImage = BufferedImage::new(TILE, image.getHeight() * i, BufferedImage::TYPE_INT_ARGB);
    let graphics: Graphics<'_> = bufferedimage.getGraphics();
    for j in 0..i
    {
        graphics.drawImage(image, -j * TILE, j * image.getHeight());
    }
    graphics.dispose();
    Rc::new(bufferedimage)
}

pub struct RenderEngine
{
    m_textureMap: RefCell<JHashMap<JString, i32>>,
    m_colors: RefCell<JHashMap<JString, Rc<RefCell<Vec<i32>>>>>,
    m_textureNameToImageMap: RefCell<JHashMap<i32, Rc<BufferedImage>>>,
    m_singleIntBuffer: IntBuffer,
    m_imageData: ByteBuffer,
    m_textureList: RefCell<Vec<Rc<dyn TextureFX>>>,
    m_urlToImageDataMap: RefCell<JHashMap<JString, Rc<ThreadDownloadImageData>>>,
    m_options: Option<Rc<GameSettings>>,
    m_clampTexture: Cell<bool>,
    m_blurTexture: Cell<bool>,
    m_texturePack: Rc<TexturePackList>,
    m_missingTextureImage: Rc<BufferedImage>,
}

impl RenderEngine
{
    pub fn useMipmaps() -> bool { USE_MIPMAPS.with(Cell::get) }

    pub fn setUseMipmaps(value: bool) { USE_MIPMAPS.with(|flag| flag.set(value)); }

    pub fn new(texturePacks: &Rc<TexturePackList>, options: Option<Rc<GameSettings>>) -> Rc<RenderEngine>
    {
        let missing: BufferedImage =
            BufferedImage::new(MISSING_TEXTURE_SIZE, MISSING_TEXTURE_SIZE, BufferedImage::TYPE_INT_ARGB);
        let mut graphics: Graphics<'_> = missing.getGraphics();
        graphics.setColor(Color::WHITE);
        graphics.fillRect(0, 0, MISSING_TEXTURE_SIZE, MISSING_TEXTURE_SIZE);
        graphics.setColor(Color::BLACK);
        graphics.drawString("missingtex", 1, 10);
        graphics.dispose();
        Rc::new(RenderEngine {
            m_textureMap: RefCell::new(JHashMap::new()),
            m_colors: RefCell::new(JHashMap::new()),
            m_textureNameToImageMap: RefCell::new(JHashMap::new()),
            m_singleIntBuffer: GLAllocation::createDirectIntBuffer(1),
            m_imageData: GLAllocation::createDirectByteBuffer(IMAGE_DATA_BYTES),
            m_textureList: RefCell::new(Vec::new()),
            m_urlToImageDataMap: RefCell::new(JHashMap::new()),
            m_options: options,
            m_clampTexture: Cell::new(false),
            m_blurTexture: Cell::new(false),
            m_texturePack: Rc::clone(texturePacks),
            m_missingTextureImage: Rc::new(missing),
        })
    }

    fn anaglyph(&self) -> bool { self.m_options.as_ref().is_some_and(|options| options.m_anaglyph.get()) }

    fn loadImage(
        &self,
        pack: &Rc<dyn TexturePackBase>,
        path: &str,
        missingFallback: bool,
    ) -> IoResult<Rc<BufferedImage>>
    {
        match textureKind(path)
        {
            TextureKind::Columns(rest) => Ok(unwrapImageByColumns(&*readTextureImage(pack.getResourceAsStream(rest))?)),
            TextureKind::Clamp(rest) =>
            {
                self.m_clampTexture.set(true);
                readTextureImage(pack.getResourceAsStream(rest))
            }
            TextureKind::Blur(rest) =>
            {
                self.m_blurTexture.set(true);
                readTextureImage(pack.getResourceAsStream(rest))
            }
            TextureKind::Plain(rest) => match pack.getResourceAsStream(rest)
            {
                None if missingFallback => Ok(Rc::clone(&self.m_missingTextureImage)),
                stream => readTextureImage(stream),
            },
        }
    }

    fn resetFilter(&self, path: &str)
    {
        match textureKind(path)
        {
            TextureKind::Clamp(_) => self.m_clampTexture.set(false),
            TextureKind::Blur(_) => self.m_blurTexture.set(false),
            TextureKind::Columns(_) | TextureKind::Plain(_) =>
            {}
        }
    }

    fn resetFilters(&self)
    {
        self.m_clampTexture.set(false);
        self.m_blurTexture.set(false);
    }

    pub fn getColors(&self, path: &str) -> Rc<RefCell<Vec<i32>>>
    {
        if let Some(colors) = self.m_colors.borrow().get(&JString::from(path))
        {
            return colors;
        }
        let pack: Rc<dyn TexturePackBase> = self.m_texturePack.selected();
        let colors: Vec<i32> = match self.loadImage(&pack, path, true)
        {
            Ok(image) =>
            {
                let colors: Vec<i32> = readColors(&image);
                self.resetFilter(path);
                colors
            }
            Err(error) =>
            {
                error.printStackTrace();
                readColors(&self.m_missingTextureImage)
            }
        };
        let colors: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(colors));
        self.m_colors.borrow_mut().put(JString::from(path), Rc::clone(&colors));
        colors
    }

    fn generateTextureName(&self) -> i32
    {
        self.m_singleIntBuffer.clear();
        GLAllocation::generateTextureNames(&self.m_singleIntBuffer);
        self.m_singleIntBuffer.getAt(0)
    }

    pub fn getTexture(&self, path: &str) -> i32
    {
        if let Some(texture) = self.m_textureMap.borrow().get(&JString::from(path))
        {
            return texture;
        }
        let pack: Rc<dyn TexturePackBase> = self.m_texturePack.selected();
        let j: i32 = self.generateTextureName();
        let texture: i32 = match self.loadImage(&pack, path, true)
        {
            Ok(image) =>
            {
                self.setupTexture(&image, j);
                self.resetFilter(path);
                j
            }
            Err(error) =>
            {
                error.printStackTrace();
                GLAllocation::generateTextureNames(&self.m_singleIntBuffer);
                let i: i32 = self.m_singleIntBuffer.getAt(0);
                self.setupTexture(&self.m_missingTextureImage, i);
                i
            }
        };
        self.m_textureMap.borrow_mut().put(JString::from(path), texture);
        texture
    }

    pub fn allocateAndSetupTexture(&self, image: Rc<BufferedImage>) -> i32
    {
        let i: i32 = self.generateTextureName();
        self.setupTexture(&image, i);
        self.m_textureNameToImageMap.borrow_mut().put(i, image);
        i
    }

    fn applyTextureParameters(&self)
    {
        let target: i32 = GL11::GL_TEXTURE_2D;
        if Self::useMipmaps()
        {
            GL11::glTexParameteri(target, GL11::GL_TEXTURE_MIN_FILTER, GL11::GL_NEAREST_MIPMAP_LINEAR);
            GL11::glTexParameteri(target, GL11::GL_TEXTURE_MAG_FILTER, GL11::GL_NEAREST);
        }
        else
        {
            GL11::glTexParameteri(target, GL11::GL_TEXTURE_MIN_FILTER, GL11::GL_NEAREST);
            GL11::glTexParameteri(target, GL11::GL_TEXTURE_MAG_FILTER, GL11::GL_NEAREST);
        }
        if self.m_blurTexture.get()
        {
            GL11::glTexParameteri(target, GL11::GL_TEXTURE_MIN_FILTER, GL11::GL_LINEAR);
            GL11::glTexParameteri(target, GL11::GL_TEXTURE_MAG_FILTER, GL11::GL_LINEAR);
        }
        let wrap: i32 = if self.m_clampTexture.get()
        {
            GL11::GL_CLAMP
        }
        else
        {
            GL11::GL_REPEAT
        };
        GL11::glTexParameteri(target, GL11::GL_TEXTURE_WRAP_S, wrap);
        GL11::glTexParameteri(target, GL11::GL_TEXTURE_WRAP_T, wrap);
    }

    fn toBytes(&self, colors: &[i32], length: usize) -> Vec<i8>
    {
        let anaglyph: bool = self.anaglyph();
        let mut abyte: Vec<i8> = vec![0; length];
        for (i, &color) in colors.iter().enumerate()
        {
            let a: i32 = color >> 24 & 0xFF;
            let mut r: i32 = color >> 16 & 0xFF;
            let mut g: i32 = color >> 8 & 0xFF;
            let mut b: i32 = color & 0xFF;
            if anaglyph
            {
                let l1: i32 = (r * 30 + g * 59 + b * 11) / 100;
                let i2: i32 = (r * 30 + g * 70) / 100;
                let j2: i32 = (r * 30 + b * 70) / 100;
                r = l1;
                g = i2;
                b = j2;
            }
            abyte[i * 4] = r as i8;
            abyte[i * 4 + 1] = g as i8;
            abyte[i * 4 + 2] = b as i8;
            abyte[i * 4 + 3] = a as i8;
        }
        abyte
    }

    fn uploadBytes(&self, bytes: &[i8])
    {
        self.m_imageData.clear();
        self.m_imageData.putSlice(bytes);
        self.m_imageData.setPosition(0).setLimit(bytes.len() as i32);
    }

    fn downsample(&self, sourceWidth: i32, width: i32, height: i32, combine: fn(i32, i32) -> i32)
    {
        let data: &ByteBuffer = &self.m_imageData;
        for x in 0..width
        {
            for y in 0..height
            {
                let at = |dx: i32, dy: i32| -> i32 { data.getIntAt((x * 2 + dx + (y * 2 + dy) * sourceWidth) * 4) };
                let color: i32 = combine(combine(at(0, 0), at(1, 0)), combine(at(1, 1), at(0, 1)));
                data.putIntAt((x + y * width) * 4, color);
            }
        }
    }

    pub fn setupTexture(&self, image: &BufferedImage, id: i32)
    {
        GL11::glBindTexture(GL11::GL_TEXTURE_2D, id);
        self.applyTextureParameters();
        let i: i32 = image.getWidth();
        let j: i32 = image.getHeight();
        let colors: Vec<i32> = readColors(image);
        let abyte: Vec<i8> = self.toBytes(&colors, (i * j * 4) as usize);
        self.uploadBytes(&abyte);
        GL11::glTexImage2D(
            GL11::GL_TEXTURE_2D,
            0,
            GL11::GL_RGBA,
            i,
            j,
            0,
            GL11::GL_RGBA,
            GL11::GL_UNSIGNED_BYTE,
            Some(&self.m_imageData),
        );
        if Self::useMipmaps()
        {
            for j3 in 1..=MIPMAP_LEVELS
            {
                let k3: i32 = i >> (j3 - 1);
                let l3: i32 = i >> j3;
                let i4: i32 = j >> j3;
                self.downsample(k3, l3, i4, weightedAverageColor);
                GL11::glTexImage2D(
                    GL11::GL_TEXTURE_2D,
                    j3,
                    GL11::GL_RGBA,
                    l3,
                    i4,
                    0,
                    GL11::GL_RGBA,
                    GL11::GL_UNSIGNED_BYTE,
                    Some(&self.m_imageData),
                );
            }
        }
    }

    pub fn bind(&self, colors: &[i32], width: i32, height: i32, id: i32)
    {
        GL11::glBindTexture(GL11::GL_TEXTURE_2D, id);
        self.applyTextureParameters();
        let abyte: Vec<i8> = self.toBytes(colors, (width * height * 4) as usize);
        self.uploadBytes(&abyte);
        GL11::glTexSubImage2D(
            GL11::GL_TEXTURE_2D,
            0,
            0,
            0,
            width,
            height,
            GL11::GL_RGBA,
            GL11::GL_UNSIGNED_BYTE,
            &self.m_imageData,
        );
    }

    pub fn deleteTexture(&self, id: i32)
    {
        self.m_textureNameToImageMap.borrow_mut().remove(&id);
        self.m_singleIntBuffer.clear();
        self.m_singleIntBuffer.put(id);
        self.m_singleIntBuffer.flip();
        GL11::glDeleteTextures(&self.m_singleIntBuffer);
    }

    pub fn getTextureForDownloadableImage(&self, url: &str, backup: Option<&str>) -> i32
    {
        let data: Option<Rc<ThreadDownloadImageData>> = self.m_urlToImageDataMap.borrow().get(&JString::from(url));
        if let Some(data) = &data
            && let Some(image) = data.image()
            && !data.m_textureSetupComplete.get()
        {
            if data.m_textureName.get() < 0
            {
                data.m_textureName.set(self.allocateAndSetupTexture(image));
            }
            else
            {
                self.setupTexture(&image, data.m_textureName.get());
            }
            data.m_textureSetupComplete.set(true);
        }
        match data.filter(|data| data.m_textureName.get() >= 0)
        {
            Some(data) => data.m_textureName.get(),
            None => backup.map_or(-1, |backup| self.getTexture(backup)),
        }
    }

    pub fn obtainImageData(
        &self,
        url: &str,
        processor: Option<Box<dyn ImageBuffer>>,
    ) -> Option<Rc<ThreadDownloadImageData>>
    {
        let data: Option<Rc<ThreadDownloadImageData>> = self.m_urlToImageDataMap.borrow().get(&JString::from(url));
        match &data
        {
            None =>
            {
                let created: Rc<ThreadDownloadImageData> = Rc::new(ThreadDownloadImageData::new(url, processor));
                self.m_urlToImageDataMap.borrow_mut().put(JString::from(url), created);
            }
            Some(data) => data.m_referenceCount.set(data.m_referenceCount.get() + 1),
        }
        data
    }

    pub fn releaseImageData(&self, url: &str)
    {
        let Some(data) = self.m_urlToImageDataMap.borrow().get(&JString::from(url))
        else
        {
            return;
        };
        data.m_referenceCount.set(data.m_referenceCount.get() - 1);
        if data.m_referenceCount.get() == 0
        {
            if data.m_textureName.get() >= 0
            {
                self.deleteTexture(data.m_textureName.get());
            }
            self.m_urlToImageDataMap.borrow_mut().remove(&JString::from(url));
        }
    }

    pub fn registerTextureFX(&self, texture: Rc<dyn TextureFX>)
    {
        self.m_textureList.borrow_mut().push(Rc::clone(&texture));
        texture.onTick();
    }

    fn uploadFX(&self, texturefx: &dyn TextureFX)
    {
        let bytes: Vec<i8> = texturefx.fx().m_imageData.borrow().clone();
        self.uploadBytes(&bytes);
    }

    pub fn updateDynamicTextures(&self)
    {
        let textures: Vec<Rc<dyn TextureFX>> = self.m_textureList.borrow().clone();
        for texturefx in &textures
        {
            let fx = texturefx.fx();
            fx.m_anaglyphEnabled.set(self.m_options.as_ref().orNpe().m_anaglyph.get());
            texturefx.onTick();
            self.uploadFX(texturefx.as_ref());
            texturefx.bindImage(self);
            let iconIndex: i32 = fx.m_iconIndex.get();
            for j in 0..fx.m_tileSize.get()
            {
                for k in 0..fx.m_tileSize.get()
                {
                    GL11::glTexSubImage2D(
                        GL11::GL_TEXTURE_2D,
                        0,
                        iconIndex % 16 * TILE + j * TILE,
                        iconIndex / 16 * TILE + k * TILE,
                        TILE,
                        TILE,
                        GL11::GL_RGBA,
                        GL11::GL_UNSIGNED_BYTE,
                        &self.m_imageData,
                    );
                    if Self::useMipmaps()
                    {
                        for l in 1..=MIPMAP_LEVELS
                        {
                            let i1: i32 = TILE >> (l - 1);
                            let j1: i32 = TILE >> l;
                            self.downsample(i1, j1, j1, averageColor);
                            GL11::glTexSubImage2D(
                                GL11::GL_TEXTURE_2D,
                                l,
                                iconIndex % 16 * j1,
                                iconIndex / 16 * j1,
                                j1,
                                j1,
                                GL11::GL_RGBA,
                                GL11::GL_UNSIGNED_BYTE,
                                &self.m_imageData,
                            );
                        }
                    }
                }
            }
        }
        for texturefx1 in &textures
        {
            let textureId: i32 = texturefx1.fx().m_textureId.get();
            if textureId <= 0
            {
                continue;
            }
            self.uploadFX(texturefx1.as_ref());
            GL11::glBindTexture(GL11::GL_TEXTURE_2D, textureId);
            GL11::glTexSubImage2D(
                GL11::GL_TEXTURE_2D,
                0,
                0,
                0,
                TILE,
                TILE,
                GL11::GL_RGBA,
                GL11::GL_UNSIGNED_BYTE,
                &self.m_imageData,
            );
            if Self::useMipmaps()
            {
                for k3 in 1..=MIPMAP_LEVELS
                {
                    let l3: i32 = TILE >> (k3 - 1);
                    let i4: i32 = TILE >> k3;
                    self.downsample(l3, i4, i4, averageColor);
                    GL11::glTexSubImage2D(
                        GL11::GL_TEXTURE_2D,
                        k3,
                        0,
                        0,
                        i4,
                        i4,
                        GL11::GL_RGBA,
                        GL11::GL_UNSIGNED_BYTE,
                        &self.m_imageData,
                    );
                }
            }
        }
    }

    pub fn bindTexture(&self, id: i32)
    {
        if id >= 0
        {
            GL11::glBindTexture(GL11::GL_TEXTURE_2D, id);
        }
    }

    pub fn refreshTextures(&self)
    {
        let pack: Rc<dyn TexturePackBase> = self.m_texturePack.selected();
        let images: Vec<(i32, Rc<BufferedImage>)> = self.m_textureNameToImageMap.borrow().entries();
        for (id, image) in images
        {
            self.setupTexture(&image, id);
        }
        for data in self.m_urlToImageDataMap.borrow().values()
        {
            data.m_textureSetupComplete.set(false);
        }
        let textures: Vec<(JString, i32)> = self.m_textureMap.borrow().entries();
        for (path, id) in textures
        {
            match self.loadImage(&pack, &path.to_string(), false)
            {
                Ok(image) =>
                {
                    self.setupTexture(&image, id);
                    self.resetFilters();
                }
                Err(error) => error.printStackTrace(),
            }
        }
        let colors: Vec<(JString, Rc<RefCell<Vec<i32>>>)> = self.m_colors.borrow().entries();
        for (path, colors) in colors
        {
            match self.loadImage(&pack, &path.to_string(), false)
            {
                Ok(image) =>
                {
                    let mut colors = colors.borrow_mut();
                    image.getRGBRegion(0, 0, image.getWidth(), image.getHeight(), &mut colors, 0, image.getWidth());
                    self.resetFilters();
                }
                Err(error) => error.printStackTrace(),
            }
        }
    }
}
