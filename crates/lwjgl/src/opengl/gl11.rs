use crate::buffers::{ByteBuffer, FloatBuffer, IntBuffer};
use crate::opengl::bindings::types::{GLboolean, GLenum};
use crate::opengl::glCall;
use std::ffi::{CStr, c_char, c_void};

pub struct GL11;

pub struct GL12;

impl GL12
{
    pub const GL_RESCALE_NORMAL: i32 = 0x803A;
}

fn boolean(value: bool) -> GLboolean { if value { 1 } else { 0 } }

fn offset(pointer: i64) -> *const c_void { pointer as usize as *const c_void }

// constants keep lwjgl's names, the game passes the values the java compiler inlined
impl GL11
{
    pub const GL_ZERO: i32 = 0;
    pub const GL_ONE: i32 = 1;
    pub const GL_NO_ERROR: i32 = 0;
    pub const GL_LINES: i32 = 1;
    pub const GL_LINE_STRIP: i32 = 3;
    pub const GL_TRIANGLES: i32 = 4;
    pub const GL_TRIANGLE_STRIP: i32 = 5;
    pub const GL_TRIANGLE_FAN: i32 = 6;
    pub const GL_QUADS: i32 = 7;
    pub const GL_DEPTH_BUFFER_BIT: i32 = 0x100;
    pub const GL_COLOR_BUFFER_BIT: i32 = 0x4000;
    pub const GL_EQUAL: i32 = 0x202;
    pub const GL_LEQUAL: i32 = 0x203;
    pub const GL_GREATER: i32 = 0x204;
    pub const GL_GEQUAL: i32 = 0x206;
    pub const GL_SRC_COLOR: i32 = 0x300;
    pub const GL_ONE_MINUS_SRC_COLOR: i32 = 0x301;
    pub const GL_SRC_ALPHA: i32 = 0x302;
    pub const GL_ONE_MINUS_SRC_ALPHA: i32 = 0x303;
    pub const GL_DST_ALPHA: i32 = 0x304;
    pub const GL_DST_COLOR: i32 = 0x306;
    pub const GL_ONE_MINUS_DST_COLOR: i32 = 0x307;
    pub const GL_FRONT: i32 = 0x404;
    pub const GL_BACK: i32 = 0x405;
    pub const GL_FRONT_AND_BACK: i32 = 0x408;
    pub const GL_EXP: i32 = 0x800;
    pub const GL_CULL_FACE: i32 = 0xB44;
    pub const GL_LIGHTING: i32 = 0xB50;
    pub const GL_LIGHT_MODEL_AMBIENT: i32 = 0xB53;
    pub const GL_COLOR_MATERIAL: i32 = 0xB57;
    pub const GL_FOG: i32 = 0xB60;
    pub const GL_FOG_DENSITY: i32 = 0xB62;
    pub const GL_FOG_START: i32 = 0xB63;
    pub const GL_FOG_END: i32 = 0xB64;
    pub const GL_FOG_MODE: i32 = 0xB65;
    pub const GL_FOG_COLOR: i32 = 0xB66;
    pub const GL_DEPTH_TEST: i32 = 0xB71;
    pub const GL_NORMALIZE: i32 = 0xBA1;
    pub const GL_MODELVIEW_MATRIX: i32 = 0xBA6;
    pub const GL_PROJECTION_MATRIX: i32 = 0xBA7;
    pub const GL_ALPHA_TEST: i32 = 0xBC0;
    pub const GL_BLEND: i32 = 0xBE2;
    pub const GL_UNPACK_ALIGNMENT: i32 = 0xCF5;
    pub const GL_PACK_ALIGNMENT: i32 = 0xD05;
    pub const GL_TEXTURE_2D: i32 = 0xDE1;
    pub const GL_AMBIENT: i32 = 0x1200;
    pub const GL_DIFFUSE: i32 = 0x1201;
    pub const GL_SPECULAR: i32 = 0x1202;
    pub const GL_POSITION: i32 = 0x1203;
    pub const GL_COMPILE: i32 = 0x1300;
    pub const GL_BYTE: i32 = 0x1400;
    pub const GL_UNSIGNED_BYTE: i32 = 0x1401;
    pub const GL_UNSIGNED_INT: i32 = 0x1405;
    pub const GL_FLOAT: i32 = 0x1406;
    pub const GL_AMBIENT_AND_DIFFUSE: i32 = 0x1602;
    pub const GL_MODELVIEW: i32 = 0x1700;
    pub const GL_PROJECTION: i32 = 0x1701;
    pub const GL_TEXTURE: i32 = 0x1702;
    pub const GL_RGB: i32 = 0x1907;
    pub const GL_RGBA: i32 = 0x1908;
    pub const GL_FLAT: i32 = 0x1D00;
    pub const GL_SMOOTH: i32 = 0x1D01;
    pub const GL_VENDOR: i32 = 0x1F00;
    pub const GL_RENDERER: i32 = 0x1F01;
    pub const GL_VERSION: i32 = 0x1F02;
    pub const GL_EXTENSIONS: i32 = 0x1F03;
    pub const GL_NEAREST: i32 = 0x2600;
    pub const GL_LINEAR: i32 = 0x2601;
    pub const GL_NEAREST_MIPMAP_LINEAR: i32 = 0x2702;
    pub const GL_TEXTURE_MAG_FILTER: i32 = 0x2800;
    pub const GL_TEXTURE_MIN_FILTER: i32 = 0x2801;
    pub const GL_TEXTURE_WRAP_S: i32 = 0x2802;
    pub const GL_TEXTURE_WRAP_T: i32 = 0x2803;
    pub const GL_CLAMP: i32 = 0x2900;
    pub const GL_REPEAT: i32 = 0x2901;
    pub const GL_LIGHT0: i32 = 0x4000;
    pub const GL_LIGHT1: i32 = 0x4001;
    pub const GL_POLYGON_OFFSET_FILL: i32 = 0x8037;
    pub const GL_VERTEX_ARRAY: i32 = 0x8074;
    pub const GL_NORMAL_ARRAY: i32 = 0x8075;
    pub const GL_COLOR_ARRAY: i32 = 0x8076;
    pub const GL_TEXTURE_COORD_ARRAY: i32 = 0x8078;
}

