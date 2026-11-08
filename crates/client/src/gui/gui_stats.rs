use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::gui_slot::{GUI_SLOT, GuiSlotClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen, GuiSlot};
use crate::render::{RenderEngine, RenderHelper, RenderItem, Tessellator};
use crate::stats::StatFileWriter;
use game::item::Item;
use game::locale::{StatCollector, StringTranslate};
use game::port::text;
use game::stats::{StatBase, StatList};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::input::Mouse;
use lwjgl::opengl::GL11;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

thread_local! {
    static ITEM_RENDERER: RenderItem = RenderItem::new();
}

struct StatsExt
{
    m_parent: Rc<GuiScreen>,
    m_title: Field<String>,
    m_generalStats: Field<Option<Rc<GuiSlot>>>,
    m_itemStats: Field<Option<Rc<GuiSlot>>>,
    m_blockStats: Field<Option<Rc<GuiSlot>>>,
    m_stats: Rc<StatFileWriter>,
    m_selectedStatsList: Field<Option<Rc<GuiSlot>>>,
}

fn fields(this: &GuiScreen) -> &StatsExt { this.ext::<StatsExt>() }

fn rowColor(index: i32) -> i32 { if index % 2 == 0 { 16777215 } else { 9474192 } }

fn renderSlot(this: &GuiScreen, x: i32, y: i32, u: i32, v: i32)
{
    let renderEngine: Rc<RenderEngine> = this.mc().m_renderEngine.get().orNpe();
    let i: i32 = renderEngine.getTexture("/gui/slot.png");
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    renderEngine.bindTexture(i);
    let tessellator: Rc<Tessellator> = Tessellator::instance();
    let z: f64 = this.m_gui.m_zLevel.get() as f64;
    let uv = |value: i32| (value as f32 * 0.0078125) as f64;
    tessellator.startDrawingQuads();
    tessellator.addVertexWithUV((x + 0) as f64, (y + 18) as f64, z, uv(u + 0), uv(v + 18));
    tessellator.addVertexWithUV((x + 18) as f64, (y + 18) as f64, z, uv(u + 18), uv(v + 18));
    tessellator.addVertexWithUV((x + 18) as f64, (y + 0) as f64, z, uv(u + 18), uv(v + 0));
    tessellator.addVertexWithUV((x + 0) as f64, (y + 0) as f64, z, uv(u + 0), uv(v + 0));
    tessellator.draw();
}

fn renderStatIcon(this: &GuiScreen, x: i32, y: i32, icon: i32)
{
    renderSlot(this, x + 1, y + 1, 0, 0);
    GL11::glEnable(32826);
    GL11::glPushMatrix();
    GL11::glRotatef(180.0, 1.0, 0.0, 0.0);
    RenderHelper::enableStandardItemLighting();
    GL11::glPopMatrix();
    let mc: Rc<Minecraft> = this.mc();
    let iconIndex: i32 = Item::byIdOrNpe(icon).getIconFromDamage(0);
    ITEM_RENDERER.with(|renderer| {
        renderer.drawItemIntoGui(
            &this.fontRenderer(),
            &mc.m_renderEngine.get().orNpe(),
            icon,
            0,
            iconIndex,
            x + 2,
            y + 2,
        );
    });
    RenderHelper::disableStandardItemLighting();
    GL11::glDisable(32826);
}

const HEADER_OFFSETS: [i32; 3] = [115, 165, 215];

#[derive(Clone, Copy)]
enum StatColumn
{
    Mined,
    Crafted,
    Used,
    Broken,
    Entry,
}

fn columnStat(column: StatColumn, entry: &Rc<StatBase>) -> Option<Rc<StatBase>>
{
    let item: i32 = entry.getItem();
    match column
    {
        StatColumn::Mined => StatList::mineBlockStat(item),
        StatColumn::Crafted => StatList::itemsCrafted(item),
        StatColumn::Used => StatList::itemsUsed(item),
        StatColumn::Broken => StatList::itemsBroken(item),
        StatColumn::Entry => Some(Rc::clone(entry)),
    }
}

