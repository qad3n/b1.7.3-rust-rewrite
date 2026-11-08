use crate::client::Minecraft;
use crate::settings::{EnumOptions, KeyBinding};
use game::locale::{StatCollector, StringTranslate};
use game::port::text;
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::{OrNpe, Throwable};
use lwjgl::input::Keyboard;
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const RENDER_DISTANCES: [&str; 4] = [
    "options.renderDistance.far",
    "options.renderDistance.normal",
    "options.renderDistance.short",
    "options.renderDistance.tiny",
];
const DIFFICULTIES: [&str; 4] =
    ["options.difficulty.peaceful", "options.difficulty.easy", "options.difficulty.normal", "options.difficulty.hard"];
const GUISCALES: [&str; 4] =
    ["options.guiScale.auto", "options.guiScale.small", "options.guiScale.normal", "options.guiScale.large"];
const LIMIT_FRAMERATES: [&str; 3] = ["performance.max", "performance.balanced", "performance.powersaver"];

pub struct GameSettings
{
    pub m_musicVolume: Cell<f32>,
    pub m_soundVolume: Cell<f32>,
    pub m_mouseSensitivity: Cell<f32>,
    pub m_invertMouse: Cell<bool>,
    pub m_renderDistance: Cell<i32>,
    pub m_viewBobbing: Cell<bool>,
    pub m_anaglyph: Cell<bool>,
    pub m_advancedOpengl: Cell<bool>,
    pub m_limitFramerate: Cell<i32>,
    pub m_fancyGraphics: Cell<bool>,
    pub m_ambientOcclusion: Cell<bool>,
    pub m_skin: Field<String>,
    pub m_keyBindForward: Rc<KeyBinding>,
    pub m_keyBindLeft: Rc<KeyBinding>,
    pub m_keyBindBack: Rc<KeyBinding>,
    pub m_keyBindRight: Rc<KeyBinding>,
    pub m_keyBindJump: Rc<KeyBinding>,
    pub m_keyBindInventory: Rc<KeyBinding>,
    pub m_keyBindDrop: Rc<KeyBinding>,
    pub m_keyBindChat: Rc<KeyBinding>,
    pub m_keyBindToggleFog: Rc<KeyBinding>,
    pub m_keyBindSneak: Rc<KeyBinding>,
    pub m_keyBindings: Vec<Rc<KeyBinding>>,
    m_mc: Field<Option<Rc<Minecraft>>>,
    m_optionsFile: Option<PathBuf>,
    pub m_difficulty: Cell<i32>,
    pub m_hideGUI: Cell<bool>,
    pub m_thirdPersonView: Cell<bool>,
    pub m_showDebugInfo: Cell<bool>,
    pub m_lastServer: Field<String>,
    pub m_discreteScroll: Cell<bool>,
    pub m_smoothCamera: Cell<bool>,
    pub m_debugCamera: Cell<bool>,
    pub m_totalDiscreteScroll: Cell<f32>,
    pub m_field_22271_G: Cell<f32>,
    pub m_guiScale: Cell<i32>,
}

fn arrayIndex(index: i32, length: usize) -> usize
{
    if !(0..length as i32).contains(&index)
    {
        jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &index.to_string());
    }
    index as usize
}

fn parseInt(string: &str) -> i32
{
    // port parseInt runs java Integer.parseInt on rust text
    game::port::parseInt(string).unwrap_or_else(|exception| jvm::throwable::rethrow(exception))
}

fn parseFloat(s: &str) -> f32
{
    match s
    {
        "true" => 1.0,
        "false" => 0.0,
        _ => jvm::floating_decimal::parseFloat(s),
    }
}

