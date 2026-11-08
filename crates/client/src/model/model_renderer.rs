use crate::model::{PositionTextureVertex, TexturedQuad};
use crate::render::{GLAllocation, Tessellator};
use lwjgl::opengl::GL11;
use std::cell::Cell;

const RADIANS_TO_DEGREES: f32 = 57.295776;

pub struct ModelRenderer
{
    m_faces: Vec<TexturedQuad>,
    m_textureOffsetX: i32,
    m_textureOffsetY: i32,
    pub m_rotationPointX: Cell<f32>,
    pub m_rotationPointY: Cell<f32>,
    pub m_rotationPointZ: Cell<f32>,
    pub m_rotateAngleX: Cell<f32>,
    pub m_rotateAngleY: Cell<f32>,
    pub m_rotateAngleZ: Cell<f32>,
    m_compiled: Cell<bool>,
    m_displayList: Cell<i32>,
    pub m_mirror: bool,
    pub m_showModel: Cell<bool>,
    pub m_invisible: Cell<bool>,
}

impl ModelRenderer
{
    pub fn new(u: i32, v: i32) -> ModelRenderer
    {
        ModelRenderer {
            m_faces: Vec::new(),
            m_textureOffsetX: u,
            m_textureOffsetY: v,
            m_rotationPointX: Cell::new(0.0),
            m_rotationPointY: Cell::new(0.0),
            m_rotationPointZ: Cell::new(0.0),
            m_rotateAngleX: Cell::new(0.0),
            m_rotateAngleY: Cell::new(0.0),
            m_rotateAngleZ: Cell::new(0.0),
            m_compiled: Cell::new(false),
            m_displayList: Cell::new(0),
            m_mirror: false,
            m_showModel: Cell::new(true),
            m_invisible: Cell::new(false),
        }
    }

    pub fn mirrored(u: i32, v: i32) -> ModelRenderer
    {
        let mut renderer: ModelRenderer = Self::new(u, v);
        renderer.m_mirror = true;
        renderer
    }

    pub fn addBox(&mut self, x: f32, y: f32, z: f32, sizeX: i32, sizeY: i32, sizeZ: i32, increase: f32)
    {
        let mut x: f32 = x;
        let mut f: f32 = x + sizeX as f32;
        let f2: f32 = y + sizeY as f32;
        let f3: f32 = z + sizeZ as f32;
        x -= increase;
        let y: f32 = y - increase;
        let z: f32 = z - increase;
        f += increase;
        let f2: f32 = f2 + increase;
        let f3: f32 = f3 + increase;
        if self.m_mirror
        {
            std::mem::swap(&mut x, &mut f);
        }
        let v1: PositionTextureVertex = PositionTextureVertex::new(x, y, z, 0.0, 0.0);
        let v2: PositionTextureVertex = PositionTextureVertex::new(f, y, z, 0.0, 8.0);
        let v3: PositionTextureVertex = PositionTextureVertex::new(f, f2, z, 8.0, 8.0);
        let v4: PositionTextureVertex = PositionTextureVertex::new(x, f2, z, 8.0, 0.0);
        let v5: PositionTextureVertex = PositionTextureVertex::new(x, y, f3, 0.0, 0.0);
        let v6: PositionTextureVertex = PositionTextureVertex::new(f, y, f3, 0.0, 8.0);
        let v7: PositionTextureVertex = PositionTextureVertex::new(f, f2, f3, 8.0, 8.0);
        let v8: PositionTextureVertex = PositionTextureVertex::new(x, f2, f3, 8.0, 0.0);
        let u: i32 = self.m_textureOffsetX;
        let v: i32 = self.m_textureOffsetY;
        let quad = |corners: [&PositionTextureVertex; 4], u1: i32, v1: i32, u2: i32, v2: i32| -> TexturedQuad {
            TexturedQuad::withTexture(corners.map(Clone::clone).to_vec(), u1, v1, u2, v2)
        };
        self.m_faces = vec![
            quad([&v6, &v2, &v3, &v7], u + sizeZ + sizeX, v + sizeZ, u + sizeZ + sizeX + sizeZ, v + sizeZ + sizeY),
            quad([&v1, &v5, &v8, &v4], u, v + sizeZ, u + sizeZ, v + sizeZ + sizeY),
            quad([&v6, &v5, &v1, &v2], u + sizeZ, v, u + sizeZ + sizeX, v + sizeZ),
            quad([&v3, &v4, &v8, &v7], u + sizeZ + sizeX, v, u + sizeZ + sizeX + sizeX, v + sizeZ),
            quad([&v2, &v1, &v4, &v3], u + sizeZ, v + sizeZ, u + sizeZ + sizeX, v + sizeZ + sizeY),
            quad(
                [&v5, &v6, &v7, &v8],
                u + sizeZ + sizeX + sizeZ,
                v + sizeZ,
                u + sizeZ + sizeX + sizeZ + sizeX,
                v + sizeZ + sizeY,
            ),
        ];
        if self.m_mirror
        {
            for face in &mut self.m_faces
            {
                face.flipFace();
            }
        }
    }

