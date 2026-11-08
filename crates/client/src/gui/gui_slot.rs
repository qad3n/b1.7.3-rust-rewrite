use crate::client::Minecraft;
use crate::gui::GuiButton;
use crate::render::Tessellator;
use jvm::throwable::OrNpe;
use lwjgl::input::Mouse;
use lwjgl::opengl::GL11;
use std::any::Any;
use std::cell::Cell;
use std::rc::{Rc, Weak};

game::vtable! {
    GuiSlotClass for GuiSlot {
        fn drawBackground();
        fn drawSlot(index: i32, x: i32, y: i32, height: i32, tessellator: &Rc<Tessellator>);
        fn elementClicked(index: i32, doubleClick: bool);
        fn getColumnHeader(column: i32) -> Option<String>;
        fn getContentHeight() -> i32;
        fn getSize() -> i32;
        fn headerClicked(x: i32, y: i32);
        fn isSelected(index: i32) -> bool;
        fn renderDecorations(mouseX: i32, mouseY: i32);
        fn renderHeader(x: i32, y: i32, tessellator: &Rc<Tessellator>);
    }
}

fn abstractMethod(name: &str) -> !
{
    jvm::throwable::throw(&jvm::throwable::ERROR, &format!("java.lang.AbstractMethodError: {name}"))
}

pub struct GuiSlot
{
    pub m_class: &'static GuiSlotClass,
    m_ext: Box<dyn Any>,
    m_this: Weak<GuiSlot>,
    m_mc: Rc<Minecraft>,
    m_width: i32,
    m_height: i32,
    pub m_top: i32,
    pub m_bottom: i32,
    m_right: i32,
    m_left: i32,
    pub m_posZ: i32,
    m_scrollUpButtonID: Cell<i32>,
    m_scrollDownButtonID: Cell<i32>,
    m_initialClickY: Cell<f32>,
    m_scrollMultiplier: Cell<f32>,
    m_amountScrolled: Cell<f32>,
    m_selectedElement: Cell<i32>,
    m_lastClicked: Cell<i64>,
    m_renderSelectionHighlight: Cell<bool>,
    m_renderHeader: Cell<bool>,
    m_headerHeight: Cell<i32>,
}

pub const GUI_SLOT: GuiSlotClass = GuiSlotClass {
    m_name: "GuiSlot",
    m_parent: None,
    m_interfaces: &[],
    drawBackground: |_| abstractMethod("drawBackground"),
    drawSlot: |_, _, _, _, _, _| abstractMethod("drawSlot"),
    elementClicked: |_, _, _| abstractMethod("elementClicked"),
    getColumnHeader: |_, _| abstractMethod("getColumnHeader"),
    getContentHeight: |this| this.getSize() * this.m_posZ + this.m_headerHeight.get(),
    getSize: |_| abstractMethod("getSize"),
    headerClicked: |_, _, _| {},
    isSelected: |_, _| abstractMethod("isSelected"),
    renderDecorations: |_, _, _| {},
    renderHeader: |_, _, _, _| {},
};

impl GuiSlot
{
    #[allow(clippy::too_many_arguments)]
    pub fn construct(
        class: &'static GuiSlotClass,
        ext: Box<dyn Any>,
        minecraft: Rc<Minecraft>,
        width: i32,
        height: i32,
        minY: i32,
        maxY: i32,
        entryHeight: i32,
    ) -> Rc<GuiSlot>
    {
        Rc::new_cyclic(|this: &Weak<GuiSlot>| GuiSlot {
            m_class: class,
            m_ext: ext,
            m_this: this.clone(),
            m_mc: minecraft,
            m_width: width,
            m_height: height,
            m_top: minY,
            m_bottom: maxY,
            m_right: width,
            m_left: 0,
            m_posZ: entryHeight,
            m_scrollUpButtonID: Cell::new(0),
            m_scrollDownButtonID: Cell::new(0),
            m_initialClickY: Cell::new(-2.0),
            m_scrollMultiplier: Cell::new(0.0),
            m_amountScrolled: Cell::new(0.0),
            m_selectedElement: Cell::new(-1),
            m_lastClicked: Cell::new(0),
            m_renderSelectionHighlight: Cell::new(true),
            m_renderHeader: Cell::new(false),
            m_headerHeight: Cell::new(0),
        })
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("slot ext matches its class") }

