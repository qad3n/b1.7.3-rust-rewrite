use super::menu::{fit, scrollWheel};
use crate::client::Minecraft;
use crate::gui::{
    FontRenderer, GUI_SCREEN, GUI_SLOT, GuiButton, GuiScreen, GuiScreenClass, GuiSlot, GuiSlotClass, GuiTextField,
};
use crate::render::Tessellator;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::input::{Keyboard, Mouse};
use modloader::config;
use modloader::mod_protocol::abi::{ConfigKind, ConfigOption};
use modloader::state::ModRuntime;
use std::cell::{Cell, Ref, RefCell};
use std::rc::{Rc, Weak};

mod layout
{
    pub const TITLE_Y: i32 = 8;
    pub const FILE_Y: i32 = 20;
    pub const TOP: i32 = 32;
    pub const BOTTOM_MARGIN: i32 = 66;
    pub const ROW_HEIGHT: i32 = 24;
    pub const CONTROL_WIDTH: i32 = 120;
    pub const CONTROL_HEIGHT: i32 = 20;
    pub const STEP_WIDTH: i32 = 20;
    pub const GAP: i32 = 2;
    pub const ROW_MAX_WIDTH: i32 = 400;
    pub const ROW_MARGIN: i32 = 30;
    pub const INFO_Y: [i32; 3] = [62, 51, 40];
    pub const INFO_MARGIN: i32 = 20;
    pub const FIELD_Y: i32 = 64;
    pub const FIELD_MAX_WIDTH: i32 = 300;
    pub const FIELD_MAX_LENGTH: i32 = 80;
    pub const BUTTON_Y: i32 = 28;
    pub const BUTTON_WIDTH: i32 = 98;
    pub const DONE: i32 = 0;
    pub const RESET: i32 = 1;
    pub const SCROLL_UP: i32 = 2;
    pub const SCROLL_DOWN: i32 = 3;
}

mod steps
{
    pub const INT: i64 = 1;
    pub const FLOAT: f64 = 0.1;
    pub const SHIFT_SCALE: i64 = 10;
    // keeps repeated float steps from drifting into long fractions
    pub const FLOAT_ROUNDING: f64 = 10000.0;
}

const WHITE: i32 = 0xFFFFFF;
const YELLOW: i32 = 0xFFFF55;
const GREY: i32 = 0xA0A0A0;
const DARK_GREY: i32 = 0x808080;
const RED: i32 = 0xFF5555;
const GREEN: i32 = 0x55FF55;
const ON: &str = "ON";
const OFF: &str = "OFF";
const GENERAL: &str = "General";
const RESTART_NOTE: &str = "Saved, restart the game to apply";

