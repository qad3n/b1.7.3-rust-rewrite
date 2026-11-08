use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub static BUFFER_OVERFLOW: jvm::throwable::ThrowableClass = jvm::throwable::ThrowableClass {
    m_name: "java.nio.BufferOverflowException",
    m_parent: Some(&jvm::throwable::RUNTIME),
};
pub static BUFFER_UNDERFLOW: jvm::throwable::ThrowableClass = jvm::throwable::ThrowableClass {
    m_name: "java.nio.BufferUnderflowException",
    m_parent: Some(&jvm::throwable::RUNTIME),
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ByteOrder
{
    BigEndian,
    LittleEndian,
}

impl ByteOrder
{
    pub fn nativeOrder() -> ByteOrder
    {
        if cfg!(target_endian = "big")
        {
            ByteOrder::BigEndian
        }
        else
        {
            ByteOrder::LittleEndian
        }
    }
}

// a java.nio element type stored in the shared byte block
pub trait Element: Copy + Default
{
    const SIZE: usize;

    fn read(bytes: &[u8], order: ByteOrder) -> Self;

    fn write(self, bytes: &mut [u8], order: ByteOrder);
}

macro_rules! element {
    ($type:ty, $size:expr) => {
        impl Element for $type
        {
            const SIZE: usize = $size;

            fn read(bytes: &[u8], order: ByteOrder) -> Self
            {
                let raw: [u8; $size] = bytes[..$size].try_into().expect("slice holds one element");
                match order
                {
                    ByteOrder::BigEndian => <$type>::from_be_bytes(raw),
                    ByteOrder::LittleEndian => <$type>::from_le_bytes(raw),
                }
            }

            fn write(self, bytes: &mut [u8], order: ByteOrder)
            {
                let raw: [u8; $size] = match order
                {
                    ByteOrder::BigEndian => self.to_be_bytes(),
                    ByteOrder::LittleEndian => self.to_le_bytes(),
                };
                bytes[..$size].copy_from_slice(&raw);
            }
        }
    };
}

element!(i8, 1);
element!(i32, 4);
element!(f32, 4);

// a direct java.nio buffer, views made by asIntBuffer share the bytes but keep their own position and limit
pub struct Buffer<T: Element>
{
    m_bytes: Rc<RefCell<Vec<u8>>>,
    m_capacity: i32,
    m_position: Cell<i32>,
    m_limit: Cell<i32>,
    m_mark: Cell<i32>,
    m_order: Cell<ByteOrder>,
    m_element: std::marker::PhantomData<T>,
}

pub type ByteBuffer = Buffer<i8>;
pub type IntBuffer = Buffer<i32>;
pub type FloatBuffer = Buffer<f32>;

impl ByteBuffer
{
    // ByteBuffer.allocateDirect starts in big endian order like every java byte buffer
    pub fn allocateDirect(capacity: i32) -> ByteBuffer
    {
        if capacity < 0
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_ARGUMENT, &capacity.to_string());
        }
        Buffer::view(Rc::new(RefCell::new(vec![0; capacity as usize])), capacity, ByteOrder::BigEndian)
    }

    pub fn order(self, order: ByteOrder) -> ByteBuffer
    {
        self.m_order.set(order);
        self
    }

    // getInt(int) and putInt(int, int), absolute byte offsets read in the buffer's order
    pub fn getIntAt(&self, index: i32) -> i32
    {
        if index < 0 || index + 4 > self.m_limit.get()
        {
            jvm::throwable::throwBare(&jvm::throwable::INDEX_OUT_OF_BOUNDS);
        }
        i32::read(&self.m_bytes.borrow()[index as usize..], self.m_order.get())
    }

    pub fn putIntAt(&self, index: i32, value: i32) -> &Self
    {
        if index < 0 || index + 4 > self.m_limit.get()
        {
            jvm::throwable::throwBare(&jvm::throwable::INDEX_OUT_OF_BOUNDS);
        }
        value.write(&mut self.m_bytes.borrow_mut()[index as usize..], self.m_order.get());
        self
    }

    // the views cover the remaining bytes, which the game only takes from a fresh buffer
    pub fn asIntBuffer(&self) -> IntBuffer { self.asView() }

    pub fn asFloatBuffer(&self) -> FloatBuffer { self.asView() }

    fn asView<U: Element>(&self) -> Buffer<U>
    {
        if self.m_position.get() != 0
        {
            jvm::throwable::throw(&jvm::throwable::ILLEGAL_STATE, "views start at the buffer origin in this stand-in");
        }
        Buffer::view(Rc::clone(&self.m_bytes), self.remaining() / U::SIZE as i32, self.m_order.get())
    }
}

impl<T: Element> Buffer<T>
{
    fn view(bytes: Rc<RefCell<Vec<u8>>>, capacity: i32, order: ByteOrder) -> Buffer<T>
    {
        Buffer {
            m_bytes: bytes,
            m_capacity: capacity,
            m_position: Cell::new(0),
            m_limit: Cell::new(capacity),
            m_mark: Cell::new(-1),
            m_order: Cell::new(order),
            m_element: std::marker::PhantomData,
        }
    }

