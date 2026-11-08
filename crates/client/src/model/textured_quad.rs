use crate::model::PositionTextureVertex;
use crate::render::Tessellator;
use game::math::Vec3D;
use std::rc::Rc;

const U_INSET: f32 = 0.0015625;
const V_INSET: f32 = 0.003125;
const TEXTURE_WIDTH: f32 = 64.0;
const TEXTURE_HEIGHT: f32 = 32.0;

pub struct TexturedQuad
{
    pub m_vertexPositions: Vec<PositionTextureVertex>,
    pub m_nVertices: i32,
}

impl TexturedQuad
{
    pub fn new(vertices: Vec<PositionTextureVertex>) -> TexturedQuad
    {
        let count: i32 = vertices.len() as i32;
        TexturedQuad { m_vertexPositions: vertices, m_nVertices: count }
    }

    pub fn withTexture(mut vertices: Vec<PositionTextureVertex>, u1: i32, v1: i32, u2: i32, v2: i32) -> TexturedQuad
    {
        let f: f32 = U_INSET;
        let f2: f32 = V_INSET;
        vertices[0] = vertices[0].setTexturePosition(u2 as f32 / TEXTURE_WIDTH - f, v1 as f32 / TEXTURE_HEIGHT + f2);
        vertices[1] = vertices[1].setTexturePosition(u1 as f32 / TEXTURE_WIDTH + f, v1 as f32 / TEXTURE_HEIGHT + f2);
        vertices[2] = vertices[2].setTexturePosition(u1 as f32 / TEXTURE_WIDTH + f, v2 as f32 / TEXTURE_HEIGHT - f2);
        vertices[3] = vertices[3].setTexturePosition(u2 as f32 / TEXTURE_WIDTH - f, v2 as f32 / TEXTURE_HEIGHT - f2);
        Self::new(vertices)
    }

    pub fn flipFace(&mut self) { self.m_vertexPositions.reverse(); }

    pub fn draw(&self, tesselator: &Tessellator, scale: f32)
    {
        let vertices: &[PositionTextureVertex] = &self.m_vertexPositions;
        let vec3D: Rc<Vec3D> = vertices[1].m_vector3D.subtract(&vertices[0].m_vector3D);
        let vec3D2: Rc<Vec3D> = vertices[1].m_vector3D.subtract(&vertices[2].m_vector3D);
        let vec3D3: Rc<Vec3D> = vec3D2.crossProduct(&vec3D).normalize();
        tesselator.startDrawingQuads();
        tesselator.setNormal(vec3D3.x() as f32, vec3D3.y() as f32, vec3D3.z() as f32);
        for vertex in &vertices[..4]
        {
            let position: &Vec3D = &vertex.m_vector3D;
            tesselator.addVertexWithUV(
                (position.x() as f32 * scale) as f64,
                (position.y() as f32 * scale) as f64,
                (position.z() as f32 * scale) as f64,
                vertex.m_texturePositionX as f64,
                vertex.m_texturePositionY as f64,
            );
        }
        tesselator.draw();
    }
}