    pub fn rc(&self) -> Rc<GuiSlot> { self.m_this.upgrade().orNpe() }

    pub fn setRenderSelectionHighlight(&self, renderSelectionHighlight: bool)
    {
        self.m_renderSelectionHighlight.set(renderSelectionHighlight);
    }

    pub fn setHeader(&self, renderHeader: bool, headerHeight: i32)
    {
        self.m_renderHeader.set(renderHeader);
        self.m_headerHeight.set(if renderHeader { headerHeight } else { 0 });
    }

    pub fn getEntryAt(self: &Rc<Self>, x: i32, y: i32) -> i32
    {
        let n: i32 = self.m_width / 2 - 110;
        let n2: i32 = self.m_width / 2 + 110;
        let n3: i32 = y - self.m_top - self.m_headerHeight.get() + self.m_amountScrolled.get() as i32 - 4;
        let n4: i32 = n3 / self.m_posZ;
        if x >= n && x <= n2 && n4 >= 0 && n3 >= 0 && n4 < self.getSize()
        {
            return n4;
        }
        -1
    }

    pub fn registerScrollButtons(
        &self,
        _buttons: &jvm::array_list::JArrayList<Rc<GuiButton>>,
        upButtonId: i32,
        downButtonId: i32,
    )
    {
        self.m_scrollUpButtonID.set(upButtonId);
        self.m_scrollDownButtonID.set(downButtonId);
    }

    fn bindAmountScrolled(self: &Rc<Self>)
    {
        let mut n: i32 = self.getContentHeight() - (self.m_bottom - self.m_top - 4);
        if n < 0
        {
            n /= 2;
        }
        if self.m_amountScrolled.get() < 0.0
        {
            self.m_amountScrolled.set(0.0);
        }
        if self.m_amountScrolled.get() > n as f32
        {
            self.m_amountScrolled.set(n as f32);
        }
    }

    pub fn actionPerformed(self: &Rc<Self>, button: &GuiButton)
    {
        if !button.m_enabled.get()
        {
            return;
        }
        let step: f32 = (self.m_posZ * 2 / 3) as f32;
        if button.m_id.get() == self.m_scrollUpButtonID.get()
        {
            self.m_amountScrolled.set(self.m_amountScrolled.get() - step);
            self.m_initialClickY.set(-2.0);
            self.bindAmountScrolled();
        }
        else if button.m_id.get() == self.m_scrollDownButtonID.get()
        {
            self.m_amountScrolled.set(self.m_amountScrolled.get() + step);
            self.m_initialClickY.set(-2.0);
            self.bindAmountScrolled();
        }
    }

    fn handleMouse(self: &Rc<Self>, mouseX: i32, mouseY: i32, n7: i32, n8: i32, n9: i32)
    {
        if !Mouse::isButtonDown(0)
        {
            self.m_initialClickY.set(-1.0);
            return;
        }
        if self.m_initialClickY.get() >= 0.0
        {
            let amount: f32 = self.m_amountScrolled.get()
                - (mouseY as f32 - self.m_initialClickY.get()) * self.m_scrollMultiplier.get();
            self.m_amountScrolled.set(amount);
            self.m_initialClickY.set(mouseY as f32);
            return;
        }
        if self.m_initialClickY.get() != -1.0
        {
            return;
        }
        if mouseY < self.m_top || mouseY > self.m_bottom
        {
            self.m_initialClickY.set(-2.0);
            return;
        }
        let mut bl: bool = true;
        let n10: i32 = self.m_width / 2 - 110;
        let n6: i32 = self.m_width / 2 + 110;
        let n5: i32 = mouseY - self.m_top - self.m_headerHeight.get() + self.m_amountScrolled.get() as i32 - 4;
        let n4: i32 = n5 / self.m_posZ;
        if mouseX >= n10 && mouseX <= n6 && n4 >= 0 && n5 >= 0 && n4 < n7
        {
            let doubleClick: bool =
                n4 == self.m_selectedElement.get() && jvm::system::currentTimeMillis() - self.m_lastClicked.get() < 250;
            self.elementClicked(n4, doubleClick);
            self.m_selectedElement.set(n4);
            self.m_lastClicked.set(jvm::system::currentTimeMillis());
        }
        else if mouseX >= n10 && mouseX <= n6 && n5 < 0
        {
            self.headerClicked(mouseX - n10, mouseY - self.m_top + self.m_amountScrolled.get() as i32 - 4);
            bl = false;
        }
        if mouseX >= n8 && mouseX <= n9
        {
            self.m_scrollMultiplier.set(-1.0);
            let n3: i32 = (self.getContentHeight() - (self.m_bottom - self.m_top - 4)).max(1);
            let range: i32 = self.m_bottom - self.m_top;
            let n2: i32 = (((range * range) as f32 / self.getContentHeight() as f32) as i32).max(32).min(range - 8);
            self.m_scrollMultiplier.set(self.m_scrollMultiplier.get() / ((range - n2) as f32 / n3 as f32));
        }
        else
        {
            self.m_scrollMultiplier.set(1.0);
        }
        self.m_initialClickY.set(if bl { mouseY as f32 } else { -2.0 });
    }

