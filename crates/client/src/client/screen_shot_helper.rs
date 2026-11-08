use crate::gui::calendar::{DateFields, dateFields};
use jvm::image::{BufferedImage, ImageIO};
use jvm::throwable::{EXCEPTION, Throwable, catchThrowable};
use lwjgl::BufferUtils;
use lwjgl::buffers::ByteBuffer;
use lwjgl::opengl::GL11;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

thread_local! {
    static BUFFER: RefCell<Option<Rc<ByteBuffer>>> = const { RefCell::new(None) };
    static PIXEL_DATA: RefCell<Vec<i8>> = const { RefCell::new(Vec::new()) };
    static IMAGE_DATA: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
}

fn formatDate(millis: i64) -> String
{
    let fields: DateFields = dateFields(millis);
    format!(
        "{:04}-{:02}-{:02}_{:02}.{:02}.{:02}",
        fields.m_year, fields.m_month, fields.m_day, fields.m_hour, fields.m_minute, fields.m_second
    )
}

fn fileName(path: &Path) -> String
{
    path.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

pub struct ScreenShotHelper;

impl ScreenShotHelper
{
    pub fn saveScreenshot(gameDir: Option<&Path>, width: i32, height: i32) -> String
    {
        match catchThrowable(&EXCEPTION, || ScreenShotHelper::save(gameDir, width, height))
        {
            Ok(Ok(name)) => format!("Saved screenshot as {name}"),
            Ok(Err(exception)) | Err(exception) =>
            {
                exception.printStackTrace();
                format!("Failed to save: {exception}")
            }
        }
    }

    fn save(gameDir: Option<&Path>, width: i32, height: i32) -> Result<String, Throwable>
    {
        let file1: PathBuf = gameDir.map_or_else(|| PathBuf::from("screenshots"), |dir| dir.join("screenshots"));
        let _ = std::fs::create_dir(&file1);
        let buffer: Rc<ByteBuffer> = BUFFER.with(|cached| {
            let mut cached = cached.borrow_mut();
            if cached.as_ref().is_none_or(|buffer| buffer.capacity() < width * height)
            {
                *cached = Some(Rc::new(BufferUtils::createByteBuffer(width * height * 3)));
            }
            cached.clone().expect("the buffer was just allocated")
        });
        if IMAGE_DATA.with_borrow(|imageData| (imageData.len() as i32) < width * height * 3)
        {
            PIXEL_DATA.set(vec![0; (width * height * 3) as usize]);
            IMAGE_DATA.set(vec![0; (width * height) as usize]);
        }
        let mut pixelData: Vec<i8> = PIXEL_DATA.take();
        let mut imageData: Vec<i32> = IMAGE_DATA.take();
        let result: Result<String, Throwable> =
            ScreenShotHelper::write(&buffer, &mut pixelData, &mut imageData, &file1, width, height);
        PIXEL_DATA.set(pixelData);
        IMAGE_DATA.set(imageData);
        result
    }

    fn write(
        buffer: &ByteBuffer,
        pixelData: &mut [i8],
        imageData: &mut [i32],
        file1: &Path,
        width: i32,
        height: i32,
    ) -> Result<String, Throwable>
    {
        GL11::glPixelStorei(3333, 1);
        GL11::glPixelStorei(3317, 1);
        buffer.clear();
        GL11::glReadPixels(0, 0, width, height, 6407, 5121, buffer);
        buffer.clear();
        let s: String = formatDate(jvm::system::currentTimeMillis());
        let mut i: i32 = 1;
        let mut file2: PathBuf;
        loop
        {
            let suffix: String = if i == 1
            {
                String::new()
            }
            else
            {
                format!("_{i}")
            };
            file2 = file1.join(format!("{s}{suffix}.png"));
            if !file2.exists()
            {
                break;
            }
            i += 1;
        }
        buffer.getInto(pixelData);
        for j in 0..width
        {
            for k in 0..height
            {
                let l: i32 = j + (height - k - 1) * width;
                let i1: i32 = pixelData[(l * 3) as usize] as i32 & 255;
                let j1: i32 = pixelData[(l * 3 + 1) as usize] as i32 & 255;
                let k1: i32 = pixelData[(l * 3 + 2) as usize] as i32 & 255;
                imageData[(j + k * width) as usize] = 0xFF000000u32 as i32 | i1 << 16 | j1 << 8 | k1;
            }
        }
        let bufferedimage: BufferedImage = BufferedImage::new(width, height, BufferedImage::TYPE_INT_RGB);
        bufferedimage.rasterData().copy_from_slice(&imageData[..(width * height) as usize]);
        ImageIO::write(&bufferedimage, "png", &file2)?;
        Ok(fileName(&file2))
    }
}
