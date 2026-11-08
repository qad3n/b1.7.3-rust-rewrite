use crate::render::GLAllocation;
use lwjgl::buffers::IntBuffer;
use lwjgl::opengl::GL11;
use std::cell::Cell;

pub struct RenderList
{
    m_x: Cell<i32>,
    m_y: Cell<i32>,
    m_z: Cell<i32>,
    m_offsetX: Cell<f32>,
    m_offsetY: Cell<f32>,
    m_offsetZ: Cell<f32>,
    m_glListBuffer: IntBuffer,
    m_initialized: Cell<bool>,
    m_rendered: Cell<bool>,
}

impl RenderList
{
    pub fn new() -> RenderList
    {
        RenderList {
            m_x: Cell::new(0),
            m_y: Cell::new(0),
            m_z: Cell::new(0),
            m_offsetX: Cell::new(0.0),
            m_offsetY: Cell::new(0.0),
            m_offsetZ: Cell::new(0.0),
            m_glListBuffer: GLAllocation::createDirectIntBuffer(65536),
            m_initialized: Cell::new(false),
            m_rendered: Cell::new(false),
        }
    }

    pub fn init(&self, x: i32, y: i32, z: i32, offsetX: f64, offsetY: f64, offsetZ: f64)
    {
        self.m_initialized.set(true);
        self.m_glListBuffer.clear();
        self.m_x.set(x);
        self.m_y.set(y);
        self.m_z.set(z);
        self.m_offsetX.set(offsetX as f32);
        self.m_offsetY.set(offsetY as f32);
        self.m_offsetZ.set(offsetZ as f32);
    }

    pub fn isAt(&self, x: i32, y: i32, z: i32) -> bool
    {
        if !self.m_initialized.get()
        {
            return false;
        }
        x == self.m_x.get() && y == self.m_y.get() && z == self.m_z.get()
    }

    pub fn addGlList(&self, glList: i32)
    {
        self.m_glListBuffer.put(glList);
        if self.m_glListBuffer.remaining() == 0
        {
            self.render();
        }
    }

    pub fn render(&self)
    {
        if !self.m_initialized.get()
        {
            return;
        }
        if !self.m_rendered.get()
        {
            self.m_glListBuffer.flip();
            self.m_rendered.set(true);
        }
        if self.m_glListBuffer.remaining() > 0
        {
            GL11::glPushMatrix();
            GL11::glTranslatef(
                self.m_x.get() as f32 - self.m_offsetX.get(),
                self.m_y.get() as f32 - self.m_offsetY.get(),
                self.m_z.get() as f32 - self.m_offsetZ.get(),
            );
            GL11::glCallLists(&self.m_glListBuffer);
            GL11::glPopMatrix();
        }
    }

    pub fn clear(&self)
    {
        self.m_initialized.set(false);
        self.m_rendered.set(false);
    }
}

impl Default for RenderList
{
    fn default() -> Self { Self::new() }
}
