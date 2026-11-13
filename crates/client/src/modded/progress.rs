use crate::gui::{FontRenderer, GUI_SCREEN, GuiButton, GuiScreen, GuiScreenClass};
use game::entity::{Entity, entity_player};
use game::item::ItemStack;
use game::stats::named::{self, ModStat};
use lwjgl::input::Mouse;
use std::cell::{Cell, Ref, RefCell};
use std::ops::Range;
use std::rc::Rc;

// mod stats on a page of their own, opened by a button on the vanilla stats screen
mod layout
{
    pub const ROW_HEIGHT: i32 = 12;
    pub const DETAIL_GAP: i32 = 4;
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
    pub const VALUE: i32 = 0xFFFFFF;
    pub const DETAIL: i32 = 0xA0A0A0;
}

// the vanilla stats screen opens the page with this button id
pub const OPEN_BUTTON: i32 = 400;

struct Row
{
    m_label: String,
    m_value: String,
    m_detail: String,
    m_color: i32,
}

struct PageExt
{
    m_parent: Rc<GuiScreen>,
    m_title: &'static str,
    m_rows: Vec<Row>,
    m_page: Cell<usize>,
    // index of the first row on each page, rows wrap their detail so pages hold what fits
    m_starts: RefCell<Vec<usize>>,
}

fn fields(this: &GuiScreen) -> &PageExt { this.ext::<PageExt>() }

fn pages(this: &GuiScreen) -> usize { fields(this).m_starts.borrow().len() }

fn width() -> i32 { layout::HALF_WIDTH * 2 }

fn rowHeight(font: &FontRenderer, row: &Row) -> i32
{
    match row.m_detail.is_empty()
    {
        true => layout::ROW_HEIGHT,
        false => layout::ROW_HEIGHT + font.splitAndGetHeight(&row.m_detail, width()) + layout::DETAIL_GAP,
    }
}

fn paginate(this: &GuiScreen)
{
    let font: Rc<FontRenderer> = this.fontRenderer();
    let room: i32 = this.m_height.get() - layout::TOP - layout::BOTTOM_MARGIN - layout::BUTTON_HEIGHT;
    let mut starts: Vec<usize> = vec![0];
    let mut used: i32 = 0;

    for (index, row) in fields(this).m_rows.iter().enumerate()
    {
        let height: i32 = rowHeight(&font, row);
        if 0 < used && room < used + height
        {
            starts.push(index);
            used = 0;
        }

        used += height;
    }

    let ext: &PageExt = fields(this);
    ext.m_page.set(ext.m_page.get().min(starts.len() - 1));
    ext.m_starts.replace(starts);
}

fn shown(this: &GuiScreen) -> Range<usize>
{
    let ext: &PageExt = fields(this);
    let starts: Ref<Vec<usize>> = ext.m_starts.borrow();
    let page: usize = ext.m_page.get();
    starts[page]..starts.get(page + 1).copied().unwrap_or(ext.m_rows.len())
}

fn build(this: &GuiScreen)
{
    this.m_controlList.clear();
    paginate(this);

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

    // the wheel turns pages without wrapping
    mouseMovedOrUp: |this, mouseX, mouseY, button|
    {
        let ext: &PageExt = fields(this);
        let last: usize = pages(this) - 1;
        match Mouse::getEventDWheel().signum()
        {
            1 => ext.m_page.set(ext.m_page.get().saturating_sub(1)),
            -1 => ext.m_page.set((ext.m_page.get() + 1).min(last)),
            _ =>
            {}
        }

        (GUI_SCREEN.mouseMovedOrUp)(this, mouseX, mouseY, button);
    },

    drawScreen: |this, mouseX, mouseY, tickDelta|
    {
        let ext: &PageExt = fields(this);
        this.drawDefaultBackground();

        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        this.m_gui.drawCenteredString(&font, ext.m_title, center, layout::TITLE_Y, color::TITLE);

        let left: i32 = center - layout::HALF_WIDTH;
        let mut y: i32 = layout::TOP;
        for row in &ext.m_rows[shown(this)]
        {
            let right: i32 = center + layout::HALF_WIDTH - font.getStringWidth(&row.m_value);
            this.m_gui.drawString(&font, &row.m_label, left, y, row.m_color);
            this.m_gui.drawString(&font, &row.m_value, right, y, color::VALUE);
            font.splitAndDraw(&row.m_detail, left, y + layout::ROW_HEIGHT, width(), color::DETAIL);
            y += rowHeight(&font, row);
        }

        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

fn page(parent: &Rc<GuiScreen>, title: &'static str, rows: Vec<Row>) -> Rc<GuiScreen>
{
    let ext: PageExt = PageExt
    {
        m_parent: Rc::clone(parent),
        m_title: title,
        m_rows: rows,
        m_page: Cell::new(0),
        m_starts: RefCell::new(Vec::new()),
    };
    GuiScreen::construct(&CLASS, Box::new(ext))
}

// every registered mod stat as beta lists its general stats, then the mod content this player counted
pub fn stats(parent: &Rc<GuiScreen>, player: &Entity) -> Rc<GuiScreen>
{
    let mut registered: Vec<Rc<ModStat>> = named::stats();
    registered.sort_by(|left, right| left.m_name.cmp(&right.m_name));

    let mut rows: Vec<Row> = registered
        .iter()
        .map(|stat| statRow(stat.m_title.clone(), stat.m_formatter.format(named::value(player, &stat.m_name))))
        .collect();

    for (name, value) in entity_player::fields(player).m_progress.stats()
    {
        if named::custom(&name)
        {
            continue;
        }

        rows.push(statRow(label(&name), value.to_string()));
    }

    page(parent, "Mod Statistics", rows)
}

fn statRow(label: String, value: String) -> Row
{
    Row { m_label: label, m_value: value, m_detail: String::new(), m_color: color::VALUE }
}

// content stats read as their kind then the name of what they count as its tooltip shows it
fn label(name: &str) -> String
{
    let (kind, id): (&str, &str) = name.split_once('/').unwrap_or(("", name));
    let item: String = itemLabel(id).unwrap_or_else(|| id.to_owned());
    format!("{kind} {item}")
}

fn itemLabel(name: &str) -> Option<String>
{
    let id: modloader::Identifier = modloader::Identifier::parse(name).ok()?;
    let numeric: i32 = modloader::registry::with(|registries| registries.itemId(&id))?;
    Some(modloader::viewer::displayName(&ItemStack::withDamage(numeric, 1, 0)))
}

pub fn hasStats(player: &Entity) -> bool
{
    !named::stats().is_empty() || !entity_player::fields(player).m_progress.stats().is_empty()
}

pub fn button(x: i32, y: i32, width: i32) -> Rc<GuiButton>
{
    GuiButton::withSize(OPEN_BUTTON, x, y, width, layout::BUTTON_HEIGHT, "Mods")
}
