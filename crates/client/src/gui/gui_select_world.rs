use crate::client::Minecraft;
use crate::controller::player_controller_sp;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::gui_slot::{GUI_SLOT, GuiSlotClass};
use crate::gui::{FontRenderer, GuiButton, GuiCreateWorld, GuiRenameWorld, GuiScreen, GuiSlot, GuiYesNo};
use crate::port::calendar::formatDefault;
use crate::render::Tessellator;
use game::locale::StringTranslate;
use game::storage::{ISaveFormat, SaveFormatComparator};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

struct SelectWorldExt
{
    m_parentScreen: Rc<GuiScreen>,
    m_screenTitle: Field<String>,
    m_selected: Cell<bool>,
    m_selectedWorld: Cell<i32>,
    m_saveList: RefCell<Vec<SaveFormatComparator>>,
    m_worldSlotContainer: Field<Option<Rc<GuiSlot>>>,
    m_worldText: Field<String>,
    m_conversionText: Field<String>,
    m_deleting: Cell<bool>,
    m_buttonRename: Field<Option<Rc<GuiButton>>>,
    m_buttonSelect: Field<Option<Rc<GuiButton>>>,
    m_buttonDelete: Field<Option<Rc<GuiButton>>>,
}

fn fields(this: &GuiScreen) -> &SelectWorldExt { this.ext::<SelectWorldExt>() }

fn saveLoader(this: &GuiScreen) -> Rc<dyn ISaveFormat> { this.mc().m_saveLoader.get().orNpe() }

fn loadSaves(this: &GuiScreen)
{
    let ext: &SelectWorldExt = fields(this);
    let mut saveList: Vec<SaveFormatComparator> = saveLoader(this).getAll();
    saveList.sort_by(|a, b| a.compareTo(b));
    *ext.m_saveList.borrow_mut() = saveList;
    ext.m_selectedWorld.set(-1);
}

fn getSaveFileName(this: &GuiScreen, index: i32) -> String
{
    fields(this).m_saveList.borrow()[index as usize].getFileName().to_string()
}

fn getSaveName(this: &GuiScreen, index: i32) -> String
{
    let string: String = fields(this).m_saveList.borrow()[index as usize].getDisplayName().to_string();
    if string.is_empty()
    {
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        return format!("{} {}", stringTranslate.translateKey("selectWorld.world"), index + 1);
    }
    string
}

fn button(slot: &Field<Option<Rc<GuiButton>>>) -> Rc<GuiButton> { slot.get().orNpe() }

pub fn initButtons(this: &GuiScreen)
{
    let ext: &SelectWorldExt = fields(this);
    let stringtranslate: Rc<StringTranslate> = StringTranslate::getInstance();
    let center: i32 = this.m_width.get() / 2;
    let height: i32 = this.m_height.get();
    let select: Rc<GuiButton> =
        GuiButton::withSize(1, center - 154, height - 52, 150, 20, &stringtranslate.translateKey("selectWorld.select"));
    ext.m_buttonSelect.set(Some(Rc::clone(&select)));
    this.m_controlList.add(select);
    let rename: Rc<GuiButton> =
        GuiButton::withSize(6, center - 154, height - 28, 70, 20, &stringtranslate.translateKey("selectWorld.rename"));
    ext.m_buttonRename.set(Some(Rc::clone(&rename)));
    this.m_controlList.add(rename);
    let delete: Rc<GuiButton> =
        GuiButton::withSize(2, center - 74, height - 28, 70, 20, &stringtranslate.translateKey("selectWorld.delete"));
    ext.m_buttonDelete.set(Some(Rc::clone(&delete)));
    this.m_controlList.add(delete);
    this.m_controlList.add(GuiButton::withSize(
        3,
        center + 4,
        height - 52,
        150,
        20,
        &stringtranslate.translateKey("selectWorld.create"),
    ));
    this.m_controlList.add(GuiButton::withSize(
        0,
        center + 4,
        height - 28,
        150,
        20,
        &stringtranslate.translateKey("gui.cancel"),
    ));
    button(&ext.m_buttonSelect).m_enabled.set(false);
    button(&ext.m_buttonRename).m_enabled.set(false);
    button(&ext.m_buttonDelete).m_enabled.set(false);
}

