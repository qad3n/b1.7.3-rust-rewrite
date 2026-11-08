use lwjgl::opengl::GLContext;

const TRY_CHECK_OCCLUSION_CAPABLE: bool = true;

pub struct OpenGlCapsChecker;

impl OpenGlCapsChecker
{
    pub fn checkARBOcclusion(&self) -> bool
    {
        TRY_CHECK_OCCLUSION_CAPABLE && GLContext::getCapabilities().GL_ARB_occlusion_query
    }
}
