use crate::model::model_minecart::side;
use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};

pub struct ModelBoat
{
    m_base: ModelBaseFields,
    pub m_boatSides: Vec<ModelRenderer>,
}

impl ModelBoat
{
    pub fn new() -> ModelBoat
    {
        let n: i32 = 24;
        let n2: i32 = 6;
        let n3: i32 = 20;
        let n4: f32 = 4.0;
        let bottom: ModelRenderer = ModelRenderer::new(0, 8)
            .withBox((-n / 2) as f32, (-n3 / 2 + 2) as f32, -3.0, n, n3 - 4, 4, 0.0)
            .at(0.0, n4, 0.0);
        bottom.m_rotateAngleX.set(1.5707964);
        let sides: [ModelRenderer; 4] = [
            side(n, n2).at((-n / 2 + 1) as f32, n4, 0.0),
            side(n, n2).at((n / 2 - 1) as f32, n4, 0.0),
            side(n, n2).at(0.0, n4, (-n3 / 2 + 1) as f32),
            side(n, n2).at(0.0, n4, (n3 / 2 - 1) as f32),
        ];
        sides[0].m_rotateAngleY.set(4.712389);
        sides[1].m_rotateAngleY.set(1.5707964);
        sides[2].m_rotateAngleY.set(std::f32::consts::PI);
        let mut boatSides: Vec<ModelRenderer> = vec![bottom];
        boatSides.extend(sides);
        ModelBoat { m_base: ModelBaseFields::default(), m_boatSides: boatSides }
    }
}

impl ModelBase for ModelBoat
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, _walk: f32, _walkSpeed: f32, _bob: f32, _yaw: f32, _pitch: f32, scale: f32)
    {
        for part in &self.m_boatSides
        {
            part.render(scale);
        }
    }
}
