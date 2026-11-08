use crate::client::{ChatAllowedCharacters, Minecraft};
use crate::controller::player_controller_sp;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen, GuiTextField};
use game::locale::StringTranslate;
use game::math::math_helper;
use game::port::text;
use game::storage::ISaveFormat;
use jvm::field::Field;
use jvm::random::Random;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::input::Keyboard;
use std::cell::Cell;
use std::rc::Rc;

struct CreateWorldExt
{
    m_parent: Rc<GuiScreen>,
    m_textboxWorldName: Field<Option<Rc<GuiTextField>>>,
    m_textboxSeed: Field<Option<Rc<GuiTextField>>>,
    m_folderName: Field<String>,
    m_createClicked: Cell<bool>,
}

fn fields(this: &GuiScreen) -> &CreateWorldExt { this.ext::<CreateWorldExt>() }

fn worldName(this: &GuiScreen) -> Rc<GuiTextField> { fields(this).m_textboxWorldName.get().orNpe() }

fn seedField(this: &GuiScreen) -> Rc<GuiTextField> { fields(this).m_textboxSeed.get().orNpe() }

fn getSaveDirectoryName(this: &GuiScreen)
{
    // port trim strips chars up to space like java String.trim
    let mut folderName: JString = text::trim(&worldName(this).getText());
    for c in ChatAllowedCharacters::ALLOWED_CHARACTERS_ARRAY
    {
        // port replaceChar swaps utf16 units like java String.replace
        folderName = text::replaceChar(&folderName, c as u16, b'_' as u16);
    }
    if math_helper::stringNullOrLengthZero(Some(&folderName))
    {
        folderName = JString::from("World");
    }
    let saveLoader: Rc<dyn ISaveFormat> = this.mc().m_saveLoader.get().orNpe();
    fields(this).m_folderName.set(GuiCreateWorld::generateUnusedFolderName(&*saveLoader, &folderName.to_string()));
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiCreateWorld",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this| {
        worldName(this).updateCursorCounter();
        seedField(this).updateCursorCounter();
    },
    initGui: |this| {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        Keyboard::enableRepeatEvents(true);
        this.m_controlList.clear();
        let x: i32 = this.m_width.get() / 2 - 100;
        let y: i32 = this.m_height.get() / 4;
        this.m_controlList.add(GuiButton::new(0, x, y + 96 + 12, &stringTranslate.translateKey("selectWorld.create")));
        this.m_controlList.add(GuiButton::new(1, x, y + 120 + 12, &stringTranslate.translateKey("gui.cancel")));
        let name: GuiTextField = GuiTextField::new(
            this.rc(),
            this.fontRenderer(),
            x,
            60,
            200,
            20,
            &stringTranslate.translateKey("selectWorld.newWorld"),
        );
        name.m_isFocused.set(true);
        name.setMaxStringLength(32);
        fields(this).m_textboxWorldName.set(Some(Rc::new(name)));
        let seed: GuiTextField = GuiTextField::new(this.rc(), this.fontRenderer(), x, 116, 200, 20, "");
        fields(this).m_textboxSeed.set(Some(Rc::new(seed)));
        getSaveDirectoryName(this);
    },
    onGuiClosed: |_| Keyboard::enableRepeatEvents(false),
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        let ext: &CreateWorldExt = fields(this);
        let mc: Rc<Minecraft> = this.mc();
        match button.m_id.get()
        {
            1 => mc.displayGuiScreen(Some(Rc::clone(&ext.m_parent))),
            0 =>
            {
                mc.displayGuiScreen(None);
                if ext.m_createClicked.get()
                {
                    return;
                }
                ext.m_createClicked.set(true);
                let mut l: i64 = Random::new().nextLong();
                let string: JString = seedField(this).getText();
                if !math_helper::stringNullOrLengthZero(Some(&string))
                {
                    match jvm::integer::parseLong(&string)
                    {
                        Ok(l2) if l2 != 0 => l = l2,
                        Ok(_) =>
                        {}
                        Err(_) => l = string.hashCode() as i64,
                    }
                }
                mc.m_playerController.set(Some(player_controller_sp::new(&mc)));
                mc.startWorld(&ext.m_folderName.get(), &worldName(this).getText().to_string(), l);
                mc.displayGuiScreen(None);
            }
            _ =>
            {}
        }
    },
    keyTyped: |this, chr, key| {
        if worldName(this).m_isFocused.get()
        {
            worldName(this).textboxKeyTyped(chr, key);
        }
        else
        {
            seedField(this).textboxKeyTyped(chr, key);
        }
        if chr == b'\r' as u16
        {
            this.actionPerformed(&this.m_controlList.get(0));
        }
        this.m_controlList.get(0).m_enabled.set(worldName(this).getText().length() > 0);
        getSaveDirectoryName(this);
    },
    mouseClicked: |this, mouseX, mouseY, button| {
        (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button);
        worldName(this).mouseClicked(mouseX, mouseY, button);
        seedField(this).mouseClicked(mouseX, mouseY, button);
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        this.drawDefaultBackground();
        let font: Rc<FontRenderer> = this.fontRenderer();
        let x: i32 = this.m_width.get() / 2 - 100;
        this.m_gui.drawCenteredString(
            &font,
            &stringTranslate.translateKey("selectWorld.create"),
            this.m_width.get() / 2,
            this.m_height.get() / 4 - 60 + 20,
            0xFFFFFF,
        );
        this.m_gui.drawString(&font, &stringTranslate.translateKey("selectWorld.enterName"), x, 47, 0xA0A0A0);
        let resultFolder: String =
            format!("{} {}", stringTranslate.translateKey("selectWorld.resultFolder"), fields(this).m_folderName.get());
        this.m_gui.drawString(&font, &resultFolder, x, 85, 0xA0A0A0);
        this.m_gui.drawString(&font, &stringTranslate.translateKey("selectWorld.enterSeed"), x, 104, 0xA0A0A0);
        this.m_gui.drawString(&font, &stringTranslate.translateKey("selectWorld.seedInfo"), x, 140, 0xA0A0A0);
        worldName(this).drawTextBox();
        seedField(this).drawTextBox();
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    selectNextField: |this| {
        let focusName: bool = !worldName(this).m_isFocused.get();
        worldName(this).setFocused(focusName);
        seedField(this).setFocused(!focusName);
    },
    ..GUI_SCREEN
};

pub struct GuiCreateWorld;

impl GuiCreateWorld
{
    pub fn new(parent: &Rc<GuiScreen>) -> Rc<GuiScreen>
    {
        let ext: CreateWorldExt = CreateWorldExt {
            m_parent: Rc::clone(parent),
            m_textboxWorldName: Field::new(None),
            m_textboxSeed: Field::new(None),
            m_folderName: Field::new(String::new()),
            m_createClicked: Cell::new(false),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }

    pub fn generateUnusedFolderName(storageSource: &dyn ISaveFormat, worldName: &str) -> String
    {
        let mut worldName: String = worldName.to_string();
        while storageSource.getData(&worldName).is_some()
        {
            worldName.push('-');
        }
        worldName
    }
}
