use crate::render::EntityRenderer;
use crate::settings::GameSettings;
use game::block::Material;
use game::entity::Entity;
use game::math::Vec3D;
use game::world::World;
use jvm::throwable::OrNpe;
use lwjgl::buffers::FloatBuffer;
use lwjgl::opengl::{GL11, GLContext};
use std::rc::Rc;

impl EntityRenderer
{
    fn setFogColor(&self, r: f32, g: f32, b: f32)
    {
        self.m_fogColorRed.set(r);
        self.m_fogColorGreen.set(g);
        self.m_fogColorBlue.set(b);
    }

    fn scaleFogColor(&self, r: f32, g: f32, b: f32)
    {
        self.setFogColor(self.m_fogColorRed.get() * r, self.m_fogColorGreen.get() * g, self.m_fogColorBlue.get() * b);
    }

    pub(super) fn updateFogColor(&self, tickDelta: f32)
    {
        let world: Rc<World> = self.theWorld();
        let entityLiving: Rc<Entity> = self.renderViewEntity();
        let settings: Rc<GameSettings> = self.gameSettings();
        let mut f3: f32 = 1.0 / (4 - settings.m_renderDistance.get()) as f32;
        f3 = 1.0 - jvm::math::pow(f3 as f64, 0.25) as f32;
        let vec3D: Rc<Vec3D> = world.getSkyColor(&entityLiving, tickDelta);
        let f4: f32 = vec3D.m_xCoord.get() as f32;
        let f5: f32 = vec3D.m_yCoord.get() as f32;
        let f6: f32 = vec3D.m_zCoord.get() as f32;
        let vec3D2: Rc<Vec3D> = world.getFogColor(tickDelta);
        self.setFogColor(vec3D2.m_xCoord.get() as f32, vec3D2.m_yCoord.get() as f32, vec3D2.m_zCoord.get() as f32);
        self.setFogColor(
            self.m_fogColorRed.get() + (f4 - self.m_fogColorRed.get()) * f3,
            self.m_fogColorGreen.get() + (f5 - self.m_fogColorGreen.get()) * f3,
            self.m_fogColorBlue.get() + (f6 - self.m_fogColorBlue.get()) * f3,
        );
        let f7: f32 = world.getRain(tickDelta);
        if f7 > 0.0
        {
            let f2: f32 = 1.0 - f7 * 0.5;
            let f: f32 = 1.0 - f7 * 0.4;
            self.scaleFogColor(f2, f2, f);
        }
        let f2: f32 = world.getThunder(tickDelta);
        if f2 > 0.0
        {
            let f: f32 = 1.0 - f2 * 0.5;
            self.scaleFogColor(f, f, f);
        }
        if self.m_cloudFog.get()
        {
            let vec3D3: Rc<Vec3D> = world.getCloudColor(tickDelta);
            self.setFogColor(vec3D3.m_xCoord.get() as f32, vec3D3.m_yCoord.get() as f32, vec3D3.m_zCoord.get() as f32);
        }
        else if entityLiving.isInsideOfMaterial(&Material::water())
        {
            self.setFogColor(0.02, 0.02, 0.2);
        }
        else if entityLiving.isInsideOfMaterial(&Material::lava())
        {
            self.setFogColor(0.6, 0.1, 0.0);
        }
        let f8: f32 = self.m_fogColor2.get() + (self.m_fogColor1.get() - self.m_fogColor2.get()) * tickDelta;
        self.scaleFogColor(f8, f8, f8);
        if settings.m_anaglyph.get()
        {
            let (r, g, b): (f32, f32, f32) =
                (self.m_fogColorRed.get(), self.m_fogColorGreen.get(), self.m_fogColorBlue.get());
            self.setFogColor(
                (r * 30.0 + g * 59.0 + b * 11.0) / 100.0,
                (r * 30.0 + g * 70.0) / 100.0,
                (r * 30.0 + b * 70.0) / 100.0,
            );
        }
        GL11::glClearColor(self.m_fogColorRed.get(), self.m_fogColorGreen.get(), self.m_fogColorBlue.get(), 0.0);
    }

    pub(super) fn setupFog(&self, mode: i32, _tickDelta: f32)
    {
        let entityLiving: Rc<Entity> = self.renderViewEntity();
        GL11::glFog(
            2918,
            self.updateColorBuffer(
                self.m_fogColorRed.get(),
                self.m_fogColorGreen.get(),
                self.m_fogColorBlue.get(),
                1.0,
            ),
        );
        GL11::glNormal3f(0.0, -1.0, 0.0);
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        if self.m_cloudFog.get()
        {
            GL11::glFogi(2917, 2048);
            GL11::glFogf(2914, 0.1);
        }
        else if entityLiving.isInsideOfMaterial(&Material::water())
        {
            GL11::glFogi(2917, 2048);
            GL11::glFogf(2914, 0.1);
        }
        else if entityLiving.isInsideOfMaterial(&Material::lava())
        {
            GL11::glFogi(2917, 2048);
            GL11::glFogf(2914, 2.0);
        }
        else
        {
            let farPlaneDistance: f32 = self.m_farPlaneDistance.get();
            GL11::glFogi(2917, 9729);
            GL11::glFogf(2915, farPlaneDistance * 0.25);
            GL11::glFogf(2916, farPlaneDistance);
            if mode < 0
            {
                GL11::glFogf(2915, 0.0);
                GL11::glFogf(2916, farPlaneDistance * 0.8);
            }
            if GLContext::getCapabilities().GL_NV_fog_distance
            {
                GL11::glFogi(34138, 34139);
            }
            if self.m_mc.m_theWorld.get().orNpe().worldProvider().m_isNether.get()
            {
                GL11::glFogf(2915, 0.0);
            }
        }
        GL11::glEnable(2903);
        GL11::glColorMaterial(1028, 4608);
    }

    fn updateColorBuffer(&self, f1: f32, f2: f32, f3: f32, f4: f32) -> &FloatBuffer
    {
        self.m_fogColorBuffer.clear();
        self.m_fogColorBuffer.put(f1).put(f2).put(f3).put(f4);
        self.m_fogColorBuffer.flip();
        &self.m_fogColorBuffer
    }
}
