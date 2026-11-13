use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen};
use crate::modded::achievements::{self, Tab};
use crate::render::{RenderEngine, RenderHelper, RenderItem};
use crate::stats::StatFileWriter;
use game::block::Block;
use game::locale::StatCollector;
use game::math::math_helper;
use game::stats::{Achievement, StatBase};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::input::Mouse;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

// beta's map scroll limits and opening offset in pixels
mod map
{
    pub const CELL: i32 = 24;
    pub const MIN_MARGIN: i32 = 112;
    pub const MAX_MARGIN: i32 = 77;
    pub const START_SPAN: i32 = 141;
    pub const START_SHIFT: i32 = 12;
}

fn minColumn(this: &GuiScreen) -> i32 { tab(this).m_bounds[0] * map::CELL - map::MIN_MARGIN }

fn minRow(this: &GuiScreen) -> i32 { tab(this).m_bounds[1] * map::CELL - map::MIN_MARGIN }

fn maxColumn(this: &GuiScreen) -> i32 { tab(this).m_bounds[2] * map::CELL - map::MAX_MARGIN }

fn maxRow(this: &GuiScreen) -> i32 { tab(this).m_bounds[3] * map::CELL - map::MAX_MARGIN }

struct AchievementsExt
{
    m_iconWidth: i32,
    m_iconHeight: i32,
    m_prevMouseX: Cell<i32>,
    m_prevMouseY: Cell<i32>,
    m_mouseX: Cell<f64>,
    m_mouseY: Cell<f64>,
    m_scaledMouseDx: Cell<f64>,
    m_scaledMouseDy: Cell<f64>,
    m_scrollX: Cell<f64>,
    m_scrollY: Cell<f64>,
    m_scroll: Cell<i32>,
    m_stats: Rc<StatFileWriter>,
    m_tabs: Vec<Tab>,
    m_tab: Cell<usize>,
}

fn fields(this: &GuiScreen) -> &AchievementsExt { this.ext::<AchievementsExt>() }

fn tab(this: &GuiScreen) -> &Tab
{
    let ext: &AchievementsExt = fields(this);
    &ext.m_tabs[ext.m_tab.get()]
}

// as beta opens on taking inventory a tab opens with its start achievement in view
fn startScroll(tab: &Tab) -> [f64; 2]
{
    let details: &Achievement = Achievement::of(&tab.m_start);
    let x: i32 = details.m_displayColumn * map::CELL - map::START_SPAN / 2 - map::START_SHIFT;
    let y: i32 = details.m_displayRow * map::CELL - map::START_SPAN / 2;
    [x as f64, y as f64]
}

fn scrollTo(ext: &AchievementsExt, [x, y]: [f64; 2])
{
    ext.m_mouseX.set(x);
    ext.m_scaledMouseDx.set(x);
    ext.m_scrollX.set(x);
    ext.m_mouseY.set(y);
    ext.m_scaledMouseDy.set(y);
    ext.m_scrollY.set(y);
}

fn frame(this: &GuiScreen) -> [i32; 2]
{
    let ext: &AchievementsExt = fields(this);
    [(this.m_width.get() - ext.m_iconWidth) / 2, (this.m_height.get() - ext.m_iconHeight) / 2]
}

fn closeToGame(mc: &Minecraft)
{
    mc.displayGuiScreen(None);
    mc.setIngameFocus();
}

fn blinkPhase() -> f64
{
    jvm::math::sin((jvm::system::currentTimeMillis() % 600) as f64 / 600.0 * std::f64::consts::PI * 2.0)
}

fn backgroundBlock(j4: i32, depth: i32, random: &Random) -> Rc<Block>
{
    if j4 > 37 || depth == 35
    {
        return Block::bedrock();
    }

    match j4
    {
        22 =>
        {
            if random.nextIntBounded(2) == 0
            {
                Block::oreDiamond()
            }
            else
            {
                Block::oreRedstone()
            }
        }

        10 => Block::oreIron(),
        8 => Block::oreCoal(),
        _ if j4 > 4 => Block::stone(),
        _ if j4 > 0 => Block::dirt(),
        _ => Block::sand(),
    }
}

