use crate::client::Minecraft;
use crate::gui::{FontRenderer, GuiButton, GuiScreen};
use crate::render::RenderEngine;
use crate::stats::StatFileWriter;
use game::item::ItemStack;
use game::stats::named::{self, ModAchievement};
use game::stats::{Achievement, AchievementList, StatBase};
use jvm::throwable::OrNpe;
use std::collections::BTreeSet;
use std::rc::Rc;

// the achievement map as tabs, beta's tree first then one per mod laid out from its parent links
mod layout
{
    // map cells between an achievement and its child and between sibling rows
    pub const STEP: i32 = 2;
    pub const TAB_SIZE: i32 = 20;
    pub const TAB_STEP: i32 = 22;
    pub const TAB_GAP: i32 = 1;
    pub const ICON_INSET: i32 = 2;
    pub const TOOLTIP_X: i32 = 12;
    pub const TOOLTIP_Y: i32 = 4;
    pub const TOOLTIP_PAD: i32 = 3;
    pub const LINE_HEIGHT: i32 = 8;
}

mod color
{
    pub const TOOLTIP: i32 = -1073741824;
    pub const TITLE: i32 = -1;
}

const BETA_TITLE: &str = "Minecraft";

// the first tab's button id, the rest follow in tab order
pub const TAB_BUTTON: i32 = 500;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Source
{
    Beta,
    Mod,
}

pub struct Tab
{
    pub m_title: String,
    pub m_source: Source,
    pub m_achievements: Vec<Rc<StatBase>>,
    // the map opens scrolled to this one as beta opens on taking inventory
    pub m_start: Rc<StatBase>,
    // lowest column and row then highest column and row, beta's bounds always take in zero
    pub m_bounds: [i32; 4],
}

impl Tab
{
    pub fn icon(&self) -> &Rc<ItemStack> { &Achievement::of(&self.m_start).m_theItemStack }

    // beta's tree reads the stats file and a mod's the player's mod progress
    pub fn earned(&self, mc: &Minecraft, stats: &StatFileWriter, achievement: &Rc<StatBase>) -> bool
    {
        match self.m_source
        {
            Source::Beta => stats.hasAchievementUnlocked(achievement),
            Source::Mod =>
            {
                let Some(player) = mc.m_thePlayer.get()
                else
                {
                    return false;
                };

                achievement.m_statGuid.with(|name| name.as_deref().is_some_and(|name| named::unlocked(&player, name)))
            }
        }
    }

    pub fn open(&self, mc: &Minecraft, stats: &StatFileWriter, achievement: &Rc<StatBase>) -> bool
    {
        Achievement::of(achievement).m_parentAchievement.as_ref().is_none_or(|parent| self.earned(mc, stats, parent))
    }
}

// places a mod's achievements as a tree, column by progress depth and one row per leaf
struct Tree<'a>
{
    m_members: &'a [Rc<ModAchievement>],
    m_done: BTreeSet<&'a str>,
    m_placed: Vec<Rc<StatBase>>,
    m_row: i32,
}