    pub fn withBox(mut self, x: f32, y: f32, z: f32, sizeX: i32, sizeY: i32, sizeZ: i32, increase: f32)
    -> ModelRenderer
    {
        self.addBox(x, y, z, sizeX, sizeY, sizeZ, increase);
        self
    }

    pub fn setRotationPoint(&self, x: f32, y: f32, z: f32)
    {
        self.m_rotationPointX.set(x);
        self.m_rotationPointY.set(y);
        self.m_rotationPointZ.set(z);
    }

    pub fn at(self, x: f32, y: f32, z: f32) -> ModelRenderer
    {
        self.setRotationPoint(x, y, z);
        self
    }

    fn isRotated(&self) -> bool
    {
        self.m_rotateAngleX.get() != 0.0 || self.m_rotateAngleY.get() != 0.0 || self.m_rotateAngleZ.get() != 0.0
    }

    fn isOffset(&self) -> bool
    {
        self.m_rotationPointX.get() != 0.0 || self.m_rotationPointY.get() != 0.0 || self.m_rotationPointZ.get() != 0.0
    }

    fn translateToPivot(&self, scale: f32)
    {
        GL11::glTranslatef(
            self.m_rotationPointX.get() * scale,
            self.m_rotationPointY.get() * scale,
            self.m_rotationPointZ.get() * scale,
        );
    }

    fn rotate(angle: f32, x: f32, y: f32, z: f32)
    {
        if angle != 0.0
        {
            GL11::glRotatef(angle * RADIANS_TO_DEGREES, x, y, z);
        }
    }

    fn rotateZYX(&self)
    {
        Self::rotate(self.m_rotateAngleZ.get(), 0.0, 0.0, 1.0);
        Self::rotate(self.m_rotateAngleY.get(), 0.0, 1.0, 0.0);
        Self::rotate(self.m_rotateAngleX.get(), 1.0, 0.0, 0.0);
    }

    fn prepare(&self, scale: f32) -> bool
    {
        if self.m_invisible.get() || !self.m_showModel.get()
        {
            return false;
        }
        if !self.m_compiled.get()
        {
            self.compileDisplayList(scale);
        }
        true
    }

    pub fn render(&self, scale: f32)
    {
        if !self.prepare(scale)
        {
            return;
        }
        if self.isRotated()
        {
            GL11::glPushMatrix();
            self.translateToPivot(scale);
            self.rotateZYX();
            GL11::glCallList(self.m_displayList.get());
            GL11::glPopMatrix();
        }
        else if self.isOffset()
        {
            self.translateToPivot(scale);
            GL11::glCallList(self.m_displayList.get());
            GL11::glTranslatef(
                -self.m_rotationPointX.get() * scale,
                -self.m_rotationPointY.get() * scale,
                -self.m_rotationPointZ.get() * scale,
            );
        }
        else
        {
            GL11::glCallList(self.m_displayList.get());
        }
    }

    pub fn renderWithRotation(&self, scale: f32)
    {
        if !self.prepare(scale)
        {
            return;
        }
        GL11::glPushMatrix();
        self.translateToPivot(scale);
        Self::rotate(self.m_rotateAngleY.get(), 0.0, 1.0, 0.0);
        Self::rotate(self.m_rotateAngleX.get(), 1.0, 0.0, 0.0);
        Self::rotate(self.m_rotateAngleZ.get(), 0.0, 0.0, 1.0);
        GL11::glCallList(self.m_displayList.get());
        GL11::glPopMatrix();
    }

    pub fn postRender(&self, scale: f32)
    {
        if !self.prepare(scale)
        {
            return;
        }
        if self.isRotated()
        {
            self.translateToPivot(scale);
            self.rotateZYX();
        }
        else if self.isOffset()
        {
            self.translateToPivot(scale);
        }
    }

    fn compileDisplayList(&self, scale: f32)
    {
        self.m_displayList.set(GLAllocation::generateDisplayLists(1));
        GL11::glNewList(self.m_displayList.get(), GL11::GL_COMPILE);
        let tessellator: std::rc::Rc<Tessellator> = Tessellator::instance();
        for face in &self.m_faces
        {
            face.draw(&tessellator, scale);
        }
        GL11::glEndList();
        self.m_compiled.set(true);
    }
}
