use super::atLeastOne;
use crate::client::{Minecraft, ThreadDownloadResources};
use crate::gui::{FontRenderer, GuiConnecting, GuiIngame, GuiMainMenu, ScaledResolution};
use crate::render::{
    EffectRenderer, EntityRenderer, ItemRenderer, OpenGlCapsChecker, RenderEngine, RenderGlobal, RenderManager,
    Tessellator, TextureCompassFX, TextureFX, TextureFlamesFX, TextureLavaFlowFX, TexturePackList, TexturePortalFX,
    TextureWatchFX, TextureWaterFlowFX,
};
use crate::settings::{Component, GameSettings, MouseHelper};
use crate::stats::StatFileWriter;
use game::stats::{Achievement, AchievementList, IStatStringFormat, StatBase};
use game::storage::SaveConverterMcRegion;
use game::worldgen::{ColorizerFoliage, ColorizerGrass, ColorizerWater};
use jvm::throwable::{EXCEPTION, OrNpe, catchThrowable};
use lwjgl::LWJGLException;
use lwjgl::input::{Controllers, Keyboard, Mouse};
use lwjgl::opengl::{Display, DisplayMode, GL11};
use lwjgl::util::glu::GLU;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

// the window title drops the Minecraft name at the user's request
const WINDOW_TITLE: &str = "beta173";
const CREATE_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(1000);

struct InventoryKeyFormat
{
    m_mc: Weak<Minecraft>,
}

impl IStatStringFormat for InventoryKeyFormat
{
    fn formatString(&self, value: &str) -> String
    {
        let mc: Rc<Minecraft> = self.m_mc.upgrade().orNpe();
        let keyName: Option<String> = Keyboard::getKeyName(mc.gameSettings().m_keyBindInventory.m_keyCode.get());
        jvm::format::format(value, &[keyName.unwrap_or_else(|| "null".to_owned())])
    }
}

impl Minecraft
{
    pub fn startGame(&self) -> Result<(), LWJGLException>
    {
        let this: Rc<Minecraft> = self.rc();
        if let Some(canvas) = self.m_mcCanvas.get()
        {
            // the awt canvas is a stand-in so the black fill before attaching is skipped
            Display::setParent(Some(&*canvas))?;
            // lwjgl sizes a parented display from the canvas peer which the stand-in lacks
            Display::setDisplayMode(DisplayMode::new(canvas.getWidth(), canvas.getHeight()))?;
        }
        else if self.m_fullscreen.get()
        {
            Display::setFullscreen(true)?;
            self.m_displayWidth.set(atLeastOne(Display::getDisplayMode().getWidth()));
            self.m_displayHeight.set(atLeastOne(Display::getDisplayMode().getHeight()));
        }
        else
        {
            Display::setDisplayMode(DisplayMode::new(self.m_displayWidth.get(), self.m_displayHeight.get()))?;
        }
        Display::setTitle(WINDOW_TITLE);
        if let Err(exception) = Display::create()
        {
            exception.printStackTrace();
            std::thread::sleep(CREATE_RETRY_DELAY);
            Display::create()?;
        }
        let dataDir: PathBuf = Minecraft::getMinecraftDir();
        self.m_mcDataDir.set(Some(dataDir.clone()));
        self.m_saveLoader.set(Some(Rc::new(SaveConverterMcRegion::new(&dataDir.join("saves")))));
        let settings: Rc<GameSettings> = GameSettings::new(Rc::clone(&this), &dataDir);
        self.m_gameSettings.set(Some(Rc::clone(&settings)));
        let packs: Rc<TexturePackList> = TexturePackList::new(&this, &dataDir);
        self.m_texturePackList.set(Some(Rc::clone(&packs)));
        let engine: Rc<RenderEngine> = RenderEngine::new(&packs, Some(Rc::clone(&settings)));
        self.m_renderEngine.set(Some(Rc::clone(&engine)));
        self.m_fontRenderer.set(Some(Rc::new(FontRenderer::new(&settings, "/font/default.png", &engine))));
        ColorizerWater::set(engine.getColors("/misc/watercolor.png"));
        ColorizerGrass::set(engine.getColors("/misc/grasscolor.png"));
        ColorizerFoliage::set(engine.getColors("/misc/foliagecolor.png"));
        self.m_entityRenderer.set(Some(Rc::new(EntityRenderer::new(&this))));
        RenderManager::instance().m_itemRenderer.set(Some(Rc::new(ItemRenderer::new(Rc::clone(&this)))));
        self.m_statFileWriter.set(Some(Rc::new(StatFileWriter::new(&self.m_session.get().orNpe(), &dataDir))));
        let openInventory: Rc<StatBase> = AchievementList::openInventory().orNpe();
        Achievement::setStatStringFormatter(&openInventory, Rc::new(InventoryKeyFormat { m_mc: Rc::downgrade(&this) }));
        self.loadScreen()?;
        Keyboard::create()?;
        Mouse::create()?;
        let component: Rc<dyn Component> = self.m_mcCanvas.get().orNpe();
        self.m_mouseHelper.set(Some(Rc::new(MouseHelper::new(component))));
        if let Err(exception) = Controllers::create()
        {
            exception.printStackTrace();
        }
        self.checkGLError("Pre startup");
        GL11::glEnable(3553);
        GL11::glShadeModel(7425);
        GL11::glClearDepth(1.0);
        GL11::glEnable(2929);
        GL11::glDepthFunc(515);
        GL11::glEnable(3008);
        GL11::glAlphaFunc(516, 0.1);
        GL11::glCullFace(1029);
        GL11::glMatrixMode(5889);
        GL11::glLoadIdentity();
        GL11::glMatrixMode(5888);
        self.checkGLError("Startup");
        self.m_glCapabilities.set(Some(Rc::new(OpenGlCapsChecker)));
        self.sndManager().loadSoundSettings(Some(Rc::clone(&settings)));
        let effects: [Rc<dyn TextureFX>; 9] = [
            self.m_textureLavaFX.get().orNpe(),
            self.m_textureWaterFX.get().orNpe(),
            Rc::new(TexturePortalFX::new()),
            Rc::new(TextureCompassFX::new(&this)),
            Rc::new(TextureWatchFX::new(&this)),
            Rc::new(TextureWaterFlowFX::new()),
            Rc::new(TextureLavaFlowFX::new()),
            Rc::new(TextureFlamesFX::new(0)),
            Rc::new(TextureFlamesFX::new(1)),
        ];
        for effect in effects
        {
            engine.registerTextureFX(effect);
        }
        self.m_renderGlobal.set(Some(RenderGlobal::new(&this, &engine)));
        GL11::glViewport(0, 0, self.m_displayWidth.get(), self.m_displayHeight.get());
        self.m_effectRenderer.set(Some(Rc::new(EffectRenderer::new(self.m_theWorld.get(), &engine))));
        self.startDownloadResources(&dataDir);
        self.checkGLError("Post startup");
        self.m_ingameGUI.set(Some(Rc::new(GuiIngame::new(Rc::clone(&this)))));
        match self.m_serverName.get()
        {
            Some(serverName) =>
            {
                self.displayGuiScreen(Some(GuiConnecting::new(&this, &serverName, self.m_serverPort.get())));
            }
            None => self.displayGuiScreen(Some(GuiMainMenu::new())),
        }
        Ok(())
    }

