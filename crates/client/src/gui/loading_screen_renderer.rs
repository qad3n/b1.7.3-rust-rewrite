use crate::client::{MINECRAFT_ERROR, Minecraft};
use crate::gui::{FontRenderer, ScaledResolution};
use crate::render::Tessellator;
use common::world::IProgressUpdate;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::opengl::{Display, GL11};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::Ordering;

pub struct LoadingScreenRenderer
{
    m_stage: Field<String>,
    m_mc: Rc<Minecraft>,
    m_title: Field<String>,
    m_lastTime: Cell<i64>,
    m_noAbort: Cell<bool>,
}

impl LoadingScreenRenderer
{
    pub fn new(minecraft: Rc<Minecraft>) -> LoadingScreenRenderer
    {
        LoadingScreenRenderer {
            m_stage: Field::new(String::new()),
            m_mc: minecraft,
            m_title: Field::new(String::new()),
            m_lastTime: Cell::new(jvm::system::currentTimeMillis()),
            m_noAbort: Cell::new(false),
        }
    }

    fn checkRunning(&self) -> bool
    {
        if self.m_mc.m_running.load(Ordering::SeqCst)
        {
            return true;
        }
        if self.m_noAbort.get()
        {
            return false;
        }
        jvm::throwable::throwBare(&MINECRAFT_ERROR)
    }

    fn setupOrtho(&self) -> ScaledResolution
    {
        let mc: &Rc<Minecraft> = &self.m_mc;
        let scaledResolution: ScaledResolution =
            ScaledResolution::new(&mc.m_gameSettings.get().orNpe(), mc.m_displayWidth.get(), mc.m_displayHeight.get());
        GL11::glClear(256);
        GL11::glMatrixMode(5889);
        GL11::glLoadIdentity();
        GL11::glOrtho(0.0, scaledResolution.m_scaledWidthD, scaledResolution.m_scaledHeightD, 0.0, 100.0, 300.0);
        GL11::glMatrixMode(5888);
        GL11::glLoadIdentity();
        GL11::glTranslatef(0.0, 0.0, -200.0);
        scaledResolution
    }

    pub fn printText(&self, title: &str)
    {
        self.m_noAbort.set(false);
        self.start(title);
    }

    pub fn start(&self, title: &str)
    {
        if !self.checkRunning()
        {
            return;
        }
        self.m_title.set(title.to_owned());
        self.setupOrtho();
    }

    fn drawProgress(&self, percentage: i32)
    {
        if !self.checkRunning()
        {
            return;
        }
        let l: i64 = jvm::system::currentTimeMillis();
        if l - self.m_lastTime.get() < 20
        {
            return;
        }
        self.m_lastTime.set(l);
        let scaledResolution: ScaledResolution = self.setupOrtho();
        let n: i32 = scaledResolution.getScaledWidth();
        let n2: i32 = scaledResolution.getScaledHeight();
        GL11::glClear(16640);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let n3: i32 = self.m_mc.m_renderEngine.get().orNpe().getTexture("/gui/background.png");
        GL11::glBindTexture(3553, n3);
        let f: f32 = 32.0;
        tessellator.startDrawingQuads();
        tessellator.setColorOpaque_I(0x404040);
        tessellator.addVertexWithUV(0.0, n2 as f64, 0.0, 0.0, (n2 as f32 / f) as f64);
        tessellator.addVertexWithUV(n as f64, n2 as f64, 0.0, (n as f32 / f) as f64, (n2 as f32 / f) as f64);
        tessellator.addVertexWithUV(n as f64, 0.0, 0.0, (n as f32 / f) as f64, 0.0);
        tessellator.addVertexWithUV(0.0, 0.0, 0.0, 0.0, 0.0);
        tessellator.draw();
        if percentage >= 0
        {
            let n4: i32 = 100;
            let n5: i32 = 2;
            let n6: i32 = n / 2 - n4 / 2;
            let n7: i32 = n2 / 2 + 16;
            GL11::glDisable(3553);
            tessellator.startDrawingQuads();
            for (color, length) in [(0x808080, n4), (0x80FF80, percentage)]
            {
                tessellator.setColorOpaque_I(color);
                tessellator.addVertex(n6 as f64, n7 as f64, 0.0);
                tessellator.addVertex(n6 as f64, (n7 + n5) as f64, 0.0);
                tessellator.addVertex((n6 + length) as f64, (n7 + n5) as f64, 0.0);
                tessellator.addVertex((n6 + length) as f64, n7 as f64, 0.0);
            }
            tessellator.draw();
            GL11::glEnable(3553);
        }
        let fontRenderer: Rc<FontRenderer> = self.m_mc.m_fontRenderer.get().orNpe();
        let title: String = self.m_title.get();
        let stage: String = self.m_stage.get();
        fontRenderer.drawStringWithShadow(
            &title,
            (n - fontRenderer.getStringWidth(&title)) / 2,
            n2 / 2 - 4 - 16,
            0xFFFFFF,
        );
        fontRenderer.drawStringWithShadow(
            &stage,
            (n - fontRenderer.getStringWidth(&stage)) / 2,
            n2 / 2 - 4 + 8,
            0xFFFFFF,
        );
        Display::update();
        std::thread::yield_now();
    }
}

impl IProgressUpdate for LoadingScreenRenderer
{
    fn progressStartNoAbort(&self, _title: &str)
    {
        self.m_noAbort.set(true);
        self.start(&self.m_title.get());
    }

    fn displayLoadingString(&self, stage: &str)
    {
        if !self.checkRunning()
        {
            return;
        }
        self.m_lastTime.set(0);
        self.m_stage.set(stage.to_owned());
        self.setLoadingProgress(-1);
        self.m_lastTime.set(0);
    }

    fn setLoadingProgress(&self, percentage: i32) { self.drawProgress(percentage); }
}
