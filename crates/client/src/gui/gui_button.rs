use crate::client::Minecraft;
use crate::gui::{FontRenderer, Gui};
use crate::settings::EnumOptions;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

game::vtable! {
    GuiButtonClass for GuiButton {
        fn getHoverState(hovered: bool) -> i32;
        fn mouseDragged(minecraft: &Rc<Minecraft>, mouseX: i32, mouseY: i32);
        fn mousePressed(minecraft: &Rc<Minecraft>, mouseX: i32, mouseY: i32) -> bool;
        fn mouseReleased(mouseX: i32, mouseY: i32);
    }
}

pub struct GuiButton
{
    pub m_class: &'static GuiButtonClass,
    m_ext: Box<dyn Any>,
    pub m_gui: Gui,
    pub m_width: Cell<i32>,
    pub m_height: Cell<i32>,
    pub m_xPosition: Cell<i32>,
    pub m_yPosition: Cell<i32>,
    pub m_displayString: Field<Option<String>>,
    pub m_id: Cell<i32>,
    pub m_enabled: Cell<bool>,
    pub m_enabled2: Cell<bool>,
}

pub const GUI_BUTTON: GuiButtonClass = GuiButtonClass {
    m_name: "GuiButton",
    m_parent: None,
    m_interfaces: &[],
    getHoverState: |this, hovered| {
        if !this.m_enabled.get()
        {
            0
        }
        else if hovered
        {
            2
        }
        else
        {
            1
        }
    },
    mouseDragged: |_, _, _, _| {},
    mousePressed: |this, _, mouseX, mouseY| this.m_enabled.get() && this.contains(mouseX, mouseY),
    mouseReleased: |_, _, _| {},
};

struct SmallButtonExt
{
    m_enumOptions: Option<EnumOptions>,
}

pub const GUI_SMALL_BUTTON: GuiButtonClass =
    GuiButtonClass { m_name: "GuiSmallButton", m_parent: Some(&GUI_BUTTON), m_interfaces: &[], ..GUI_BUTTON };

impl GuiButton
{
    pub fn construct(
        class: &'static GuiButtonClass,
        ext: Box<dyn Any>,
        id: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        message: &str,
    ) -> Rc<GuiButton>
    {
        Rc::new(GuiButton {
            m_class: class,
            m_ext: ext,
            m_gui: Gui::new(),
            m_width: Cell::new(width),
            m_height: Cell::new(height),
            m_xPosition: Cell::new(x),
            m_yPosition: Cell::new(y),
            m_displayString: Field::new(Some(message.to_owned())),
            m_id: Cell::new(id),
            m_enabled: Cell::new(true),
            m_enabled2: Cell::new(true),
        })
    }

    pub fn new(id: i32, x: i32, y: i32, message: &str) -> Rc<GuiButton>
    {
        GuiButton::withSize(id, x, y, 200, 20, message)
    }

    pub fn withSize(id: i32, x: i32, y: i32, width: i32, height: i32, message: &str) -> Rc<GuiButton>
    {
        GuiButton::construct(&GUI_BUTTON, Box::new(()), id, x, y, width, height, message)
    }

    pub fn small(id: i32, x: i32, y: i32, message: &str) -> Rc<GuiButton>
    {
        GuiButton::smallOption(id, x, y, None, message)
    }

    pub fn smallWithSize(id: i32, x: i32, y: i32, width: i32, height: i32, message: &str) -> Rc<GuiButton>
    {
        let ext: Box<SmallButtonExt> = Box::new(SmallButtonExt { m_enumOptions: None });
        GuiButton::construct(&GUI_SMALL_BUTTON, ext, id, x, y, width, height, message)
    }

    pub fn smallOption(id: i32, x: i32, y: i32, option: Option<EnumOptions>, message: &str) -> Rc<GuiButton>
    {
        let ext: Box<SmallButtonExt> = Box::new(SmallButtonExt { m_enumOptions: option });
        GuiButton::construct(&GUI_SMALL_BUTTON, ext, id, x, y, 150, 20, message)
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("button ext matches its class") }

    pub fn returnEnumOptions(&self) -> Option<EnumOptions> { self.ext::<SmallButtonExt>().m_enumOptions }

    fn contains(&self, mouseX: i32, mouseY: i32) -> bool
    {
        mouseX >= self.m_xPosition.get()
            && mouseY >= self.m_yPosition.get()
            && mouseX < self.m_xPosition.get() + self.m_width.get()
            && mouseY < self.m_yPosition.get() + self.m_height.get()
    }

    pub fn drawButton(self: &Rc<Self>, minecraft: &Rc<Minecraft>, mouseX: i32, mouseY: i32)
    {
        if !self.m_enabled2.get()
        {
            return;
        }
        let fontRenderer: Rc<FontRenderer> = minecraft.m_fontRenderer.get().orNpe();
        GL11::glBindTexture(3553, minecraft.m_renderEngine.get().orNpe().getTexture("/gui/gui.png"));
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        let bl: bool = self.contains(mouseX, mouseY);
        let n: i32 = self.getHoverState(bl);
        let (x, y, width, height): (i32, i32, i32, i32) =
            (self.m_xPosition.get(), self.m_yPosition.get(), self.m_width.get(), self.m_height.get());
        self.m_gui.drawTexturedModalRect(x, y, 0, 46 + n * 20, width / 2, height);
        self.m_gui.drawTexturedModalRect(x + width / 2, y, 200 - width / 2, 46 + n * 20, width / 2, height);
        self.mouseDragged(minecraft, mouseX, mouseY);
        let color: i32 = if !self.m_enabled.get()
        {
            -6250336
        }
        else if bl
        {
            0xFFFFA0
        }
        else
        {
            0xE0E0E0
        };
        if let Some(displayString) = self.m_displayString.get()
        {
            self.m_gui.drawCenteredString(&fontRenderer, &displayString, x + width / 2, y + (height - 8) / 2, color);
        }
    }
}