fn clampScroll(value: i32, min: i32, max: i32) -> i32
{
    let mut value: i32 = value;
    if value < min
    {
        value = min;
    }

    if value >= max
    {
        value = max - 1;
    }

    value
}

fn renderIcons(this: &Rc<GuiScreen>, mouseX: i32, mouseY: i32, tickdelta: f32)
{
    let ext: &AchievementsExt = fields(this);
    let mc: Rc<Minecraft> = this.mc();
    let renderEngine: Rc<RenderEngine> = mc.m_renderEngine.get().orNpe();
    let delta: f64 = tickdelta as f64;
    let i: i32 = clampScroll(
        math_helper::floor_double(ext.m_mouseX.get() + (ext.m_scaledMouseDx.get() - ext.m_mouseX.get()) * delta),
        minColumn(this),
        maxColumn(this),
    );

    let j: i32 = clampScroll(
        math_helper::floor_double(ext.m_mouseY.get() + (ext.m_scaledMouseDy.get() - ext.m_mouseY.get()) * delta),
        minRow(this),
        maxRow(this),
    );

    let k: i32 = renderEngine.getTexture("/terrain.png");
    let l: i32 = renderEngine.getTexture("/achievement/bg.png");
    let i1: i32 = (this.m_width.get() - ext.m_iconWidth) / 2;
    let j1: i32 = (this.m_height.get() - ext.m_iconHeight) / 2;
    let k1: i32 = i1 + 16;
    let l1: i32 = j1 + 17;

    this.m_gui.m_zLevel.set(0.0);
    GL11::glDepthFunc(518);
    GL11::glPushMatrix();
    GL11::glTranslatef(0.0, 0.0, -200.0);
    GL11::glEnable(3553);
    GL11::glDisable(2896);
    GL11::glEnable(32826);
    GL11::glEnable(2903);
    renderEngine.bindTexture(k);

    let i2: i32 = i + 288 >> 4;
    let j2: i32 = j + 288 >> 4;
    let k2: i32 = (i + 288) % 16;
    let l2: i32 = (j + 288) % 16;
    let random: Random = Random::new();
    let mut l3: i32 = 0;
    while l3 * 16 - l2 < 155
    {
        let f: f32 = 0.6 - (j2 + l3) as f32 / 25.0 * 0.3;
        GL11::glColor4f(f, f, f, 1.0);

        let mut i4: i32 = 0;
        while i4 * 16 - k2 < 224
        {
            random.setSeed((1234 + i2 + i4) as i64);
            random.nextInt();

            let j4: i32 = random.nextIntBounded(1 + j2 + l3) + (j2 + l3) / 2;
            let k4: i32 = backgroundBlock(j4, j2 + l3, &random).m_blockIndexInTexture.get();
            this.m_gui.drawTexturedModalRect(k1 + i4 * 16 - k2, l1 + l3 * 16 - l2, k4 % 16 << 4, k4 >> 4 << 4, 16, 16);
            i4 += 1;
        }

        l3 += 1;
    }

    GL11::glEnable(2929);
    GL11::glDepthFunc(515);
    GL11::glDisable(3553);

    let tab: &Tab = tab(this);
    let stats: &StatFileWriter = &ext.m_stats;

    for achievement1 in &tab.m_achievements
    {
        let details: &Achievement = Achievement::of(achievement1);
        if let Some(parent) = &details.m_parentAchievement
        {
            let parentDetails: &Achievement = Achievement::of(parent);
            let k2: i32 = details.m_displayColumn * 24 - i + 11 + k1;
            let l2: i32 = details.m_displayRow * 24 - j + 11 + l1;
            let i3: i32 = parentDetails.m_displayColumn * 24 - i + 11 + k1;
            let j3: i32 = parentDetails.m_displayRow * 24 - j + 11 + l1;
            let flag: bool = tab.earned(&mc, stats, achievement1);
            let flag1: bool = tab.open(&mc, stats, achievement1);
            let k7: i32 = if blinkPhase() > 0.6 { 255 } else { 130 };

            let k3: i32 = if flag
            {
                -9408400
            }
            else if flag1
            {
                65280i32.wrapping_add(k7 << 24)
            }
            else
            {
                -16777216
            };
            this.m_gui.drawHorizontalLine(k2, i3, l2, k3);
            this.m_gui.drawVerticalLine(i3, l2, j3, k3);
        }
    }

    let mut achievement: Option<Rc<StatBase>> = None;
    let renderitem: RenderItem = RenderItem::new();
    let customColor: &std::cell::Cell<bool> = renderitem.useCustomDisplayColor();

    GL11::glPushMatrix();
    GL11::glRotatef(180.0, 1.0, 0.0, 0.0);
    RenderHelper::enableStandardItemLighting();
    GL11::glPopMatrix();
    GL11::glDisable(2896);
    GL11::glEnable(32826);
    GL11::glEnable(2903);

    let fontRenderer: Rc<FontRenderer> = mc.m_fontRenderer.get().orNpe();

    for achievement3 in &tab.m_achievements
    {
        let details: &Achievement = Achievement::of(achievement3);
        let j5: i32 = details.m_displayColumn * 24 - i;
        let k5: i32 = details.m_displayRow * 24 - j;
        if !(j5 >= -24 && k5 >= -24 && j5 <= 224 && k5 <= 155)
        {
            continue;
        }

        let color: f32 = if tab.earned(&mc, stats, achievement3)
        {
            1.0
        }
        else if tab.open(&mc, stats, achievement3)
        {
            if blinkPhase() < 0.6 { 0.6 } else { 0.8 }
        }
        else
        {
            0.3
        };
        GL11::glColor4f(color, color, color, 1.0);
        renderEngine.bindTexture(l);

        let i6: i32 = k1 + j5;
        let k6: i32 = l1 + k5;

        let u: i32 = if Achievement::getSpecial(achievement3)
        {
            26
        }
        else
        {
            0
        };
        this.m_gui.drawTexturedModalRect(i6 - 2, k6 - 2, u, 202, 26, 26);

        if !tab.open(&mc, stats, achievement3)
        {
            let f4: f32 = 0.1;
            GL11::glColor4f(f4, f4, f4, 1.0);
            customColor.set(false);
        }

        GL11::glEnable(2896);
        GL11::glEnable(2884);
        renderitem.renderItemIntoGUI(&fontRenderer, &renderEngine, Some(&details.m_theItemStack), i6 + 3, k6 + 3);
        GL11::glDisable(2896);

        if !tab.open(&mc, stats, achievement3)
        {
            customColor.set(true);
        }

        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);

        if mouseX >= k1
            && mouseY >= l1
            && mouseX < k1 + 224
            && mouseY < l1 + 155
            && mouseX >= i6
            && mouseX <= i6 + 22
            && mouseY >= k6
            && mouseY <= k6 + 22
        {
            achievement = Some(Rc::clone(achievement3));
        }
    }

    GL11::glDisable(2929);
    GL11::glEnable(3042);
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    renderEngine.bindTexture(l);
    this.m_gui.drawTexturedModalRect(i1, j1, 0, 0, ext.m_iconWidth, ext.m_iconHeight);
    GL11::glPopMatrix();
    this.m_gui.m_zLevel.set(0.0);
    GL11::glDepthFunc(515);
    GL11::glDisable(2929);
    GL11::glEnable(3553);
    (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickdelta);

    if let Some(achievement2) = achievement
    {
        drawTooltip(this, &achievement2, mouseX, mouseY);
    }

    GL11::glEnable(2929);
    GL11::glEnable(2896);
    RenderHelper::disableStandardItemLighting();
}

