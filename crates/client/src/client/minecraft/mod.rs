mod debug;
mod input;
mod launch;
mod run;
mod startup;
mod worlds;

use crate::client::{Canvas, Frame, MinecraftApplet, Session, ThreadDownloadResources, Timer};
use crate::controller::PlayerController;
use crate::gui::{FontRenderer, GuiAchievement, GuiIngame, GuiScreen, LoadingScreenRenderer};
use crate::model::ModelBiped;
use crate::render::{
    EffectRenderer, EntityRenderer, OpenGlCapsChecker, RenderEngine, RenderGlobal, TextureLavaFX, TexturePackList,
    TextureWaterFX,
};
use crate::settings::{GameSettings, MouseHelper};
use crate::sound::SoundManager;
use crate::stats::StatFileWriter;
use game::entity::Entity;
use game::math::MovingObjectPosition;
use game::stats::StatList;
use game::storage::ISaveFormat;
use game::world::World;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

thread_local! {
    static THE_MINECRAFT: Field<Option<Rc<Minecraft>>> = Field::new(None);
    static FRAME_TIMES: RefCell<Vec<i64>> = RefCell::new(vec![0; 512]);
    static TICK_TIMES: RefCell<Vec<i64>> = RefCell::new(vec![0; 512]);
    static NUM_RECORDED_FRAME_TIMES: Cell<i32> = const { Cell::new(0) };
    static MINECRAFT_DIR: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static MEMORY_RESERVED_FOR_CRASH: RefCell<Vec<u8>> = RefCell::new(vec![0; 0xA00000]);
}

fn atLeastOne(size: i32) -> i32
{
    if size <= 0
    {
        return 1;
    }
    size
}

static HAS_PAID_CHECK_TIME: AtomicI64 = AtomicI64::new(0);

pub struct Minecraft
{
    pub m_playerController: Field<Option<Rc<PlayerController>>>,
    pub m_fullscreen: Cell<bool>,
    pub m_hasCrashed: Cell<bool>,
    pub m_displayWidth: Cell<i32>,
    pub m_displayHeight: Cell<i32>,
    pub m_glCapabilities: Field<Option<Rc<OpenGlCapsChecker>>>,
    pub m_timer: Field<Option<Rc<Timer>>>,
    pub m_theWorld: Field<Option<Rc<World>>>,
    pub m_renderGlobal: Field<Option<Rc<RenderGlobal>>>,
    pub m_thePlayer: Field<Option<Rc<Entity>>>,
    pub m_renderViewEntity: Field<Option<Rc<Entity>>>,
    pub m_effectRenderer: Field<Option<Rc<EffectRenderer>>>,
    pub m_session: Field<Option<Rc<Session>>>,
    pub m_minecraftUri: Field<Option<String>>,
    pub m_mcCanvas: Field<Option<Rc<Canvas>>>,
    pub m_hideQuitButton: Cell<bool>,
    pub m_isGamePaused: AtomicBool,
    pub m_renderEngine: Field<Option<Rc<RenderEngine>>>,
    pub m_fontRenderer: Field<Option<Rc<FontRenderer>>>,
    pub m_currentScreen: Field<Option<Rc<GuiScreen>>>,
    pub m_loadingScreen: Field<Option<Rc<LoadingScreenRenderer>>>,
    pub m_entityRenderer: Field<Option<Rc<EntityRenderer>>>,
    pub m_downloadResourcesThread: Field<Option<Rc<ThreadDownloadResources>>>,
    pub m_ticksRan: Cell<i32>,
    pub m_leftClickCounter: Cell<i32>,
    pub m_tempDisplayWidth: Cell<i32>,
    pub m_tempDisplayHeight: Cell<i32>,
    pub m_guiAchievement: Field<Option<Rc<GuiAchievement>>>,
    pub m_ingameGUI: Field<Option<Rc<GuiIngame>>>,
    pub m_skipRenderWorld: Cell<bool>,
    pub m_humanoidModel: Field<Option<Rc<ModelBiped>>>,
    pub m_objectMouseOver: Field<Option<MovingObjectPosition>>,
    pub m_gameSettings: Field<Option<Rc<GameSettings>>>,
    pub m_mcApplet: Field<Option<Rc<MinecraftApplet>>>,
    pub m_sndManager: Field<Option<Rc<SoundManager>>>,
    pub m_mouseHelper: Field<Option<Rc<MouseHelper>>>,
    pub m_texturePackList: Field<Option<Rc<TexturePackList>>>,
    pub m_mcDataDir: Field<Option<PathBuf>>,
    pub m_saveLoader: Field<Option<Rc<dyn ISaveFormat>>>,
    pub m_statFileWriter: Field<Option<Rc<StatFileWriter>>>,
    pub m_serverName: Field<Option<String>>,
    pub m_serverPort: Cell<i32>,
    pub m_textureWaterFX: Field<Option<Rc<TextureWaterFX>>>,
    pub m_textureLavaFX: Field<Option<Rc<TextureLavaFX>>>,
    pub m_running: AtomicBool,
    pub m_debug: Field<String>,
    pub m_isTakingScreenshot: Cell<bool>,
    pub m_prevFrameTime: Cell<i64>,
    pub m_inGameHasFocus: Cell<bool>,
    pub m_mouseTicksRan: Cell<i32>,
    pub m_isRaining: Cell<bool>,
    pub m_systemTime: Cell<i64>,
    pub m_joinPlayerCounter: Cell<i32>,
    pub m_frame: Field<Option<Rc<Frame>>>,
    m_this: Weak<Minecraft>,
}