// buffer arguments start at the buffer position and cover its remaining elements, as lwjgl passes them
impl GL11
{
    pub fn glAlphaFunc(func: i32, reference: f32)
    {
        glCall!(AlphaFunc(func as GLenum, reference));
    }

    pub fn glBindTexture(target: i32, texture: i32)
    {
        glCall!(BindTexture(target as GLenum, texture as u32));
    }

    pub fn glBlendFunc(sfactor: i32, dfactor: i32)
    {
        glCall!(BlendFunc(sfactor as GLenum, dfactor as GLenum));
    }

    pub fn glCallList(list: i32)
    {
        glCall!(CallList(list as u32));
    }

    pub fn glCallLists(lists: &IntBuffer)
    {
        glCall!(CallLists(lists.remaining(), Self::GL_UNSIGNED_INT as GLenum, lists.address()));
    }

    pub fn glClear(mask: i32)
    {
        glCall!(Clear(mask as u32));
    }

    pub fn glClearColor(red: f32, green: f32, blue: f32, alpha: f32)
    {
        glCall!(ClearColor(red, green, blue, alpha));
    }

    pub fn glClearDepth(depth: f64)
    {
        glCall!(ClearDepth(depth));
    }

    pub fn glColor3f(red: f32, green: f32, blue: f32)
    {
        glCall!(Color3f(red, green, blue));
    }

    pub fn glColor4f(red: f32, green: f32, blue: f32, alpha: f32)
    {
        glCall!(Color4f(red, green, blue, alpha));
    }

    pub fn glColorMask(red: bool, green: bool, blue: bool, alpha: bool)
    {
        glCall!(ColorMask(boolean(red), boolean(green), boolean(blue), boolean(alpha)));
    }

    pub fn glColorMaterial(face: i32, mode: i32)
    {
        glCall!(ColorMaterial(face as GLenum, mode as GLenum));
    }

    pub fn glColorPointer(size: i32, unsigned: bool, stride: i32, pointer: &ByteBuffer)
    {
        let kind: i32 = if unsigned
        {
            Self::GL_UNSIGNED_BYTE
        }
        else
        {
            Self::GL_BYTE
        };
        glCall!(ColorPointer(size, kind as GLenum, stride, pointer.address()));
    }

