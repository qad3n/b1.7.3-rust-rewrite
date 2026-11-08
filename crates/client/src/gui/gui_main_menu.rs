use crate::assets;
use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiMultiplayer, GuiOptions, GuiScreen, GuiSelectWorld, GuiTexturePacks};
use crate::port::calendar::{DateFields, dateFields};
use crate::render::Tessellator;
use game::locale::StringTranslate;
use game::math::math_helper;
use game::port::text;
use jvm::field::Field;
use jvm::io::InputStream;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static RAND: Random = Random::new();
}

struct MainMenuExt
{
    m_updateCounter: Cell<f32>,
    m_splashText: Field<String>,
    m_multiplayerButton: Field<Option<Rc<GuiButton>>>,
}

fn fields(this: &GuiScreen) -> &MainMenuExt { this.ext::<MainMenuExt>() }

fn readSplashes() -> Option<Vec<String>>
{
    let mut stream = assets::getResourceAsStream("/title/splashes.txt")?;
    let mut bytes: Vec<u8> = Vec::new();
    loop
    {
        let value: i32 = stream.read().ok()?;
        if value < 0
        {
            break;
        }
        bytes.push(value as u8);
    }
    let text: String = String::from_utf8_lossy(&bytes).into_owned();
    // port readLines and trimStr give java BufferedReader.readLine and String.trim
    Some(
        text::readLines(&text)
            .into_iter()
            .map(text::trimStr)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

const HOLIDAY_SPLASHES: [(i32, i32, &str); 4] = [
    (11, 9, "Happy birthday, ez!"),
    (6, 1, "Happy birthday, Notch!"),
    (12, 24, "Merry X-mas!"),
    (1, 1, "Happy new year!"),
];

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiMainMenu",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this| fields(this).m_updateCounter.set(fields(this).m_updateCounter.get() + 1.0),
    keyTyped: |_, _, _| {},
    initGui: |this| {
        let ext: &MainMenuExt = fields(this);
        // port dateFields stands in for java Calendar fields in local time
        let date: DateFields = dateFields(jvm::system::currentTimeMillis());
        if let Some((_, _, splash)) =
            HOLIDAY_SPLASHES.iter().find(|(month, day, _)| date.m_month == *month && date.m_day == *day)
        {
            ext.m_splashText.set(splash.to_string());
        }
        let stringtranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let i: i32 = this.m_height.get() / 4 + 48;
        let x: i32 = this.m_width.get() / 2 - 100;
        this.m_controlList.add(GuiButton::new(1, x, i, &stringtranslate.translateKey("menu.singleplayer")));
        let multiplayerButton: Rc<GuiButton> =
            GuiButton::new(2, x, i + 24, &stringtranslate.translateKey("menu.multiplayer"));
        ext.m_multiplayerButton.set(Some(Rc::clone(&multiplayerButton)));
        this.m_controlList.add(multiplayerButton);
        this.m_controlList.add(GuiButton::new(3, x, i + 48, &stringtranslate.translateKey("menu.mods")));
        let mc: Rc<Minecraft> = this.mc();
        if mc.m_hideQuitButton.get()
        {
            this.m_controlList.add(GuiButton::new(0, x, i + 72, &stringtranslate.translateKey("menu.options")));
        }
        else
        {
            this.m_controlList.add(GuiButton::withSize(
                0,
                x,
                i + 72 + 12,
                98,
                20,
                &stringtranslate.translateKey("menu.options"),
            ));
            this.m_controlList.add(GuiButton::withSize(
                4,
                this.m_width.get() / 2 + 2,
                i + 72 + 12,
                98,
                20,
                &stringtranslate.translateKey("menu.quit"),
            ));
        }
        if mc.m_session.get().is_none()
        {
            ext.m_multiplayerButton.get().orNpe().m_enabled.set(false);
        }
    },
    actionPerformed: |this, button| {
        let mc: Rc<Minecraft> = this.mc();
        let parent: Rc<GuiScreen> = this.rc();
        match button.m_id.get()
        {
            0 => mc.displayGuiScreen(Some(GuiOptions::new(&parent, &mc.m_gameSettings.get().orNpe()))),
            1 => mc.displayGuiScreen(Some(GuiSelectWorld::new(&parent))),
            2 => mc.displayGuiScreen(Some(GuiMultiplayer::new(&parent))),
            3 => mc.displayGuiScreen(Some(GuiTexturePacks::new(&parent))),
            4 => mc.shutdown(),
            _ =>
            {}
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        this.drawDefaultBackground();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let short1: i32 = 274;
        let i: i32 = this.m_width.get() / 2 - short1 / 2;
        let b0: i32 = 30;
        let mc: Rc<Minecraft> = this.mc();
        GL11::glBindTexture(3553, mc.m_renderEngine.get().orNpe().getTexture("/title/mclogo.png"));
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        this.m_gui.drawTexturedModalRect(i + 0, b0 + 0, 0, 0, 155, 44);
        this.m_gui.drawTexturedModalRect(i + 155, b0 + 0, 0, 45, 155, 44);
        tessellator.setColorOpaque_I(16777215);
        GL11::glPushMatrix();
        GL11::glTranslatef((this.m_width.get() / 2 + 90) as f32, 70.0, 0.0);
        GL11::glRotatef(-20.0, 0.0, 0.0, 1.0);
        let font: Rc<FontRenderer> = this.fontRenderer();
        let splashText: String = fields(this).m_splashText.get();
        let phase: f32 = (jvm::system::currentTimeMillis() % 1000) as f32 / 1000.0 * std::f64::consts::PI as f32 * 2.0;
        let mut f: f32 = 1.8 - math_helper::abs(math_helper::sin(phase) * 0.1);
        f = f * 100.0 / (font.getStringWidth(&splashText) + 32) as f32;
        GL11::glScalef(f, f, f);
        this.m_gui.drawCenteredString(&font, &splashText, 0, -8, 16776960);
        GL11::glPopMatrix();
        this.m_gui.drawString(&font, "Minecraft Beta 1.7.3", 2, 2, 5263440);
        let s: &str = "Copyright Mojang AB. Do not distribute.";
        this.m_gui.drawString(
            &font,
            s,
            this.m_width.get() - font.getStringWidth(s) - 2,
            this.m_height.get() - 10,
            16777215,
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiMainMenu;

impl GuiMainMenu
{
    pub fn new() -> Rc<GuiScreen>
    {
        let ext: MainMenuExt = MainMenuExt {
            m_updateCounter: Cell::new(0.0),
            m_splashText: Field::new("missingno".to_string()),
            m_multiplayerButton: Field::new(None),
        };
        if let Some(splashes) = readSplashes()
            && !splashes.is_empty()
        {
            let index: i32 = RAND.with(|rand| rand.nextIntBounded(splashes.len() as i32));
            ext.m_splashText.set(splashes[index as usize].clone());
        }
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