impl Minecraft
{
    pub fn theMinecraft() -> Option<Rc<Minecraft>> { THE_MINECRAFT.with(Field::get) }

    pub fn setTheMinecraft(minecraft: Option<Rc<Minecraft>>) { THE_MINECRAFT.with(|instance| instance.set(minecraft)); }

    pub fn hasPaidCheckTime() -> i64 { HAS_PAID_CHECK_TIME.load(Ordering::Relaxed) }

    fn setHasPaidCheckTime(time: i64) { HAS_PAID_CHECK_TIME.store(time, Ordering::Relaxed); }

    fn setNumRecordedFrameTimes(count: i32) { NUM_RECORDED_FRAME_TIMES.with(|frames| frames.set(count)); }

    fn releaseMemoryReservedForCrash() { MEMORY_RESERVED_FOR_CRASH.with(|memory| *memory.borrow_mut() = Vec::new()); }

    pub fn numRecordedFrameTimes() -> i32 { NUM_RECORDED_FRAME_TIMES.with(Cell::get) }

    pub fn minecraftDir() -> Option<PathBuf> { MINECRAFT_DIR.with(|dir| dir.borrow().clone()) }

    fn setMinecraftDir(dir: PathBuf) { MINECRAFT_DIR.with(|current| *current.borrow_mut() = Some(dir)); }

    pub fn frameTimes<R>(body: impl FnOnce(&mut Vec<i64>) -> R) -> R
    {
        FRAME_TIMES.with(|times| body(&mut times.borrow_mut()))
    }

    pub fn tickTimes<R>(body: impl FnOnce(&mut Vec<i64>) -> R) -> R
    {
        TICK_TIMES.with(|times| body(&mut times.borrow_mut()))
    }

