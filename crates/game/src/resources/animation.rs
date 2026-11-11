use jvm::image::BufferedImage;
use jvm::warning;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;

// a changed frame to upload at its place on the sheet
pub struct Frame<'a>
{
    pub m_x: i32,
    pub m_y: i32,
    pub m_size: i32,
    pub m_rgba: &'a [u8],
}

// one frame shown for some ticks
struct Step
{
    m_frame: usize,
    m_ticks: u32,
}

// a texture pack sprite with frames that plays into its place on a sheet
pub(crate) struct Animation
{
    m_x: i32,
    m_y: i32,
    m_size: i32,
    // rgba bytes already scaled to the sheet tile so a tick only uploads
    m_frames: Vec<Vec<u8>>,
    m_steps: Vec<Step>,
    m_length: u32,
    m_shown: Option<usize>,
}

#[derive(Default)]
struct Sheet
{
    m_ticks: u32,
    // sheet slots a texture pack painted so the beta texture effects leave them alone
    m_painted: Vec<usize>,
    m_animations: Vec<Animation>,
}

thread_local!
{
    static SHEETS: RefCell<HashMap<String, Sheet>> = RefCell::new(HashMap::new());
}

fn frametime(value: Option<&Value>) -> u32
{
    value.and_then(Value::as_u64).map_or(1, |ticks| ticks.clamp(1, u32::MAX as u64) as u32)
}

// the modern animation section with frametime and a frames list of indices or index and time objects
fn steps(meta: Option<&[u8]>, frames: usize) -> Vec<Step>
{
    let parsed: Option<Value> = meta.and_then(|bytes| match serde_json::from_slice::<Value>(bytes)
    {
        Ok(value) => Some(value),

        Err(error) =>
        {
            warning!("Ignoring a texture pack animation file: {error}");
            None
        }
    });
    let animation: Option<&Value> = parsed.as_ref().and_then(|value| value.get("animation"));
    let time: u32 = frametime(animation.and_then(|animation| animation.get("frametime")));
    let listed: Option<&Vec<Value>> = animation.and_then(|animation| animation.get("frames")).and_then(Value::as_array);

    let Some(listed) = listed
    else
    {
        return (0..frames).map(|frame| Step { m_frame: frame, m_ticks: time }).collect();
    };

    listed
        .iter()
        .filter_map(|entry| match entry
        {
            Value::Number(index) => Some(Step { m_frame: index.as_u64()? as usize, m_ticks: time }),
            Value::Object(step) => Some(Step
            {
                m_frame: step.get("index")?.as_u64()? as usize,
                m_ticks: step.get("time").map_or(time, |ticks| frametime(Some(ticks))),
            }),
            _ => None,
        })
        .filter(|step| step.m_frame < frames)
        .collect()
}

// one frame of a strip as rgba bytes at the target size
fn frame(strip: &BufferedImage, index: i32, size: i32) -> Vec<u8>
{
    let width: i32 = strip.getWidth();
    let mut rgba: Vec<u8> = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size
    {
        for x in 0..size
        {
            let argb: u32 = strip.getRGB(x * width / size, index * width + y * width / size) as u32;
            rgba.extend_from_slice(&[(argb >> 16) as u8, (argb >> 8) as u8, argb as u8, (argb >> 24) as u8]);
        }
    }

    rgba
}

impl Animation
{
    // none for a still sprite or a strip whose frames list names nothing
    pub(crate) fn new(strip: &BufferedImage, meta: Option<&[u8]>, place: (i32, i32, i32)) -> Option<Self>
    {
        let frames: i32 = strip.getHeight() / strip.getWidth();
        if frames < 2
        {
            return None;
        }

        let steps: Vec<Step> = steps(meta, frames as usize);
        let length: u32 = steps.iter().map(|step| step.m_ticks).sum();
        if length == 0
        {
            return None;
        }

        let (x, y, size): (i32, i32, i32) = place;
        let rendered: Vec<Vec<u8>> = (0..frames).map(|index| frame(strip, index, size)).collect();
        Some(Self { m_x: x, m_y: y, m_size: size, m_frames: rendered, m_steps: steps, m_length: length, m_shown: None })
    }

    fn current(&self, ticks: u32) -> usize
    {
        let mut left: u32 = ticks % self.m_length;

        for step in &self.m_steps
        {
            if left < step.m_ticks
            {
                return step.m_frame;
            }

            left -= step.m_ticks;
        }

        self.m_steps[0].m_frame
    }
}

// the result of the last sheet overlay replaces what the sheet had
pub(crate) fn set(path: &str, painted: Vec<usize>, animations: Vec<Animation>)
{
    SHEETS.with(|sheets|
    {
        sheets.borrow_mut().insert(path.to_owned(), Sheet { m_ticks: 0, m_painted: painted, m_animations: animations })
    });
}

// true when a texture pack painted the slot of a beta sheet such as terrain.png
pub fn painted(path: &str, slot: usize) -> bool
{
    let path: &str = path.trim_start_matches('/');
    SHEETS.with(|sheets| sheets.borrow().get(path).is_some_and(|sheet| sheet.m_painted.contains(&slot)))
}

// one tick of every animation on a sheet and the upload of each frame that changed
pub fn animate(path: &str, upload: &mut dyn FnMut(Frame<'_>))
{
    let path: &str = path.trim_start_matches('/');
    SHEETS.with(|sheets|
    {
        let mut sheets: std::cell::RefMut<HashMap<String, Sheet>> = sheets.borrow_mut();
        let Some(sheet) = sheets.get_mut(path)
        else
        {
            return;
        };

        sheet.m_ticks = sheet.m_ticks.wrapping_add(1);

        for animation in &mut sheet.m_animations
        {
            let frame: usize = animation.current(sheet.m_ticks);
            if animation.m_shown == Some(frame)
            {
                continue;
            }

            animation.m_shown = Some(frame);
            upload(Frame
            {
                m_x: animation.m_x,
                m_y: animation.m_y,
                m_size: animation.m_size,
                m_rgba: &animation.m_frames[frame],
            });
        }
    });
}
