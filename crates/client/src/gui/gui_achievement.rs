use crate::client::Minecraft;
use crate::gui::{FontRenderer, Gui, ScaledResolution};
use crate::render::{RenderEngine, RenderHelper, RenderItem};
use game::locale::StatCollector;
use game::stats::{Achievement, StatBase};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

pub struct GuiAchievement
{
    pub m_gui: Gui,
    m_theGame: Rc<Minecraft>,
    m_achievementWindowWidth: Cell<i32>,
    m_achievementWindowHeight: Cell<i32>,
    m_title: Field<String>,
    m_description: Field<String>,
    m_theAchievement: Field<Option<Rc<StatBase>>>,
    m_startTime: Cell<i64>,
    m_itemRender: Rc<RenderItem>,
    m_tutorial: Cell<bool>,
}

impl GuiAchievement
{
    pub fn new(minecraft: Rc<Minecraft>) -> GuiAchievement
    {
        GuiAchievement {
            m_gui: Gui::new(),
            m_theGame: minecraft,
            m_achievementWindowWidth: Cell::new(0),
            m_achievementWindowHeight: Cell::new(0),
            m_title: Field::new(String::new()),
            m_description: Field::new(String::new()),
            m_theAchievement: Field::new(None),
            m_startTime: Cell::new(0),
            m_itemRender: Rc::new(RenderItem::new()),
            m_tutorial: Cell::new(false),
        }
    }

    pub fn queueTakenAchievement(&self, achievement: &Rc<StatBase>)
    {
        self.m_title.set(StatCollector::translateToLocal("achievement.get"));
        self.m_description.set(achievement.m_statName.clone());
        self.m_startTime.set(jvm::system::currentTimeMillis());
        self.m_theAchievement.set(Some(Rc::clone(achievement)));
        self.m_tutorial.set(false);
    }

    pub fn queueAchievementInformation(&self, achievement: &Rc<StatBase>)
    {
        self.m_title.set(achievement.m_statName.clone());
        self.m_description.set(Achievement::getDescription(achievement));
        self.m_startTime.set(jvm::system::currentTimeMillis() - 2500);
        self.m_theAchievement.set(Some(Rc::clone(achievement)));
        self.m_tutorial.set(true);
    }

    fn updateAchievementWindowScale(&self)
    {
        let game: &Rc<Minecraft> = &self.m_theGame;
        GL11::glViewport(0, 0, game.m_displayWidth.get(), game.m_displayHeight.get());
        GL11::glMatrixMode(5889);
        GL11::glLoadIdentity();
        GL11::glMatrixMode(5888);
        GL11::glLoadIdentity();
        self.m_achievementWindowWidth.set(game.m_displayWidth.get());
        self.m_achievementWindowHeight.set(game.m_displayHeight.get());
        let scaledResolution: ScaledResolution = ScaledResolution::new(
            &game.m_gameSettings.get().orNpe(),
            game.m_displayWidth.get(),
            game.m_displayHeight.get(),
        );
        self.m_achievementWindowWidth.set(scaledResolution.getScaledWidth());
        self.m_achievementWindowHeight.set(scaledResolution.getScaledHeight());
        GL11::glClear(256);
        GL11::glMatrixMode(5889);
        GL11::glLoadIdentity();
        GL11::glOrtho(
            0.0,
            self.m_achievementWindowWidth.get() as f64,
            self.m_achievementWindowHeight.get() as f64,
            0.0,
            1000.0,
            3000.0,
        );
        GL11::glMatrixMode(5888);
        GL11::glLoadIdentity();
        GL11::glTranslatef(0.0, 0.0, -2000.0);
    }

    pub fn updateAchievementWindow(&self)
    {
        if Minecraft::hasPaidCheckTime() > 0
        {
            let fontRenderer: Rc<FontRenderer> = self.m_theGame.m_fontRenderer.get().orNpe();
            GL11::glDisable(2929);
            GL11::glDepthMask(false);
            RenderHelper::disableStandardItemLighting();
            self.updateAchievementWindowScale();
            fontRenderer.drawStringWithShadow("Minecraft Beta 1.7.3   Unlicensed Copy :(", 2, 2, 0xFFFFFF);
            fontRenderer.drawStringWithShadow("(Or logged in from another location)", 2, 11, 0xFFFFFF);
            fontRenderer.drawStringWithShadow("Purchase at minecraft.net", 2, 20, 0xFFFFFF);
            GL11::glDepthMask(true);
            GL11::glEnable(2929);
        }
        let Some(achievement) = self.m_theAchievement.get()
        else
        {
            return;
        };
        if self.m_startTime.get() == 0
        {
            return;
        }
        let d: f64 = (jvm::system::currentTimeMillis() - self.m_startTime.get()) as f64 / 3000.0;
        if !self.m_tutorial.get() && !(0.0..=1.0).contains(&d)
        {
            self.m_startTime.set(0);
            return;
        }
        self.updateAchievementWindowScale();
        GL11::glDisable(2929);
        GL11::glDepthMask(false);
        let mut d2: f64 = d * 2.0;
        if d2 > 1.0
        {
            d2 = 2.0 - d2;
        }
        d2 *= 4.0;
        d2 = (1.0 - d2).max(0.0);
        d2 *= d2;
        d2 *= d2;
        let n: i32 = self.m_achievementWindowWidth.get() - 160;
        let n2: i32 = -((d2 * 36.0) as i32);
        let renderEngine: Rc<RenderEngine> = self.m_theGame.m_renderEngine.get().orNpe();
        let n3: i32 = renderEngine.getTexture("/achievement/bg.png");
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glEnable(3553);
        GL11::glBindTexture(3553, n3);
        GL11::glDisable(2896);
        self.m_gui.drawTexturedModalRect(n, n2, 96, 202, 160, 32);
        let fontRenderer: Rc<FontRenderer> = self.m_theGame.m_fontRenderer.get().orNpe();
        if self.m_tutorial.get()
        {
            fontRenderer.splitAndDraw(&self.m_description.get(), n + 30, n2 + 7, 120, -1);
        }
        else
        {
            fontRenderer.drawString(&self.m_title.get(), n + 30, n2 + 7, -256);
            fontRenderer.drawString(&self.m_description.get(), n + 30, n2 + 18, -1);
        }
        GL11::glPushMatrix();
        GL11::glRotatef(180.0, 1.0, 0.0, 0.0);
        RenderHelper::enableStandardItemLighting();
        GL11::glPopMatrix();
        GL11::glDisable(2896);
        GL11::glEnable(32826);
        GL11::glEnable(2903);
        GL11::glEnable(2896);
        let icon: Rc<game::item::ItemStack> = Rc::clone(&Achievement::of(&achievement).m_theItemStack);
        self.m_itemRender.renderItemIntoGUI(&fontRenderer, &renderEngine, Some(&icon), n + 8, n2 + 8);
        GL11::glDisable(2896);
        GL11::glDepthMask(true);
        GL11::glEnable(2929);
    }
}
