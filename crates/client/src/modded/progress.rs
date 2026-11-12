use crate::gui::{FontRenderer, GUI_SCREEN, GuiButton, GuiScreen, GuiScreenClass};
use game::entity::{Entity, entity_player};
use game::stats::named;
use std::cell::Cell;
use std::rc::Rc;

// mod achievements and mod stats on pages of their own, opened by a button on the vanilla screens
mod layout
{
    pub const ROWS: usize = 12;
    pub const ROW_HEIGHT: i32 = 12;
    pub const TOP: i32 = 40;
    pub const TITLE_Y: i32 = 20;
    pub const HALF_WIDTH: i32 = 150;
    pub const BUTTON_HEIGHT: i32 = 20;
    pub const DONE_HALF_WIDTH: i32 = 100;
    pub const PAGE_WIDTH: i32 = 20;
    pub const GAP: i32 = 6;
    pub const BOTTOM_MARGIN: i32 = 28;
}

mod ids
{
    pub const DONE: i32 = 0;
    pub const PREVIOUS: i32 = 1;
    pub const NEXT: i32 = 2;
}

mod color
{
    pub const TITLE: i32 = 0xFFFFFF;
    pub const DONE: i32 = 0x55FF55;
    pub const LOCKED: i32 = 0x808080;
    pub const VALUE: i32 = 0xFFFFFF;
}

// the vanilla achievement and stats screens open these pages with this button id
pub const OPEN_BUTTON: i32 = 400;

struct Row
{
    m_label: String,
    m_value: String,
    m_color: i32,
}

struct PageExt
{
    m_parent: Rc<GuiScreen>,
    m_title: &'static str,
    m_rows: Vec<Row>,
    m_page: Cell<usize>,
}

fn fields(this: &GuiScreen) -> &PageExt { this.ext::<PageExt>() }

fn pages(this: &GuiScreen) -> usize { fields(this).m_rows.len().div_ceil(layout::ROWS).max(1) }

fn build(this: &GuiScreen)
{
    this.m_controlList.clear();

    let center: i32 = this.m_width.get() / 2;
    let bottom: i32 = this.m_height.get() - layout::BOTTOM_MARGIN;
    this.m_controlList.add(GuiButton::new(ids::DONE, center - layout::DONE_HALF_WIDTH, bottom, "Done"));

    if pages(this) > 1
    {
        let previous: i32 = center - layout::DONE_HALF_WIDTH - layout::PAGE_WIDTH - layout::GAP;
        let next: i32 = center + layout::DONE_HALF_WIDTH + layout::GAP;
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
    m_name: "GuiModProgress",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| build(this),

    actionPerformed: |this, button|
    {
        let ext: &PageExt = fields(this);
        let count: usize = pages(this);
        match button.m_id.get()
        {
            ids::DONE => this.mc().displayGuiScreen(Some(Rc::clone(&ext.m_parent))),
            ids::PREVIOUS => ext.m_page.set((ext.m_page.get() + count - 1) % count),
            ids::NEXT => ext.m_page.set((ext.m_page.get() + 1) % count),
            _ =>
            {}
        }
    },

    drawScreen: |this, mouseX, mouseY, tickDelta|
    {
        let ext: &PageExt = fields(this);
        this.drawDefaultBackground();

        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        this.m_gui.drawCenteredString(&font, ext.m_title, center, layout::TITLE_Y, color::TITLE);

        let rows = ext.m_rows.iter().skip(ext.m_page.get() * layout::ROWS).take(layout::ROWS);
        for (index, row) in rows.enumerate()
        {
            let y: i32 = layout::TOP + index as i32 * layout::ROW_HEIGHT;
            let right: i32 = center + layout::HALF_WIDTH - font.getStringWidth(&row.m_value);
            this.m_gui.drawString(&font, &row.m_label, center - layout::HALF_WIDTH, y, row.m_color);
            this.m_gui.drawString(&font, &row.m_value, right, y, color::VALUE);
        }

        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

fn page(parent: &Rc<GuiScreen>, title: &'static str, rows: Vec<Row>) -> Rc<GuiScreen>
{
    let ext: PageExt = PageExt { m_parent: Rc::clone(parent), m_title: title, m_rows: rows, m_page: Cell::new(0) };
    GuiScreen::construct(&CLASS, Box::new(ext))
}

// every mod achievement, earned ones in green
pub fn achievements(parent: &Rc<GuiScreen>, player: &Entity) -> Rc<GuiScreen>
{
    let rows: Vec<Row> = named::achievements()
        .iter()
        .map(|achievement|
        {
            let earned: bool = named::unlocked(player, &achievement.m_name);
            Row
            {
                m_label: achievement.m_title.clone(),
                m_value: achievement.m_description.clone(),
                m_color: if earned { color::DONE } else { color::LOCKED },
            }
        })
        .collect();
    page(parent, "Mod Achievements", rows)
}

// every stat mods count, by name
pub fn stats(parent: &Rc<GuiScreen>, player: &Entity) -> Rc<GuiScreen>
{
    let rows: Vec<Row> = entity_player::fields(player)
        .m_progress
        .stats()
        .into_iter()
        .map(|(name, value)| Row { m_label: label(&name), m_value: value.to_string(), m_color: color::VALUE })
        .collect();
    page(parent, "Mod Statistics", rows)
}

// content stats read as their kind then the translated name of what they count
fn label(name: &str) -> String
{
    let (kind, id) = name.split_once('/').unwrap_or(("", name));
    let item: String = modloader::content::lang::translate(id).unwrap_or_else(|| id.to_owned());
    match kind
    {
        "" => item,
        kind => format!("{kind} {item}"),
    }
}

pub fn hasAchievements() -> bool { !named::achievements().is_empty() }

pub fn hasStats(player: &Entity) -> bool { !entity_player::fields(player).m_progress.stats().is_empty() }

pub fn button(x: i32, y: i32, width: i32) -> Rc<GuiButton>
{
    GuiButton::withSize(OPEN_BUTTON, x, y, width, layout::BUTTON_HEIGHT, "Mods")
}