pub fn selectWorld(this: &GuiScreen, id: i32)
{
    let mc: Rc<Minecraft> = this.mc();
    mc.displayGuiScreen(None);
    let ext: &SelectWorldExt = fields(this);
    if ext.m_selected.get()
    {
        return;
    }
    ext.m_selected.set(true);
    mc.m_playerController.set(Some(player_controller_sp::new(&mc)));
    let string: String = getSaveFileName(this, id);
    mc.startWorld(&string, &getSaveName(this, id), 0);
    mc.displayGuiScreen(None);
}

struct WorldSlotExt
{
    m_screen: Weak<GuiScreen>,
}

fn screen(slot: &GuiSlot) -> Rc<GuiScreen> { slot.ext::<WorldSlotExt>().m_screen.upgrade().orNpe() }

fn drawSlot(slot: &Rc<GuiSlot>, index: i32, x: i32, y: i32, _height: i32, _tessellator: &Rc<Tessellator>)
{
    let this: Rc<GuiScreen> = screen(slot);
    let ext: &SelectWorldExt = fields(&this);
    let saveList: std::cell::Ref<'_, Vec<SaveFormatComparator>> = ext.m_saveList.borrow();
    let saveFormatComparator: &SaveFormatComparator = &saveList[index as usize];
    let mut string: String = saveFormatComparator.getDisplayName().to_string();
    if string.is_empty()
    {
        string = format!("{} {}", ext.m_worldText.get(), index + 1);
    }
    let l: i64 = saveFormatComparator.getSize();
    let megabytes: f32 = (l / 1024 * 100 / 1024) as f32 / 100.0;
    // port formatDefault stands in for the java SimpleDateFormat default pattern
    let string2: String = format!(
        "{} ({}, {} MB)",
        saveFormatComparator.getFileName(),
        formatDefault(saveFormatComparator.getLastPlayed()),
        jvm::floating_decimal::floatToString(megabytes)
    );
    let mut string3: String = String::new();
    if saveFormatComparator.isSameVersion()
    {
        string3 = format!("{} {string3}", ext.m_conversionText.get());
    }
    let font: Rc<FontRenderer> = this.fontRenderer();
    this.m_gui.drawString(&font, &string, x + 2, y + 1, 0xFFFFFF);
    this.m_gui.drawString(&font, &string2, x + 2, y + 12, 0x808080);
    this.m_gui.drawString(&font, &string3, x + 2, y + 12 + 10, 0x808080);
}

