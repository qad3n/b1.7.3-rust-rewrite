use super::atLeastOne;
use crate::client::{MINECRAFT_ERROR, MINECRAFT_EXCEPTION, Minecraft, ScreenShotHelper, UnexpectedThrowable};
use crate::gui::{GuiConflictWarning, GuiErrorScreen};
use crate::render::{CHUNKS_UPDATED, GLAllocation, RenderBlocks, Tessellator};
use crate::settings::Component;
use game::math::{AxisAlignedBB, Vec3D};
use jvm::throwable::{EXCEPTION, OUT_OF_MEMORY, OrNpe, THROWABLE, Throwable, ThrowableClass, catchThrowable, rethrow};
use lwjgl::LWJGLException;
use lwjgl::input::{Keyboard, Mouse};
use lwjgl::opengl::{Display, GL11};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::Ordering;

static LWJGL_EXCEPTION: ThrowableClass =
    ThrowableClass { m_name: "org.lwjgl.LWJGLException", m_parent: Some(&EXCEPTION) };

const INACTIVE_SLEEP: std::time::Duration = std::time::Duration::from_millis(10);

fn lwjglThrowable(exception: &LWJGLException) -> Throwable
{
    Throwable::new(&LWJGL_EXCEPTION, exception.getMessage().map(str::to_owned))
}

struct FrameCounter
{
    m_lastSecond: i64,
    m_frames: i32,
}

impl Minecraft
{
    pub fn onMinecraftCrash(&self, report: UnexpectedThrowable)
    {
        self.m_hasCrashed.set(true);
        self.displayUnexpectedThrowable(&report);
    }

    pub fn setServer(&self, address: &str, port: i32)
    {
        self.m_serverName.set(Some(address.to_owned()));
        self.m_serverPort.set(port);
    }

    pub fn run(&self)
    {
        self.m_running.store(true, Ordering::Relaxed);
        let started: Result<Result<(), LWJGLException>, Throwable> = catchThrowable(&EXCEPTION, || self.startGame());
        if let Some(exception) = match started
        {
            Ok(Ok(())) => None,
            Ok(Err(exception)) => Some(lwjglThrowable(&exception)),
            Err(exception) => Some(exception),
        }
        {
            exception.printStackTrace();
            self.onMinecraftCrash(UnexpectedThrowable::new("Failed to start game".to_owned(), exception));
            return;
        }
        if let Err(throwable) = catchThrowable(&THROWABLE, || self.runLoop())
            && !throwable.instanceOf(&MINECRAFT_ERROR)
        {
            self.cleanHeap();
            throwable.printStackTrace();
            self.onMinecraftCrash(UnexpectedThrowable::new("Unexpected error".to_owned(), throwable));
        }
        self.shutdownMinecraftApplet();
    }

    fn runLoop(&self)
    {
        let mut counter: FrameCounter = FrameCounter { m_lastSecond: jvm::system::currentTimeMillis(), m_frames: 0 };
        while self.isRunning()
        {
            match catchThrowable(&THROWABLE, || self.runFrame(&mut counter))
            {
                Ok(true) =>
                {}
                Ok(false) => break,
                Err(exception) if exception.instanceOf(&MINECRAFT_EXCEPTION) =>
                {
                    self.m_theWorld.set(None);
                    self.changeWorld1(None);
                    self.displayGuiScreen(Some(GuiConflictWarning::new()));
                }
                Err(error) if error.instanceOf(&OUT_OF_MEMORY) =>
                {
                    self.cleanHeap();
                    self.displayGuiScreen(Some(GuiErrorScreen::new()));
                }
                Err(throwable) => rethrow(throwable),
            }
        }
    }

