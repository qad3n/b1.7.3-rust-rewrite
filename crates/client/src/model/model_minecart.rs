use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};

pub struct ModelMinecart
{
    m_base: ModelBaseFields,
    pub m_sideModels: Vec<ModelRenderer>,
}

pub(super) fn side(n: i32, n2: i32) -> ModelRenderer
{
    ModelRenderer::new(0, 0).withBox((-n / 2 + 2) as f32, (-n2 - 1) as f32, -1.0, n - 4, n2, 2, 0.0)
}

impl ModelMinecart
{
    pub fn new() -> ModelMinecart
    {
        let n: i32 = 20;
        let n2: i32 = 8;
        let n3: i32 = 16;
        let n4: f32 = 4.0;
        let floor: ModelRenderer =
            ModelRenderer::new(0, 10).withBox((-n / 2) as f32, (-n3 / 2) as f32, -1.0, n, n3, 2, 0.0).at(0.0, n4, 0.0);
        let inside: ModelRenderer = ModelRenderer::new(44, 10)
            .withBox((-n / 2 + 1) as f32, (-n3 / 2 + 1) as f32, -1.0, n - 2, n3 - 2, 1, 0.0)
            .at(0.0, n4, 0.0);
        let sides: [ModelRenderer; 4] = [
            side(n, n2).at((-n / 2 + 1) as f32, n4, 0.0),
            side(n, n2).at((n / 2 - 1) as f32, n4, 0.0),
            side(n, n2).at(0.0, n4, (-n3 / 2 + 1) as f32),
            side(n, n2).at(0.0, n4, (n3 / 2 - 1) as f32),
        ];
        floor.m_rotateAngleX.set(1.5707964);
        sides[0].m_rotateAngleY.set(4.712389);
        sides[1].m_rotateAngleY.set(1.5707964);
        sides[2].m_rotateAngleY.set(std::f32::consts::PI);
        inside.m_rotateAngleX.set(-1.5707964);
        let mut sideModels: Vec<ModelRenderer> = vec![floor];
        sideModels.extend(sides);
        sideModels.push(inside);
        ModelMinecart { m_base: ModelBaseFields::default(), m_sideModels: sideModels }
    }
}

impl ModelBase for ModelMinecart
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, _walk: f32, _walkSpeed: f32, bob: f32, _yaw: f32, _pitch: f32, scale: f32)
    {
        self.m_sideModels[5].m_rotationPointY.set(4.0 - bob);
        for part in &self.m_sideModels[..6]
        {
            part.render(scale);
        }
    }
}
