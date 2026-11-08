use crate::client::Minecraft;
use crate::gui::{FontRenderer, Gui, GuiButton, GuiParticle};
use crate::render::Tessellator;
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::input::{Keyboard, Mouse};
use lwjgl::opengl::GL11;
use std::any::Any;
use std::cell::Cell;
use std::rc::{Rc, Weak};

game::vtable! {
    GuiScreenClass for GuiScreen {
        fn actionPerformed(button: &Rc<GuiButton>);
        fn deleteWorld(result: bool, id: i32);
        fn doesGuiPauseGame() -> bool;
        fn drawGuiContainerBackgroundLayer(tickDelta: f32);
        fn drawGuiContainerForegroundLayer();
        fn drawScreen(mouseX: i32, mouseY: i32, tickDelta: f32);
        fn initGui();
        fn keyTyped(chr: u16, key: i32);
        fn mouseClicked(mouseX: i32, mouseY: i32, button: i32);
        fn mouseMovedOrUp(mouseX: i32, mouseY: i32, button: i32);
        fn onGuiClosed();
        fn selectNextField();
        fn updateScreen();
    }
}

fn abstractMethod(name: &str) -> !
{
    jvm::throwable::throw(&jvm::throwable::ERROR, &format!("java.lang.AbstractMethodError: {name}"))
}

pub struct GuiScreen
{
    pub m_class: &'static GuiScreenClass,
    m_ext: Box<dyn Any>,
    m_this: Weak<GuiScreen>,
    pub m_gui: Gui,
    pub m_mc: Field<Option<Rc<Minecraft>>>,
    pub m_width: Cell<i32>,
    pub m_height: Cell<i32>,
    pub m_controlList: JArrayList<Rc<GuiButton>>,
    pub m_passEvents: Cell<bool>,
    pub m_fontRenderer: Field<Option<Rc<FontRenderer>>>,
    pub m_particlesGui: Field<Option<Rc<GuiParticle>>>,
    m_selectedButton: Field<Option<Rc<GuiButton>>>,
}

pub const GUI_SCREEN: GuiScreenClass = GuiScreenClass {
    m_name: "GuiScreen",
    m_parent: None,
    m_interfaces: &[],
    actionPerformed: |_, _| {},
    deleteWorld: |_, _, _| {},
    doesGuiPauseGame: |_| true,
    drawGuiContainerBackgroundLayer: |_, _| abstractMethod("drawGuiContainerBackgroundLayer"),
    drawGuiContainerForegroundLayer: |_| abstractMethod("drawGuiContainerForegroundLayer"),
    drawScreen: |this, mouseX, mouseY, _| {
        let mc: Rc<Minecraft> = this.mc();
        for i in 0..this.m_controlList.size()
        {
            this.m_controlList.get(i).drawButton(&mc, mouseX, mouseY);
        }
    },
    initGui: |_| {},
    keyTyped: |this, _, key| {
        if key == 1
        {
            let mc: Rc<Minecraft> = this.mc();
            mc.displayGuiScreen(None);
            mc.setIngameFocus();
        }
    },
    mouseClicked: |this, mouseX, mouseY, button| {
        if button != 0
        {
            return;
        }
        let mc: Rc<Minecraft> = this.mc();
        let mut i: i32 = 0;
        while i < this.m_controlList.size()
        {
            let guibutton: Rc<GuiButton> = this.m_controlList.get(i);
            if guibutton.mousePressed(&mc, mouseX, mouseY)
            {
                this.m_selectedButton.set(Some(Rc::clone(&guibutton)));
                mc.m_sndManager.get().orNpe().playSoundFX("random.click", 1.0, 1.0);
                this.actionPerformed(&guibutton);
            }
            i += 1;
        }
    },
    mouseMovedOrUp: |this, mouseX, mouseY, button| {
        if let Some(selected) = this.m_selectedButton.get()
            && button == 0
        {
            selected.mouseReleased(mouseX, mouseY);
            this.m_selectedButton.set(None);
        }
    },
    onGuiClosed: |_| {},
    selectNextField: |_| {},
    updateScreen: |_| {},
};

