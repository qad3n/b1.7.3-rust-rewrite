use crate::settings::GameSettings;
use std::rc::Rc;

pub struct ScaledResolution
{
    m_scaledWidth: i32,
    m_scaledHeight: i32,
    pub m_scaledWidthD: f64,
    pub m_scaledHeightD: f64,
    pub m_scaleFactor: i32,
}

impl ScaledResolution
{
    pub fn new(options: &Rc<GameSettings>, width: i32, height: i32) -> ScaledResolution
    {
        let mut scaleFactor: i32 = 1;
        let mut n: i32 = options.m_guiScale.get();
        if n == 0
        {
            n = 1000;
        }
        while scaleFactor < n && width / (scaleFactor + 1) >= 320 && height / (scaleFactor + 1) >= 240
        {
            scaleFactor += 1;
        }
        let scaledWidthD: f64 = width as f64 / scaleFactor as f64;
        let scaledHeightD: f64 = height as f64 / scaleFactor as f64;
        ScaledResolution {
            m_scaledWidth: scaledWidthD.ceil() as i32,
            m_scaledHeight: scaledHeightD.ceil() as i32,
            m_scaledWidthD: scaledWidthD,
            m_scaledHeightD: scaledHeightD,
            m_scaleFactor: scaleFactor,
        }
    }

    pub fn getScaledWidth(&self) -> i32 { self.m_scaledWidth }

    pub fn getScaledHeight(&self) -> i32 { self.m_scaledHeight }

    pub fn scaledWidthD(&self) -> f64 { self.m_scaledWidthD }

    pub fn scaledHeightD(&self) -> f64 { self.m_scaledHeightD }
}