    pub fn new(
        canvas: Option<Rc<Canvas>>,
        applet: Option<Rc<MinecraftApplet>>,
        width: i32,
        height: i32,
        fullscreen: bool,
    ) -> Rc<Minecraft>
    {
        let minecraft: Rc<Minecraft> = Rc::new_cyclic(|this| Minecraft {
            m_playerController: Field::new(None),
            m_fullscreen: Cell::new(false),
            m_hasCrashed: Cell::new(false),
            m_displayWidth: Cell::new(0),
            m_displayHeight: Cell::new(0),
            m_glCapabilities: Field::new(None),
            // the windows timer resolution thread is left out because it only sleeps
            m_timer: Field::new(Some(Rc::new(Timer::new(20.0)))),
            m_theWorld: Field::new(None),
            m_renderGlobal: Field::new(None),
            m_thePlayer: Field::new(None),
            m_renderViewEntity: Field::new(None),
            m_effectRenderer: Field::new(None),
            m_session: Field::new(None),
            m_minecraftUri: Field::new(None),
            m_mcCanvas: Field::new(None),
            m_hideQuitButton: Cell::new(true),
            m_isGamePaused: AtomicBool::new(false),
            m_renderEngine: Field::new(None),
            m_fontRenderer: Field::new(None),
            m_currentScreen: Field::new(None),
            m_loadingScreen: Field::new(None),
            m_entityRenderer: Field::new(None),
            m_downloadResourcesThread: Field::new(None),
            m_ticksRan: Cell::new(0),
            m_leftClickCounter: Cell::new(0),
            m_tempDisplayWidth: Cell::new(0),
            m_tempDisplayHeight: Cell::new(0),
            m_guiAchievement: Field::new(None),
            m_ingameGUI: Field::new(None),
            m_skipRenderWorld: Cell::new(false),
            m_humanoidModel: Field::new(Some(Rc::new(ModelBiped::new(0.0, 0.0)))),
            m_objectMouseOver: Field::new(None),
            m_gameSettings: Field::new(None),
            m_mcApplet: Field::new(None),
            m_sndManager: Field::new(Some(Rc::new(SoundManager::new()))),
            m_mouseHelper: Field::new(None),
            m_texturePackList: Field::new(None),
            m_mcDataDir: Field::new(None),
            m_saveLoader: Field::new(None),
            m_statFileWriter: Field::new(None),
            m_serverName: Field::new(None),
            m_serverPort: Cell::new(0),
            m_textureWaterFX: Field::new(Some(Rc::new(TextureWaterFX::new()))),
            m_textureLavaFX: Field::new(Some(Rc::new(TextureLavaFX::new()))),
            m_running: AtomicBool::new(true),
            m_debug: Field::new(String::new()),
            m_isTakingScreenshot: Cell::new(false),
            m_prevFrameTime: Cell::new(-1),
            m_inGameHasFocus: Cell::new(false),
            m_mouseTicksRan: Cell::new(0),
            m_isRaining: Cell::new(false),
            m_systemTime: Cell::new(jvm::system::currentTimeMillis()),
            m_joinPlayerCounter: Cell::new(0),
            m_frame: Field::new(None),
            m_this: this.clone(),
        });
        minecraft.m_loadingScreen.set(Some(Rc::new(LoadingScreenRenderer::new(Rc::clone(&minecraft)))));
        minecraft.m_guiAchievement.set(Some(Rc::new(GuiAchievement::new(Rc::clone(&minecraft)))));
        StatList::init();
        minecraft.m_tempDisplayHeight.set(height);
        minecraft.m_fullscreen.set(fullscreen);
        minecraft.m_mcApplet.set(applet.clone());
        minecraft.m_mcCanvas.set(canvas);
        minecraft.m_displayWidth.set(width);
        minecraft.m_displayHeight.set(height);
        minecraft.m_fullscreen.set(fullscreen);
        if applet.is_none_or(|applet| applet.getParameter("stand-alone").as_deref() == Some("true"))
        {
            minecraft.m_hideQuitButton.set(false);
        }
        Minecraft::setTheMinecraft(Some(Rc::clone(&minecraft)));
        minecraft
    }

    pub fn rc(&self) -> Rc<Minecraft> { self.m_this.upgrade().orNpe() }

    fn gameSettings(&self) -> Rc<GameSettings> { self.m_gameSettings.get().orNpe() }

    fn theWorld(&self) -> Rc<World> { self.m_theWorld.get().orNpe() }

    fn thePlayer(&self) -> Rc<Entity> { self.m_thePlayer.get().orNpe() }

    fn timer(&self) -> Rc<Timer> { self.m_timer.get().orNpe() }

    fn renderEngine(&self) -> Rc<RenderEngine> { self.m_renderEngine.get().orNpe() }

    fn statFileWriter(&self) -> Rc<StatFileWriter> { self.m_statFileWriter.get().orNpe() }

    fn playerController(&self) -> Rc<PlayerController> { self.m_playerController.get().orNpe() }

    fn sndManager(&self) -> Rc<SoundManager> { self.m_sndManager.get().orNpe() }

    fn ingameGUI(&self) -> Rc<GuiIngame> { self.m_ingameGUI.get().orNpe() }

    fn entityRenderer(&self) -> Rc<EntityRenderer> { self.m_entityRenderer.get().orNpe() }

    fn loadingScreen(&self) -> Rc<LoadingScreenRenderer> { self.m_loadingScreen.get().orNpe() }

    fn isGamePaused(&self) -> bool { self.m_isGamePaused.load(Ordering::Relaxed) }

    fn isRunning(&self) -> bool { self.m_running.load(Ordering::Relaxed) }
}
