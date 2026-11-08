use super::config_menu::configScreen;
use crate::client::Minecraft;
use crate::gui::{FontRenderer, GUI_SCREEN, GUI_SLOT, GuiButton, GuiScreen, GuiScreenClass, GuiSlot, GuiSlotClass};
use crate::render::Tessellator;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::Sys;
use lwjgl::input::{Keyboard, Mouse};
use modloader::log;
use modloader::manifest::Manifest;
use modloader::resolver::Problem;
use modloader::state::{self, ModRuntime};
use std::cell::{Cell, Ref, RefCell};
use std::path::Path;
use std::rc::{Rc, Weak};

mod listing
{
    pub const TITLE_Y: i32 = 8;
    pub const SUMMARY_Y: i32 = 20;
    pub const TOP: i32 = 32;
    pub const BOTTOM_MARGIN: i32 = 36;
    pub const ENTRY_HEIGHT: i32 = 36;
    pub const TEXT_WIDTH: i32 = 214;
    pub const DETAIL_LINES: usize = 2;
    pub const LINE_Y: [i32; 3] = [1, 12, 22];
    pub const BUTTON_WIDTH: i32 = 98;
    pub const BUTTON_GAP: i32 = 4;
    pub const BUTTON_Y: i32 = 28;
    pub const DONE: i32 = 0;
    pub const OPEN_FOLDER: i32 = 2;
    pub const SCROLL_UP: i32 = 3;
    pub const SCROLL_DOWN: i32 = 4;
}

pub const MODS_BUTTON: i32 = 50;

const RED: i32 = 0xFF5555;
const WHITE: i32 = 0xFFFFFF;
const GREY: i32 = 0xA0A0A0;
const DARK_GREY: i32 = 0x808080;
const ELLIPSIS: &str = "...";
const LOADER_TITLE: &str = "Mod Loader";
const KEY_CONFLICT_TITLE: &str = "Key conflicts";
const MODIFY_CONFLICT_TITLE: &str = "Block changes";
const CONFIG_HINT: &str = "click a mod to change its config";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status
{
    Loaded,
    Warning,
    Failed,
    Skipped,
}

struct StatusRow
{
    m_status: Status,
    m_label: &'static str,
    m_summary: &'static str,
    m_color: i32,
}

const STATUSES: [StatusRow; 4] = [
    StatusRow { m_status: Status::Loaded, m_label: "Loaded", m_summary: "loaded", m_color: 0x55FF55 },
    StatusRow { m_status: Status::Warning, m_label: "Warning", m_summary: "with warnings", m_color: 0xFFFF55 },
    StatusRow { m_status: Status::Failed, m_label: "Failed", m_summary: "failed", m_color: RED },
    StatusRow { m_status: Status::Skipped, m_label: "Skipped", m_summary: "skipped", m_color: GREY },
];

impl Status
{
    fn row(self) -> &'static StatusRow
    {
        STATUSES.iter().find(|row| row.m_status == self).expect("every status has a row")
    }
}

struct Entry
{
    m_title: String,
    m_status: Status,
    m_lines: Vec<(String, i32)>,
    m_runtime: Option<Rc<ModRuntime>>,
}

struct ModList
{
    m_parent: Rc<GuiScreen>,
    m_entries: RefCell<Vec<Entry>>,
    m_summary: RefCell<String>,
    m_selected: Cell<i32>,
    m_slot: Field<Option<Rc<GuiSlot>>>,
}

fn list(this: &GuiScreen) -> &ModList { this.ext::<ModList>() }

struct ModSlot
{
    m_screen: Weak<GuiScreen>,
}

fn screen(slot: &GuiSlot) -> Rc<GuiScreen> { slot.ext::<ModSlot>().m_screen.upgrade().orNpe() }

pub(super) fn fit(font: &FontRenderer, text: &str, width: i32) -> String
{
    if font.getStringWidth(text) <= width
    {
        return text.to_owned();
    }
    let mut cut: String = text.to_owned();
    while !cut.is_empty() && font.getStringWidth(&format!("{cut}{ELLIPSIS}")) > width
    {
        cut.pop();
    }
    format!("{cut}{ELLIPSIS}")
}