struct StatsListKind
{
    m_class: &'static GuiSlotClass,
    m_source: fn() -> Vec<Rc<StatBase>>,
    m_filter: [StatColumn; 2],
    m_sortColumns: [StatColumn; 3],
    m_headerIcons: [i32; 3],
    m_columns: [StatColumn; 3],
    m_headers: [&'static str; 3],
}

const BLOCK_STATS: StatsListKind = StatsListKind {
    m_class: &SLOT_STATS_BLOCK,
    m_source: || StatList::mined().toVec(),
    m_filter: [StatColumn::Used, StatColumn::Crafted],
    m_sortColumns: [StatColumn::Crafted, StatColumn::Used, StatColumn::Mined],
    m_headerIcons: [18, 36, 54],
    m_columns: [StatColumn::Crafted, StatColumn::Used, StatColumn::Entry],
    m_headers: ["stat.crafted", "stat.used", "stat.mined"],
};

const ITEM_STATS: StatsListKind = StatsListKind {
    m_class: &SLOT_STATS_ITEM,
    m_source: || StatList::used().toVec(),
    m_filter: [StatColumn::Broken, StatColumn::Crafted],
    m_sortColumns: [StatColumn::Broken, StatColumn::Crafted, StatColumn::Used],
    m_headerIcons: [72, 18, 36],
    m_columns: [StatColumn::Broken, StatColumn::Crafted, StatColumn::Entry],
    m_headers: ["stat.depleted", "stat.crafted", "stat.used"],
};

struct SlotStatsExt
{
    m_screen: Weak<GuiScreen>,
    m_kind: &'static StatsListKind,
    m_clickedIconId: Cell<i32>,
    m_entries: RefCell<Vec<Rc<StatBase>>>,
    m_selectedTab: Cell<i32>,
    m_statSortOrder: Cell<i32>,
}

fn slotFields(slot: &GuiSlot) -> &SlotStatsExt { slot.ext::<SlotStatsExt>() }

fn slotScreen(slot: &GuiSlot) -> Rc<GuiScreen> { slotFields(slot).m_screen.upgrade().orNpe() }

fn compareEntries(slot: &GuiSlot, var1: &Rc<StatBase>, var2: &Rc<StatBase>) -> i32
{
    let ext: &SlotStatsExt = slotFields(slot);
    let j: i32 = var1.getItem();
    let k: i32 = var2.getItem();
    let tab: i32 = ext.m_selectedTab.get();
    if (0..3).contains(&tab)
    {
        let column: StatColumn = ext.m_kind.m_sortColumns[tab as usize];
        let statbase: Option<Rc<StatBase>> = columnStat(column, var1);
        let statbase1: Option<Rc<StatBase>> = columnStat(column, var2);
        if statbase.is_some() || statbase1.is_some()
        {
            let (Some(statbase), Some(statbase1)) = (statbase, statbase1)
            else
            {
                return if columnStat(column, var1).is_none()
                {
                    1
                }
                else
                {
                    -1
                };
            };
            let screen: Rc<GuiScreen> = slotScreen(slot);
            let stats: &StatFileWriter = &fields(&screen).m_stats;
            let l: i32 = stats.writeStat(&statbase);
            let i1: i32 = stats.writeStat(&statbase1);
            if l != i1
            {
                return l.wrapping_sub(i1).wrapping_mul(ext.m_statSortOrder.get());
            }
        }
    }
    j - k
}

fn click(slot: &GuiSlot, buttonDownTime: i32)
{
    let ext: &SlotStatsExt = slotFields(slot);
    if buttonDownTime != ext.m_selectedTab.get()
    {
        ext.m_selectedTab.set(buttonDownTime);
        ext.m_statSortOrder.set(-1);
    }
    else if ext.m_statSortOrder.get() == -1
    {
        ext.m_statSortOrder.set(1);
    }
    else
    {
        ext.m_selectedTab.set(-1);
        ext.m_statSortOrder.set(0);
    }
    let mut entries: Vec<Rc<StatBase>> = ext.m_entries.borrow().clone();
    jvm::sort::sort(&mut entries, &mut |a, b| compareEntries(slot, a, b));
    *ext.m_entries.borrow_mut() = entries;
}

fn renderColumnStat(slot: &GuiSlot, stat: Option<&Rc<StatBase>>, x: i32, y: i32, isRowEven: bool)
{
    let this: Rc<GuiScreen> = slotScreen(slot);
    let font: Rc<FontRenderer> = this.fontRenderer();
    let s: String = match stat
    {
        Some(stat) => stat.format(fields(&this).m_stats.writeStat(stat)),
        None => "-".to_string(),
    };
    this.m_gui.drawString(&font, &s, x - font.getStringWidth(&s), y + 5, if isRowEven { 16777215 } else { 9474192 });
}

fn drawTooltip(this: &GuiScreen, s: &str, mouseX: i32, mouseY: i32)
{
    if s.is_empty()
    {
        return;
    }
    let font: Rc<FontRenderer> = this.fontRenderer();
    let k: i32 = mouseX + 12;
    let l: i32 = mouseY - 12;
    let i1: i32 = font.getStringWidth(s);
    this.m_gui.drawGradientRect(k - 3, l - 3, k + i1 + 3, l + 8 + 3, -1073741824, -1073741824);
    font.drawStringWithShadow(s, k, l, -1);
}

fn renderItemName(slot: &GuiSlot, stat: &Rc<StatBase>, x: i32, z: i32)
{
    let item: Rc<Item> = Item::byIdOrNpe(stat.getItem());
    let name: Option<String> = item.getItemName();
    let s: String = StringTranslate::getInstance().translateNamedKey(name.as_deref().unwrap_or("null"));
    // port trim strips chars up to space like java String.trim
    drawTooltip(&slotScreen(slot), text::trimStr(&s), x, z);
}

fn renderDecorations(slot: &Rc<GuiSlot>, mouseX: i32, mouseY: i32)
{
    if mouseY < slot.m_top || mouseY > slot.m_bottom
    {
        return;
    }
    let this: Rc<GuiScreen> = slotScreen(slot);
    let i: i32 = slot.getEntryAt(mouseX, mouseY);
    let j: i32 = this.m_width.get() / 2 - 92 - 16;
    if i >= 0
    {
        if mouseX < j + 40 || mouseX > j + 40 + 20
        {
            return;
        }
        let statcrafting: Rc<StatBase> = Rc::clone(&slotFields(slot).m_entries.borrow()[i as usize]);
        renderItemName(slot, &statcrafting, mouseX, mouseY);
        return;
    }
    let column: i32 = if mouseX >= j + 115 - 18 && mouseX <= j + 115
    {
        0
    }
    else if mouseX >= j + 165 - 18 && mouseX <= j + 165
    {
        1
    }
    else if mouseX >= j + 215 - 18 && mouseX <= j + 215
    {
        2
    }
    else
    {
        return;
    };
    let header: Option<String> = slot.getColumnHeader(column);
    let s: String = StringTranslate::getInstance().translateKey(header.as_deref().unwrap_or("null"));
    // port trim strips chars up to space like java String.trim
    drawTooltip(&this, text::trimStr(&s), mouseX, mouseY);
}

fn renderHeader(slot: &Rc<GuiSlot>, x: i32, y: i32, _tesselator: &Rc<Tessellator>)
{
    let ext: &SlotStatsExt = slotFields(slot);
    let this: Rc<GuiScreen> = slotScreen(slot);
    if !Mouse::isButtonDown(0)
    {
        ext.m_clickedIconId.set(-1);
    }
    for (column, offset) in HEADER_OFFSETS.iter().enumerate()
    {
        let v: i32 = if ext.m_clickedIconId.get() == column as i32
        {
            0
        }
        else
        {
            18
        };
        renderSlot(&this, x + offset - 18, y + 1, 0, v);
    }
    if ext.m_selectedTab.get() != -1
    {
        let short1: i32 = match ext.m_selectedTab.get()
        {
            1 => 129,
            2 => 179,
            _ => 79,
        };
        let b0: i32 = if ext.m_statSortOrder.get() == 1
        {
            36
        }
        else
        {
            18
        };
        renderSlot(&this, x + short1, y + 1, b0, 0);
    }
}

fn renderKindHeader(slot: &Rc<GuiSlot>, x: i32, y: i32, tesselator: &Rc<Tessellator>)
{
    (SLOT_STATS.renderHeader)(slot, x, y, tesselator);
    let ext: &SlotStatsExt = slotFields(slot);
    let this: Rc<GuiScreen> = slotScreen(slot);
    for (column, offset) in HEADER_OFFSETS.iter().enumerate()
    {
        let u: i32 = ext.m_kind.m_headerIcons[column];
        if ext.m_clickedIconId.get() == column as i32
        {
            renderSlot(&this, x + offset - 18 + 1, y + 1 + 1, u, 18);
        }
        else
        {
            renderSlot(&this, x + offset - 18, y + 1, u, 18);
        }
    }
}

fn getKindColumnHeader(slot: &Rc<GuiSlot>, column: i32) -> Option<String>
{
    let headers: &[&str; 3] = &slotFields(slot).m_kind.m_headers;
    Some(headers[(column as usize).min(2)].to_string())
}

fn drawStatsSlot(slot: &Rc<GuiSlot>, index: i32, x: i32, y: i32, _height: i32, _tesselator: &Rc<Tessellator>)
{
    let ext: &SlotStatsExt = slotFields(slot);
    let statcrafting: Rc<StatBase> = Rc::clone(&ext.m_entries.borrow()[index as usize]);
    renderStatIcon(&slotScreen(slot), x + 40, y, statcrafting.getItem());
    for (column, offset) in ext.m_kind.m_columns.iter().zip(HEADER_OFFSETS)
    {
        renderColumnStat(slot, columnStat(*column, &statcrafting).as_ref(), x + offset, y, index % 2 == 0);
    }
}

const SLOT_STATS: GuiSlotClass = GuiSlotClass {
    m_name: "GuiStats$GuiSlotStats",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    elementClicked: |_, _, _| {},
    isSelected: |_, _| false,
    drawBackground: |slot| slotScreen(slot).drawDefaultBackground(),
    renderHeader,
    headerClicked: |slot, x, _y| {
        let ext: &SlotStatsExt = slotFields(slot);
        let clicked: i32 = match x
        {
            79..=114 => 0,
            129..=164 => 1,
            179..=214 => 2,
            _ => -1,
        };
        ext.m_clickedIconId.set(clicked);
        if clicked >= 0
        {
            click(slot, clicked);
            slotScreen(slot).mc().m_sndManager.get().orNpe().playSoundFX("random.click", 1.0, 1.0);
        }
    },
    getSize: |slot| slotFields(slot).m_entries.borrow().len() as i32,
    renderDecorations,
    ..GUI_SLOT
};

const SLOT_STATS_BLOCK: GuiSlotClass = GuiSlotClass {
    m_name: "GuiSlotStatsBlock",
    m_parent: Some(&SLOT_STATS),
    m_interfaces: &[],
    renderHeader: renderKindHeader,
    drawSlot: drawStatsSlot,
    getColumnHeader: getKindColumnHeader,
    ..SLOT_STATS
};

const SLOT_STATS_ITEM: GuiSlotClass = GuiSlotClass {
    m_name: "GuiSlotStatsItem",
    m_parent: Some(&SLOT_STATS),
    m_interfaces: &[],
    renderHeader: renderKindHeader,
    drawSlot: drawStatsSlot,
    getColumnHeader: getKindColumnHeader,
    ..SLOT_STATS
};

fn newSlotStats(this: &GuiScreen, kind: &'static StatsListKind) -> Rc<GuiSlot>
{
    let stats: &StatFileWriter = &fields(this).m_stats;
    let mut entries: Vec<Rc<StatBase>> = Vec::new();
    for statcrafting in (kind.m_source)()
    {
        let flag: bool = stats.writeStat(&statcrafting) > 0
            || kind
                .m_filter
                .iter()
                .any(|column| columnStat(*column, &statcrafting).is_some_and(|stat| stats.writeStat(&stat) > 0));
        if flag
        {
            entries.push(statcrafting);
        }
    }
    let ext: SlotStatsExt = SlotStatsExt {
        m_screen: Rc::downgrade(&this.rc()),
        m_kind: kind,
        m_clickedIconId: Cell::new(-1),
        m_entries: RefCell::new(entries),
        m_selectedTab: Cell::new(-1),
        m_statSortOrder: Cell::new(0),
    };
    let slot: Rc<GuiSlot> = GuiSlot::construct(
        kind.m_class,
        Box::new(ext),
        this.mc(),
        this.m_width.get(),
        this.m_height.get(),
        32,
        this.m_height.get() - 64,
        20,
    );
    slot.setRenderSelectionHighlight(false);
    slot.setHeader(true, 20);
    slot
}

struct SlotGeneralExt
{
    m_screen: Weak<GuiScreen>,
}

fn generalScreen(slot: &GuiSlot) -> Rc<GuiScreen> { slot.ext::<SlotGeneralExt>().m_screen.upgrade().orNpe() }

const SLOT_STATS_GENERAL: GuiSlotClass = GuiSlotClass {
    m_name: "GuiSlotStatsGeneral",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    getSize: |_| StatList::general().size(),
    elementClicked: |_, _, _| {},
    isSelected: |_, _| false,
    getContentHeight: |slot| slot.getSize() * 10,
    drawBackground: |slot| generalScreen(slot).drawDefaultBackground(),
    drawSlot: |slot, index, x, y, _, _| {
        let this: Rc<GuiScreen> = generalScreen(slot);
        let font: Rc<FontRenderer> = this.fontRenderer();
        let statbase: Rc<StatBase> = StatList::general().get(index);
        this.m_gui.drawString(&font, &statbase.m_statName, x + 2, y + 1, rowColor(index));
        let s: String = statbase.format(fields(&this).m_stats.writeStat(&statbase));
        this.m_gui.drawString(&font, &s, x + 2 + 213 - font.getStringWidth(&s), y + 1, rowColor(index));
    },
    ..GUI_SLOT
};

fn selectedList(this: &GuiScreen) -> Rc<GuiSlot> { fields(this).m_selectedStatsList.get().orNpe() }

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiStats",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let ext: &StatsExt = fields(this);
        ext.m_title.set(StatCollector::translateToLocal("gui.stats"));
        let general: Rc<GuiSlot> = GuiSlot::construct(
            &SLOT_STATS_GENERAL,
            Box::new(SlotGeneralExt { m_screen: Rc::downgrade(&this.rc()) }),
            this.mc(),
            this.m_width.get(),
            this.m_height.get(),
            32,
            this.m_height.get() - 64,
            10,
        );
        general.setRenderSelectionHighlight(false);
        general.registerScrollButtons(&this.m_controlList, 1, 1);
        ext.m_generalStats.set(Some(Rc::clone(&general)));
        let items: Rc<GuiSlot> = newSlotStats(this, &ITEM_STATS);
        items.registerScrollButtons(&this.m_controlList, 1, 1);
        ext.m_itemStats.set(Some(items));
        let blocks: Rc<GuiSlot> = newSlotStats(this, &BLOCK_STATS);
        blocks.registerScrollButtons(&this.m_controlList, 1, 1);
        ext.m_blockStats.set(Some(blocks));
        ext.m_selectedStatsList.set(Some(general));
        createButtons(this);
    },
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        let ext: &StatsExt = fields(this);
        match button.m_id.get()
        {
            0 => this.mc().displayGuiScreen(Some(Rc::clone(&ext.m_parent))),
            1 => ext.m_selectedStatsList.set(ext.m_generalStats.get()),
            3 => ext.m_selectedStatsList.set(ext.m_itemStats.get()),
            2 => ext.m_selectedStatsList.set(ext.m_blockStats.get()),
            _ => selectedList(this).actionPerformed(button),
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        selectedList(this).drawScreen(mouseX, mouseY, tickDelta);
        this.m_gui.drawCenteredString(
            &this.fontRenderer(),
            &fields(this).m_title.get(),
            this.m_width.get() / 2,
            20,
            16777215,
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub fn createButtons(this: &GuiScreen)
{
    let ext: &StatsExt = fields(this);
    let stringtranslate: Rc<StringTranslate> = StringTranslate::getInstance();
    let center: i32 = this.m_width.get() / 2;
    let height: i32 = this.m_height.get();
    this.m_controlList.add(GuiButton::withSize(
        0,
        center + 4,
        height - 28,
        150,
        20,
        &stringtranslate.translateKey("gui.done"),
    ));
    this.m_controlList.add(GuiButton::withSize(
        1,
        center - 154,
        height - 52,
        100,
        20,
        &stringtranslate.translateKey("stat.generalButton"),
    ));
    let guibutton: Rc<GuiButton> =
        GuiButton::withSize(2, center - 46, height - 52, 100, 20, &stringtranslate.translateKey("stat.blocksButton"));
    this.m_controlList.add(Rc::clone(&guibutton));
    let guibutton1: Rc<GuiButton> =
        GuiButton::withSize(3, center + 62, height - 52, 100, 20, &stringtranslate.translateKey("stat.itemsButton"));
    this.m_controlList.add(Rc::clone(&guibutton1));
    if ext.m_blockStats.get().orNpe().getSize() == 0
    {
        guibutton.m_enabled.set(false);
    }
    if ext.m_itemStats.get().orNpe().getSize() == 0
    {
        guibutton1.m_enabled.set(false);
    }
}

pub struct GuiStats;

impl GuiStats
{
    pub fn new(parent: Rc<GuiScreen>, stats: Rc<StatFileWriter>) -> Rc<GuiScreen>
    {
        let ext: StatsExt = StatsExt {
            m_parent: parent,
            m_title: Field::new("Select world".to_string()),
            m_generalStats: Field::new(None),
            m_itemStats: Field::new(None),
            m_blockStats: Field::new(None),
            m_stats: stats,
            m_selectedStatsList: Field::new(None),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