    fn runFrame(&self, counter: &mut FrameCounter) -> bool
    {
        if self.m_mcApplet.get().is_some_and(|applet| !applet.isActive())
        {
            return false;
        }
        AxisAlignedBB::clearBoundingBoxPool();
        Vec3D::initialize();
        match self.m_frame.get()
        {
            Some(frame) => frame.dispatchEvents(self),
            None if self.m_mcCanvas.with(Option::is_none) && Display::isCloseRequested() => self.shutdown(),
            None =>
            {}
        }
        if self.isGamePaused() && self.m_theWorld.with(Option::is_some)
        {
            let f: f32 = self.timer().m_renderPartialTicks.get();
            self.timer().updateTimer();
            self.timer().m_renderPartialTicks.set(f);
        }
        else
        {
            self.timer().updateTimer();
        }
        let l: i64 = jvm::system::nanoTime();
        for _ in 0..self.timer().m_elapsedTicks.get()
        {
            self.m_ticksRan.set(self.m_ticksRan.get() + 1);
            if catchThrowable(&MINECRAFT_EXCEPTION, || self.runTick()).is_err()
            {
                self.m_theWorld.set(None);
                self.changeWorld1(None);
                self.displayGuiScreen(Some(GuiConflictWarning::new()));
            }
        }
        let i1: i64 = jvm::system::nanoTime() - l;
        self.checkGLError("Pre render");
        RenderBlocks::setFancyGrass(self.gameSettings().m_fancyGraphics.get());
        self.sndManager().update(self.m_thePlayer.get().as_ref(), self.timer().m_renderPartialTicks.get());
        GL11::glEnable(3553);
        if let Some(world) = self.m_theWorld.get()
        {
            world.updatingLighting();
        }
        if !Keyboard::isKeyDown(65)
        {
            Display::update();
        }
        if self.m_thePlayer.get().is_some_and(|player| player.isEntityInsideOpaqueBlock())
        {
            self.gameSettings().m_thirdPersonView.set(false);
        }
        if !self.m_skipRenderWorld.get()
        {
            if let Some(controller) = self.m_playerController.get()
            {
                controller.setPartialTime(self.timer().m_renderPartialTicks.get());
            }
            self.entityRenderer().updateCameraAndRender(self.timer().m_renderPartialTicks.get());
        }
        if !Display::isActive()
        {
            if self.m_fullscreen.get()
            {
                self.toggleFullscreen();
            }
            std::thread::sleep(INACTIVE_SLEEP);
        }
        if self.gameSettings().m_showDebugInfo.get()
        {
            self.displayDebugInfo(i1);
        }
        else
        {
            self.m_prevFrameTime.set(jvm::system::nanoTime());
        }
        self.m_guiAchievement.get().orNpe().updateAchievementWindow();
        std::thread::yield_now();
        if Keyboard::isKeyDown(65)
        {
            Display::update();
        }
        self.screenshotListener();
        self.followCanvasSize();
        self.checkGLError("Post render");
        counter.m_frames += 1;
        self.m_isGamePaused.store(
            !self.isMultiplayerWorld() && self.m_currentScreen.get().is_some_and(|screen| screen.doesGuiPauseGame()),
            Ordering::Relaxed,
        );
        while jvm::system::currentTimeMillis() >= counter.m_lastSecond + 1000
        {
            self.m_debug.set(format!(
                "{} fps, {} chunk updates",
                counter.m_frames,
                CHUNKS_UPDATED.load(Ordering::Relaxed)
            ));
            CHUNKS_UPDATED.store(0, Ordering::Relaxed);
            counter.m_lastSecond += 1000;
            counter.m_frames = 0;
        }
        true
    }

    fn followCanvasSize(&self)
    {
        let Some(canvas) = self.m_mcCanvas.get()
        else
        {
            return;
        };
        if self.m_fullscreen.get()
            || (canvas.getWidth() == self.m_displayWidth.get() && canvas.getHeight() == self.m_displayHeight.get())
        {
            return;
        }
        self.m_displayWidth.set(atLeastOne(canvas.getWidth()));
        self.m_displayHeight.set(atLeastOne(canvas.getHeight()));
        self.resize(self.m_displayWidth.get(), self.m_displayHeight.get());
    }

    pub fn cleanHeap(&self)
    {
        let _ = catchThrowable(&THROWABLE, || {
            Minecraft::releaseMemoryReservedForCrash();
            self.m_renderGlobal.get().orNpe().releaseGlLists();
        });
        let _ = catchThrowable(&THROWABLE, || {
            AxisAlignedBB::clearCache();
            Vec3D::clearPool();
        });
        let _ = catchThrowable(&THROWABLE, || self.changeWorld1(None));
    }

    pub fn shutdownMinecraftApplet(&self)
    {
        let result: Result<(), Throwable> = catchThrowable(&THROWABLE, || {
            self.statFileWriter().func_27175_b();
            self.statFileWriter().syncStats();
            if let Some(applet) = self.m_mcApplet.get()
            {
                applet.clearApplet();
            }
            let _ = catchThrowable(&EXCEPTION, || {
                if let Some(thread) = self.m_downloadResourcesThread.get()
                {
                    thread.closeMinecraft();
                }
            });
            jvm::system::println("Stopping!");
            let _ = catchThrowable(&THROWABLE, || self.changeWorld1(None));
            let _ = catchThrowable(&THROWABLE, GLAllocation::deleteTexturesAndDisplayLists);
            self.sndManager().closeMinecraft();
            Mouse::destroy();
            Keyboard::destroy();
        });
        Display::destroy();
        if !self.m_hasCrashed.get()
        {
            std::process::exit(0);
        }
        if let Err(throwable) = result
        {
            rethrow(throwable);
        }
    }