// a word wider than the line stays whole here and is cut by fit
fn wrap(font: &FontRenderer, text: &str, width: i32) -> Vec<String>
{
    let mut lines: Vec<String> = Vec::new();
    let mut line: String = String::new();
    for word in text.split_whitespace()
    {
        let joined: String = if line.is_empty()
        {
            word.to_owned()
        }
        else
        {
            format!("{line} {word}")
        };
        if line.is_empty() || font.getStringWidth(&joined) <= width
        {
            line = joined;
        }
        else
        {
            lines.push(std::mem::replace(&mut line, word.to_owned()));
        }
    }
    if !line.is_empty()
    {
        lines.push(line);
    }
    lines
}

fn detailLines(font: &FontRenderer, details: Vec<(String, i32)>) -> Vec<(String, i32)>
{
    let mut lines: Vec<(String, i32)> = details
        .into_iter()
        .flat_map(|(text, color)| wrap(font, &text, listing::TEXT_WIDTH).into_iter().map(move |line| (line, color)))
        .collect();
    let hidden: bool = lines.len() > listing::DETAIL_LINES;
    lines.truncate(listing::DETAIL_LINES);
    if let (true, Some(last)) = (hidden, lines.last_mut())
    {
        last.0 = format!("{} {ELLIPSIS}", last.0);
    }
    lines.into_iter().map(|(text, color)| (fit(font, &text, listing::TEXT_WIDTH), color)).collect()
}

fn entry(font: &FontRenderer, title: &str, status: Status, details: Vec<(String, i32)>) -> Entry
{
    let titleWidth: i32 = listing::TEXT_WIDTH - font.getStringWidth(status.row().m_label) - 6;
    Entry {
        m_title: fit(font, title, titleWidth),
        m_status: status,
        m_lines: detailLines(font, details),
        m_runtime: None,
    }
}