    // the resource download runs on the game thread because it shares game objects
    fn startDownloadResources(&self, dataDir: &std::path::Path)
    {
        let _ = catchThrowable(&EXCEPTION, || {
            let thread: Rc<ThreadDownloadResources> = Rc::new(ThreadDownloadResources::new(dataDir, self.rc()));
            self.m_downloadResourcesThread.set(Some(Rc::clone(&thread)));
            thread.run();
        });
    }

    fn loadScreen(&self) -> Result<(), LWJGLException>
    {
        let scaledResolution: ScaledResolution =
            ScaledResolution::new(&self.gameSettings(), self.m_displayWidth.get(), self.m_displayHeight.get());
        GL11::glClear(16640);
        GL11::glMatrixMode(5889);
        GL11::glLoadIdentity();
        GL11::glOrtho(0.0, scaledResolution.m_scaledWidthD, scaledResolution.m_scaledHeightD, 0.0, 1000.0, 3000.0);
        GL11::glMatrixMode(5888);
        GL11::glLoadIdentity();
        GL11::glTranslatef(0.0, 0.0, -2000.0);
        GL11::glViewport(0, 0, self.m_displayWidth.get(), self.m_displayHeight.get());
        GL11::glClearColor(0.0, 0.0, 0.0, 0.0);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glDisable(2896);
        GL11::glEnable(3553);
        GL11::glDisable(2912);
        GL11::glBindTexture(3553, self.renderEngine().getTexture("/title/mojang.png"));
        let width: f64 = self.m_displayWidth.get() as f64;
        let height: f64 = self.m_displayHeight.get() as f64;
        tessellator.startDrawingQuads();
        tessellator.setColorOpaque_I(0xFFFFFF);
        tessellator.addVertexWithUV(0.0, height, 0.0, 0.0, 0.0);
        tessellator.addVertexWithUV(width, height, 0.0, 0.0, 0.0);
        tessellator.addVertexWithUV(width, 0.0, 0.0, 0.0, 0.0);
        tessellator.addVertexWithUV(0.0, 0.0, 0.0, 0.0, 0.0);
        tessellator.draw();
        let n: i32 = 256;
        let n2: i32 = 256;
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        tessellator.setColorOpaque_I(0xFFFFFF);
        self.draw(
            (scaledResolution.getScaledWidth() - n) / 2,
            (scaledResolution.getScaledHeight() - n2) / 2,
            0,
            0,
            n,
            n2,
        );
        GL11::glDisable(2896);
        GL11::glDisable(2912);
        GL11::glEnable(3008);
        GL11::glAlphaFunc(516, 0.1);
        Display::swapBuffers()
    }

    pub fn draw(&self, x: i32, y: i32, u: i32, v: i32, width: i32, height: i32)
    {
        let f: f32 = 0.00390625;
        let f2: f32 = 0.00390625;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        tessellator.addVertexWithUV(
            (x + 0) as f64,
            (y + height) as f64,
            0.0,
            ((u + 0) as f32 * f) as f64,
            ((v + height) as f32 * f2) as f64,
        );
        tessellator.addVertexWithUV(
            (x + width) as f64,
            (y + height) as f64,
            0.0,
            ((u + width) as f32 * f) as f64,
            ((v + height) as f32 * f2) as f64,
        );
        tessellator.addVertexWithUV(
            (x + width) as f64,
            (y + 0) as f64,
            0.0,
            ((u + width) as f32 * f) as f64,
            ((v + 0) as f32 * f2) as f64,
        );
        tessellator.addVertexWithUV(
            (x + 0) as f64,
            (y + 0) as f64,
            0.0,
            ((u + 0) as f32 * f) as f64,
            ((v + 0) as f32 * f2) as f64,
        );
        tessellator.draw();
    }

    pub(super) fn checkGLError(&self, message: &str)
    {
        let n: i32 = GL11::glGetError();
        if n != 0
        {
            let string: &str = GLU::gluErrorString(n).unwrap_or("null");
            jvm::system::println("########## GL ERROR ##########");
            jvm::system::println(&format!("@ {message}"));
            jvm::system::println(&format!("{n}: {string}"));
        }
    }
}