    pub fn capacity(&self) -> i32 { self.m_capacity }

    pub fn position(&self) -> i32 { self.m_position.get() }

    pub fn setPosition(&self, position: i32) -> &Self
    {
        if position > self.m_limit.get() || position < 0
        {
            jvm::throwable::throwBare(&jvm::throwable::ILLEGAL_ARGUMENT);
        }
        if self.m_mark.get() > position
        {
            self.m_mark.set(-1);
        }
        self.m_position.set(position);
        self
    }

    pub fn limit(&self) -> i32 { self.m_limit.get() }

    pub fn setLimit(&self, limit: i32) -> &Self
    {
        if limit > self.m_capacity || limit < 0
        {
            jvm::throwable::throwBare(&jvm::throwable::ILLEGAL_ARGUMENT);
        }
        self.m_limit.set(limit);
        if self.m_position.get() > limit
        {
            self.m_position.set(limit);
        }
        if self.m_mark.get() > limit
        {
            self.m_mark.set(-1);
        }
        self
    }

    pub fn clear(&self) -> &Self
    {
        self.m_position.set(0);
        self.m_limit.set(self.m_capacity);
        self.m_mark.set(-1);
        self
    }

    pub fn flip(&self) -> &Self
    {
        self.m_limit.set(self.m_position.get());
        self.m_position.set(0);
        self.m_mark.set(-1);
        self
    }

    pub fn rewind(&self) -> &Self
    {
        self.m_position.set(0);
        self.m_mark.set(-1);
        self
    }

    pub fn remaining(&self) -> i32 { self.m_limit.get() - self.m_position.get() }

    pub fn hasRemaining(&self) -> bool { self.m_position.get() < self.m_limit.get() }

    fn checkIndex(&self, index: i32) -> usize
    {
        if index < 0 || index >= self.m_limit.get()
        {
            jvm::throwable::throwBare(&jvm::throwable::INDEX_OUT_OF_BOUNDS);
        }
        index as usize * T::SIZE
    }

    fn nextIndex(&self, count: i32, overflow: &'static jvm::throwable::ThrowableClass) -> i32
    {
        let position: i32 = self.m_position.get();
        if self.m_limit.get() - position < count
        {
            jvm::throwable::throwBare(overflow);
        }
        self.m_position.set(position + count);
        position
    }

    pub fn getAt(&self, index: i32) -> T
    {
        let offset: usize = self.checkIndex(index);
        T::read(&self.m_bytes.borrow()[offset..], self.m_order.get())
    }

    pub fn putAt(&self, index: i32, value: T) -> &Self
    {
        let offset: usize = self.checkIndex(index);
        value.write(&mut self.m_bytes.borrow_mut()[offset..], self.m_order.get());
        self
    }

    pub fn get(&self) -> T
    {
        let index: i32 = self.nextIndex(1, &BUFFER_UNDERFLOW);
        self.getAt(index)
    }

    pub fn put(&self, value: T) -> &Self
    {
        let index: i32 = self.nextIndex(1, &BUFFER_OVERFLOW);
        self.putAt(index, value)
    }

    // get(T[]) fills the whole slice
    pub fn getInto(&self, dst: &mut [T]) -> &Self
    {
        let start: i32 = self.nextIndex(dst.len() as i32, &BUFFER_UNDERFLOW);
        for (i, value) in dst.iter_mut().enumerate()
        {
            *value = self.getAt(start + i as i32);
        }
        self
    }

    // put(T[], int, int)
    pub fn putFrom(&self, src: &[T], offset: i32, length: i32) -> &Self
    {
        let start: i32 = self.nextIndex(length, &BUFFER_OVERFLOW);
        for i in 0..length
        {
            self.putAt(start + i, src[(offset + i) as usize]);
        }
        self
    }

    pub fn putSlice(&self, src: &[T]) -> &Self { self.putFrom(src, 0, src.len() as i32) }

    // address of the element at the position, what lwjgl hands to the gl call
    pub(crate) fn address(&self) -> *const std::ffi::c_void
    {
        let offset: usize = self.m_position.get() as usize * T::SIZE;
        self.m_bytes.borrow()[offset..].as_ptr().cast()
    }

    pub(crate) fn addressMut(&self) -> *mut std::ffi::c_void
    {
        let offset: usize = self.m_position.get() as usize * T::SIZE;
        self.m_bytes.borrow_mut()[offset..].as_mut_ptr().cast()
    }
}

// org.lwjgl.BufferUtils, direct buffers in native order
pub struct BufferUtils;

impl BufferUtils
{
    pub fn createByteBuffer(size: i32) -> ByteBuffer
    {
        ByteBuffer::allocateDirect(size).order(ByteOrder::nativeOrder())
    }

    pub fn createIntBuffer(size: i32) -> IntBuffer { Self::createByteBuffer(size << 2).asIntBuffer() }

    pub fn createFloatBuffer(size: i32) -> FloatBuffer { Self::createByteBuffer(size << 2).asFloatBuffer() }
}
