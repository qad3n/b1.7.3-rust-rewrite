use jvm::array_list::JArrayList;
use lwjgl::buffers::{ByteBuffer, ByteOrder, FloatBuffer, IntBuffer};
use lwjgl::opengl::GL11;

thread_local! {
    static DISPLAY_LISTS: JArrayList<i32> = JArrayList::new();
    static TEXTURE_NAMES: JArrayList<i32> = JArrayList::new();
}

pub struct GLAllocation;

impl GLAllocation
{
    pub fn generateDisplayLists(s: i32) -> i32
    {
        let n: i32 = GL11::glGenLists(s);
        DISPLAY_LISTS.with(|lists| {
            lists.add(n);
            lists.add(s);
        });
        n
    }

    pub fn generateTextureNames(lists: &IntBuffer)
    {
        GL11::glGenTextures(lists);
        TEXTURE_NAMES.with(|names| {
            for i in lists.position()..lists.limit()
            {
                names.add(lists.getAt(i));
            }
        });
    }

    pub fn releaseList(list: i32)
    {
        DISPLAY_LISTS.with(|lists| {
            let n: i32 = lists.indexOf(&list);
            GL11::glDeleteLists(lists.get(n), lists.get(n + 1));
            lists.removeAt(n);
            lists.removeAt(n);
        });
    }

    pub fn deleteTexturesAndDisplayLists()
    {
        DISPLAY_LISTS.with(|lists| {
            let mut i: i32 = 0;
            while i < lists.size()
            {
                GL11::glDeleteLists(lists.get(i), lists.get(i + 1));
                i += 2;
            }
        });
        TEXTURE_NAMES.with(|names| {
            let intBuffer: IntBuffer = Self::createDirectIntBuffer(names.size());
            intBuffer.flip();
            GL11::glDeleteTextures(&intBuffer);
            for i in 0..names.size()
            {
                intBuffer.put(names.get(i));
            }
            intBuffer.flip();
            GL11::glDeleteTextures(&intBuffer);
        });
        DISPLAY_LISTS.with(|lists| lists.clear());
        TEXTURE_NAMES.with(|names| names.clear());
    }

    pub fn createDirectByteBuffer(capacity: i32) -> ByteBuffer
    {
        ByteBuffer::allocateDirect(capacity).order(ByteOrder::nativeOrder())
    }

    pub fn createDirectIntBuffer(capacity: i32) -> IntBuffer
    {
        Self::createDirectByteBuffer(capacity << 2).asIntBuffer()
    }

    pub fn createDirectFloatBuffer(capacity: i32) -> FloatBuffer
    {
        Self::createDirectByteBuffer(capacity << 2).asFloatBuffer()
    }
}
