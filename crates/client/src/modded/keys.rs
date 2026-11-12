use crate::gui::{FontRenderer, GUI_SCREEN, GuiButton, GuiScreen, GuiScreenClass};
use lwjgl::input::Keyboard;
use modloader::keybinds::{self, Keybind};
use std::cell::Cell;
use std::rc::Rc;

// laid out as the vanilla controls screen lays out its keys, two columns of five rows a page
mod layout
{
    pub const PER_PAGE: usize = 10;
    pub const COLUMNS: i32 = 2;
    pub const COLUMN_WIDTH: i32 = 160;
    pub const ROW_HEIGHT: i32 = 24;
    pub const KEY_WIDTH: i32 = 70;
    pub const BUTTON_HEIGHT: i32 = 20;
    pub const LIST_HALF_WIDTH: i32 = 155;
    pub const LABEL_GAP: i32 = 6;
    pub const LABEL_DROP: i32 = 7;
    pub const BOTTOM_ROW: i32 = 168;
    pub const PAGE_WIDTH: i32 = 20;
    pub const DONE_HALF_WIDTH: i32 = 100;
    pub const TITLE_Y: i32 = 20;
}

mod ids
{
    pub const DONE: i32 = 200;
    pub const PREVIOUS: i32 = 201;
    pub const NEXT: i32 = 202;
}

const WHITE: i32 = 0xFFFFFF;
// the vanilla controls labels draw in -1, opaque white
const LABEL: i32 = -1;
const CONFLICT: i32 = 0xFF5555;
const TITLE: &str = "Mod Controls";
// the controls screen button that opens this page, only shown while a mod has keybinds
pub const OPEN_BUTTON: i32 = 300;

struct KeysExt
{
    m_parent: Rc<GuiScreen>,
    m_page: Cell<usize>,
    m_waiting: Cell<Option<usize>>,
}

fn fields(this: &GuiScreen) -> &KeysExt { this.ext::<KeysExt>() }

fn binds() -> Vec<Keybind> { keybinds::all() }

fn pageBinds(this: &GuiScreen) -> Vec<Keybind>
{
    binds().into_iter().skip(fields(this).m_page.get() * layout::PER_PAGE).take(layout::PER_PAGE).collect()
}

fn left(this: &GuiScreen) -> i32 { this.m_width.get() / 2 - layout::LIST_HALF_WIDTH }

fn slot(this: &GuiScreen, index: usize) -> (i32, i32)
{
    let index: i32 = index as i32;
    let x: i32 = left(this) + index % layout::COLUMNS * layout::COLUMN_WIDTH;
    let y: i32 = this.m_height.get() / 6 + layout::ROW_HEIGHT * (index / layout::COLUMNS);
    (x, y)
}

fn keyName(key: i32) -> String { Keyboard::getKeyName(key).unwrap_or_else(|| key.to_string()) }

fn pages() -> usize { binds().len().div_ceil(layout::PER_PAGE).max(1) }

fn build(this: &GuiScreen)
{
    this.m_controlList.clear();
    let waiting: Option<usize> = fields(this).m_waiting.get();

    for (index, bind) in pageBinds(this).iter().enumerate()
    {
        let (x, y): (i32, i32) = slot(this, index);
        let label: String = match waiting == Some(index)
        {
            true => format!("> {} <", keyName(bind.m_key)),
            false => keyName(bind.m_key),
        };
        this.m_controlList.add(GuiButton::smallWithSize(
            index as i32,
            x,
            y,
            layout::KEY_WIDTH,
            layout::BUTTON_HEIGHT,
            &label,
        ));
    }

    let bottom: i32 = this.m_height.get() / 6 + layout::BOTTOM_ROW;
    let center: i32 = this.m_width.get() / 2;
    this.m_controlList.add(GuiButton::new(ids::DONE, center - layout::DONE_HALF_WIDTH, bottom, "Done"));

    if pages() > 1
    {
        let previous: i32 = center - layout::DONE_HALF_WIDTH - layout::PAGE_WIDTH - layout::LABEL_GAP;
        let next: i32 = center + layout::DONE_HALF_WIDTH + layout::LABEL_GAP;
        this.m_controlList.add(GuiButton::withSize(
            ids::PREVIOUS,
            previous,
            bottom,
            layout::PAGE_WIDTH,
            layout::BUTTON_HEIGHT,
            "<",
        ));
        this.m_controlList.add(GuiButton::withSize(
            ids::NEXT,
            next,
            bottom,
            layout::PAGE_WIDTH,
            layout::BUTTON_HEIGHT,
            ">",
        ));
    }
}

