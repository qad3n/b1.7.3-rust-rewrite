mod arb;
mod bindings;
mod display;
mod display_mode;
mod gl11;

pub use arb::{ARBOcclusionQuery, ARBVertexBufferObject, NVFogDistance};
pub use display::Display;
pub use display_mode::DisplayMode;
pub use gl11::{GL11, GL12};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static GL: RefCell<Option<Rc<bindings::Gl>>> = const { RefCell::new(None) };
    static CAPABILITIES: RefCell<Option<Rc<ContextCapabilities>>> = const { RefCell::new(None) };
}

// SAFETY: the bindings were loaded from the current context and every pointer argument comes from a live
// buffer of the size the call reads or writes, as lwjgl's own checks guarantee
macro_rules! glCall {
    ($function:ident($($argument:expr),* $(,)?)) => {
        $crate::opengl::gl(|gl| unsafe { gl.$function($($argument),*) })
    };
}

pub(crate) use glCall;

// runs a raw gl call, lwjgl throws the same way when no context is current
pub(crate) fn gl<R>(body: impl FnOnce(&bindings::Gl) -> R) -> R
{
    let gl: Rc<bindings::Gl> = GL.with(|gl| gl.borrow().clone()).unwrap_or_else(|| {
        jvm::throwable::throw(&jvm::throwable::RUNTIME, "No OpenGL context found in the current thread.")
    });
    body(&gl)
}

// the extension flags the game reads from GLContext.getCapabilities()
pub struct ContextCapabilities
{
    pub GL_ARB_occlusion_query: bool,
    pub GL_ARB_vertex_buffer_object: bool,
    pub GL_NV_fog_distance: bool,
}

pub struct GLContext;

impl GLContext
{
    pub(crate) fn load(loader: impl FnMut(&'static str) -> *const std::ffi::c_void)
    {
        GL.with(|gl| *gl.borrow_mut() = Some(Rc::new(bindings::Gl::load_with(loader))));
        let extensions: String = GL11::glGetString(GL11::GL_EXTENSIONS).unwrap_or_default();
        let has = |name: &str| -> bool { extensions.split_ascii_whitespace().any(|extension| extension == name) };
        let capabilities: ContextCapabilities = ContextCapabilities {
            GL_ARB_occlusion_query: has("GL_ARB_occlusion_query"),
            GL_ARB_vertex_buffer_object: has("GL_ARB_vertex_buffer_object"),
            GL_NV_fog_distance: has("GL_NV_fog_distance"),
        };
        CAPABILITIES.with(|current| *current.borrow_mut() = Some(Rc::new(capabilities)));
    }

    pub(crate) fn unload()
    {
        GL.with(|gl| *gl.borrow_mut() = None);
        CAPABILITIES.with(|current| *current.borrow_mut() = None);
    }

    pub fn getCapabilities() -> Rc<ContextCapabilities>
    {
        CAPABILITIES.with(|current| current.borrow().clone()).unwrap_or_else(|| {
            jvm::throwable::throw(&jvm::throwable::RUNTIME, "No OpenGL context found in the current thread.")
        })
    }
}
