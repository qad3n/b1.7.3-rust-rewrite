#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumOptions
{
    Music,
    Sound,
    InvertMouse,
    Sensitivity,
    RenderDistance,
    ViewBobbing,
    Anaglyph,
    AdvancedOpengl,
    FramerateLimit,
    Difficulty,
    Graphics,
    AmbientOcclusion,
    GuiScale,
}

struct OptionRow
{
    m_option: EnumOptions,
    m_enumString: &'static str,
    m_enumFloat: bool,
    m_enumBoolean: bool,
}

const fn row(option: EnumOptions, name: &'static str, isFloat: bool, isBoolean: bool) -> OptionRow
{
    OptionRow { m_option: option, m_enumString: name, m_enumFloat: isFloat, m_enumBoolean: isBoolean }
}

const OPTIONS: [OptionRow; 13] = [
    row(EnumOptions::Music, "options.music", true, false),
    row(EnumOptions::Sound, "options.sound", true, false),
    row(EnumOptions::InvertMouse, "options.invertMouse", false, true),
    row(EnumOptions::Sensitivity, "options.sensitivity", true, false),
    row(EnumOptions::RenderDistance, "options.renderDistance", false, false),
    row(EnumOptions::ViewBobbing, "options.viewBobbing", false, true),
    row(EnumOptions::Anaglyph, "options.anaglyph", false, true),
    row(EnumOptions::AdvancedOpengl, "options.advancedOpengl", false, true),
    row(EnumOptions::FramerateLimit, "options.framerateLimit", false, false),
    row(EnumOptions::Difficulty, "options.difficulty", false, false),
    row(EnumOptions::Graphics, "options.graphics", false, false),
    row(EnumOptions::AmbientOcclusion, "options.ao", false, true),
    row(EnumOptions::GuiScale, "options.guiScale", false, false),
];

impl EnumOptions
{
    fn row(self) -> &'static OptionRow { &OPTIONS[self as usize] }

    pub fn values() -> impl Iterator<Item = EnumOptions> { OPTIONS.iter().map(|row| row.m_option) }

    pub fn getEnumOptions(id: i32) -> Option<EnumOptions>
    {
        EnumOptions::values().find(|option| option.returnEnumOrdinal() == id)
    }

    pub fn getEnumFloat(self) -> bool { self.row().m_enumFloat }

    pub fn getEnumBoolean(self) -> bool { self.row().m_enumBoolean }

    pub fn returnEnumOrdinal(self) -> i32 { self as i32 }

    pub fn getEnumString(self) -> &'static str { self.row().m_enumString }
}