impl GuiScreen
{
    pub fn construct(class: &'static GuiScreenClass, ext: Box<dyn Any>) -> Rc<GuiScreen>
    {
        Rc::new_cyclic(|this: &Weak<GuiScreen>| GuiScreen {
            m_class: class,
            m_ext: ext,
            m_this: this.clone(),
            m_gui: Gui::new(),
            m_mc: Field::new(None),
            m_width: Cell::new(0),
            m_height: Cell::new(0),
            m_controlList: JArrayList::new(),
            m_passEvents: Cell::new(false),
            m_fontRenderer: Field::new(None),
            m_particlesGui: Field::new(None),
            m_selectedButton: Field::new(None),
        })
    }

    pub fn new() -> Rc<GuiScreen> { GuiScreen::construct(&GUI_SCREEN, Box::new(())) }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("screen ext matches its class") }

    pub fn rc(&self) -> Rc<GuiScreen> { self.m_this.upgrade().orNpe() }

    pub fn mc(&self) -> Rc<Minecraft> { self.m_mc.get().orNpe() }

    pub fn fontRenderer(&self) -> Rc<FontRenderer> { self.m_fontRenderer.get().orNpe() }

    pub fn getClipboardString() -> Option<String> { None }

    pub fn setWorldAndResolution(self: &Rc<Self>, minecraft: &Rc<Minecraft>, width: i32, height: i32)
    {
        self.m_particlesGui.set(Some(Rc::new(GuiParticle::new(Rc::clone(minecraft)))));
        self.m_mc.set(Some(Rc::clone(minecraft)));
        self.m_fontRenderer.set(minecraft.m_fontRenderer.get());
        self.m_width.set(width);
        self.m_height.set(height);
        self.m_controlList.clear();
        self.initGui();
    }

    pub fn handleInput(self: &Rc<Self>)
    {
        while Mouse::next()
        {
            self.handleMouseInput();
        }
        while Keyboard::next()
        {
            self.handleKeyboardInput();
        }
    }

    pub fn handleMouseInput(self: &Rc<Self>)
    {
        let mc: Rc<Minecraft> = self.mc();
        let n: i32 = Mouse::getEventX() * self.m_width.get() / mc.m_displayWidth.get();
        let n2: i32 = self.m_height.get() - Mouse::getEventY() * self.m_height.get() / mc.m_displayHeight.get() - 1;
        if Mouse::getEventButtonState()
        {
            self.mouseClicked(n, n2, Mouse::getEventButton());
        }
        else
        {
            self.mouseMovedOrUp(n, n2, Mouse::getEventButton());
        }
    }

    pub fn handleKeyboardInput(self: &Rc<Self>)
    {
        if Keyboard::getEventKeyState()
        {
            if Keyboard::getEventKey() == 87
            {
                self.mc().toggleFullscreen();
                return;
            }
            self.keyTyped(Keyboard::getEventCharacter(), Keyboard::getEventKey());
        }
    }

    pub fn drawDefaultBackground(&self) { self.drawWorldBackground(0); }

    pub fn drawWorldBackground(&self, offset: i32)
    {
        if self.mc().m_theWorld.get().is_some()
        {
            self.m_gui.drawGradientRect(0, 0, self.m_width.get(), self.m_height.get(), -1072689136, -804253680);
        }
        else
        {
            self.drawBackground(offset);
        }
    }

    pub fn drawBackground(&self, offset: i32)
    {
        GL11::glDisable(2896);
        GL11::glDisable(2912);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glBindTexture(3553, self.mc().m_renderEngine.get().orNpe().getTexture("/gui/background.png"));
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        let f: f32 = 32.0;
        let (width, height): (i32, i32) = (self.m_width.get(), self.m_height.get());
        let bottom: f64 = (height as f32 / f + offset as f32) as f64;
        tessellator.startDrawingQuads();
        tessellator.setColorOpaque_I(0x404040);
        tessellator.addVertexWithUV(0.0, height as f64, 0.0, 0.0, bottom);
        tessellator.addVertexWithUV(width as f64, height as f64, 0.0, (width as f32 / f) as f64, bottom);
        tessellator.addVertexWithUV(width as f64, 0.0, 0.0, (width as f32 / f) as f64, offset as f64);
        tessellator.addVertexWithUV(0.0, 0.0, 0.0, 0.0, offset as f64);
        tessellator.draw();
    }

    pub fn renderParticlesGui(&self, tickDelta: f32) { self.m_particlesGui.get().orNpe().render(tickDelta); }
}