impl GameSettings
{
    fn alloc(minecraft: Option<Rc<Minecraft>>, optionsFile: Option<PathBuf>) -> GameSettings
    {
        let keyBindForward: Rc<KeyBinding> = KeyBinding::new("key.forward", 17);
        let keyBindLeft: Rc<KeyBinding> = KeyBinding::new("key.left", 30);
        let keyBindBack: Rc<KeyBinding> = KeyBinding::new("key.back", 31);
        let keyBindRight: Rc<KeyBinding> = KeyBinding::new("key.right", 32);
        let keyBindJump: Rc<KeyBinding> = KeyBinding::new("key.jump", 57);
        let keyBindInventory: Rc<KeyBinding> = KeyBinding::new("key.inventory", 18);
        let keyBindDrop: Rc<KeyBinding> = KeyBinding::new("key.drop", 16);
        let keyBindChat: Rc<KeyBinding> = KeyBinding::new("key.chat", 20);
        let keyBindToggleFog: Rc<KeyBinding> = KeyBinding::new("key.fog", 33);
        let keyBindSneak: Rc<KeyBinding> = KeyBinding::new("key.sneak", 42);
        let keyBindings: Vec<Rc<KeyBinding>> = vec![
            Rc::clone(&keyBindForward),
            Rc::clone(&keyBindLeft),
            Rc::clone(&keyBindBack),
            Rc::clone(&keyBindRight),
            Rc::clone(&keyBindJump),
            Rc::clone(&keyBindSneak),
            Rc::clone(&keyBindDrop),
            Rc::clone(&keyBindInventory),
            Rc::clone(&keyBindChat),
            Rc::clone(&keyBindToggleFog),
        ];
        GameSettings {
            m_musicVolume: Cell::new(1.0),
            m_soundVolume: Cell::new(1.0),
            m_mouseSensitivity: Cell::new(0.5),
            m_invertMouse: Cell::new(false),
            m_renderDistance: Cell::new(0),
            m_viewBobbing: Cell::new(true),
            m_anaglyph: Cell::new(false),
            m_advancedOpengl: Cell::new(false),
            m_limitFramerate: Cell::new(1),
            m_fancyGraphics: Cell::new(true),
            m_ambientOcclusion: Cell::new(true),
            m_skin: Field::new("Default".to_owned()),
            m_keyBindForward: keyBindForward,
            m_keyBindLeft: keyBindLeft,
            m_keyBindBack: keyBindBack,
            m_keyBindRight: keyBindRight,
            m_keyBindJump: keyBindJump,
            m_keyBindInventory: keyBindInventory,
            m_keyBindDrop: keyBindDrop,
            m_keyBindChat: keyBindChat,
            m_keyBindToggleFog: keyBindToggleFog,
            m_keyBindSneak: keyBindSneak,
            m_keyBindings: keyBindings,
            m_mc: Field::new(minecraft),
            m_optionsFile: optionsFile,
            m_difficulty: Cell::new(2),
            m_hideGUI: Cell::new(false),
            m_thirdPersonView: Cell::new(false),
            m_showDebugInfo: Cell::new(false),
            m_lastServer: Field::new(String::new()),
            m_discreteScroll: Cell::new(false),
            m_smoothCamera: Cell::new(false),
            m_debugCamera: Cell::new(false),
            m_totalDiscreteScroll: Cell::new(1.0),
            m_field_22271_G: Cell::new(1.0),
            m_guiScale: Cell::new(0),
        }
    }

    pub fn new(minecraft: Rc<Minecraft>, dir: &Path) -> Rc<GameSettings>
    {
        let settings: Rc<GameSettings> = Rc::new(GameSettings::alloc(Some(minecraft), Some(dir.join("options.txt"))));
        settings.loadOptions();
        settings
    }

    pub fn empty() -> Rc<GameSettings> { Rc::new(GameSettings::alloc(None, None)) }

    fn mc(&self) -> Rc<Minecraft> { self.m_mc.get().orNpe() }

    fn keyBinding(&self, keyBinding: i32) -> &Rc<KeyBinding>
    {
        &self.m_keyBindings[arrayIndex(keyBinding, self.m_keyBindings.len())]
    }

    pub fn getKeyBindingDescription(&self, keyBinding: i32) -> String
    {
        StringTranslate::getInstance().translateKey(&self.keyBinding(keyBinding).m_keyDescription)
    }

    pub fn getOptionDisplayString(&self, keyBinding: i32) -> Option<String>
    {
        Keyboard::getKeyName(self.keyBinding(keyBinding).m_keyCode.get())
    }

    pub fn setKeyBinding(&self, keyBinding: i32, keyCode: i32)
    {
        self.keyBinding(keyBinding).m_keyCode.set(keyCode);
        self.saveOptions();
    }

    pub fn setOptionFloatValue(&self, option: EnumOptions, value: f32)
    {
        match option
        {
            EnumOptions::Music =>
            {
                self.m_musicVolume.set(value);
                self.mc().m_sndManager.get().orNpe().onSoundOptionsChanged();
            }
            EnumOptions::Sound =>
            {
                self.m_soundVolume.set(value);
                self.mc().m_sndManager.get().orNpe().onSoundOptionsChanged();
            }
            EnumOptions::Sensitivity => self.m_mouseSensitivity.set(value),
            _ =>
            {}
        }
    }