    // glColorPointer(int, int, int, long) into the bound buffer object
    pub fn glColorPointerOffset(size: i32, kind: i32, stride: i32, pointer: i64)
    {
        glCall!(ColorPointer(size, kind as GLenum, stride, offset(pointer)));
    }

    pub fn glCullFace(mode: i32)
    {
        glCall!(CullFace(mode as GLenum));
    }

    pub fn glDeleteLists(list: i32, range: i32)
    {
        glCall!(DeleteLists(list as u32, range));
    }

    pub fn glDeleteTextures(textures: &IntBuffer)
    {
        glCall!(DeleteTextures(textures.remaining(), textures.address().cast()));
    }

    pub fn glDepthFunc(func: i32)
    {
        glCall!(DepthFunc(func as GLenum));
    }

    pub fn glDepthMask(flag: bool)
    {
        glCall!(DepthMask(boolean(flag)));
    }

    pub fn glDisable(cap: i32)
    {
        glCall!(Disable(cap as GLenum));
    }

    pub fn glDisableClientState(cap: i32)
    {
        glCall!(DisableClientState(cap as GLenum));
    }

    pub fn glDrawArrays(mode: i32, first: i32, count: i32)
    {
        glCall!(DrawArrays(mode as GLenum, first, count));
    }

    pub fn glEnable(cap: i32)
    {
        glCall!(Enable(cap as GLenum));
    }

    pub fn glEnableClientState(cap: i32)
    {
        glCall!(EnableClientState(cap as GLenum));
    }

    pub fn glEndList()
    {
        glCall!(EndList());
    }

    pub fn glFog(pname: i32, params: &FloatBuffer)
    {
        glCall!(Fogfv(pname as GLenum, params.address().cast()));
    }

    pub fn glFogf(pname: i32, param: f32)
    {
        glCall!(Fogf(pname as GLenum, param));
    }

    pub fn glFogi(pname: i32, param: i32)
    {
        glCall!(Fogi(pname as GLenum, param));
    }

    pub fn glGenLists(range: i32) -> i32 { glCall!(GenLists(range)) as i32 }

    pub fn glGenTextures(textures: &IntBuffer)
    {
        glCall!(GenTextures(textures.remaining(), textures.addressMut().cast()));
    }

    pub fn glGetError() -> i32 { glCall!(GetError()) as i32 }

    pub fn glGetFloat(pname: i32, params: &FloatBuffer)
    {
        glCall!(GetFloatv(pname as GLenum, params.addressMut().cast()));
    }

    pub fn glGetString(name: i32) -> Option<String>
    {
        let text: *const u8 = glCall!(GetString(name as GLenum));
        if text.is_null()
        {
            return None;
        }
        // SAFETY: a non null glGetString result is a nul terminated string the driver owns
        let text: &CStr = unsafe { CStr::from_ptr(text as *const c_char) };
        Some(text.to_string_lossy().into_owned())
    }

    pub fn glLight(light: i32, pname: i32, params: &FloatBuffer)
    {
        glCall!(Lightfv(light as GLenum, pname as GLenum, params.address().cast()));
    }

    pub fn glLightModel(pname: i32, params: &FloatBuffer)
    {
        glCall!(LightModelfv(pname as GLenum, params.address().cast()));
    }

    pub fn glLineWidth(width: f32)
    {
        glCall!(LineWidth(width));
    }

    pub fn glLoadIdentity()
    {
        glCall!(LoadIdentity());
    }

    pub fn glMatrixMode(mode: i32)
    {
        glCall!(MatrixMode(mode as GLenum));
    }

    pub fn glMultMatrix(m: &FloatBuffer)
    {
        glCall!(MultMatrixf(m.address().cast()));
    }

    pub fn glNewList(list: i32, mode: i32)
    {
        glCall!(NewList(list as u32, mode as GLenum));
    }

    pub fn glNormal3f(x: f32, y: f32, z: f32)
    {
        glCall!(Normal3f(x, y, z));
    }

    pub fn glNormalPointer(stride: i32, pointer: &ByteBuffer)
    {
        glCall!(NormalPointer(Self::GL_BYTE as GLenum, stride, pointer.address()));
    }

