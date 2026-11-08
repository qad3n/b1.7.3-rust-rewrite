use crate::buffers::{ByteBuffer, IntBuffer};
use crate::opengl::glCall;

pub struct ARBOcclusionQuery;

impl ARBOcclusionQuery
{
    pub const GL_SAMPLES_PASSED_ARB: i32 = 0x8914;
    pub const GL_QUERY_RESULT_ARB: i32 = 0x8866;
    pub const GL_QUERY_RESULT_AVAILABLE_ARB: i32 = 0x8867;

    pub fn glGenQueriesARB(ids: &IntBuffer)
    {
        glCall!(GenQueriesARB(ids.remaining(), ids.addressMut().cast()));
    }

    pub fn glBeginQueryARB(target: i32, id: i32)
    {
        glCall!(BeginQueryARB(target as u32, id as u32));
    }

    pub fn glEndQueryARB(target: i32)
    {
        glCall!(EndQueryARB(target as u32));
    }

    pub fn glGetQueryObjectuARB(id: i32, pname: i32, params: &IntBuffer)
    {
        glCall!(GetQueryObjectuivARB(id as u32, pname as u32, params.addressMut().cast()));
    }
}

pub struct ARBVertexBufferObject;

impl ARBVertexBufferObject
{
    pub const GL_ARRAY_BUFFER_ARB: i32 = 0x8892;
    pub const GL_STREAM_DRAW_ARB: i32 = 0x88E0;

    pub fn glGenBuffersARB(buffers: &IntBuffer)
    {
        glCall!(GenBuffersARB(buffers.remaining(), buffers.addressMut().cast()));
    }

    pub fn glBindBufferARB(target: i32, buffer: i32)
    {
        glCall!(BindBufferARB(target as u32, buffer as u32));
    }

    // uploads the bytes between position and limit
    pub fn glBufferDataARB(target: i32, data: &ByteBuffer, usage: i32)
    {
        glCall!(BufferDataARB(target as u32, data.remaining() as isize, data.address(), usage as u32));
    }
}

pub struct NVFogDistance;

impl NVFogDistance
{
    pub const GL_FOG_DISTANCE_MODE_NV: i32 = 0x855A;
    pub const GL_EYE_RADIAL_NV: i32 = 0x855B;
}
