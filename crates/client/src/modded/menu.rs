use crate::gui::{FontRenderer, GUI_SCREEN, GuiButton, GuiScreen, GuiScreenClass, GuiTextField};
use modloader::state::{self, ModRuntime};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

mod layout
{
    pub const TOP: i32 = 32;
    pub const ROW: i32 = 22;
    pub const MAX_ROWS: usize = 7;
    pub const BUTTON_WIDTH: i32 = 200;
    pub const DONE: i32 = 0;
    pub const CONFIG: i32 = 1;
    pub const SAVE: i32 = 1;
    pub const FIRST_ROW: i32 = 100;
}

const RED: i32 = 0xFF5555;
const WHITE: i32 = 0xFFFFFF;
const GREY: i32 = 0xA0A0A0;

struct ModList
{
    m_parent: Rc<GuiScreen>,
    m_selected: Cell<usize>,
}

fn list(this: &GuiScreen) -> &ModList { this.ext::<ModList>() }

fn label(runtime: &ModRuntime) -> String
{
    let status: &str = if runtime.isActive() { "" } else { " (failed)" };
    format!("{} {}{status}", runtime.m_manifest.displayName(), runtime.m_manifest.version)
}

const MOD_LIST: GuiScreenClass = GuiScreenClass {
    m_name: "GuiModList",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let x: i32 = this.m_width.get() / 2 - layout::BUTTON_WIDTH / 2;
        for (index, runtime) in state::mods().iter().take(layout::MAX_ROWS).enumerate()
        {
            let y: i32 = layout::TOP + index as i32 * layout::ROW;
            this.m_controlList.add(GuiButton::withSize(
                layout::FIRST_ROW + index as i32,
                x,
                y,
                layout::BUTTON_WIDTH,
                20,
                &label(runtime),
            ));
        }
        let bottom: i32 = this.m_height.get() - 28;
        this.m_controlList.add(GuiButton::withSize(layout::CONFIG, x, bottom, 98, 20, "Config"));
        this.m_controlList.add(GuiButton::withSize(layout::DONE, x + 102, bottom, 98, 20, "Done"));
    },
    actionPerformed: |this, button| {
        let mc = this.mc();
        match button.m_id.get()
        {
            layout::DONE => mc.displayGuiScreen(Some(Rc::clone(&list(this).m_parent))),
            layout::CONFIG =>
            {
                if let Some(runtime) = state::mods().get(list(this).m_selected.get())
                {
                    mc.displayGuiScreen(Some(configScreen(&this.rc(), Rc::clone(runtime))));
                }
            }
            id => list(this).m_selected.set((id - layout::FIRST_ROW).max(0) as usize),
        }
    },
    drawScreen: |this, mouseX, mouseY, partial| {
        this.drawDefaultBackground();
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        this.m_gui.drawCenteredString(&font, "Mods", center, 12, WHITE);
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, partial);
        let mut y: i32 = layout::TOP + layout::MAX_ROWS as i32 * layout::ROW + 4;
        if let Some(runtime) = state::mods().get(list(this).m_selected.get())
        {
            let manifest = &runtime.m_manifest;
            let detail: String = format!("{} - {} - {}", manifest.id, manifest.description, runtime.m_file);
            this.m_gui.drawCenteredString(&font, &detail, center, y, GREY);
            y += 12;
        }
        for problem in state::problems().iter().take(6)
        {
            this.m_gui.drawCenteredString(
                &font,
                &format!("{} ({}): {}", problem.m_mod, problem.m_file, problem.m_reason),
                center,
                y,
                RED,
            );
            y += 10;
        }
        for (first, second) in modloader::keybinds::conflicts()
        {
            this.m_gui.drawCenteredString(&font, &format!("key conflict: {first} and {second}"), center, y, RED);
            y += 10;
        }
    },
    ..GUI_SCREEN
};

pub fn modsScreen(parent: &Rc<GuiScreen>) -> Rc<GuiScreen>
{
    GuiScreen::construct(&MOD_LIST, Box::new(ModList { m_parent: Rc::clone(parent), m_selected: Cell::new(0) }))
}

pub fn modsButtonVisible() -> bool { modloader::play::present() }

// load problems show before the title screen, every one of them at once
pub fn startScreen(main: Rc<GuiScreen>) -> Rc<GuiScreen>
{
    if state::problems().is_empty()
    {
        main
    }
    else
    {
        modsScreen(&main)
    }
}

struct ModConfig
{
    m_parent: Rc<GuiScreen>,
    m_runtime: Rc<ModRuntime>,
    m_field: RefCell<Option<GuiTextField>>,
    m_key: RefCell<Option<String>>,
}