const WORLD_SLOT: GuiSlotClass = GuiSlotClass {
    m_name: "GuiWorldSlot",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    getSize: |slot| fields(&screen(slot)).m_saveList.borrow().len() as i32,
    elementClicked: |slot, index, doubleClick| {
        let this: Rc<GuiScreen> = screen(slot);
        let ext: &SelectWorldExt = fields(&this);
        ext.m_selectedWorld.set(index);
        let flag: bool = ext.m_selectedWorld.get() >= 0 && ext.m_selectedWorld.get() < slot.getSize();
        button(&ext.m_buttonSelect).m_enabled.set(flag);
        button(&ext.m_buttonRename).m_enabled.set(flag);
        button(&ext.m_buttonDelete).m_enabled.set(flag);
        if doubleClick && flag
        {
            selectWorld(&this, index);
        }
    },
    isSelected: |slot, index| index == fields(&screen(slot)).m_selectedWorld.get(),
    getContentHeight: |slot| fields(&screen(slot)).m_saveList.borrow().len() as i32 * 36,
    drawBackground: |slot| screen(slot).drawDefaultBackground(),
    drawSlot,
    ..GUI_SLOT
};

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiSelectWorld",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let ext: &SelectWorldExt = fields(this);
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        ext.m_screenTitle.set(stringTranslate.translateKey("selectWorld.title"));
        ext.m_worldText.set(stringTranslate.translateKey("selectWorld.world"));
        ext.m_conversionText.set(stringTranslate.translateKey("selectWorld.conversion"));
        loadSaves(this);
        let slot: Rc<GuiSlot> = GuiSlot::construct(
            &WORLD_SLOT,
            Box::new(WorldSlotExt { m_screen: Rc::downgrade(&this.rc()) }),
            this.mc(),
            this.m_width.get(),
            this.m_height.get(),
            32,
            this.m_height.get() - 64,
            36,
        );
        slot.registerScrollButtons(&this.m_controlList, 4, 5);
        ext.m_worldSlotContainer.set(Some(slot));
        initButtons(this);
    },
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        let ext: &SelectWorldExt = fields(this);
        let mc: Rc<Minecraft> = this.mc();
        match button.m_id.get()
        {
            2 =>
            {
                let string: String = getSaveName(this, ext.m_selectedWorld.get());
                ext.m_deleting.set(true);
                let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
                let string2: String = stringTranslate.translateKey("selectWorld.deleteQuestion");
                let string3: String =
                    format!("'{string}' {}", stringTranslate.translateKey("selectWorld.deleteWarning"));
                let string4: String = stringTranslate.translateKey("selectWorld.deleteButton");
                let string5: String = stringTranslate.translateKey("gui.cancel");
                let guiYesNo: Rc<GuiScreen> =
                    GuiYesNo::new(&this.rc(), &string2, &string3, &string4, &string5, ext.m_selectedWorld.get());
                mc.displayGuiScreen(Some(guiYesNo));
            }
            1 => selectWorld(this, ext.m_selectedWorld.get()),
            3 => mc.displayGuiScreen(Some(GuiCreateWorld::new(&this.rc()))),
            6 => mc.displayGuiScreen(Some(GuiRenameWorld::new(
                &this.rc(),
                &getSaveFileName(this, ext.m_selectedWorld.get()),
            ))),
            0 => mc.displayGuiScreen(Some(Rc::clone(&ext.m_parentScreen))),
            _ => ext.m_worldSlotContainer.get().orNpe().actionPerformed(button),
        }
    },
    deleteWorld: |this, result, id| {
        let ext: &SelectWorldExt = fields(this);
        if !ext.m_deleting.get()
        {
            return;
        }
        ext.m_deleting.set(false);
        if result
        {
            let iSaveFormat: Rc<dyn ISaveFormat> = saveLoader(this);
            iSaveFormat.flushCache();
            iSaveFormat.delete(&getSaveFileName(this, id));
            loadSaves(this);
        }
        this.mc().displayGuiScreen(Some(this.rc()));
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let ext: &SelectWorldExt = fields(this);
        ext.m_worldSlotContainer.get().orNpe().drawScreen(mouseX, mouseY, tickDelta);
        this.m_gui.drawCenteredString(
            &this.fontRenderer(),
            &ext.m_screenTitle.get(),
            this.m_width.get() / 2,
            20,
            0xFFFFFF,
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiSelectWorld;

impl GuiSelectWorld
{
    pub fn new(parent: &Rc<GuiScreen>) -> Rc<GuiScreen>
    {
        let ext: SelectWorldExt = SelectWorldExt {
            m_parentScreen: Rc::clone(parent),
            m_screenTitle: Field::new("Select world".to_string()),
            m_selected: Cell::new(false),
            m_selectedWorld: Cell::new(0),
            m_saveList: RefCell::new(Vec::new()),
            m_worldSlotContainer: Field::new(None),
            m_worldText: Field::new(String::new()),
            m_conversionText: Field::new(String::new()),
            m_deleting: Cell::new(false),
            m_buttonRename: Field::new(None),
            m_buttonSelect: Field::new(None),
            m_buttonDelete: Field::new(None),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
