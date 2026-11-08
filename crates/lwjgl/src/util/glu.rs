use crate::buffers::{BufferUtils, FloatBuffer};
use crate::opengl::GL11;

// org.lwjgl.util.glu.GLU.PI is a float literal
const PI: f32 = 3.14159265358979323846;

// gluErrorString checks the glu codes, then Util.translateGLErrorString
const ERROR_STRINGS: [(i32, &str); 12] = [
    (GLU::GLU_INVALID_ENUM, "Invalid enum (glu)"),
    (GLU::GLU_INVALID_VALUE, "Invalid value (glu)"),
    (GLU::GLU_OUT_OF_MEMORY, "Out of memory (glu)"),
    (0x0000, "No error"),
    (0x0500, "Invalid enum"),
    (0x0501, "Invalid value"),
    (0x0502, "Invalid operation"),
    (0x0503, "Stack overflow"),
    (0x0504, "Stack underflow"),
    (0x0505, "Out of memory"),
    (0x8031, "Table too large"),
    (0x0506, "Invalid framebuffer operation"),
];

pub struct GLU;

impl GLU
{
    pub const GLU_INVALID_ENUM: i32 = 100900;
    pub const GLU_INVALID_VALUE: i32 = 100901;
    pub const GLU_OUT_OF_MEMORY: i32 = 100902;

    pub fn gluErrorString(errorCode: i32) -> Option<&'static str>
    {
        ERROR_STRINGS.iter().find(|(code, _)| *code == errorCode).map(|(_, text)| *text)
    }

    // Project.gluPerspective builds the matrix in a float buffer and multiplies it in
    pub fn gluPerspective(fovy: f32, aspect: f32, zNear: f32, zFar: f32)
    {
        let radians: f32 = fovy / 2.0 * PI / 180.0;
        let deltaZ: f32 = zFar - zNear;
        let sine: f32 = jvm::math::sin(radians as f64) as f32;
        if deltaZ == 0.0 || sine == 0.0 || aspect == 0.0
        {
            return;
        }
        let cotangent: f32 = jvm::math::cos(radians as f64) as f32 / sine;
        let matrix: FloatBuffer = BufferUtils::createFloatBuffer(16);
        for i in 0..4
        {
            matrix.putAt(i * 4 + i, 1.0);
        }
        matrix.putAt(0 * 4 + 0, cotangent / aspect);
        matrix.putAt(1 * 4 + 1, cotangent);
        matrix.putAt(2 * 4 + 2, -(zFar + zNear) / deltaZ);
        matrix.putAt(2 * 4 + 3, -1.0);
        matrix.putAt(3 * 4 + 2, -2.0 * zNear * zFar / deltaZ);
        matrix.putAt(3 * 4 + 3, 0.0);
        GL11::glMultMatrix(&matrix);
    }
}