fn setTitle(this: &GuiScreen)
{
    let ext: &AchievementsExt = fields(this);
    let i: i32 = (this.m_width.get() - ext.m_iconWidth) / 2;
    let j: i32 = (this.m_height.get() - ext.m_iconHeight) / 2;
    this.fontRenderer().drawString("Achievements", i + 15, j + 5, 4210752);
}

fn drawTooltip(this: &GuiScreen, achievement2: &Rc<StatBase>, mouseX: i32, mouseY: i32)
{
    let stats: &StatFileWriter = &fields(this).m_stats;
    let tab: &Tab = tab(this);
    let mc: Rc<Minecraft> = this.mc();
    let font: Rc<FontRenderer> = this.fontRenderer();
    let s: &str = &achievement2.m_statName;
    let s1: String = Achievement::getDescription(achievement2);
    let l5: i32 = mouseX + 12;
    let j6: i32 = mouseY - 4;
    let unlocked: bool = tab.open(&mc, stats, achievement2);
    if unlocked
    {
        let l6: i32 = font.getStringWidth(s).max(120);
        let mut j7: i32 = font.splitAndGetHeight(&s1, l6);

        if tab.earned(&mc, stats, achievement2)
        {
            j7 += 12;
        }

        this.m_gui.drawGradientRect(l5 - 3, j6 - 3, l5 + l6 + 3, j6 + j7 + 3 + 12, -1073741824, -1073741824);
        font.splitAndDraw(&s1, l5, j6 + 12, l6, -6250336);

        if tab.earned(&mc, stats, achievement2)
        {
            font.drawStringWithShadow(&StatCollector::translateToLocal("achievement.taken"), l5, j6 + j7 + 4, -7302913);
        }
    }
    else
    {
        let i7: i32 = font.getStringWidth(s).max(120);
        let parent: &Rc<StatBase> = Achievement::of(achievement2).m_parentAchievement.as_ref().orNpe();
        let s2: String =
            StatCollector::translateToLocalFormatted("achievement.requires", std::slice::from_ref(&parent.m_statName));
        let l7: i32 = font.splitAndGetHeight(&s2, i7);
        this.m_gui.drawGradientRect(l5 - 3, j6 - 3, l5 + i7 + 3, j6 + l7 + 12 + 3, -1073741824, -1073741824);
        font.splitAndDraw(&s2, l5, j6 + 12, i7, -9416624);
    }
    let special: bool = Achievement::getSpecial(achievement2);
    let color: i32 = match (unlocked, special)
    {
        (true, true) => -128,
        (true, false) => -1,
        (false, true) => -8355776,
        (false, false) => -8355712,
    };
    font.drawStringWithShadow(s, l5, j6, color);
}