fn fileName(file: &str) -> String
{
    Path::new(file).file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

// loader problems for a running mod may carry no file
fn owns(runtime: &ModRuntime, problem: &Problem) -> bool
{
    problem.m_mod == runtime.id() && (problem.m_file.is_empty() || problem.m_file == runtime.m_file)
}

fn modEntry(font: &FontRenderer, runtime: Rc<ModRuntime>, problems: &mut Vec<Problem>) -> Entry
{
    let (own, rest): (Vec<Problem>, Vec<Problem>) =
        std::mem::take(problems).into_iter().partition(|problem| owns(&runtime, problem));
    *problems = rest;
    let status: Status = match (runtime.isActive() && runtime.m_instance.is_some(), own.is_empty())
    {
        (false, _) => Status::Failed,
        (true, true) => Status::Loaded,
        (true, false) => Status::Warning,
    };
    let manifest: &Manifest = &runtime.m_manifest;
    let mut details: Vec<(String, i32)> =
        own.into_iter().map(|problem| (problem.m_reason, status.row().m_color)).collect();
    if !manifest.description.is_empty()
    {
        details.push((manifest.description.clone(), GREY));
    }
    details.push((format!("{} - {}", manifest.id, runtime.m_file), DARK_GREY));
    let title: String = format!("{} {}", manifest.displayName(), manifest.version);
    Entry { m_runtime: Some(Rc::clone(&runtime)), ..entry(font, &title, status, details) }
}

// problems of one file that never became a running mod
fn problemEntry(font: &FontRenderer, problems: Vec<Problem>) -> Entry
{
    let first: &Problem = &problems[0];
    let file: String = fileName(&first.m_file);
    let title: String = match first.m_mod.as_str()
    {
        state::UNKNOWN_MOD => file.clone(),
        log::LOADER => LOADER_TITLE.to_owned(),
        id => id.to_owned(),
    };
    let showFile: bool = !file.is_empty() && file != title;
    let mut details: Vec<(String, i32)> = problems.into_iter().map(|problem| (problem.m_reason, RED)).collect();
    if showFile
    {
        details.push((file, DARK_GREY));
    }
    entry(font, &title, Status::Failed, details)
}

fn entries(font: &FontRenderer) -> Vec<Entry>
{
    let mut problems: Vec<Problem> = state::problems();
    let mut entries: Vec<Entry> =
        state::mods().into_iter().map(|runtime| modEntry(font, runtime, &mut problems)).collect();
    while let Some(first) = problems.first()
    {
        let (modId, file): (String, String) = (first.m_mod.clone(), first.m_file.clone());
        let (own, rest): (Vec<Problem>, Vec<Problem>) =
            problems.into_iter().partition(|problem| problem.m_mod == modId && problem.m_file == file);
        problems = rest;
        entries.push(problemEntry(font, own));
    }
    for (id, reason) in state::skipped()
    {
        entries.push(entry(font, &id, Status::Skipped, vec![(reason, GREY)]));
    }
    let conflicts: Vec<(String, String)> = modloader::keybinds::conflicts();
    if !conflicts.is_empty()
    {
        let details: Vec<(String, i32)> =
            conflicts.into_iter().map(|(first, second)| (format!("{first} and {second}"), RED)).collect();
        entries.push(entry(font, KEY_CONFLICT_TITLE, Status::Warning, details));
    }
    let changes: Vec<String> = modloader::modify::conflicts();
    if !changes.is_empty()
    {
        let details: Vec<(String, i32)> = changes.into_iter().map(|change| (change, RED)).collect();
        entries.push(entry(font, MODIFY_CONFLICT_TITLE, Status::Warning, details));
    }
    entries
}

fn summary(entries: &[Entry]) -> String
{
    let mut parts: Vec<String> = STATUSES
        .iter()
        .filter_map(|row| {
            let count: usize = entries.iter().filter(|entry| entry.m_status == row.m_status).count();
            (count > 0).then(|| format!("{count} {}", row.m_summary))
        })
        .collect();
    if entries.iter().any(|entry| configurable(entry).is_some())
    {
        parts.push(CONFIG_HINT.to_owned());
    }
    parts.join(", ")
}

// the wheel arrives as a release event and scrolls like the vanilla scroll buttons
pub(super) fn scrollWheel(slot: &Rc<GuiSlot>, wheel: i32, up: i32, down: i32)
{
    if wheel != 0
    {
        let id: i32 = if wheel > 0 { up } else { down };
        slot.actionPerformed(&GuiButton::new(id, 0, 0, ""));
    }
}

fn configurable(entry: &Entry) -> Option<Rc<ModRuntime>>
{
    entry.m_runtime.as_ref().filter(|runtime| !runtime.m_config.borrow().options().is_empty()).cloned()
}

fn selectedConfig(this: &GuiScreen) -> Option<Rc<ModRuntime>>
{
    let state: &ModList = list(this);
    let entries: Ref<'_, Vec<Entry>> = state.m_entries.borrow();
    usize::try_from(state.m_selected.get()).ok().and_then(|index| entries.get(index)).and_then(configurable)
}

fn openConfig(this: &GuiScreen)
{
    if let Some(runtime) = selectedConfig(this)
    {
        this.mc().displayGuiScreen(Some(configScreen(&this.rc(), runtime)));
    }
}

fn openModsFolder()
{
    let Some(paths) = modloader::paths()
    else
    {
        return;
    };
    let folder: &Path = &paths.m_mods;
    let absolute: std::path::PathBuf = std::path::absolute(folder).unwrap_or_else(|_| folder.to_path_buf());
    if !Sys::openURL(&format!("file://{}", absolute.display()))
    {
        log::warn(log::LOADER, &format!("could not open {}", absolute.display()));
    }
}

fn drawSlot(slot: &Rc<GuiSlot>, index: i32, x: i32, y: i32, _height: i32, _tessellator: &Rc<Tessellator>)
{
    let this: Rc<GuiScreen> = screen(slot);
    let font: Rc<FontRenderer> = this.fontRenderer();
    let entries: Ref<'_, Vec<Entry>> = list(&this).m_entries.borrow();
    let entry: &Entry = &entries[index as usize];
    let row: &StatusRow = entry.m_status.row();
    let [titleY, detailYs @ ..] = listing::LINE_Y;
    this.m_gui.drawString(&font, &entry.m_title, x, y + titleY, WHITE);
    let labelX: i32 = x + listing::TEXT_WIDTH - font.getStringWidth(row.m_label);
    this.m_gui.drawString(&font, row.m_label, labelX, y + titleY, row.m_color);
    for ((text, color), lineY) in entry.m_lines.iter().zip(detailYs)
    {
        this.m_gui.drawString(&font, text, x, y + lineY, *color);
    }
}

const MOD_SLOT: GuiSlotClass = GuiSlotClass {
    m_name: "GuiModSlot",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    getSize: |slot| list(&screen(slot)).m_entries.borrow().len() as i32,
    elementClicked: |slot, index, _| {
        let this: Rc<GuiScreen> = screen(slot);
        list(&this).m_selected.set(index);
        openConfig(&this);
    },
    isSelected: |slot, index| list(&screen(slot)).m_selected.get() == index,
    drawBackground: |slot| screen(slot).drawDefaultBackground(),
    drawSlot,
    ..GUI_SLOT
};

const MOD_LIST: GuiScreenClass = GuiScreenClass {
    m_name: "GuiModList",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let state: &ModList = list(this);
        let found: Vec<Entry> = entries(&this.fontRenderer());
        *state.m_summary.borrow_mut() = summary(&found);
        *state.m_entries.borrow_mut() = found;
        let (width, height): (i32, i32) = (this.m_width.get(), this.m_height.get());
        let mc: Rc<Minecraft> = this.mc();
        let slot: Rc<GuiSlot> = GuiSlot::construct(
            &MOD_SLOT,
            Box::new(ModSlot { m_screen: Rc::downgrade(&this.rc()) }),
            mc,
            width,
            height,
            listing::TOP,
            height - listing::BOTTOM_MARGIN,
            listing::ENTRY_HEIGHT,
        );
        slot.registerScrollButtons(&this.m_controlList, listing::SCROLL_UP, listing::SCROLL_DOWN);
        state.m_slot.set(Some(slot));
        let step: i32 = listing::BUTTON_WIDTH + listing::BUTTON_GAP;
        let left: i32 = width / 2 - step + listing::BUTTON_GAP / 2;
        let y: i32 = height - listing::BUTTON_Y;
        let button = |id: i32, column: i32, label: &str| {
            GuiButton::withSize(id, left + column * step, y, listing::BUTTON_WIDTH, 20, label)
        };
        this.m_controlList.add(button(listing::OPEN_FOLDER, 0, "Mods Folder"));
        this.m_controlList.add(button(listing::DONE, 1, "Done"));
    },
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        match button.m_id.get()
        {
            listing::DONE => this.mc().displayGuiScreen(Some(Rc::clone(&list(this).m_parent))),
            listing::OPEN_FOLDER => openModsFolder(),
            _ => list(this).m_slot.get().orNpe().actionPerformed(button),
        }
    },
    keyTyped: |this, _, key| {
        if key == Keyboard::KEY_ESCAPE
        {
            this.mc().displayGuiScreen(Some(Rc::clone(&list(this).m_parent)));
        }
    },
    mouseMovedOrUp: |this, mouseX, mouseY, button| {
        let slot: Rc<GuiSlot> = list(this).m_slot.get().orNpe();
        scrollWheel(&slot, Mouse::getEventDWheel(), listing::SCROLL_UP, listing::SCROLL_DOWN);
        (GUI_SCREEN.mouseMovedOrUp)(this, mouseX, mouseY, button);
    },
    drawScreen: |this, mouseX, mouseY, partial| {
        let state: &ModList = list(this);
        state.m_slot.get().orNpe().drawScreen(mouseX, mouseY, partial);
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        this.m_gui.drawCenteredString(&font, "Mods", center, listing::TITLE_Y, WHITE);
        this.m_gui.drawCenteredString(&font, &state.m_summary.borrow(), center, listing::SUMMARY_Y, GREY);
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, partial);
    },
    ..GUI_SCREEN
};

pub fn modsScreen(parent: &Rc<GuiScreen>) -> Rc<GuiScreen>
{
    let state: ModList = ModList {
        m_parent: Rc::clone(parent),
        m_entries: RefCell::new(Vec::new()),
        m_summary: RefCell::new(String::new()),
        m_selected: Cell::new(-1),
        m_slot: Field::new(None),
    };
    GuiScreen::construct(&MOD_LIST, Box::new(state))
}

// the button only appears when mod files are present
pub fn addModsButton(screen: &GuiScreen, x: i32, y: i32)
{
    if modloader::play::present()
    {
        screen.m_controlList.add(GuiButton::small(MODS_BUTTON, x, y, "Mods"));
    }
}

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
