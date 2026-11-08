use crate::io::{self, InputStream, IoResult};
use crate::throwable::{self, ThrowableClass};
use std::cell::{RefCell, RefMut};
use std::sync::OnceLock;

pub static IIO: ThrowableClass =
    ThrowableClass { m_name: "javax.imageio.IIOException", m_parent: Some(&throwable::IO) };

const READ_CHUNK: usize = 8192;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

// java.awt.Color as the ARGB int it carries
pub struct Color;

impl Color
{
    pub const WHITE: i32 = 0xFFFFFFFFu32 as i32;
    pub const BLACK: i32 = 0xFF000000u32 as i32;
}

// java.awt.image.BufferedImage held as ARGB ints, the layout TYPE_INT_ARGB stores and getRGB returns
pub struct BufferedImage
{
    m_width: i32,
    m_height: i32,
    m_type: i32,
    m_pixels: RefCell<Vec<i32>>,
}

impl BufferedImage
{
    pub const TYPE_INT_RGB: i32 = 1;
    pub const TYPE_INT_ARGB: i32 = 2;

    pub fn new(width: i32, height: i32, kind: i32) -> BufferedImage
    {
        if width <= 0 || height <= 0
        {
            throwable::throw(
                &throwable::ILLEGAL_ARGUMENT,
                &format!("Width ({width}) and height ({height}) cannot be <= 0"),
            );
        }
        BufferedImage {
            m_width: width,
            m_height: height,
            m_type: kind,
            m_pixels: RefCell::new(vec![0; (width * height) as usize]),
        }
    }

    fn fromPixels(width: i32, height: i32, pixels: Vec<i32>) -> BufferedImage
    {
        BufferedImage { m_width: width, m_height: height, m_type: Self::TYPE_INT_ARGB, m_pixels: RefCell::new(pixels) }
    }

    pub fn getWidth(&self) -> i32 { self.m_width }

    pub fn getHeight(&self) -> i32 { self.m_height }

    pub fn getType(&self) -> i32 { self.m_type }

    fn index(&self, x: i32, y: i32) -> usize
    {
        if x < 0 || y < 0 || x >= self.m_width || y >= self.m_height
        {
            throwable::throw(&throwable::ARRAY_INDEX, "Coordinate out of bounds!");
        }
        (x + y * self.m_width) as usize
    }

    pub fn getRGB(&self, x: i32, y: i32) -> i32 { self.m_pixels.borrow()[self.index(x, y)] }

    pub fn setRGB(&self, x: i32, y: i32, rgb: i32)
    {
        let index: usize = self.index(x, y);
        self.m_pixels.borrow_mut()[index] = rgb;
    }

    // getRGB(int, int, int, int, int[], int, int)
    pub fn getRGBRegion(
        &self,
        startX: i32,
        startY: i32,
        width: i32,
        height: i32,
        rgbArray: &mut [i32],
        offset: i32,
        scansize: i32,
    )
    {
        for y in 0..height
        {
            for x in 0..width
            {
                rgbArray[(offset + y * scansize + x) as usize] = self.getRGB(startX + x, startY + y);
            }
        }
    }

    // the int array behind getRaster().getDataBuffer()
    pub fn rasterData(&self) -> RefMut<'_, Vec<i32>> { self.m_pixels.borrow_mut() }

    pub fn getGraphics(&self) -> Graphics<'_> { Graphics { m_target: self, m_color: Color::BLACK } }
}

// the java2d alpha tables of AlphaMath.c the SrcOver blits round through
struct AlphaTables
{
    m_mul8: Vec<[u8; 256]>,
    m_div8: Vec<[u8; 256]>,
}

fn alphaTables() -> &'static AlphaTables
{
    static TABLES: OnceLock<AlphaTables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut mul8: Vec<[u8; 256]> = vec![[0; 256]; 256];
        let mut div8: Vec<[u8; 256]> = vec![[0; 256]; 256];
        for i in 1..256u32
        {
            let inc: u32 = (i << 16) + (i << 8) + i;
            let mut val: u32 = inc + (1 << 23);
            for j in 1..256usize
            {
                mul8[i as usize][j] = (val >> 24) as u8;
                val = val.wrapping_add(inc);
            }
        }
        for i in 1..256u32
        {
            let inc: u32 = ((0xFFu32 << 24) + i / 2) / i;
            let mut val: u32 = 1 << 23;
            for j in 0..256u32
            {
                div8[i as usize][j as usize] = if j < i { (val >> 24) as u8 } else { 255 };
                val = val.wrapping_add(inc);
            }
        }
        AlphaTables { m_mul8: mul8, m_div8: div8 }
    })
}

fn channel(pixel: u32, shift: u32) -> u32 { (pixel >> shift) & 0xFF }

// IntArgb onto IntArgb with AlphaComposite.SrcOver
fn srcOver(src: i32, dst: i32) -> i32
{
    let src: u32 = src as u32;
    let dst: u32 = dst as u32;
    let srcA: u32 = channel(src, 24);
    if srcA == 0
    {
        return dst as i32;
    }
    if srcA == 0xFF
    {
        return src as i32;
    }
    let tables: &AlphaTables = alphaTables();
    let mul8 = |a: u32, v: u32| -> u32 { tables.m_mul8[a as usize][v as usize] as u32 };
    let dstF: u32 = 0xFF - srcA;
    let dstA: u32 = mul8(dstF, channel(dst, 24));
    let mut res: [u32; 3] = [0; 3];
    for (i, shift) in [16u32, 8, 0].into_iter().enumerate()
    {
        res[i] = mul8(srcA, channel(src, shift)) + mul8(dstA, channel(dst, shift));
    }
    let resA: u32 = srcA + dstA;
    if resA < 0xFF
    {
        for value in res.iter_mut()
        {
            *value = tables.m_div8[resA as usize][*value as usize] as u32;
        }
    }
    ((resA << 24) | (res[0] << 16) | (res[1] << 8) | res[2]) as i32
}