    pub fn drawScreen(self: &Rc<Self>, mouseX: i32, mouseY: i32, _tickDelta: f32)
    {
        self.drawBackground();
        let n7: i32 = self.getSize();
        let n8: i32 = self.m_width / 2 + 124;
        let n9: i32 = n8 + 6;
        self.handleMouse(mouseX, mouseY, n7, n8, n9);
        self.bindAmountScrolled();
        GL11::glDisable(2896);
        GL11::glDisable(2912);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glBindTexture(3553, self.m_mc.m_renderEngine.get().orNpe().getTexture("/gui/background.png"));
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        let f: f32 = 32.0;
        let scrolled: i32 = self.m_amountScrolled.get() as i32;
        let (left, right, top, bottom): (i32, i32, i32, i32) = (self.m_left, self.m_right, self.m_top, self.m_bottom);
        let uv = |value: i32| (value as f32 / f) as f64;
        tessellator.startDrawingQuads();
        tessellator.setColorOpaque_I(0x202020);
        tessellator.addVertexWithUV(left as f64, bottom as f64, 0.0, uv(left), uv(bottom + scrolled));
        tessellator.addVertexWithUV(right as f64, bottom as f64, 0.0, uv(right), uv(bottom + scrolled));
        tessellator.addVertexWithUV(right as f64, top as f64, 0.0, uv(right), uv(top + scrolled));
        tessellator.addVertexWithUV(left as f64, top as f64, 0.0, uv(left), uv(top + scrolled));
        tessellator.draw();
        let n6: i32 = self.m_width / 2 - 92 - 16;
        let n5: i32 = top + 4 - scrolled;
        if self.m_renderHeader.get()
        {
            self.renderHeader(n6, n5, &tessellator);
        }
        let quad = |x1: i32, y1: i32, x2: i32, y2: i32| {
            tessellator.addVertexWithUV(x1 as f64, y2 as f64, 0.0, 0.0, 1.0);
            tessellator.addVertexWithUV(x2 as f64, y2 as f64, 0.0, 1.0, 1.0);
            tessellator.addVertexWithUV(x2 as f64, y1 as f64, 0.0, 1.0, 0.0);
            tessellator.addVertexWithUV(x1 as f64, y1 as f64, 0.0, 0.0, 0.0);
        };
        for n4 in 0..n7
        {
            let n3: i32 = n5 + n4 * self.m_posZ + self.m_headerHeight.get();
            let n2: i32 = self.m_posZ - 4;
            if n3 > bottom || n3 + n2 < top
            {
                continue;
            }
            if self.m_renderSelectionHighlight.get() && self.isSelected(n4)
            {
                let n: i32 = self.m_width / 2 - 110;
                let n11: i32 = self.m_width / 2 + 110;
                GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
                GL11::glDisable(3553);
                tessellator.startDrawingQuads();
                tessellator.setColorOpaque_I(0x808080);
                quad(n, n3 - 2, n11, n3 + n2 + 2);
                tessellator.setColorOpaque_I(0);
                quad(n + 1, n3 - 1, n11 - 1, n3 + n2 + 1);
                tessellator.draw();
                GL11::glEnable(3553);
            }
            self.drawSlot(n4, n6, n3, n2, &tessellator);
        }
        GL11::glDisable(2929);
        let n4: i32 = 4;
        self.overlayBackground(0, top, 255, 255);
        self.overlayBackground(bottom, self.m_height, 255, 255);
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        GL11::glDisable(3008);
        GL11::glShadeModel(7425);
        GL11::glDisable(3553);
        tessellator.startDrawingQuads();
        tessellator.setColorRGBA_I(0, 0);
        tessellator.addVertexWithUV(left as f64, (top + n4) as f64, 0.0, 0.0, 1.0);
        tessellator.addVertexWithUV(right as f64, (top + n4) as f64, 0.0, 1.0, 1.0);
        tessellator.setColorRGBA_I(0, 255);
        tessellator.addVertexWithUV(right as f64, top as f64, 0.0, 1.0, 0.0);
        tessellator.addVertexWithUV(left as f64, top as f64, 0.0, 0.0, 0.0);
        tessellator.draw();
        tessellator.startDrawingQuads();
        tessellator.setColorRGBA_I(0, 255);
        tessellator.addVertexWithUV(left as f64, bottom as f64, 0.0, 0.0, 1.0);
        tessellator.addVertexWithUV(right as f64, bottom as f64, 0.0, 1.0, 1.0);
        tessellator.setColorRGBA_I(0, 0);
        tessellator.addVertexWithUV(right as f64, (bottom - n4) as f64, 0.0, 1.0, 0.0);
        tessellator.addVertexWithUV(left as f64, (bottom - n4) as f64, 0.0, 0.0, 0.0);
        tessellator.draw();
        let n3: i32 = self.getContentHeight() - (bottom - top - 4);
        if n3 > 0
        {
            let n2: i32 = ((bottom - top) * (bottom - top) / self.getContentHeight()).max(32).min(bottom - top - 8);
            let n: i32 = (scrolled * (bottom - top - n2) / n3 + top).max(top);
            let bar = |color: i32, x2: i32, y2: i32| {
                tessellator.startDrawingQuads();
                tessellator.setColorRGBA_I(color, 255);
                tessellator.addVertexWithUV(n8 as f64, y2 as f64, 0.0, 0.0, 1.0);
                tessellator.addVertexWithUV(x2 as f64, y2 as f64, 0.0, 1.0, 1.0);
                tessellator.addVertexWithUV(x2 as f64, n as f64, 0.0, 1.0, 0.0);
                tessellator.addVertexWithUV(n8 as f64, n as f64, 0.0, 0.0, 0.0);
                tessellator.draw();
            };
            tessellator.startDrawingQuads();
            tessellator.setColorRGBA_I(0, 255);
            tessellator.addVertexWithUV(n8 as f64, bottom as f64, 0.0, 0.0, 1.0);
            tessellator.addVertexWithUV(n9 as f64, bottom as f64, 0.0, 1.0, 1.0);
            tessellator.addVertexWithUV(n9 as f64, top as f64, 0.0, 1.0, 0.0);
            tessellator.addVertexWithUV(n8 as f64, top as f64, 0.0, 0.0, 0.0);
            tessellator.draw();
            bar(0x808080, n9, n + n2);
            bar(0xC0C0C0, n9 - 1, n + n2 - 1);
        }
        self.renderDecorations(mouseX, mouseY);
        GL11::glEnable(3553);
        GL11::glShadeModel(7424);
        GL11::glEnable(3008);
        GL11::glDisable(3042);
    }

    fn overlayBackground(&self, top: i32, bottom: i32, topAlpha: i32, bottomAlpha: i32)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glBindTexture(3553, self.m_mc.m_renderEngine.get().orNpe().getTexture("/gui/background.png"));
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        let f: f32 = 32.0;
        tessellator.startDrawingQuads();
        tessellator.setColorRGBA_I(0x404040, bottomAlpha);
        tessellator.addVertexWithUV(0.0, bottom as f64, 0.0, 0.0, (bottom as f32 / f) as f64);
        tessellator.addVertexWithUV(
            self.m_width as f64,
            bottom as f64,
            0.0,
            (self.m_width as f32 / f) as f64,
            (bottom as f32 / f) as f64,
        );
        tessellator.setColorRGBA_I(0x404040, topAlpha);
        tessellator.addVertexWithUV(
            self.m_width as f64,
            top as f64,
            0.0,
            (self.m_width as f32 / f) as f64,
            (top as f32 / f) as f64,
        );
        tessellator.addVertexWithUV(0.0, top as f64, 0.0, 0.0, (top as f32 / f) as f64);
        tessellator.draw();
    }
}