fn dragMap(this: &GuiScreen, mouseX: i32, mouseY: i32)
{
    let ext: &AchievementsExt = fields(this);

    if !Mouse::isButtonDown(0)
    {
        ext.m_scroll.set(0);
        return;
    }

    let k: i32 = (this.m_width.get() - ext.m_iconWidth) / 2 + 8;
    let l: i32 = (this.m_height.get() - ext.m_iconHeight) / 2 + 17;
    if (ext.m_scroll.get() == 0 || ext.m_scroll.get() == 1)
        && mouseX >= k
        && mouseX < k + 224
        && mouseY >= l
        && mouseY < l + 155
    {
        if ext.m_scroll.get() == 0
        {
            ext.m_scroll.set(1);
        }
        else
        {
            ext.m_scaledMouseDx.set(ext.m_scaledMouseDx.get() - (mouseX - ext.m_prevMouseX.get()) as f64);
            ext.m_scaledMouseDy.set(ext.m_scaledMouseDy.get() - (mouseY - ext.m_prevMouseY.get()) as f64);
            ext.m_mouseX.set(ext.m_scaledMouseDx.get());
            ext.m_scrollX.set(ext.m_scaledMouseDx.get());
            ext.m_mouseY.set(ext.m_scaledMouseDy.get());
            ext.m_scrollY.set(ext.m_scaledMouseDy.get());
        }
        ext.m_prevMouseX.set(mouseX);
        ext.m_prevMouseY.set(mouseY);
    }

    if ext.m_scrollX.get() < minColumn(this) as f64
    {
        ext.m_scrollX.set(minColumn(this) as f64);
    }

    if ext.m_scrollY.get() < minRow(this) as f64
    {
        ext.m_scrollY.set(minRow(this) as f64);
    }

    if ext.m_scrollX.get() >= maxColumn(this) as f64
    {
        ext.m_scrollX.set((maxColumn(this) - 1) as f64);
    }

    if ext.m_scrollY.get() >= maxRow(this) as f64
    {
        ext.m_scrollY.set((maxRow(this) - 1) as f64);
    }
}