const CLASS: GuiScreenClass = GuiScreenClass
{
    m_name: "GuiModControls",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| build(this),

    actionPerformed: |this, button|
    {
        let ext: &KeysExt = fields(this);
        match button.m_id.get()
        {
            ids::DONE => this.mc().displayGuiScreen(Some(Rc::clone(&ext.m_parent))),
            ids::PREVIOUS => ext.m_page.set((ext.m_page.get() + pages() - 1) % pages()),
            ids::NEXT => ext.m_page.set((ext.m_page.get() + 1) % pages()),
            index => ext.m_waiting.set(usize::try_from(index).ok()),
        }

        build(this);
    },

    keyTyped: |this, chr, key|
    {
        let ext: &KeysExt = fields(this);
        let Some(index) = ext.m_waiting.take()
        else
        {
            (GUI_SCREEN.keyTyped)(this, chr, key);
            return;
        };

        if let Some(bind) = pageBinds(this).get(index)
        {
            keybinds::setKey(&bind.m_name, key);
        }

        build(this);
    },

    drawScreen: |this, mouseX, mouseY, tickDelta|
    {
        this.drawDefaultBackground();

        let font: Rc<FontRenderer> = this.fontRenderer();
        this.m_gui.drawCenteredString(&font, TITLE, this.m_width.get() / 2, layout::TITLE_Y, WHITE);

        let conflicted: Vec<String> =
            keybinds::conflicts().into_iter().flat_map(|(first, second)| [first, second]).collect();

        for (index, bind) in pageBinds(this).iter().enumerate()
        {
            let (x, y): (i32, i32) = slot(this, index);
            // a key two binds share shows red, as later versions mark conflicts
            let color: i32 = if conflicted.contains(&bind.m_name)
            {
                CONFLICT
            }
            else
            {
                LABEL
            };
            let label: String = label(&bind.m_name);
            this.m_gui.drawString(
                &font,
                &label,
                x + layout::KEY_WIDTH + layout::LABEL_GAP,
                y + layout::LABEL_DROP,
                color,
            );
        }

        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

// later versions name a key key.<namespace>.<path> in lang files, a key the mod wrote as its own path also counts
fn label(name: &str) -> String
{
    let (namespace, path): (&str, &str) = name.split_once(':').unwrap_or(("", name));
    modloader::content::lang::translate(&format!("key.{namespace}.{path}"))
        .or_else(|| modloader::content::lang::translate(path))
        .unwrap_or_else(|| readable(path))
}

// a key no lang file names reads as its last word so ic2.key.suit shows as Suit
fn readable(path: &str) -> String
{
    let word: &str = path.rsplit('.').next().unwrap_or(path);
    modloader::config::humanize(word)
}

pub fn screen(parent: &Rc<GuiScreen>) -> Rc<GuiScreen>
{
    let ext: KeysExt = KeysExt { m_parent: Rc::clone(parent), m_page: Cell::new(0), m_waiting: Cell::new(None) };
    GuiScreen::construct(&CLASS, Box::new(ext))
}

// the toggle button right of the vanilla controls screen's done button, never a replacement of the screen
pub fn addButton(screen: &GuiScreen)
{
    let x: i32 = screen.m_width.get() / 2 + layout::DONE_HALF_WIDTH + layout::LABEL_GAP;
    let y: i32 = screen.m_height.get() / 6 + layout::BOTTOM_ROW;

    if !keybinds::all().is_empty()
    {
        screen.m_controlList.add(GuiButton::withSize(
            OPEN_BUTTON,
            x,
            y,
            layout::KEY_WIDTH,
            layout::BUTTON_HEIGHT,
            "Mod Keys",
        ));
    }
}