    pub fn setOptionValue(&self, option: EnumOptions, value: i32)
    {
        match option
        {
            EnumOptions::InvertMouse => self.m_invertMouse.set(!self.m_invertMouse.get()),
            EnumOptions::RenderDistance => self.m_renderDistance.set((self.m_renderDistance.get() + value) & 3),
            EnumOptions::GuiScale => self.m_guiScale.set((self.m_guiScale.get() + value) & 3),
            EnumOptions::ViewBobbing => self.m_viewBobbing.set(!self.m_viewBobbing.get()),
            EnumOptions::AdvancedOpengl =>
            {
                self.m_advancedOpengl.set(!self.m_advancedOpengl.get());
                self.mc().m_renderGlobal.get().orNpe().loadRenderers();
            }
            EnumOptions::Anaglyph =>
            {
                self.m_anaglyph.set(!self.m_anaglyph.get());
                self.mc().m_renderEngine.get().orNpe().refreshTextures();
            }
            EnumOptions::FramerateLimit => self.m_limitFramerate.set((self.m_limitFramerate.get() + value + 3) % 3),
            EnumOptions::Difficulty => self.m_difficulty.set((self.m_difficulty.get() + value) & 3),
            EnumOptions::Graphics =>
            {
                self.m_fancyGraphics.set(!self.m_fancyGraphics.get());
                self.mc().m_renderGlobal.get().orNpe().loadRenderers();
            }
            EnumOptions::AmbientOcclusion =>
            {
                self.m_ambientOcclusion.set(!self.m_ambientOcclusion.get());
                self.mc().m_renderGlobal.get().orNpe().loadRenderers();
            }
            _ =>
            {}
        }
        self.saveOptions();
    }

    pub fn getOptionFloatValue(&self, option: EnumOptions) -> f32
    {
        match option
        {
            EnumOptions::Music => self.m_musicVolume.get(),
            EnumOptions::Sound => self.m_soundVolume.get(),
            EnumOptions::Sensitivity => self.m_mouseSensitivity.get(),
            _ => 0.0,
        }
    }

    pub fn getOptionOrdinalValue(&self, option: EnumOptions) -> bool
    {
        match option
        {
            EnumOptions::InvertMouse => self.m_invertMouse.get(),
            EnumOptions::ViewBobbing => self.m_viewBobbing.get(),
            EnumOptions::Anaglyph => self.m_anaglyph.get(),
            EnumOptions::AdvancedOpengl => self.m_advancedOpengl.get(),
            EnumOptions::AmbientOcclusion => self.m_ambientOcclusion.get(),
            _ => false,
        }
    }

    pub fn getKeyBinding(&self, option: EnumOptions) -> String
    {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let string: String = format!("{}: ", stringTranslate.translateKey(option.getEnumString()));
        if option.getEnumFloat()
        {
            let f: f32 = self.getOptionFloatValue(option);
            if option == EnumOptions::Sensitivity
            {
                if f == 0.0
                {
                    return string + &stringTranslate.translateKey("options.sensitivity.min");
                }
                if f == 1.0
                {
                    return string + &stringTranslate.translateKey("options.sensitivity.max");
                }
                return format!("{string}{}%", (f * 200.0) as i32);
            }
            if f == 0.0
            {
                return string + &stringTranslate.translateKey("options.off");
            }
            return format!("{string}{}%", (f * 100.0) as i32);
        }
        if option.getEnumBoolean()
        {
            let key: &str = if self.getOptionOrdinalValue(option)
            {
                "options.on"
            }
            else
            {
                "options.off"
            };
            return string + &stringTranslate.translateKey(key);
        }
        match option
        {
            EnumOptions::RenderDistance =>
            {
                let index: usize = arrayIndex(self.m_renderDistance.get(), RENDER_DISTANCES.len());
                string + &stringTranslate.translateKey(RENDER_DISTANCES[index])
            }
            EnumOptions::Difficulty =>
            {
                let index: usize = arrayIndex(self.m_difficulty.get(), DIFFICULTIES.len());
                string + &stringTranslate.translateKey(DIFFICULTIES[index])
            }
            EnumOptions::GuiScale =>
            {
                let index: usize = arrayIndex(self.m_guiScale.get(), GUISCALES.len());
                string + &stringTranslate.translateKey(GUISCALES[index])
            }
            EnumOptions::FramerateLimit =>
            {
                let index: usize = arrayIndex(self.m_limitFramerate.get(), LIMIT_FRAMERATES.len());
                string + &StatCollector::translateToLocal(LIMIT_FRAMERATES[index])
            }
            EnumOptions::Graphics =>
            {
                let key: &str = if self.m_fancyGraphics.get()
                {
                    "options.graphics.fancy"
                }
                else
                {
                    "options.graphics.fast"
                };
                string + &stringTranslate.translateKey(key)
            }
            _ => string,
        }
    }

