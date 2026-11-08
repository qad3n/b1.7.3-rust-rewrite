use crate::render::ImageBuffer;
use jvm::field::Field;
use jvm::image::{BufferedImage, ImageIO};
use jvm::io::ByteArrayInputStream;
use jvm::net::openStream;
use jvm::throwable::{self, Throwable};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct ThreadDownloadImageData
{
    m_downloaded: Arc<Mutex<Option<BufferedImage>>>,
    m_image: Field<Option<Rc<BufferedImage>>>,
    pub m_referenceCount: Cell<i32>,
    pub m_textureName: Cell<i32>,
    pub m_textureSetupComplete: Cell<bool>,
}

fn download(url: &str, processor: Option<Box<dyn ImageBuffer>>) -> Result<Option<BufferedImage>, Throwable>
{
    let bytes: Vec<u8> = openStream(url)?;
    let mut input: ByteArrayInputStream = ByteArrayInputStream::new(bytes);
    let image: Option<BufferedImage> = ImageIO::read(Some(&mut input))?;
    Ok(match processor
    {
        Some(mut processor) => processor.parseUserSkin(image),
        None => image,
    })
}

impl ThreadDownloadImageData
{
    pub fn new(url: &str, processor: Option<Box<dyn ImageBuffer>>) -> ThreadDownloadImageData
    {
        let downloaded: Arc<Mutex<Option<BufferedImage>>> = Arc::new(Mutex::new(None));
        let target: Arc<Mutex<Option<BufferedImage>>> = Arc::clone(&downloaded);
        let url: String = url.to_owned();
        let started: std::io::Result<std::thread::JoinHandle<()>> =
            jvm::thread::start(jvm::thread::nextName(), move || {
                let result: Result<Result<Option<BufferedImage>, Throwable>, Throwable> =
                    throwable::catchThrowable(&throwable::EXCEPTION, || download(&url, processor));
                match result.and_then(|inner| inner)
                {
                    Ok(image) => *target.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = image,
                    Err(error) => error.printStackTrace(),
                }
            });
        if let Err(error) = started
        {
            jvm::io::fromStd(&error).printStackTrace();
        }
        ThreadDownloadImageData {
            m_downloaded: downloaded,
            m_image: Field::new(None),
            m_referenceCount: Cell::new(1),
            m_textureName: Cell::new(-1),
            m_textureSetupComplete: Cell::new(false),
        }
    }

    pub fn image(&self) -> Option<Rc<BufferedImage>>
    {
        if self.m_image.with(Option::is_none)
        {
            let finished: Option<BufferedImage> =
                self.m_downloaded.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).take();
            if let Some(image) = finished
            {
                self.m_image.set(Some(Rc::new(image)));
            }
        }
        self.m_image.get()
    }
}
