use crate::render::GLAllocation;
use lwjgl::buffers::{ByteBuffer, ByteOrder, FloatBuffer, IntBuffer};
use lwjgl::opengl::{ARBVertexBufferObject, GL11, GLContext};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const CONVERT_QUADS_TO_TRIANGLES: bool = true;
const TRY_VBO: bool = false;
const INSTANCE_SIZE: i32 = 0x200000;

mod vertex
{
    pub const INTS: i32 = 8;
    pub const BYTES: i32 = 32;
    pub const UV: i32 = 3;
    pub const COLOR: i32 = 5;
    pub const NORMAL: i32 = 6;
    pub const UV_BYTES: i64 = 12;
    pub const COLOR_BYTES: i32 = 20;
    pub const NORMAL_BYTES: i32 = 24;
}

thread_local! {
    static INSTANCE: Rc<Tessellator> = Rc::new(Tessellator::new(INSTANCE_SIZE));
}

pub struct Tessellator
{
    m_byteBuffer: ByteBuffer,
    m_intBuffer: IntBuffer,
    m_floatBuffer: FloatBuffer,
    m_rawBuffer: RefCell<Vec<i32>>,
    m_vertexCount: Cell<i32>,
    m_textureU: Cell<f64>,
    m_textureV: Cell<f64>,
    m_color: Cell<i32>,
    m_hasColor: Cell<bool>,
    m_hasTexture: Cell<bool>,
    m_hasNormals: Cell<bool>,
    m_rawBufferIndex: Cell<i32>,
    m_addedVertices: Cell<i32>,
    m_isColorDisabled: Cell<bool>,
    m_drawMode: Cell<i32>,
    m_xOffset: Cell<f64>,
    m_yOffset: Cell<f64>,
    m_zOffset: Cell<f64>,
    m_normal: Cell<i32>,
    m_isDrawing: Cell<bool>,
    m_useVBO: bool,
    m_vertexBuffers: Option<IntBuffer>,
    m_vboIndex: Cell<i32>,
    m_vboCount: i32,
    m_bufferSize: i32,
}

impl Tessellator
{
    pub fn instance() -> Rc<Tessellator> { INSTANCE.with(Rc::clone) }

    fn new(size: i32) -> Tessellator
    {
        let byteBuffer: ByteBuffer = GLAllocation::createDirectByteBuffer(size * 4);
        let intBuffer: IntBuffer = byteBuffer.asIntBuffer();
        let floatBuffer: FloatBuffer = byteBuffer.asFloatBuffer();
        let vboCount: i32 = 10;
        let useVBO: bool = TRY_VBO && GLContext::getCapabilities().GL_ARB_vertex_buffer_object;
        let vertexBuffers: Option<IntBuffer> = useVBO.then(|| {
            let buffers: IntBuffer = GLAllocation::createDirectIntBuffer(vboCount);
            ARBVertexBufferObject::glGenBuffersARB(&buffers);
            buffers
        });
        Tessellator {
            m_byteBuffer: byteBuffer,
            m_intBuffer: intBuffer,
            m_floatBuffer: floatBuffer,
            m_rawBuffer: RefCell::new(vec![0; size as usize]),
            m_vertexCount: Cell::new(0),
            m_textureU: Cell::new(0.0),
            m_textureV: Cell::new(0.0),
            m_color: Cell::new(0),
            m_hasColor: Cell::new(false),
            m_hasTexture: Cell::new(false),
            m_hasNormals: Cell::new(false),
            m_rawBufferIndex: Cell::new(0),
            m_addedVertices: Cell::new(0),
            m_isColorDisabled: Cell::new(false),
            m_drawMode: Cell::new(0),
            m_xOffset: Cell::new(0.0),
            m_yOffset: Cell::new(0.0),
            m_zOffset: Cell::new(0.0),
            m_normal: Cell::new(0),
            m_isDrawing: Cell::new(false),
            m_useVBO: useVBO,
            m_vertexBuffers: vertexBuffers,
            m_vboIndex: Cell::new(0),
            m_vboCount: vboCount,
            m_bufferSize: size,
        }
    }

