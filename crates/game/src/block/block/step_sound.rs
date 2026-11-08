use std::rc::Rc;

pub struct StepSound
{
    pub m_key: &'static str,
    pub m_volume: f32,
    pub m_pitch: f32,
    m_dir: Option<&'static str>,
}

pub struct StepSounds
{
    pub m_powder: Rc<StepSound>,
    pub m_wood: Rc<StepSound>,
    pub m_gravel: Rc<StepSound>,
    pub m_grass: Rc<StepSound>,
    pub m_stone: Rc<StepSound>,
    pub m_metal: Rc<StepSound>,
    pub m_glass: Rc<StepSound>,
    pub m_cloth: Rc<StepSound>,
    pub m_sand: Rc<StepSound>,
}

thread_local! {
    static SOUNDS: StepSounds = StepSounds {
        m_powder: StepSound::new("stone", 1.0, 1.0, None),
        m_wood: StepSound::new("wood", 1.0, 1.0, None),
        m_gravel: StepSound::new("gravel", 1.0, 1.0, None),
        m_grass: StepSound::new("grass", 1.0, 1.0, None),
        m_stone: StepSound::new("stone", 1.0, 1.0, None),
        m_metal: StepSound::new("stone", 1.0, 1.5, None),
        m_glass: StepSound::new("stone", 1.0, 1.0, Some("random.glass")),
        m_cloth: StepSound::new("cloth", 1.0, 1.0, None),
        m_sand: StepSound::new("sand", 1.0, 1.0, Some("step.gravel")),
    };
}

impl StepSound
{
    fn new(key: &'static str, volume: f32, pitch: f32, dir: Option<&'static str>) -> Rc<Self>
    {
        Rc::new(Self { m_key: key, m_volume: volume, m_pitch: pitch, m_dir: dir })
    }

    pub fn sounds<R>(body: impl FnOnce(&StepSounds) -> R) -> R { SOUNDS.with(body) }

    pub fn powder() -> Rc<Self> { Self::sounds(|sounds| Rc::clone(&sounds.m_powder)) }

    pub fn getVolume(&self) -> f32 { self.m_volume }

    pub fn getPitch(&self) -> f32 { self.m_pitch }

    pub fn stepSoundDir(&self) -> String
    {
        match self.m_dir
        {
            Some(dir) => dir.to_owned(),
            None => format!("step.{}", self.m_key),
        }
    }

    pub fn getStepping(&self) -> String { format!("step.{}", self.m_key) }
}
