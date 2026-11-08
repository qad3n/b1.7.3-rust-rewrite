use crate::render::{ClippingHelper, GLAllocation};
use game::math::math_helper;
use lwjgl::buffers::FloatBuffer;
use lwjgl::opengl::GL11;
use std::rc::Rc;

thread_local! {
    static INSTANCE: Rc<ClippingHelperImpl> = Rc::new(ClippingHelperImpl::new());
}

pub struct ClippingHelperImpl
{
    pub m_helper: ClippingHelper,
    m_projectionMatrixBuffer: FloatBuffer,
    m_modelviewMatrixBuffer: FloatBuffer,
    m_clipBuffer: FloatBuffer,
}

fn normalize(frustum: &mut [[f32; 16]; 16], face: usize)
{
    let plane: &mut [f32; 16] = &mut frustum[face];
    let sqrt_float: f32 = math_helper::sqrt_float(plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]);
    plane[0] /= sqrt_float;
    plane[1] /= sqrt_float;
    plane[2] /= sqrt_float;
    plane[3] /= sqrt_float;
}

impl ClippingHelperImpl
{
    fn new() -> ClippingHelperImpl
    {
        ClippingHelperImpl {
            m_helper: ClippingHelper::new(),
            m_projectionMatrixBuffer: GLAllocation::createDirectFloatBuffer(16),
            m_modelviewMatrixBuffer: GLAllocation::createDirectFloatBuffer(16),
            m_clipBuffer: GLAllocation::createDirectFloatBuffer(16),
        }
    }

    pub fn getInstance() -> Rc<ClippingHelperImpl>
    {
        let instance: Rc<ClippingHelperImpl> = INSTANCE.with(Rc::clone);
        instance.init();
        instance
    }

    fn init(&self)
    {
        self.m_projectionMatrixBuffer.clear();
        self.m_modelviewMatrixBuffer.clear();
        self.m_clipBuffer.clear();
        GL11::glGetFloat(2983, &self.m_projectionMatrixBuffer);
        GL11::glGetFloat(2982, &self.m_modelviewMatrixBuffer);
        self.m_projectionMatrixBuffer.flip().setLimit(16);
        self.m_projectionMatrixBuffer.getInto(&mut *self.m_helper.m_projectionMatrix.borrow_mut());
        self.m_modelviewMatrixBuffer.flip().setLimit(16);
        self.m_modelviewMatrixBuffer.getInto(&mut *self.m_helper.m_modelviewMatrix.borrow_mut());
        let modelview: [f32; 16] = *self.m_helper.m_modelviewMatrix.borrow();
        let projection: [f32; 16] = *self.m_helper.m_projectionMatrix.borrow();
        let mut clip: [f32; 16] = [0.0; 16];
        for row in 0..4
        {
            for column in 0..4
            {
                clip[row * 4 + column] = modelview[row * 4] * projection[column]
                    + modelview[row * 4 + 1] * projection[4 + column]
                    + modelview[row * 4 + 2] * projection[8 + column]
                    + modelview[row * 4 + 3] * projection[12 + column];
            }
        }
        *self.m_helper.m_clippingMatrix.borrow_mut() = clip;
        let mut frustum: std::cell::RefMut<'_, [[f32; 16]; 16]> = self.m_helper.m_frustum.borrow_mut();
        let planes: [(usize, f32); 6] = [(0, -1.0), (0, 1.0), (1, 1.0), (1, -1.0), (2, -1.0), (2, 1.0)];
        for (face, (column, sign)) in planes.iter().enumerate()
        {
            for row in 0..4
            {
                frustum[face][row] = clip[row * 4 + 3] + sign * clip[row * 4 + column];
            }
            normalize(&mut frustum, face);
        }
    }
}
