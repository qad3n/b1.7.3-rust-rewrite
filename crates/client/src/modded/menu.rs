use super::config_menu::configScreen;
use crate::client::Minecraft;
use crate::gui::{FontRenderer, GUI_SCREEN, GUI_SLOT, GuiButton, GuiScreen, GuiScreenClass, GuiSlot, GuiSlotClass};
use crate::render::Tessellator;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::Sys;
use lwjgl::input::{Keyboard, Mouse};
use modloader::log;
use modloader::manifest::{Manifest, ModKind};
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
    pub const BOTTOM_MARGIN: i32 = 80;
    pub const ENTRY_HEIGHT: i32 = 24;
    pub const ROW_MAX_WIDTH: i32 = 320;
    pub const LINE_Y: [i32; 2] = [1, 12];
    pub const STATUS_GAP: i32 = 6;
    pub const INFO_Y: i32 = 76;
    pub const INFO_LINES: usize = 4;
    pub const BUTTON_WIDTH: i32 = 98;
    pub const BUTTON_GAP: i32 = 4;
    pub const BUTTON_Y: i32 = 28;
    pub const DONE: i32 = 0;
    pub const CONFIG: i32 = 1;
    pub const OPEN_FOLDER: i32 = 2;
    pub const SCROLL_UP: i32 = 3;
    pub const SCROLL_DOWN: i32 = 4;
}

