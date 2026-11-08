use crate::model::ModelRenderer;

const MODEL_SCALE: f32 = 0.0625;

pub struct SignModel
{
    pub m_signBoard: ModelRenderer,
    pub m_signStick: ModelRenderer,
}

impl SignModel
{
    pub fn new() -> SignModel
    {
        SignModel {
            m_signBoard: ModelRenderer::new(0, 0).withBox(-12.0, -14.0, -1.0, 24, 12, 2, 0.0),
            m_signStick: ModelRenderer::new(0, 14).withBox(-1.0, -2.0, -1.0, 2, 14, 2, 0.0),
        }
    }

    pub fn render(&self)
    {
        self.m_signBoard.render(MODEL_SCALE);
        self.m_signStick.render(MODEL_SCALE);
    }
}
