use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen, GuiTextField};
use game::locale::StringTranslate;
use game::port::text;
use game::storage::ISaveFormat;
use game::world::WorldInfo;
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::input::Keyboard;
use std::rc::Rc;

struct RenameWorldExt
{
    m_parent: Rc<GuiScreen>,
    m_searchField: Field<Option<Rc<GuiTextField>>>,
    m_worldName: String,
}

fn fields(this: &GuiScreen) -> &RenameWorldExt { this.ext::<RenameWorldExt>() }

fn searchField(this: &GuiScreen) -> Rc<GuiTextField> { fields(this).m_searchField.get().orNpe() }

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiRenameWorld",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    updateScreen: |this| searchField(this).updateCursorCounter(),
    initGui: |this| {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        Keyboard::enableRepeatEvents(true);
        this.m_controlList.clear();
        let x: i32 = this.m_width.get() / 2 - 100;
        let y: i32 = this.m_height.get() / 4;
        this.m_controlList.add(GuiButton::new(
            0,
            x,
            y + 96 + 12,
            &stringTranslate.translateKey("selectWorld.renameButton"),
        ));
        this.m_controlList.add(GuiButton::new(1, x, y + 120 + 12, &stringTranslate.translateKey("gui.cancel")));
        let iSaveFormat: Rc<dyn ISaveFormat> = this.mc().m_saveLoader.get().orNpe();
        let worldInfo: Rc<WorldInfo> = iSaveFormat.getData(&fields(this).m_worldName).orNpe();
        let string: String = JString::display(worldInfo.getWorldName().as_ref());
        let field: GuiTextField = GuiTextField::new(this.rc(), this.fontRenderer(), x, 60, 200, 20, &string);
        field.m_isFocused.set(true);
        field.setMaxStringLength(32);
        fields(this).m_searchField.set(Some(Rc::new(field)));
    },
    onGuiClosed: |_| Keyboard::enableRepeatEvents(false),
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        let ext: &RenameWorldExt = fields(this);
        match button.m_id.get()
        {
            1 => this.mc().displayGuiScreen(Some(Rc::clone(&ext.m_parent))),
            0 =>
            {
                let iSaveFormat: Rc<dyn ISaveFormat> = this.mc().m_saveLoader.get().orNpe();
                // port trim strips chars up to space like java String.trim
                iSaveFormat.rename(&ext.m_worldName, &text::trim(&searchField(this).getText()).to_string());
                this.mc().displayGuiScreen(Some(Rc::clone(&ext.m_parent)));
            }
            _ =>
            {}
        }
    },
    keyTyped: |this, chr, key| {
        let field: Rc<GuiTextField> = searchField(this);
        field.textboxKeyTyped(chr, key);
        // port trim strips chars up to space like java String.trim
        this.m_controlList.get(0).m_enabled.set(text::trim(&field.getText()).length() > 0);
        if chr == b'\r' as u16
        {
            this.actionPerformed(&this.m_controlList.get(0));
        }
    },
    mouseClicked: |this, mouseX, mouseY, button| {
        (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button);
        searchField(this).mouseClicked(mouseX, mouseY, button);
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        this.drawDefaultBackground();
        let font: Rc<FontRenderer> = this.fontRenderer();
        this.m_gui.drawCenteredString(
            &font,
            &stringTranslate.translateKey("selectWorld.renameTitle"),
            this.m_width.get() / 2,
            this.m_height.get() / 4 - 60 + 20,
            0xFFFFFF,
        );
        this.m_gui.drawString(
            &font,
            &stringTranslate.translateKey("selectWorld.enterName"),
            this.m_width.get() / 2 - 100,
            47,
            0xA0A0A0,
        );
        searchField(this).drawTextBox();
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiRenameWorld;

impl GuiRenameWorld
{
    pub fn new(parent: &Rc<GuiScreen>, worldName: &str) -> Rc<GuiScreen>
    {
        let ext: RenameWorldExt = RenameWorldExt {
            m_parent: Rc::clone(parent),
            m_searchField: Field::new(None),
            m_worldName: worldName.to_string(),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