pub(crate) mod text
{
    pub const LINE_HEIGHT: i32 = 10;
    pub const ROW_MARGIN: i32 = 30;
    // the slot draws rows 2 in from the highlight edge
    pub const ROW_INSET: i32 = 2;
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
const SELECT_HINT: &str = "Select a mod to see its details";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status
{
    Loaded,
    Library,
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

const STATUSES: [StatusRow; 5] = [
    StatusRow { m_status: Status::Loaded, m_label: "Loaded", m_summary: "loaded", m_color: 0x55FF55 },
    StatusRow { m_status: Status::Library, m_label: "Library", m_summary: "library", m_color: 0x55FFFF },
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

type Line = (String, i32);

struct Entry
{
    m_title: String,
    m_status: Status,
    m_details: Vec<Line>,
    m_runtime: Option<Rc<ModRuntime>>,
    m_shownTitle: String,
    m_preview: Line,
    m_info: Vec<Line>,
}

struct ModList
{
    m_parent: Rc<GuiScreen>,
    m_entries: RefCell<Vec<Entry>>,
    m_summary: RefCell<String>,
    m_selected: Cell<i32>,
    m_slot: Field<Option<Rc<GuiSlot>>>,
    m_configButton: Field<Option<Rc<GuiButton>>>,
}

fn list(this: &GuiScreen) -> &ModList { this.ext::<ModList>() }

struct ModSlot
{
    m_screen: Weak<GuiScreen>,
}

fn screen(slot: &GuiSlot) -> Rc<GuiScreen> { slot.ext::<ModSlot>().m_screen.upgrade().orNpe() }

pub(crate) fn fit(font: &FontRenderer, text: &str, width: i32) -> String
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

// wraps every text to the width and ends the last kept line with an ellipsis when lines are cut
pub(crate) fn wrapLines(font: &FontRenderer, texts: &[Line], width: i32, limit: usize) -> Vec<Line>
{
    let mut lines: Vec<Line> = texts
        .iter()
        .flat_map(|(text, color)| wrap(font, text, width).into_iter().map(move |line| (line, *color)))
        .collect();
    let hidden: bool = lines.len() > limit;

    lines.truncate(limit);

    if let (true, Some(last)) = (hidden, lines.last_mut())
    {
        last.0 = format!("{} {ELLIPSIS}", last.0);
    }

    lines.into_iter().map(|(text, color)| (fit(font, &text, width), color)).collect()
}

pub(crate) fn rowWidth(width: i32, max: i32) -> i32 { (width - 2 * text::ROW_MARGIN).min(max) }

fn textWidth(width: i32) -> i32 { rowWidth(width, listing::ROW_MAX_WIDTH) - 2 * text::ROW_INSET }

fn entry(title: &str, status: Status, details: Vec<Line>) -> Entry
{
    Entry
    {
        m_title: title.to_owned(),
        m_status: status,
        m_details: details,
        m_runtime: None,
        m_shownTitle: String::new(),
        m_preview: (String::new(), GREY),
        m_info: Vec::new(),
    }
}

// fits the row and info texts once per layout instead of every frame
fn place(font: &FontRenderer, entry: &mut Entry, width: i32)
{
    let titleWidth: i32 = width - font.getStringWidth(entry.m_status.row().m_label) - listing::STATUS_GAP;
    entry.m_shownTitle = fit(font, &entry.m_title, titleWidth);

    if let Some((preview, color)) = entry.m_details.first()
    {
        entry.m_preview = (fit(font, preview, width), *color);
    }

    entry.m_info = wrapLines(font, &entry.m_details, width, listing::INFO_LINES);
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

fn modEntry(runtime: Rc<ModRuntime>, problems: &mut Vec<Problem>) -> Entry
{
    let (own, rest): (Vec<Problem>, Vec<Problem>) =
        std::mem::take(problems).into_iter().partition(|problem| owns(&runtime, problem));
    *problems = rest;

    let manifest: &Manifest = &runtime.m_manifest;

    let status: Status = match (runtime.isActive() && runtime.m_instance.is_some(), own.is_empty(), manifest.kind)
    {
        (false, _, _) => Status::Failed,
        (true, true, ModKind::Mod) => Status::Loaded,
        (true, true, ModKind::Library) => Status::Library,
        (true, false, _) => Status::Warning,
    };

    let mut details: Vec<Line> = own.into_iter().map(|problem| (problem.m_reason, status.row().m_color)).collect();

    if !manifest.description.is_empty()
    {
        details.push((manifest.description.clone(), GREY));
    }

    details.push((format!("{} - {}", manifest.id, fileName(&runtime.m_file)), DARK_GREY));

    let title: String = format!("{} {}", manifest.displayName(), manifest.version);
    Entry { m_runtime: Some(Rc::clone(&runtime)), ..entry(&title, status, details) }
}

// problems of one file that never became a running mod
fn problemEntry(problems: Vec<Problem>) -> Entry
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
    let mut details: Vec<Line> = problems.into_iter().map(|problem| (problem.m_reason, RED)).collect();

    if showFile
    {
        details.push((file, DARK_GREY));
    }

    entry(&title, Status::Failed, details)
}

fn entries() -> Vec<Entry>
{
    let mut problems: Vec<Problem> = state::problems();
    let mut entries: Vec<Entry> = state::mods().into_iter().map(|runtime| modEntry(runtime, &mut problems)).collect();

    while let Some(first) = problems.first()
    {
        let (modId, file): (String, String) = (first.m_mod.clone(), first.m_file.clone());
        let (own, rest): (Vec<Problem>, Vec<Problem>) =
            problems.into_iter().partition(|problem| problem.m_mod == modId && problem.m_file == file);
        problems = rest;
        entries.push(problemEntry(own));
    }

    for (id, reason) in state::skipped()
    {
        entries.push(entry(&id, Status::Skipped, vec![(reason, GREY)]));
    }

    let conflicts: Vec<(String, String)> = modloader::keybinds::conflicts();
    if !conflicts.is_empty()
    {
        let details: Vec<Line> =
            conflicts.into_iter().map(|(first, second)| (format!("{first} and {second}"), RED)).collect();
        entries.push(entry(KEY_CONFLICT_TITLE, Status::Warning, details));
    }

    let changes: Vec<String> = modloader::modify::conflicts();
    if !changes.is_empty()
    {
        let details: Vec<Line> = changes.into_iter().map(|change| (change, RED)).collect();
        entries.push(entry(MODIFY_CONFLICT_TITLE, Status::Warning, details));
    }

    entries
}

fn summary(entries: &[Entry]) -> String
{
    let parts: Vec<String> = STATUSES
        .iter()
        .filter_map(|row|
        {
            let count: usize = entries.iter().filter(|entry| entry.m_status == row.m_status).count();
            (count > 0).then(|| format!("{count} {}", row.m_summary))
        })
        .collect();

    parts.join(", ")
}

// the wheel arrives as a release event and scrolls like the vanilla scroll buttons
pub(crate) fn scrollWheel(slot: &Rc<GuiSlot>, wheel: i32, up: i32, down: i32)
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

fn select(this: &GuiScreen, index: i32)
{
    let state: &ModList = list(this);
    state.m_selected.set(index);

    if let Some(button) = state.m_configButton.get()
    {
        button.m_enabled.set(selectedConfig(this).is_some());
    }
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
    let [titleY, previewY] = listing::LINE_Y;
    this.m_gui.drawString(&font, &entry.m_shownTitle, x, y + titleY, WHITE);

    let labelX: i32 = x + textWidth(this.m_width.get()) - font.getStringWidth(row.m_label);
    this.m_gui.drawString(&font, row.m_label, labelX, y + titleY, row.m_color);

    let (preview, color): &Line = &entry.m_preview;
    this.m_gui.drawString(&font, preview, x, y + previewY, *color);
}

fn drawInfo(this: &GuiScreen, font: &FontRenderer)
{
    let state: &ModList = list(this);
    let (width, height): (i32, i32) = (this.m_width.get(), this.m_height.get());
    let x: i32 = width / 2 - rowWidth(width, listing::ROW_MAX_WIDTH) / 2 + text::ROW_INSET;
    let top: i32 = height - listing::INFO_Y;
    let entries: Ref<'_, Vec<Entry>> = state.m_entries.borrow();

    let Some(entry) = usize::try_from(state.m_selected.get()).ok().and_then(|index| entries.get(index))
    else
    {
        this.m_gui.drawCenteredString(font, SELECT_HINT, width / 2, top, DARK_GREY);
        return;
    };

    for (line, (text, color)) in (0..).zip(&entry.m_info)
    {
        this.m_gui.drawString(font, text, x, top + line * text::LINE_HEIGHT, *color);
    }
}

const MOD_SLOT: GuiSlotClass = GuiSlotClass
{
    m_name: "GuiModSlot",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    getSize: |slot| list(&screen(slot)).m_entries.borrow().len() as i32,
    elementClicked: |slot, index, doubleClick|
    {
        let this: Rc<GuiScreen> = screen(slot);
        select(&this, index);

        if doubleClick
        {
            openConfig(&this);
        }
    },
    isSelected: |slot, index| list(&screen(slot)).m_selected.get() == index,
    drawBackground: |slot| screen(slot).drawDefaultBackground(),
    drawSlot,
    ..GUI_SLOT
};

const MOD_LIST: GuiScreenClass = GuiScreenClass
{
    m_name: "GuiModList",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this|
    {
        let state: &ModList = list(this);
        let (width, height): (i32, i32) = (this.m_width.get(), this.m_height.get());
        let font: Rc<FontRenderer> = this.fontRenderer();
        let mut found: Vec<Entry> = entries();

        for entry in &mut found
        {
            place(&font, entry, textWidth(width));
        }

        *state.m_summary.borrow_mut() = fit(&font, &summary(&found), width - 2 * text::ROW_MARGIN);
        *state.m_entries.borrow_mut() = found;

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
        slot.setRowWidth(rowWidth(width, listing::ROW_MAX_WIDTH));
        slot.registerScrollButtons(&this.m_controlList, listing::SCROLL_UP, listing::SCROLL_DOWN);
        state.m_slot.set(Some(slot));

        let step: i32 = listing::BUTTON_WIDTH + listing::BUTTON_GAP;
        let left: i32 = width / 2 - step - listing::BUTTON_WIDTH / 2;
        let y: i32 = height - listing::BUTTON_Y;

        let button = |id: i32, column: i32, label: &str|
        {
            GuiButton::withSize(id, left + column * step, y, listing::BUTTON_WIDTH, 20, label)
        };
        let config: Rc<GuiButton> = button(listing::CONFIG, 1, "Config");

        this.m_controlList.add(button(listing::OPEN_FOLDER, 0, "Mods Folder"));
        this.m_controlList.add(Rc::clone(&config));
        this.m_controlList.add(button(listing::DONE, 2, "Done"));
        state.m_configButton.set(Some(config));
        select(this, state.m_selected.get());
    },

    actionPerformed: |this, button|
    {
        if !button.m_enabled.get()
        {
            return;
        }

        match button.m_id.get()
        {
            listing::DONE => this.mc().displayGuiScreen(Some(Rc::clone(&list(this).m_parent))),
            listing::CONFIG => openConfig(this),
            listing::OPEN_FOLDER => openModsFolder(),
            _ => list(this).m_slot.get().orNpe().actionPerformed(button),
        }
    },

    keyTyped: |this, _, key|
    {
        if key == Keyboard::KEY_ESCAPE
        {
            this.mc().displayGuiScreen(Some(Rc::clone(&list(this).m_parent)));
        }
    },

    mouseMovedOrUp: |this, mouseX, mouseY, button|
    {
        let slot: Rc<GuiSlot> = list(this).m_slot.get().orNpe();
        scrollWheel(&slot, Mouse::getEventDWheel(), listing::SCROLL_UP, listing::SCROLL_DOWN);
        (GUI_SCREEN.mouseMovedOrUp)(this, mouseX, mouseY, button);
    },

    drawScreen: |this, mouseX, mouseY, partial|
    {
        let state: &ModList = list(this);
        state.m_slot.get().orNpe().drawScreen(mouseX, mouseY, partial);

        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        this.m_gui.drawCenteredString(&font, "Mods", center, listing::TITLE_Y, WHITE);
        this.m_gui.drawCenteredString(&font, &state.m_summary.borrow(), center, listing::SUMMARY_Y, GREY);
        drawInfo(this, &font);
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, partial);
    },
    ..GUI_SCREEN
};

pub fn modsScreen(parent: &Rc<GuiScreen>) -> Rc<GuiScreen>
{
    let state: ModList = ModList
    {
        m_parent: Rc::clone(parent),
        m_entries: RefCell::new(Vec::new()),
        m_summary: RefCell::new(String::new()),
        m_selected: Cell::new(-1),
        m_slot: Field::new(None),
        m_configButton: Field::new(None),
    };
    GuiScreen::construct(&MOD_LIST, Box::new(state))
}

// the button only appears when mod files are present
pub fn addModsButton(screen: &GuiScreen, x: i32, y: i32)
{
    if modloader::play::present()
    {
        screen.m_controlList.add(GuiButton::withSize(MODS_BUTTON, x, y, 60, 20, "Mods"));
    }
}

// the title screen line above the copyright, the loader version and how many mods loaded
pub fn drawModCount(screen: &GuiScreen, font: &FontRenderer)
{
    if !modloader::play::present()
    {
        return;
    }

    // GuiMainMenu draws the copyright 10 from the bottom and 2 from the right edge
    const BOTTOM: i32 = 20;
    const RIGHT: i32 = 2;

    let count: usize = state::mods().len();
    let noun: &str = if count == 1 { "mod" } else { "mods" };
    let text: String = format!("Modloader v{}. {count} {noun} loaded.", modloader::LOADER_VERSION);
    let x: i32 = screen.m_width.get() - font.getStringWidth(&text) - RIGHT;
    screen.m_gui.drawString(font, &text, x, screen.m_height.get() - BOTTOM, WHITE);
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
