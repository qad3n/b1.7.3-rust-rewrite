use crate::render::GLAllocation;
use game::math::Vec3D;
use lwjgl::buffers::FloatBuffer;
use lwjgl::opengl::GL11;
use std::rc::Rc;

thread_local! {
    static BUFFER: FloatBuffer = GLAllocation::createDirectFloatBuffer(16);
}

pub struct RenderHelper;

fn lightBuffer(r: f32, g: f32, b: f32, a: f32, body: impl FnOnce(&FloatBuffer))
{
    BUFFER.with(|buffer| {
        buffer.clear();
        buffer.put(r).put(g).put(b).put(a);
        buffer.flip();
        body(buffer);
    });
}

fn setLight(light: i32, direction: &Vec3D, diffuse: f32, specular: f32)
{
    lightBuffer(direction.x() as f32, direction.y() as f32, direction.z() as f32, 0.0, |buffer| {
        GL11::glLight(light, GL11::GL_POSITION, buffer)
    });
    lightBuffer(diffuse, diffuse, diffuse, 1.0, |buffer| GL11::glLight(light, GL11::GL_DIFFUSE, buffer));
    lightBuffer(0.0, 0.0, 0.0, 1.0, |buffer| GL11::glLight(light, GL11::GL_AMBIENT, buffer));
    lightBuffer(specular, specular, specular, 1.0, |buffer| GL11::glLight(light, GL11::GL_SPECULAR, buffer));
}

impl RenderHelper
{
    pub fn disableStandardItemLighting()
    {
        GL11::glDisable(GL11::GL_LIGHTING);
        GL11::glDisable(GL11::GL_LIGHT0);
        GL11::glDisable(GL11::GL_LIGHT1);
        GL11::glDisable(GL11::GL_COLOR_MATERIAL);
    }

    pub fn enableStandardItemLighting()
    {
        GL11::glEnable(GL11::GL_LIGHTING);
        GL11::glEnable(GL11::GL_LIGHT0);
        GL11::glEnable(GL11::GL_LIGHT1);
        GL11::glEnable(GL11::GL_COLOR_MATERIAL);
        GL11::glColorMaterial(GL11::GL_FRONT_AND_BACK, GL11::GL_AMBIENT_AND_DIFFUSE);
        let f: f32 = 0.4;
        let f2: f32 = 0.6;
        let f3: f32 = 0.0;
        let vec3D: Rc<Vec3D> = Vec3D::createVector(0.2f32 as f64, 1.0, -0.7f32 as f64).normalize();
        setLight(GL11::GL_LIGHT0, &vec3D, f2, f3);
        let vec3D: Rc<Vec3D> = Vec3D::createVector(-0.2f32 as f64, 1.0, 0.7f32 as f64).normalize();
        setLight(GL11::GL_LIGHT1, &vec3D, f2, f3);
        GL11::glShadeModel(GL11::GL_FLAT);
        lightBuffer(f, f, f, 1.0, |buffer| GL11::glLightModel(GL11::GL_LIGHT_MODEL_AMBIENT, buffer));
    }
}