    // glNormalPointer(int, int, long) into the bound buffer object
    pub fn glNormalPointerOffset(kind: i32, stride: i32, pointer: i64)
    {
        glCall!(NormalPointer(kind as GLenum, stride, offset(pointer)));
    }

    pub fn glOrtho(left: f64, right: f64, bottom: f64, top: f64, zNear: f64, zFar: f64)
    {
        glCall!(Ortho(left, right, bottom, top, zNear, zFar));
    }

    pub fn glPixelStorei(pname: i32, param: i32)
    {
        glCall!(PixelStorei(pname as GLenum, param));
    }

    pub fn glPolygonOffset(factor: f32, units: f32)
    {
        glCall!(PolygonOffset(factor, units));
    }

    pub fn glPopMatrix()
    {
        glCall!(PopMatrix());
    }

    pub fn glPushMatrix()
    {
        glCall!(PushMatrix());
    }

    pub fn glReadPixels(x: i32, y: i32, width: i32, height: i32, format: i32, kind: i32, pixels: &ByteBuffer)
    {
        glCall!(ReadPixels(x, y, width, height, format as GLenum, kind as GLenum, pixels.addressMut()));
    }

    pub fn glRotatef(angle: f32, x: f32, y: f32, z: f32)
    {
        glCall!(Rotatef(angle, x, y, z));
    }

    pub fn glScaled(x: f64, y: f64, z: f64)
    {
        glCall!(Scaled(x, y, z));
    }

    pub fn glScalef(x: f32, y: f32, z: f32)
    {
        glCall!(Scalef(x, y, z));
    }

    pub fn glShadeModel(mode: i32)
    {
        glCall!(ShadeModel(mode as GLenum));
    }

    pub fn glTexCoordPointer(size: i32, stride: i32, pointer: &FloatBuffer)
    {
        glCall!(TexCoordPointer(size, Self::GL_FLOAT as GLenum, stride, pointer.address()));
    }

    // glTexCoordPointer(int, int, int, long) into the bound buffer object
    pub fn glTexCoordPointerOffset(size: i32, kind: i32, stride: i32, pointer: i64)
    {
        glCall!(TexCoordPointer(size, kind as GLenum, stride, offset(pointer)));
    }

    pub fn glTexImage2D(
        target: i32,
        level: i32,
        internalformat: i32,
        width: i32,
        height: i32,
        border: i32,
        format: i32,
        kind: i32,
        pixels: Option<&ByteBuffer>,
    )
    {
        let data: *const c_void = pixels.map_or(std::ptr::null(), ByteBuffer::address);
        glCall!(TexImage2D(
            target as GLenum,
            level,
            internalformat,
            width,
            height,
            border,
            format as GLenum,
            kind as GLenum,
            data
        ));
    }

    pub fn glTexParameteri(target: i32, pname: i32, param: i32)
    {
        glCall!(TexParameteri(target as GLenum, pname as GLenum, param));
    }

    pub fn glTexSubImage2D(
        target: i32,
        level: i32,
        xoffset: i32,
        yoffset: i32,
        width: i32,
        height: i32,
        format: i32,
        kind: i32,
        pixels: &ByteBuffer,
    )
    {
        glCall!(TexSubImage2D(
            target as GLenum,
            level,
            xoffset,
            yoffset,
            width,
            height,
            format as GLenum,
            kind as GLenum,
            pixels.address(),
        ));
    }

    pub fn glTranslatef(x: f32, y: f32, z: f32)
    {
        glCall!(Translatef(x, y, z));
    }

    pub fn glVertexPointer(size: i32, stride: i32, pointer: &FloatBuffer)
    {
        glCall!(VertexPointer(size, Self::GL_FLOAT as GLenum, stride, pointer.address()));
    }

    // glVertexPointer(int, int, int, long) into the bound buffer object
    pub fn glVertexPointerOffset(size: i32, kind: i32, stride: i32, pointer: i64)
    {
        glCall!(VertexPointer(size, kind as GLenum, stride, offset(pointer)));
    }

    pub fn glViewport(x: i32, y: i32, width: i32, height: i32)
    {
        glCall!(Viewport(x, y, width, height));
    }
}
