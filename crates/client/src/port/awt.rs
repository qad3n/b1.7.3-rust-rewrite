pub trait Component
{
    fn getWidth(&self) -> i32;

    fn getHeight(&self) -> i32;
}

pub fn hsbToRgb(hue: f32, saturation: f32, brightness: f32) -> i32
{
    let channel = |value: f32| (value * 255.0 + 0.5) as i32;
    let (r, g, b): (i32, i32, i32) = if saturation == 0.0
    {
        (channel(brightness), channel(brightness), channel(brightness))
    }
    else
    {
        let h: f32 = (hue - hue.floor()) * 6.0;
        let f: f32 = h - h.floor();
        let p: f32 = brightness * (1.0 - saturation);
        let q: f32 = brightness * (1.0 - saturation * f);
        let t: f32 = brightness * (1.0 - saturation * (1.0 - f));
        match h as i32
        {
            0 => (channel(brightness), channel(t), channel(p)),
            1 => (channel(q), channel(brightness), channel(p)),
            2 => (channel(p), channel(brightness), channel(t)),
            3 => (channel(p), channel(q), channel(brightness)),
            4 => (channel(t), channel(p), channel(brightness)),
            5 => (channel(brightness), channel(p), channel(q)),
            _ => (0, 0, 0),
        }
    };
    0xFF000000u32 as i32 | r << 16 | g << 8 | b
}