const CLASS: GuiScreenClass = GuiScreenClass
{
    m_name: "GuiAchievements",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this|
    {
        this.m_controlList.clear();
        this.m_controlList.add(GuiButton::smallWithSize(
            1,
            this.m_width.get() / 2 + 24,
            this.m_height.get() / 2 + 74,
            80,
            20,
            &StatCollector::translateToLocal("gui.done"),
        ));

        // mod achievements each get a tab of their own beside beta's tree
        let ext: &AchievementsExt = fields(this);
        achievements::addTabs(this, &ext.m_tabs, ext.m_tab.get(), frame(this), ext.m_iconWidth);
    },

    actionPerformed: |this, button|
    {
        if button.m_id.get() == 1
        {
            closeToGame(&this.mc());
        }

        let ext: &AchievementsExt = fields(this);
        if let Some(index) = achievements::tabOf(button.m_id.get(), ext.m_tabs.len())
        {
            ext.m_tab.set(index);
            scrollTo(ext, startScroll(tab(this)));
            (CLASS.initGui)(this);
        }

        (GUI_SCREEN.actionPerformed)(this, button);
    },

    keyTyped: |this, chr, key|
    {
        let mc: Rc<Minecraft> = this.mc();
        if key == mc.m_gameSettings.get().orNpe().m_keyBindInventory.m_keyCode.get()
        {
            closeToGame(&mc);
        }
        else
        {
            (GUI_SCREEN.keyTyped)(this, chr, key);
        }
    },

    drawScreen: |this, mouseX, mouseY, tickDelta|
    {
        dragMap(this, mouseX, mouseY);
        this.drawDefaultBackground();
        renderIcons(this, mouseX, mouseY, tickDelta);
        GL11::glDisable(2896);
        GL11::glDisable(2929);
        setTitle(this);

        let ext: &AchievementsExt = fields(this);
        achievements::drawTabs(this, &ext.m_tabs, frame(this), ext.m_iconWidth, [mouseX, mouseY]);
        GL11::glEnable(2896);
        GL11::glEnable(2929);
    },

    updateScreen: |this|
    {
        let ext: &AchievementsExt = fields(this);
        ext.m_mouseX.set(ext.m_scaledMouseDx.get());
        ext.m_mouseY.set(ext.m_scaledMouseDy.get());

        let d0: f64 = ext.m_scrollX.get() - ext.m_scaledMouseDx.get();
        let d1: f64 = ext.m_scrollY.get() - ext.m_scaledMouseDy.get();
        if d0 * d0 + d1 * d1 < 4.0
        {
            ext.m_scaledMouseDx.set(ext.m_scaledMouseDx.get() + d0);
            ext.m_scaledMouseDy.set(ext.m_scaledMouseDy.get() + d1);
        }
        else
        {
            ext.m_scaledMouseDx.set(ext.m_scaledMouseDx.get() + d0 * 0.85);
            ext.m_scaledMouseDy.set(ext.m_scaledMouseDy.get() + d1 * 0.85);
        }
    },
    doesGuiPauseGame: |_| true,
    ..GUI_SCREEN
};

pub struct GuiAchievements;

impl GuiAchievements
{
    pub fn new(stats: Rc<StatFileWriter>) -> Rc<GuiScreen>
    {
        let tabs: Vec<Tab> = achievements::tabs();
        let [x, y]: [f64; 2] = startScroll(&tabs[0]);

        let ext: AchievementsExt = AchievementsExt
        {
            m_iconWidth: 256,
            m_iconHeight: 202,
            m_prevMouseX: Cell::new(0),
            m_prevMouseY: Cell::new(0),
            m_mouseX: Cell::new(x),
            m_mouseY: Cell::new(y),
            m_scaledMouseDx: Cell::new(x),
            m_scaledMouseDy: Cell::new(y),
            m_scrollX: Cell::new(x),
            m_scrollY: Cell::new(y),
            m_scroll: Cell::new(0),
            m_stats: stats,
            m_tabs: tabs,
            m_tab: Cell::new(0),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