impl<'a> Tree<'a>
{
    fn place(&mut self, member: &'a ModAchievement, parent: Option<Rc<StatBase>>, depth: i32)
    {
        if !self.m_done.insert(member.m_name.as_str())
        {
            return;
        }

        let icon: Rc<ItemStack> = ItemStack::withDamage(member.m_icon.get(), 1, 0);
        let cell: [i32; 2] = [depth * layout::STEP, self.m_row];
        let stat: Rc<StatBase> =
            Achievement::placed(member.m_title.clone(), member.m_description.clone(), icon, cell, parent);
        // the registry name lets the map read the player's mod progress
        stat.m_statGuid.set(Some(member.m_name.clone()));
        self.m_placed.push(Rc::clone(&stat));

        let row: i32 = self.m_row;
        let members: &'a [Rc<ModAchievement>] = self.m_members;

        for child in members.iter().filter(|child| child.m_parent.as_deref() == Some(member.m_name.as_str()))
        {
            self.place(child, Some(Rc::clone(&stat)), depth + 1);
        }

        // a leaf takes a row of its own and a parent sits level with its first child
        if self.m_row == row
        {
            self.m_row += layout::STEP;
        }
    }
}

pub fn tabs() -> Vec<Tab>
{
    let all: Vec<Rc<ModAchievement>> = named::achievements();
    let mut namespaces: Vec<&str> = Vec::new();

    for achievement in &all
    {
        let namespace: &str = namespace(&achievement.m_name);
        if !namespaces.contains(&namespace)
        {
            namespaces.push(namespace);
        }
    }

    let mut tabs: Vec<Tab> = vec![beta()];

    for namespace in namespaces
    {
        let members: Vec<Rc<ModAchievement>> =
            all.iter().filter(|achievement| self::namespace(&achievement.m_name) == namespace).cloned().collect();
        tabs.push(modTab(namespace, &members));
    }

    tabs
}

fn namespace(name: &str) -> &str { name.split_once(':').map_or(name, |(namespace, _)| namespace) }

fn beta() -> Tab
{
    let bounds: [i32; 4] = [
        AchievementList::minDisplayColumn(),
        AchievementList::minDisplayRow(),
        AchievementList::maxDisplayColumn(),
        AchievementList::maxDisplayRow(),
    ];

    Tab
    {
        m_title: BETA_TITLE.to_owned(),
        m_source: Source::Beta,
        m_achievements: AchievementList::achievementList().toVec(),
        m_start: AchievementList::openInventory().orNpe(),
        m_bounds: bounds,
    }
}

fn modTab(namespace: &str, members: &[Rc<ModAchievement>]) -> Tab
{
    let names: BTreeSet<&str> = members.iter().map(|member| member.m_name.as_str()).collect();
    let mut tree: Tree = Tree { m_members: members, m_done: BTreeSet::new(), m_placed: Vec::new(), m_row: 0 };

    for member in
        members.iter().filter(|member| member.m_parent.as_deref().is_none_or(|parent| !names.contains(parent)))
    {
        tree.place(member, None, 0);
    }

    // a parent cycle leaves no root so what is left places as roots
    for member in members
    {
        tree.place(member, None, 0);
    }

    let title: String = modloader::state::byId(namespace)
        .map_or_else(|| namespace.to_owned(), |runtime| runtime.m_manifest.displayName().to_owned());
    let start: Rc<StatBase> = tree.m_placed.first().cloned().expect("a mod tab holds at least one achievement");

    Tab
    {
        m_title: title,
        m_source: Source::Mod,
        m_bounds: bounds(&tree.m_placed),
        m_start: start,
        m_achievements: tree.m_placed,
    }
}

fn bounds(achievements: &[Rc<StatBase>]) -> [i32; 4]
{
    achievements.iter().map(|stat| Achievement::of(stat)).fold([0; 4], |[left, top, right, bottom], achievement|
    {
        let [column, row]: [i32; 2] = [achievement.m_displayColumn, achievement.m_displayRow];
        [left.min(column), top.min(row), right.max(column), bottom.max(row)]
    })
}

// tabs line up along the frame's top edge and further rows stack upward
fn tabPosition(index: usize, frame: [i32; 2], width: i32) -> [i32; 2]
{
    let perRow: usize = (width / layout::TAB_STEP) as usize;
    let column: i32 = (index % perRow) as i32;
    let row: i32 = (index / perRow) as i32;
    [frame[0] + column * layout::TAB_STEP, frame[1] - layout::TAB_GAP - layout::TAB_SIZE - row * layout::TAB_STEP]
}

// a lone beta tab needs no strip so the screen stays vanilla without mod achievements
pub fn addTabs(this: &GuiScreen, tabs: &[Tab], selected: usize, frame: [i32; 2], width: i32)
{
    if tabs.len() < 2
    {
        return;
    }

    for index in 0..tabs.len()
    {
        let [x, y]: [i32; 2] = tabPosition(index, frame, width);
        let button: Rc<GuiButton> =
            GuiButton::withSize(TAB_BUTTON + index as i32, x, y, layout::TAB_SIZE, layout::TAB_SIZE, "");
        // the open tab is a disabled button as beta's stats screen marks its open list
        button.m_enabled.set(index != selected);
        this.m_controlList.add(button);
    }
}

pub fn tabOf(id: i32, count: usize) -> Option<usize>
{
    usize::try_from(id - TAB_BUTTON).ok().filter(|index| *index < count)
}

pub fn drawTabs(this: &GuiScreen, tabs: &[Tab], frame: [i32; 2], width: i32, [mouseX, mouseY]: [i32; 2])
{
    if tabs.len() < 2
    {
        return;
    }

    let font: Rc<FontRenderer> = this.fontRenderer();
    let engine: Rc<RenderEngine> = this.mc().m_renderEngine.get().orNpe();
    let mut hovered: Option<&Tab> = None;

    for (index, tab) in tabs.iter().enumerate()
    {
        let [x, y]: [i32; 2] = tabPosition(index, frame, width);
        super::draw::drawItem(&font, &engine, tab.icon(), x + layout::ICON_INSET, y + layout::ICON_INSET);

        let inside: bool = mouseX >= x && mouseY >= y && mouseX < x + layout::TAB_SIZE && mouseY < y + layout::TAB_SIZE;
        if inside
        {
            hovered = Some(tab);
        }
    }

    let Some(tab) = hovered
    else
    {
        return;
    };

    let x: i32 = mouseX + layout::TOOLTIP_X;
    let y: i32 = mouseY - layout::TOOLTIP_Y;
    let right: i32 = x + font.getStringWidth(&tab.m_title) + layout::TOOLTIP_PAD;
    let bottom: i32 = y + layout::LINE_HEIGHT + layout::TOOLTIP_PAD;
    this.m_gui.drawGradientRect(
        x - layout::TOOLTIP_PAD,
        y - layout::TOOLTIP_PAD,
        right,
        bottom,
        color::TOOLTIP,
        color::TOOLTIP,
    );
    font.drawStringWithShadow(&tab.m_title, x, y, color::TITLE);
}