// java.awt.Graphics over a BufferedImage, as far as the game draws with it
pub struct Graphics<'a>
{
    m_target: &'a BufferedImage,
    m_color: i32,
}

impl Graphics<'_>
{
    pub fn setColor(&mut self, color: i32) { self.m_color = color; }

    pub fn fillRect(&self, x: i32, y: i32, width: i32, height: i32)
    {
        let target: &BufferedImage = self.m_target;
        for py in y.max(0)..(y + height).min(target.m_height)
        {
            for px in x.max(0)..(x + width).min(target.m_width)
            {
                target.setRGB(px, py, srcOver(self.m_color, target.getRGB(px, py)));
            }
        }
    }

    // the stand-in has no fonts, so text is left out of the image
    pub fn drawString(&self, _text: &str, _x: i32, _y: i32) {}

    pub fn drawImage(&self, image: &BufferedImage, x: i32, y: i32) -> bool
    {
        let target: &BufferedImage = self.m_target;
        for sy in 0..image.m_height
        {
            let ty: i32 = y + sy;
            if ty < 0 || ty >= target.m_height
            {
                continue;
            }
            for sx in 0..image.m_width
            {
                let tx: i32 = x + sx;
                if tx < 0 || tx >= target.m_width
                {
                    continue;
                }
                let src: i32 = image.getRGB(sx, sy);
                target.setRGB(tx, ty, srcOver(src, target.getRGB(tx, ty)));
            }
        }
        true
    }

    pub fn dispose(&self) {}
}

pub struct ImageIO;

impl ImageIO
{
    // null when no reader takes the stream, IIOException when the png reader fails, the stream stays open
    pub fn read<S: InputStream>(input: Option<&mut S>) -> IoResult<Option<BufferedImage>>
    {
        let Some(input) = input
        else
        {
            throwable::throw(&throwable::ILLEGAL_ARGUMENT, "input == null!");
        };
        let mut bytes: Vec<u8> = Vec::new();
        let mut chunk: [u8; READ_CHUNK] = [0; READ_CHUNK];
        loop
        {
            let count: i32 = input.readInto(&mut chunk, 0, READ_CHUNK as i32)?;
            if count < 0
            {
                break;
            }
            bytes.extend_from_slice(&chunk[..count as usize]);
        }
        if !bytes.starts_with(&PNG_SIGNATURE)
        {
            return Ok(None);
        }
        Self::decodePng(&bytes)
            .map(Some)
            .map_err(|error| io::ioError(&IIO, &format!("Error reading PNG image data: {error}")))
    }

    // the game ships 8 bit rgba and palette pngs, expanded these give the ARGB ImageIO's getRGB reports
    fn decodePng(bytes: &[u8]) -> Result<BufferedImage, png::DecodingError>
    {
        let mut decoder: png::Decoder<&[u8]> = png::Decoder::new(bytes);
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader: png::Reader<&[u8]> = decoder.read_info()?;
        let mut buffer: Vec<u8> = vec![0; reader.output_buffer_size()];
        let frame: png::OutputInfo = reader.next_frame(&mut buffer)?;
        let channels: usize = frame.color_type.samples();
        let pixels: Vec<i32> = buffer[..frame.buffer_size()]
            .chunks_exact(channels)
            .map(|sample| {
                let (r, g, b, a): (u8, u8, u8, u8) = match sample.len()
                {
                    1 => (sample[0], sample[0], sample[0], 0xFF),
                    2 => (sample[0], sample[0], sample[0], sample[1]),
                    3 => (sample[0], sample[1], sample[2], 0xFF),
                    _ => (sample[0], sample[1], sample[2], sample[3]),
                };
                i32::from_be_bytes([a, r, g, b])
            })
            .collect();
        Ok(BufferedImage::fromPixels(frame.width as i32, frame.height as i32, pixels))
    }

    // the png writer, false for any other format name like a missing ImageWriter
    pub fn write(image: &BufferedImage, formatName: &str, output: &std::path::Path) -> IoResult<bool>
    {
        if !formatName.eq_ignore_ascii_case("png")
        {
            return Ok(false);
        }
        let file: std::fs::File = std::fs::File::create(output).map_err(|error| io::fromStd(&error))?;
        Self::encodePng(image, std::io::BufWriter::new(file))
            .map_err(|error| io::ioError(&IIO, &format!("Error writing PNG image data: {error}")))?;
        Ok(true)
    }

    fn encodePng(image: &BufferedImage, output: impl std::io::Write) -> Result<(), png::EncodingError>
    {
        let alpha: bool = image.getType() == BufferedImage::TYPE_INT_ARGB;
        let mut encoder: png::Encoder<_> = png::Encoder::new(output, image.getWidth() as u32, image.getHeight() as u32);
        encoder.set_color(
            if alpha
            {
                png::ColorType::Rgba
            }
            else
            {
                png::ColorType::Rgb
            },
        );
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer: png::Writer<_> = encoder.write_header()?;
        let data: Vec<u8> = image
            .m_pixels
            .borrow()
            .iter()
            .flat_map(|pixel| {
                let [a, r, g, b]: [u8; 4] = pixel.to_be_bytes();
                if alpha
                {
                    vec![r, g, b, a]
                }
                else
                {
                    vec![r, g, b]
                }
            })
            .collect();
        writer.write_image_data(&data)?;
        writer.finish()
    }
}