    pub fn draw(&self)
    {
        if !self.m_isDrawing.get()
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_STATE, "Not tesselating!");
        }
        self.m_isDrawing.set(false);
        if self.m_vertexCount.get() > 0
        {
            self.m_intBuffer.clear();
            self.m_intBuffer.putFrom(&self.m_rawBuffer.borrow(), 0, self.m_rawBufferIndex.get());
            self.m_byteBuffer.setPosition(0);
            self.m_byteBuffer.setLimit(self.m_rawBufferIndex.get() * 4);
            if self.m_useVBO
            {
                self.m_vboIndex.set((self.m_vboIndex.get() + 1) % self.m_vboCount);
                let buffers: &IntBuffer = self.m_vertexBuffers.as_ref().expect("vbo names exist when vbo is used");
                ARBVertexBufferObject::glBindBufferARB(
                    ARBVertexBufferObject::GL_ARRAY_BUFFER_ARB,
                    buffers.getAt(self.m_vboIndex.get()),
                );
                ARBVertexBufferObject::glBufferDataARB(
                    ARBVertexBufferObject::GL_ARRAY_BUFFER_ARB,
                    &self.m_byteBuffer,
                    ARBVertexBufferObject::GL_STREAM_DRAW_ARB,
                );
            }
            if self.m_hasTexture.get()
            {
                if self.m_useVBO
                {
                    GL11::glTexCoordPointerOffset(2, GL11::GL_FLOAT, vertex::BYTES, vertex::UV_BYTES);
                }
                else
                {
                    self.m_floatBuffer.setPosition(vertex::UV);
                    GL11::glTexCoordPointer(2, vertex::BYTES, &self.m_floatBuffer);
                }
                GL11::glEnableClientState(GL11::GL_TEXTURE_COORD_ARRAY);
            }
            if self.m_hasColor.get()
            {
                if self.m_useVBO
                {
                    GL11::glColorPointerOffset(4, GL11::GL_UNSIGNED_BYTE, vertex::BYTES, vertex::COLOR_BYTES as i64);
                }
                else
                {
                    self.m_byteBuffer.setPosition(vertex::COLOR_BYTES);
                    GL11::glColorPointer(4, true, vertex::BYTES, &self.m_byteBuffer);
                }
                GL11::glEnableClientState(GL11::GL_COLOR_ARRAY);
            }
            if self.m_hasNormals.get()
            {
                if self.m_useVBO
                {
                    GL11::glNormalPointerOffset(GL11::GL_BYTE, vertex::BYTES, vertex::NORMAL_BYTES as i64);
                }
                else
                {
                    self.m_byteBuffer.setPosition(vertex::NORMAL_BYTES);
                    GL11::glNormalPointer(vertex::BYTES, &self.m_byteBuffer);
                }
                GL11::glEnableClientState(GL11::GL_NORMAL_ARRAY);
            }
            if self.m_useVBO
            {
                GL11::glVertexPointerOffset(3, GL11::GL_FLOAT, vertex::BYTES, 0);
            }
            else
            {
                self.m_floatBuffer.setPosition(0);
                GL11::glVertexPointer(3, vertex::BYTES, &self.m_floatBuffer);
            }
            GL11::glEnableClientState(GL11::GL_VERTEX_ARRAY);
            if self.m_drawMode.get() == GL11::GL_QUADS && CONVERT_QUADS_TO_TRIANGLES
            {
                GL11::glDrawArrays(GL11::GL_TRIANGLES, 0, self.m_vertexCount.get());
            }
            else
            {
                GL11::glDrawArrays(self.m_drawMode.get(), 0, self.m_vertexCount.get());
            }
            GL11::glDisableClientState(GL11::GL_VERTEX_ARRAY);
            if self.m_hasTexture.get()
            {
                GL11::glDisableClientState(GL11::GL_TEXTURE_COORD_ARRAY);
            }
            if self.m_hasColor.get()
            {
                GL11::glDisableClientState(GL11::GL_COLOR_ARRAY);
            }
            if self.m_hasNormals.get()
            {
                GL11::glDisableClientState(GL11::GL_NORMAL_ARRAY);
            }
        }
        self.reset();
    }

    fn reset(&self)
    {
        self.m_vertexCount.set(0);
        self.m_byteBuffer.clear();
        self.m_rawBufferIndex.set(0);
        self.m_addedVertices.set(0);
    }

    pub fn startDrawingQuads(&self) { self.startDrawing(GL11::GL_QUADS); }

    pub fn startDrawing(&self, drawMode: i32)
    {
        if self.m_isDrawing.get()
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_STATE, "Already tesselating!");
        }
        self.m_isDrawing.set(true);
        self.reset();
        self.m_drawMode.set(drawMode);
        self.m_hasNormals.set(false);
        self.m_hasColor.set(false);
        self.m_hasTexture.set(false);
        self.m_isColorDisabled.set(false);
    }

    pub fn setTextureUV(&self, u: f64, v: f64)
    {
        self.m_hasTexture.set(true);
        self.m_textureU.set(u);
        self.m_textureV.set(v);
    }

    pub fn setColorOpaque_F(&self, r: f32, g: f32, b: f32)
    {
        self.setColorOpaque((r * 255.0) as i32, (g * 255.0) as i32, (b * 255.0) as i32);
    }

    pub fn setColorRGBA_F(&self, r: f32, g: f32, b: f32, a: f32)
    {
        self.setColorRGBA((r * 255.0) as i32, (g * 255.0) as i32, (b * 255.0) as i32, (a * 255.0) as i32);
    }

    pub fn setColorOpaque(&self, r: i32, g: i32, b: i32) { self.setColorRGBA(r, g, b, 255); }

    pub fn setColorRGBA(&self, r: i32, g: i32, b: i32, a: i32)
    {
        if self.m_isColorDisabled.get()
        {
            return;
        }
        let r: i32 = r.clamp(0, 255);
        let g: i32 = g.clamp(0, 255);
        let b: i32 = b.clamp(0, 255);
        let a: i32 = a.clamp(0, 255);
        self.m_hasColor.set(true);
        if ByteOrder::nativeOrder() == ByteOrder::LittleEndian
        {
            self.m_color.set(a << 24 | b << 16 | g << 8 | r);
        }
        else
        {
            self.m_color.set(r << 24 | g << 16 | b << 8 | a);
        }
    }

    pub fn addVertexWithUV(&self, x: f64, y: f64, z: f64, u: f64, v: f64)
    {
        self.setTextureUV(u, v);
        self.addVertex(x, y, z);
    }

    pub fn addVertex(&self, x: f64, y: f64, z: f64)
    {
        self.m_addedVertices.set(self.m_addedVertices.get() + 1);
        {
            let mut raw = self.m_rawBuffer.borrow_mut();
            let at = |offset: i32| -> usize { (self.m_rawBufferIndex.get() + offset) as usize };
            if self.m_drawMode.get() == GL11::GL_QUADS
                && CONVERT_QUADS_TO_TRIANGLES
                && self.m_addedVertices.get() % 4 == 0
            {
                for i in 0..2
                {
                    let n: i32 = vertex::INTS * (3 - i);
                    if self.m_hasTexture.get()
                    {
                        raw[at(vertex::UV)] = raw[at(-n + vertex::UV)];
                        raw[at(vertex::UV + 1)] = raw[at(-n + vertex::UV + 1)];
                    }
                    if self.m_hasColor.get()
                    {
                        raw[at(vertex::COLOR)] = raw[at(-n + vertex::COLOR)];
                    }
                    raw[at(0)] = raw[at(-n)];
                    raw[at(1)] = raw[at(-n + 1)];
                    raw[at(2)] = raw[at(-n + 2)];
                    self.m_vertexCount.set(self.m_vertexCount.get() + 1);
                    self.m_rawBufferIndex.set(self.m_rawBufferIndex.get() + vertex::INTS);
                }
            }
            if self.m_hasTexture.get()
            {
                raw[at(vertex::UV)] = (self.m_textureU.get() as f32).to_bits() as i32;
                raw[at(vertex::UV + 1)] = (self.m_textureV.get() as f32).to_bits() as i32;
            }
            if self.m_hasColor.get()
            {
                raw[at(vertex::COLOR)] = self.m_color.get();
            }
            if self.m_hasNormals.get()
            {
                raw[at(vertex::NORMAL)] = self.m_normal.get();
            }
            raw[at(0)] = ((x + self.m_xOffset.get()) as f32).to_bits() as i32;
            raw[at(1)] = ((y + self.m_yOffset.get()) as f32).to_bits() as i32;
            raw[at(2)] = ((z + self.m_zOffset.get()) as f32).to_bits() as i32;
        }
        self.m_rawBufferIndex.set(self.m_rawBufferIndex.get() + vertex::INTS);
        self.m_vertexCount.set(self.m_vertexCount.get() + 1);
        if self.m_vertexCount.get() % 4 == 0 && self.m_rawBufferIndex.get() >= self.m_bufferSize - vertex::BYTES
        {
            self.draw();
            self.m_isDrawing.set(true);
        }
    }

    pub fn setColorOpaque_I(&self, rgb: i32) { self.setColorOpaque(rgb >> 16 & 0xFF, rgb >> 8 & 0xFF, rgb & 0xFF); }

    pub fn setColorRGBA_I(&self, rgb: i32, a: i32)
    {
        self.setColorRGBA(rgb >> 16 & 0xFF, rgb >> 8 & 0xFF, rgb & 0xFF, a);
    }

    pub fn disableColor(&self) { self.m_isColorDisabled.set(true); }

    pub fn setNormal(&self, x: f32, y: f32, z: f32)
    {
        if !self.m_isDrawing.get()
        {
            jvm::system::println("But..");
        }
        self.m_hasNormals.set(true);
        let by: i8 = (x * 128.0) as i32 as i8;
        let by2: i8 = (y * 127.0) as i32 as i8;
        let by3: i8 = (z * 127.0) as i32 as i8;
        self.m_normal.set(by as i32 | (by2 as i32) << 8 | (by3 as i32) << 16);
    }

    pub fn setTranslationD(&self, offsetX: f64, offsetY: f64, offsetZ: f64)
    {
        self.m_xOffset.set(offsetX);
        self.m_yOffset.set(offsetY);
        self.m_zOffset.set(offsetZ);
    }

    pub fn setTranslationF(&self, offsetX: f32, offsetY: f32, offsetZ: f32)
    {
        self.m_xOffset.set(self.m_xOffset.get() + offsetX as f64);
        self.m_yOffset.set(self.m_yOffset.get() + offsetY as f64);
        self.m_zOffset.set(self.m_zOffset.get() + offsetZ as f64);
    }
}