    fn screenshotListener(&self)
    {
        if Keyboard::isKeyDown(60)
        {
            if !self.m_isTakingScreenshot.get()
            {
                self.m_isTakingScreenshot.set(true);
                let dir: Option<PathBuf> = Minecraft::minecraftDir();
                let message: String = ScreenShotHelper::saveScreenshot(
                    dir.as_deref(),
                    self.m_displayWidth.get(),
                    self.m_displayHeight.get(),
                );
                self.ingameGUI().addChatMessage(&message);
            }
        }
        else
        {
            self.m_isTakingScreenshot.set(false);
        }
    }

    fn displayDebugInfo(&self, tickTime: i64)
    {
        let l: i64 = 16666666;
        if self.m_prevFrameTime.get() == -1
        {
            self.m_prevFrameTime.set(jvm::system::nanoTime());
        }
        let l2: i64 = jvm::system::nanoTime();
        let length: i32 = Minecraft::frameTimes(|times| times.len() as i32);
        let recorded: i32 = Minecraft::numRecordedFrameTimes();
        Minecraft::tickTimes(|times| times[(recorded & (length - 1)) as usize] = tickTime);
        Minecraft::frameTimes(|times| times[(recorded & (length - 1)) as usize] = l2 - self.m_prevFrameTime.get());
        Minecraft::setNumRecordedFrameTimes(recorded.wrapping_add(1));
        self.m_prevFrameTime.set(l2);
        GL11::glClear(256);
        GL11::glMatrixMode(5889);
        GL11::glLoadIdentity();
        GL11::glOrtho(0.0, self.m_displayWidth.get() as f64, self.m_displayHeight.get() as f64, 0.0, 1000.0, 3000.0);
        GL11::glMatrixMode(5888);
        GL11::glLoadIdentity();
        GL11::glTranslatef(0.0, 0.0, -2000.0);
        GL11::glLineWidth(1.0);
        GL11::glDisable(3553);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let height: i32 = self.m_displayHeight.get();
        let width: f64 = length as f64;
        tessellator.startDrawing(7);
        let n2: i32 = (l / 200000) as i32;
        tessellator.setColorOpaque_I(0x20000000);
        tessellator.addVertex(0.0, (height - n2) as f64, 0.0);
        tessellator.addVertex(0.0, height as f64, 0.0);
        tessellator.addVertex(width, height as f64, 0.0);
        tessellator.addVertex(width, (height - n2) as f64, 0.0);
        tessellator.setColorOpaque_I(0x20200000);
        tessellator.addVertex(0.0, (height - n2 * 2) as f64, 0.0);
        tessellator.addVertex(0.0, (height - n2) as f64, 0.0);
        tessellator.addVertex(width, (height - n2) as f64, 0.0);
        tessellator.addVertex(width, (height - n2 * 2) as f64, 0.0);
        tessellator.draw();
        let frameTimes: Vec<i64> = Minecraft::frameTimes(|times| times.clone());
        let tickTimes: Vec<i64> = Minecraft::tickTimes(|times| times.clone());
        let l3: i64 = frameTimes.iter().fold(0i64, |sum, time| sum.wrapping_add(*time));
        let n: i32 = (l3 / 200000 / length as i64) as i32;
        tessellator.startDrawing(7);
        tessellator.setColorOpaque_I(0x20400000);
        tessellator.addVertex(0.0, (height - n) as f64, 0.0);
        tessellator.addVertex(0.0, height as f64, 0.0);
        tessellator.addVertex(width, height as f64, 0.0);
        tessellator.addVertex(width, (height - n) as f64, 0.0);
        tessellator.draw();
        tessellator.startDrawing(1);
        let recorded: i32 = Minecraft::numRecordedFrameTimes();
        for i in 0..length
        {
            let n3: i32 = ((i - recorded) & (length - 1)) * 255 / length;
            let mut n4: i32 = n3 * n3 / 255;
            n4 = n4 * n4 / 255;
            if frameTimes[i as usize] > l
            {
                tessellator.setColorOpaque_I(-16777216 + n4 * 65536);
            }
            else
            {
                tessellator.setColorOpaque_I(-16777216 + n4 * 256);
            }
            let l4: i64 = frameTimes[i as usize] / 200000;
            let l5: i64 = tickTimes[i as usize] / 200000;
            let x: f64 = (i as f32 + 0.5) as f64;
            tessellator.addVertex(x, ((height as i64 - l4) as f32 + 0.5) as f64, 0.0);
            tessellator.addVertex(x, (height as f32 + 0.5) as f64, 0.0);
            tessellator.setColorOpaque_I(-16777216 + n4 * 65536 + n4 * 256 + n4 * 1);
            tessellator.addVertex(x, ((height as i64 - l4) as f32 + 0.5) as f64, 0.0);
            tessellator.addVertex(x, ((height as i64 - (l4 - l5)) as f32 + 0.5) as f64, 0.0);
        }
        tessellator.draw();
        GL11::glEnable(3553);
    }

    pub fn shutdown(&self) { self.m_running.store(false, Ordering::Relaxed); }
}