enum Row
{
    Header(String),
    Option(ConfigOption),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Part
{
    Toggle,
    Cycle,
    Minus,
    Plus,
    Edit,
}

struct Control
{
    m_part: Part,
    m_x: i32,
    m_width: i32,
    m_text: String,
}

struct ConfigMenu
{
    m_parent: Rc<GuiScreen>,
    m_runtime: Rc<ModRuntime>,
    m_rows: RefCell<Vec<Row>>,
    m_selected: Cell<i32>,
    m_mouse: Cell<(i32, i32)>,
    m_editing: RefCell<Option<String>>,
    m_field: RefCell<Option<GuiTextField>>,
    m_message: RefCell<Option<(String, i32)>>,
    m_slot: Field<Option<Rc<GuiSlot>>>,
    m_resetButton: Field<Option<Rc<GuiButton>>>,
}

fn menu(this: &GuiScreen) -> &ConfigMenu { this.ext::<ConfigMenu>() }

struct ConfigSlot
{
    m_screen: Weak<GuiScreen>,
}

fn screen(slot: &GuiSlot) -> Rc<GuiScreen> { slot.ext::<ConfigSlot>().m_screen.upgrade().orNpe() }

fn rows(options: &[ConfigOption]) -> Vec<Row>
{
    let mut categories: Vec<Option<&str>> = Vec::new();
    for found in options.iter().map(config::category)
    {
        if !categories.contains(&found)
        {
            categories.push(found);
        }
    }
    categories.sort_by_key(Option::is_some);
    let headed: bool = categories.iter().any(Option::is_some);
    let mut rows: Vec<Row> = Vec::new();
    for current in categories
    {
        if headed
        {
            rows.push(Row::Header(current.map_or_else(|| GENERAL.to_owned(), config::humanize)));
        }
        rows.extend(options.iter().filter(|option| config::category(option) == current).cloned().map(Row::Option));
    }
    rows
}

fn value(this: &GuiScreen, option: &ConfigOption) -> String
{
    menu(this).m_runtime.m_config.borrow().value(&option.key).unwrap_or_else(|| option.kind.defaultText())
}

fn controls(font: &FontRenderer, option: &ConfigOption, current: &str, x: i32) -> Vec<Control>
{
    let control = |part: Part, offset: i32, width: i32, text: &str| Control {
        m_part: part,
        m_x: x + offset,
        m_width: width,
        m_text: fit(font, text, width - 6),
    };
    match &option.kind
    {
        ConfigKind::Bool { .. } =>
        {
            vec![control(Part::Toggle, 0, layout::CONTROL_WIDTH, if current == "true" { ON } else { OFF })]
        }
        ConfigKind::Choice { .. } => vec![control(Part::Cycle, 0, layout::CONTROL_WIDTH, &config::humanize(current))],
        ConfigKind::Int { .. } | ConfigKind::Float { .. } =>
        {
            let middle: i32 = layout::CONTROL_WIDTH - 2 * (layout::STEP_WIDTH + layout::GAP);
            let plusX: i32 = layout::CONTROL_WIDTH - layout::STEP_WIDTH;
            vec![
                control(Part::Minus, 0, layout::STEP_WIDTH, "-"),
                control(Part::Edit, layout::STEP_WIDTH + layout::GAP, middle, current),
                control(Part::Plus, plusX, layout::STEP_WIDTH, "+"),
            ]
        }
        ConfigKind::Text { .. } => vec![control(Part::Edit, 0, layout::CONTROL_WIDTH, current)],
    }
}

fn rowWidth(width: i32) -> i32 { (width - 2 * layout::ROW_MARGIN).min(layout::ROW_MAX_WIDTH) }

fn controlX(this: &GuiScreen) -> i32
{
    let width: i32 = this.m_width.get();
    width / 2 + rowWidth(width) / 2 - layout::GAP - layout::CONTROL_WIDTH
}

fn shifted() -> bool { Keyboard::isKeyDown(Keyboard::KEY_LSHIFT) || Keyboard::isKeyDown(Keyboard::KEY_RSHIFT) }

fn nextChoice(choices: &[String], current: &str) -> String
{
    let index: usize = choices.iter().position(|choice| choice == current).map_or(0, |index| index + 1);
    choices[index % choices.len()].clone()
}

// the stepped value clamped to the option range
fn stepped(kind: &ConfigKind, current: &str, direction: i64) -> Option<String>
{
    let scale: i64 = if shifted() { steps::SHIFT_SCALE } else { 1 };
    match kind
    {
        ConfigKind::Int { min, max, .. } =>
        {
            let base: i64 = current.parse().ok()?;
            Some(base.saturating_add(direction * steps::INT * scale).clamp(*min, *max).to_string())
        }
        ConfigKind::Float { min, max, .. } =>
        {
            let base: f64 = current.parse().ok()?;
            let next: f64 = base + (direction * scale) as f64 * steps::FLOAT;
            let rounded: f64 = (next * steps::FLOAT_ROUNDING).round() / steps::FLOAT_ROUNDING;
            Some(rounded.clamp(*min, *max).to_string())
        }
        _ => None,
    }
}

fn apply(this: &GuiScreen, option: &ConfigOption, text: &str) -> bool
{
    let state: &ConfigMenu = menu(this);
    let result: Result<(), String> = config::change(&state.m_runtime, &option.key, text);
    let message: Option<(String, i32)> = match &result
    {
        Ok(()) if option.restart => Some((RESTART_NOTE.to_owned(), GREEN)),
        Ok(()) => None,
        Err(reason) => Some((reason.clone(), RED)),
    };
    *state.m_message.borrow_mut() = message;
    result.is_ok()
}

fn startEditing(this: &GuiScreen, option: &ConfigOption)
{
    let state: &ConfigMenu = menu(this);
    if let Some(field) = state.m_field.borrow().as_ref()
    {
        field.setText(&value(this, option));
        field.setFocused(true);
    }
    *state.m_editing.borrow_mut() = Some(option.key.clone());
}

fn stopEditing(this: &GuiScreen)
{
    let state: &ConfigMenu = menu(this);
    *state.m_editing.borrow_mut() = None;
    if let Some(field) = state.m_field.borrow().as_ref()
    {
        field.setFocused(false);
    }
}

fn commitEdit(this: &GuiScreen)
{
    let state: &ConfigMenu = menu(this);
    let Some(key) = state.m_editing.borrow().clone()
    else
    {
        return;
    };
    let text: String = state.m_field.borrow().as_ref().map(|field| field.getText().to_string()).unwrap_or_default();
    let option: Option<ConfigOption> =
        state.m_runtime.m_config.borrow().options().iter().find(|option| option.key == key).cloned();
    if let Some(option) = option
        && apply(this, &option, &text)
    {
        stopEditing(this);
    }
}

fn press(this: &GuiScreen, option: &ConfigOption, part: Part)
{
    let current: String = value(this, option);
    let next: Option<String> = match (part, &option.kind)
    {
        (Part::Toggle, _) => Some((current != "true").to_string()),
        (Part::Cycle, ConfigKind::Choice { choices, .. }) => Some(nextChoice(choices, &current)),
        (Part::Minus, kind) => stepped(kind, &current, -1),
        (Part::Plus, kind) => stepped(kind, &current, 1),
        (Part::Edit, _) =>
        {
            startEditing(this, option);
            None
        }
        _ => None,
    };
    if let Some(next) = next
    {
        stopEditing(this);
        apply(this, option, &next);
    }
}

fn selectedOption(this: &GuiScreen) -> Option<ConfigOption>
{
    let state: &ConfigMenu = menu(this);
    let rows: Ref<'_, Vec<Row>> = state.m_rows.borrow();
    match usize::try_from(state.m_selected.get()).ok().and_then(|index| rows.get(index))
    {
        Some(Row::Option(option)) => Some(option.clone()),
        _ => None,
    }
}

fn select(this: &GuiScreen, index: i32)
{
    let state: &ConfigMenu = menu(this);
    state.m_selected.set(index);
    *state.m_message.borrow_mut() = None;
    if let Some(button) = state.m_resetButton.get()
    {
        button.m_enabled.set(selectedOption(this).is_some());
    }
}

fn drawSlot(slot: &Rc<GuiSlot>, index: i32, x: i32, y: i32, height: i32, _tessellator: &Rc<Tessellator>)
{
    let this: Rc<GuiScreen> = screen(slot);
    let font: Rc<FontRenderer> = this.fontRenderer();
    let state: &ConfigMenu = menu(&this);
    let rows: Ref<'_, Vec<Row>> = state.m_rows.borrow();
    let textY: i32 = y + (height - 8) / 2;
    match &rows[index as usize]
    {
        Row::Header(title) => this.m_gui.drawCenteredString(&font, title, this.m_width.get() / 2, textY, YELLOW),
        Row::Option(option) =>
        {
            let left: i32 = controlX(&this);
            let labelColor: i32 = if value(&this, option) == option.kind.defaultText()
            {
                WHITE
            }
            else
            {
                YELLOW
            };
            this.m_gui.drawString(&font, &fit(&font, &config::label(option), left - x - 4), x, textY, labelColor);
            let mc: Rc<Minecraft> = this.mc();
            let (mouseX, mouseY): (i32, i32) = state.m_mouse.get();
            for control in controls(&font, option, &value(&this, option), left)
            {
                let button: Rc<GuiButton> =
                    GuiButton::withSize(0, control.m_x, y, control.m_width, layout::CONTROL_HEIGHT, &control.m_text);
                button.drawButton(&mc, mouseX, mouseY);
            }
        }
    }
}

const CONFIG_SLOT: GuiSlotClass = GuiSlotClass {
    m_name: "GuiModConfigSlot",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    getSize: |slot| menu(&screen(slot)).m_rows.borrow().len() as i32,
    elementClicked: |slot, index, _| {
        let this: Rc<GuiScreen> = screen(slot);
        let option: Option<ConfigOption> = match menu(&this).m_rows.borrow().get(index as usize)
        {
            Some(Row::Option(option)) => Some(option.clone()),
            _ => None,
        };
        let Some(option) = option
        else
        {
            return;
        };
        select(&this, index);
        let font: Rc<FontRenderer> = this.fontRenderer();
        let mouseX: i32 = menu(&this).m_mouse.get().0;
        let hit: Option<Part> = controls(&font, &option, &value(&this, &option), controlX(&this))
            .into_iter()
            .find(|control| mouseX >= control.m_x && mouseX < control.m_x + control.m_width)
            .map(|control| control.m_part);
        if let Some(part) = hit
        {
            this.mc().m_sndManager.get().orNpe().playSoundFX("random.click", 1.0, 1.0);
            press(&this, &option, part);
        }
    },
    isSelected: |slot, index| menu(&screen(slot)).m_selected.get() == index,
    drawBackground: |slot| screen(slot).drawDefaultBackground(),
    drawSlot,
    ..GUI_SLOT
};

fn drawInfo(this: &GuiScreen, font: &FontRenderer)
{
    let state: &ConfigMenu = menu(this);
    let (width, height): (i32, i32) = (this.m_width.get(), this.m_height.get());
    let center: i32 = width / 2;
    let infoWidth: i32 = width - 2 * layout::INFO_MARGIN;
    let Some(option) = selectedOption(this)
    else
    {
        return;
    };
    let [labelY, commentY, detailY] = layout::INFO_Y;
    // the edit field covers the label and comment lines while open
    if state.m_editing.borrow().is_none()
    {
        this.m_gui.drawCenteredString(
            font,
            &fit(font, &config::label(&option), infoWidth),
            center,
            height - labelY,
            WHITE,
        );
        this.m_gui.drawCenteredString(font, &fit(font, &option.comment, infoWidth), center, height - commentY, GREY);
    }

    let (detail, color): (String, i32) =
        state.m_message.borrow().clone().unwrap_or_else(|| (config::describe(&option), DARK_GREY));
    this.m_gui.drawCenteredString(font, &fit(font, &detail, infoWidth), center, height - detailY, color);
}

const MOD_CONFIG: GuiScreenClass = GuiScreenClass {
    m_name: "GuiModConfig",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let state: &ConfigMenu = menu(this);
        *state.m_rows.borrow_mut() = rows(state.m_runtime.m_config.borrow().options());
        let (width, height): (i32, i32) = (this.m_width.get(), this.m_height.get());
        let slot: Rc<GuiSlot> = GuiSlot::construct(
            &CONFIG_SLOT,
            Box::new(ConfigSlot { m_screen: Rc::downgrade(&this.rc()) }),
            this.mc(),
            width,
            height,
            layout::TOP,
            height - layout::BOTTOM_MARGIN,
            layout::ROW_HEIGHT,
        );
        slot.setRowWidth(rowWidth(width));
        slot.registerScrollButtons(&this.m_controlList, layout::SCROLL_UP, layout::SCROLL_DOWN);
        state.m_slot.set(Some(slot));
        let fieldWidth: i32 = (width - 2 * layout::INFO_MARGIN).min(layout::FIELD_MAX_WIDTH);
        let field: GuiTextField = GuiTextField::new(
            this.rc(),
            this.fontRenderer(),
            width / 2 - fieldWidth / 2,
            height - layout::FIELD_Y,
            fieldWidth,
            layout::CONTROL_HEIGHT,
            "",
        );
        field.setMaxStringLength(layout::FIELD_MAX_LENGTH);
        *state.m_field.borrow_mut() = Some(field);
        *state.m_editing.borrow_mut() = None;
        let y: i32 = height - layout::BUTTON_Y;
        let reset: Rc<GuiButton> = GuiButton::withSize(
            layout::RESET,
            width / 2 - layout::BUTTON_WIDTH - 2,
            y,
            layout::BUTTON_WIDTH,
            20,
            "Reset",
        );
        this.m_controlList.add(Rc::clone(&reset));
        this.m_controlList.add(GuiButton::withSize(layout::DONE, width / 2 + 2, y, layout::BUTTON_WIDTH, 20, "Done"));
        state.m_resetButton.set(Some(reset));
        select(this, state.m_selected.get());
    },
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        match button.m_id.get()
        {
            layout::DONE => this.mc().displayGuiScreen(Some(Rc::clone(&menu(this).m_parent))),
            layout::RESET =>
            {
                if let Some(option) = selectedOption(this)
                {
                    stopEditing(this);
                    apply(this, &option, &option.kind.defaultText());
                }
            }
            _ => menu(this).m_slot.get().orNpe().actionPerformed(button),
        }
    },
    keyTyped: |this, chr, key| {
        let editing: bool = menu(this).m_editing.borrow().is_some();
        match key
        {
            Keyboard::KEY_ESCAPE if editing => stopEditing(this),
            Keyboard::KEY_ESCAPE => this.mc().displayGuiScreen(Some(Rc::clone(&menu(this).m_parent))),
            Keyboard::KEY_RETURN if editing => commitEdit(this),
            _ =>
            {
                if let Some(field) = menu(this).m_field.borrow().as_ref()
                {
                    field.textboxKeyTyped(chr, key);
                }
            }
        }
    },
    mouseMovedOrUp: |this, mouseX, mouseY, button| {
        scrollWheel(&menu(this).m_slot.get().orNpe(), Mouse::getEventDWheel(), layout::SCROLL_UP, layout::SCROLL_DOWN);
        (GUI_SCREEN.mouseMovedOrUp)(this, mouseX, mouseY, button);
    },
    updateScreen: |this| {
        if let Some(field) = menu(this).m_field.borrow().as_ref()
        {
            field.updateCursorCounter();
        }
    },
    drawScreen: |this, mouseX, mouseY, partial| {
        let state: &ConfigMenu = menu(this);
        state.m_mouse.set((mouseX, mouseY));
        state.m_slot.get().orNpe().drawScreen(mouseX, mouseY, partial);
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        let title: String = format!("{} config", state.m_runtime.m_manifest.displayName());
        this.m_gui.drawCenteredString(&font, &title, center, layout::TITLE_Y, WHITE);
        let file: String = state.m_runtime.m_config.borrow().fileName();
        this.m_gui.drawCenteredString(&font, &file, center, layout::FILE_Y, GREY);
        drawInfo(this, &font);
        if state.m_editing.borrow().is_some()
            && let Some(field) = state.m_field.borrow().as_ref()
        {
            field.drawTextBox();
        }
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, partial);
    },
    ..GUI_SCREEN
};

pub fn configScreen(parent: &Rc<GuiScreen>, runtime: Rc<ModRuntime>) -> Rc<GuiScreen>
{
    let state: ConfigMenu = ConfigMenu {
        m_parent: Rc::clone(parent),
        m_runtime: runtime,
        m_rows: RefCell::new(Vec::new()),
        m_selected: Cell::new(-1),
        m_mouse: Cell::new((0, 0)),
        m_editing: RefCell::new(None),
        m_field: RefCell::new(None),
        m_message: RefCell::new(None),
        m_slot: Field::new(None),
        m_resetButton: Field::new(None),
    };
    GuiScreen::construct(&MOD_CONFIG, Box::new(state))
}