    fn loadOption(&self, line: &str)
    {
        // port split drops trailing empty parts like java String.split
        let astring: Vec<&str> = text::split(line, ':');
        let value = |index: usize| -> &str {
            match astring.get(index)
            {
                Some(part) => part,
                None => jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &index.to_string()),
            }
        };
        match value(0)
        {
            "music" => self.m_musicVolume.set(parseFloat(value(1))),
            "sound" => self.m_soundVolume.set(parseFloat(value(1))),
            "mouseSensitivity" => self.m_mouseSensitivity.set(parseFloat(value(1))),
            "invertYMouse" => self.m_invertMouse.set(value(1) == "true"),
            "viewDistance" => self.m_renderDistance.set(parseInt(value(1))),
            "guiScale" => self.m_guiScale.set(parseInt(value(1))),
            "bobView" => self.m_viewBobbing.set(value(1) == "true"),
            "anaglyph3d" => self.m_anaglyph.set(value(1) == "true"),
            "advancedOpengl" => self.m_advancedOpengl.set(value(1) == "true"),
            "fpsLimit" => self.m_limitFramerate.set(parseInt(value(1))),
            "difficulty" => self.m_difficulty.set(parseInt(value(1))),
            "fancyGraphics" => self.m_fancyGraphics.set(value(1) == "true"),
            "ao" => self.m_ambientOcclusion.set(value(1) == "true"),
            "skin" => self.m_skin.set(value(1).to_owned()),
            "lastServer" if astring.len() >= 2 => self.m_lastServer.set(value(1).to_owned()),
            _ =>
            {}
        }
        for keyBinding in &self.m_keyBindings
        {
            if value(0) == format!("key_{}", keyBinding.m_keyDescription)
            {
                keyBinding.m_keyCode.set(parseInt(value(1)));
            }
        }
    }

    fn readOptions(&self, optionsFile: &Path) -> Result<(), Throwable>
    {
        let bytes: Vec<u8> = std::fs::read(optionsFile).map_err(|error| jvm::io::fileNotFound(optionsFile, &error))?;
        // port decodePlatform decodes in the platform charset like a java Reader
        let text: String = text::decodePlatform(&bytes);
        // port readLines splits lines like BufferedReader.readLine
        for s in text::readLines(&text)
        {
            if jvm::throwable::catchThrowable(&jvm::throwable::EXCEPTION, || self.loadOption(s)).is_err()
            {
                jvm::system::println(&format!("Skipping bad option: {s}"));
            }
        }
        Ok(())
    }

    pub fn loadOptions(&self)
    {
        let optionsFile: &Path = self.m_optionsFile.as_deref().orNpe();
        if !optionsFile.exists()
        {
            return;
        }
        if let Err(exception) = self.readOptions(optionsFile)
        {
            jvm::system::println("Failed to load options");
            exception.printStackTrace();
        }
    }

    fn optionsText(&self) -> String
    {
        let mut lines: Vec<String> = vec![
            format!("music:{}", jvm::floating_decimal::floatToString(self.m_musicVolume.get())),
            format!("sound:{}", jvm::floating_decimal::floatToString(self.m_soundVolume.get())),
            format!("invertYMouse:{}", self.m_invertMouse.get()),
            format!("mouseSensitivity:{}", jvm::floating_decimal::floatToString(self.m_mouseSensitivity.get())),
            format!("viewDistance:{}", self.m_renderDistance.get()),
            format!("guiScale:{}", self.m_guiScale.get()),
            format!("bobView:{}", self.m_viewBobbing.get()),
            format!("anaglyph3d:{}", self.m_anaglyph.get()),
            format!("advancedOpengl:{}", self.m_advancedOpengl.get()),
            format!("fpsLimit:{}", self.m_limitFramerate.get()),
            format!("difficulty:{}", self.m_difficulty.get()),
            format!("fancyGraphics:{}", self.m_fancyGraphics.get()),
            format!("ao:{}", self.m_ambientOcclusion.get()),
            format!("skin:{}", self.m_skin.get()),
            format!("lastServer:{}", self.m_lastServer.get()),
        ];
        for keyBinding in &self.m_keyBindings
        {
            lines.push(format!("key_{}:{}", keyBinding.m_keyDescription, keyBinding.m_keyCode.get()));
        }
        lines.iter().map(|line| format!("{line}\n")).collect()
    }

    pub fn saveOptions(&self)
    {
        let optionsFile: &Path = self.m_optionsFile.as_deref().orNpe();
        match std::fs::File::create(optionsFile)
        {
            Ok(mut file) =>
            {
                let _ = std::io::Write::write_all(&mut file, &JString::from(self.optionsText().as_str()).getBytes());
            }
            Err(error) =>
            {
                jvm::system::println("Failed to save options");
                jvm::io::fileNotFound(optionsFile, &error).printStackTrace();
            }
        }
    }
}