fn config(this: &GuiScreen) -> &ModConfig { this.ext::<ModConfig>() }

fn entryLabel(runtime: &ModRuntime, key: &str) -> String
{
    format!("{key} = {}", runtime.m_config.borrow().value(key).unwrap_or_default())
}

const MOD_CONFIG: GuiScreenClass = GuiScreenClass {
    m_name: "GuiModConfig",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let state: &ModConfig = config(this);
        let x: i32 = this.m_width.get() / 2 - layout::BUTTON_WIDTH / 2;
        let keys: Vec<String> =
            state.m_runtime.m_config.borrow().entries().iter().map(|entry| entry.key.clone()).collect();
        for (index, key) in keys.iter().take(layout::MAX_ROWS).enumerate()
        {
            let y: i32 = layout::TOP + index as i32 * layout::ROW;
            let label: String = entryLabel(&state.m_runtime, key);
            this.m_controlList.add(GuiButton::withSize(
                layout::FIRST_ROW + index as i32,
                x,
                y,
                layout::BUTTON_WIDTH,
                20,
                &label,
            ));
        }
        let bottom: i32 = this.m_height.get() - 28;
        let field: GuiTextField =
            GuiTextField::new(this.rc(), this.fontRenderer(), x, bottom - 26, layout::BUTTON_WIDTH, 20, "");
        field.setMaxStringLength(128);
        *state.m_field.borrow_mut() = Some(field);
        this.m_controlList.add(GuiButton::withSize(layout::SAVE, x, bottom, 98, 20, "Save"));
        this.m_controlList.add(GuiButton::withSize(layout::DONE, x + 102, bottom, 98, 20, "Done"));
    },
    actionPerformed: |this, button| {
        let state: &ModConfig = config(this);
        match button.m_id.get()
        {
            layout::DONE => this.mc().displayGuiScreen(Some(Rc::clone(&state.m_parent))),
            layout::SAVE =>
            {
                let key: Option<String> = state.m_key.borrow().clone();
                let value: Option<String> = state.m_field.borrow().as_ref().map(|field| field.getText().to_string());
                if let (Some(key), Some(value)) = (key, value)
                {
                    if let Err(error) = state.m_runtime.m_config.borrow_mut().set(&key, &value)
                    {
                        modloader::log::error(state.m_runtime.id(), &format!("config save failed: {error}"));
                    }
                    this.mc().displayGuiScreen(Some(configScreen(&state.m_parent, Rc::clone(&state.m_runtime))));
                }
            }
            id =>
            {
                let keys: Vec<String> =
                    state.m_runtime.m_config.borrow().entries().iter().map(|entry| entry.key.clone()).collect();
                if let Some(key) = keys.get((id - layout::FIRST_ROW).max(0) as usize)
                {
                    let value: String = state.m_runtime.m_config.borrow().value(key).unwrap_or_default();
                    if let Some(field) = state.m_field.borrow().as_ref()
                    {
                        field.setText(&value);
                        field.setFocused(true);
                    }
                    *state.m_key.borrow_mut() = Some(key.clone());
                }
            }
        }
    },
    keyTyped: |this, chr, key| {
        if let Some(field) = config(this).m_field.borrow().as_ref()
        {
            field.textboxKeyTyped(chr, key);
        }
        if key == 1
        {
            this.mc().displayGuiScreen(Some(Rc::clone(&config(this).m_parent)));
        }
    },
    mouseClicked: |this, mouseX, mouseY, button| {
        (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button);
        if let Some(field) = config(this).m_field.borrow().as_ref()
        {
            field.mouseClicked(mouseX, mouseY, button);
        }
    },
    updateScreen: |this| {
        if let Some(field) = config(this).m_field.borrow().as_ref()
        {
            field.updateCursorCounter();
        }
    },
    drawScreen: |this, mouseX, mouseY, partial| {
        this.drawDefaultBackground();
        let font: Rc<FontRenderer> = this.fontRenderer();
        let title: String = format!("{} config", config(this).m_runtime.m_manifest.displayName());
        this.m_gui.drawCenteredString(&font, &title, this.m_width.get() / 2, 12, WHITE);
        if let Some(field) = config(this).m_field.borrow().as_ref()
        {
            field.drawTextBox();
        }
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, partial);
    },
    ..GUI_SCREEN
};

fn configScreen(parent: &Rc<GuiScreen>, runtime: Rc<ModRuntime>) -> Rc<GuiScreen>
{
    let state: ModConfig = ModConfig {
        m_parent: Rc::clone(parent),
        m_runtime: runtime,
        m_field: RefCell::new(None),
        m_key: RefCell::new(None),
    };
    GuiScreen::construct(&MOD_CONFIG, Box::new(state))
}
