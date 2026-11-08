use crate::render::{EntityRenderer, RenderGlobal, RenderHelper, Tessellator};
use game::entity::Entity;
use game::math::{Vec3D, math_helper};
use game::world::{World, WorldProvider};
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::rc::Rc;

fn anaglyphColor(r: f32, g: f32, b: f32) -> (f32, f32, f32)
{
    ((r * 30.0 + g * 59.0 + b * 11.0) / 100.0, (r * 30.0 + g * 70.0) / 100.0, (r * 30.0 + b * 70.0) / 100.0)
}

fn colorMaskForAnaglyph(anaglyph: bool)
{
    if !anaglyph
    {
        GL11::glColorMask(true, true, true, true);
    }
    else if EntityRenderer::anaglyphField() == 0
    {
        GL11::glColorMask(false, true, true, true);
    }
    else
    {
        GL11::glColorMask(true, false, false, true);
    }
}

impl RenderGlobal
{
    pub fn renderSky(&self, tickDelta: f32)
    {
        if self.m_mc.m_theWorld.get().orNpe().worldProvider().m_isNether.get()
        {
            return;
        }
        GL11::glDisable(3553);
        let world: Rc<World> = self.worldObj();
        let vec3D: Rc<Vec3D> = world.getSkyColor(&self.renderViewEntity(), tickDelta);
        let mut f5: f32 = vec3D.m_xCoord.get() as f32;
        let mut f6: f32 = vec3D.m_yCoord.get() as f32;
        let mut f7: f32 = vec3D.m_zCoord.get() as f32;
        let anaglyph: bool = self.gameSettings().m_anaglyph.get();
        if anaglyph
        {
            (f5, f6, f7) = anaglyphColor(f5, f6, f7);
        }
        GL11::glColor3f(f5, f6, f7);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glDepthMask(false);
        GL11::glEnable(2912);
        GL11::glColor3f(f5, f6, f7);
        GL11::glCallList(self.m_glSkyList);
        GL11::glDisable(2912);
        GL11::glDisable(3008);
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        RenderHelper::disableStandardItemLighting();
        let provider: Rc<WorldProvider> = world.worldProvider();
        let colors: Option<[f32; 4]> = provider.calcSunriseSunsetColors(world.getCelestialAngle(tickDelta), tickDelta);
        if let Some(fArray) = colors
        {
            GL11::glDisable(3553);
            GL11::glShadeModel(7425);
            GL11::glPushMatrix();
            GL11::glRotatef(90.0, 1.0, 0.0, 0.0);
            let f4: f32 = world.getCelestialAngle(tickDelta);
            GL11::glRotatef(if f4 > 0.5 { 180.0 } else { 0.0 }, 0.0, 0.0, 1.0);
            let (mut f3, mut f2, mut f): (f32, f32, f32) = (fArray[0], fArray[1], fArray[2]);
            if anaglyph
            {
                (f3, f2, f) = anaglyphColor(f3, f2, f);
            }
            tessellator.startDrawing(6);
            tessellator.setColorRGBA_F(f3, f2, f, fArray[3]);
            tessellator.addVertex(0.0, 100.0, 0.0);
            let n: i32 = 16;
            tessellator.setColorRGBA_F(fArray[0], fArray[1], fArray[2], 0.0);
            for i in 0..=n
            {
                let f10: f32 = i as f32 * std::f64::consts::PI as f32 * 2.0 / n as f32;
                let f13: f32 = math_helper::sin(f10);
                let f14: f32 = math_helper::cos(f10);
                tessellator.addVertex((f13 * 120.0) as f64, (f14 * 120.0) as f64, (-f14 * 40.0 * fArray[3]) as f64);
            }
            tessellator.draw();
            GL11::glPopMatrix();
            GL11::glShadeModel(7424);
        }
        GL11::glEnable(3553);
        GL11::glBlendFunc(770, 1);
        GL11::glPushMatrix();
        let f15: f32 = 1.0 - world.getRain(tickDelta);
        GL11::glColor4f(1.0, 1.0, 1.0, f15);
        GL11::glTranslatef(0.0, 0.0, 0.0);
        GL11::glRotatef(0.0, 0.0, 0.0, 1.0);
        GL11::glRotatef(world.getCelestialAngle(tickDelta) * 360.0, 1.0, 0.0, 0.0);
        let mut f: f64 = 30.0;
        GL11::glBindTexture(3553, self.m_renderEngine.getTexture("/terrain/sun.png"));
        tessellator.startDrawingQuads();
        tessellator.addVertexWithUV(-f, 100.0, -f, 0.0, 0.0);
        tessellator.addVertexWithUV(f, 100.0, -f, 1.0, 0.0);
        tessellator.addVertexWithUV(f, 100.0, f, 1.0, 1.0);
        tessellator.addVertexWithUV(-f, 100.0, f, 0.0, 1.0);
        tessellator.draw();
        f = 20.0;
        GL11::glBindTexture(3553, self.m_renderEngine.getTexture("/terrain/moon.png"));
        tessellator.startDrawingQuads();
        tessellator.addVertexWithUV(-f, -100.0, f, 1.0, 1.0);
        tessellator.addVertexWithUV(f, -100.0, f, 0.0, 1.0);
        tessellator.addVertexWithUV(f, -100.0, -f, 0.0, 0.0);
        tessellator.addVertexWithUV(-f, -100.0, -f, 1.0, 0.0);
        tessellator.draw();
        GL11::glDisable(3553);
        let f16: f32 = world.getStarBrightness(tickDelta) * f15;
        if f16 > 0.0
        {
            GL11::glColor4f(f16, f16, f16, f16);
            GL11::glCallList(self.m_starGLCallList);
        }
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glDisable(3042);
        GL11::glEnable(3008);
        GL11::glEnable(2912);
        GL11::glPopMatrix();
        if provider.hasGround()
        {
            GL11::glColor3f(f5 * 0.2 + 0.04, f6 * 0.2 + 0.04, f7 * 0.6 + 0.1);
        }
        else
        {
            GL11::glColor3f(f5, f6, f7);
        }
        GL11::glDisable(3553);
        GL11::glCallList(self.m_glSkyList2);
        GL11::glEnable(3553);
        GL11::glDepthMask(true);
    }

    fn cloudColor(&self, tickDelta: f32) -> (f32, f32, f32)
    {
        let vec3d: Rc<Vec3D> = self.worldObj().getCloudColor(tickDelta);
        let color: (f32, f32, f32) =
            (vec3d.m_xCoord.get() as f32, vec3d.m_yCoord.get() as f32, vec3d.m_zCoord.get() as f32);
        if self.gameSettings().m_anaglyph.get()
        {
            anaglyphColor(color.0, color.1, color.2)
        }
        else
        {
            color
        }
    }

    pub fn renderClouds(&self, tickDelta: f32)
    {
        if self.m_mc.m_theWorld.get().orNpe().worldProvider().m_isNether.get()
        {
            return;
        }
        if self.gameSettings().m_fancyGraphics.get()
        {
            self.renderCloudsFancy(tickDelta);
            return;
        }
        GL11::glDisable(2884);
        let view: Rc<Entity> = self.renderViewEntity();
        let delta: f64 = tickDelta as f64;
        let f: f32 = (view.m_lastTickPosY.get() + (view.posY() - view.m_lastTickPosY.get()) * delta) as f32;
        let b0: i32 = 32;
        let i: i32 = 256 / b0;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glBindTexture(3553, self.m_renderEngine.getTexture("/environment/clouds.png"));
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        let (f1, f2, f3): (f32, f32, f32) = self.cloudColor(tickDelta);
        let f10: f32 = 4.8828125E-4;
        let mut d1: f64 = view.m_prevPosX.get()
            + (view.posX() - view.m_prevPosX.get()) * delta
            + ((self.m_cloudOffsetX.get() as f32 + tickDelta) * 0.03) as f64;
        let mut d0: f64 = view.m_prevPosZ.get() + (view.posZ() - view.m_prevPosZ.get()) * delta;
        let j: i32 = math_helper::floor_double(d1 / 2048.0);
        let k: i32 = math_helper::floor_double(d0 / 2048.0);
        d1 -= (j * 2048) as f64;
        d0 -= (k * 2048) as f64;
        let f7: f32 = self.worldObj().worldProvider().getCloudHeight() - f + 0.33;
        let f8: f32 = (d1 * f10 as f64) as f32;
        let f9: f32 = (d0 * f10 as f64) as f32;
        tessellator.startDrawingQuads();
        tessellator.setColorRGBA_F(f1, f2, f3, 0.8);
        let mut l: i32 = -b0 * i;
        while l < b0 * i
        {
            let mut i1: i32 = -b0 * i;
            while i1 < b0 * i
            {
                let vertex = |x: i32, z: i32| {
                    tessellator.addVertexWithUV(
                        x as f64,
                        f7 as f64,
                        z as f64,
                        (x as f32 * f10 + f8) as f64,
                        (z as f32 * f10 + f9) as f64,
                    );
                };
                vertex(l + 0, i1 + b0);
                vertex(l + b0, i1 + b0);
                vertex(l + b0, i1 + 0);
                vertex(l + 0, i1 + 0);
                i1 += b0;
            }
            l += b0;
        }
        tessellator.draw();
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glDisable(3042);
        GL11::glEnable(2884);
    }

    pub fn hasThiccFog(&self, _x: f64, _y: f64, _z: f64, _tickDelta: f32) -> bool { false }

    pub fn renderCloudsFancy(&self, tickDelta: f32)
    {
        GL11::glDisable(2884);
        let view: Rc<Entity> = self.renderViewEntity();
        let delta: f64 = tickDelta as f64;
        let f: f32 = (view.m_lastTickPosY.get() + (view.posY() - view.m_lastTickPosY.get()) * delta) as f32;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let f1: f32 = 12.0;
        let f2: f32 = 4.0;
        let mut d0: f64 = (view.m_prevPosX.get()
            + (view.posX() - view.m_prevPosX.get()) * delta
            + ((self.m_cloudOffsetX.get() as f32 + tickDelta) * 0.03) as f64)
            / f1 as f64;
        let mut d1: f64 =
            (view.m_prevPosZ.get() + (view.posZ() - view.m_prevPosZ.get()) * delta) / f1 as f64 + 0.33f32 as f64;
        let f3: f32 = self.worldObj().worldProvider().getCloudHeight() - f + 0.33;
        let i: i32 = math_helper::floor_double(d0 / 2048.0);
        let j: i32 = math_helper::floor_double(d1 / 2048.0);
        d0 -= (i * 2048) as f64;
        d1 -= (j * 2048) as f64;
        GL11::glBindTexture(3553, self.m_renderEngine.getTexture("/environment/clouds.png"));
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        let (f4, f5, f6): (f32, f32, f32) = self.cloudColor(tickDelta);
        let f19: f32 = 0.00390625;
        let f17: f32 = math_helper::floor_double(d0) as f32 * f19;
        let f18: f32 = math_helper::floor_double(d1) as f32 * f19;
        let f10: f32 = (d0 - math_helper::floor_double(d0) as f64) as f32;
        let f11: f32 = (d1 - math_helper::floor_double(d1) as f64) as f32;
        let b0: f32 = 8.0;
        let b1: i32 = 3;
        let f12: f32 = 9.765625E-4;
        GL11::glScalef(f1, 1.0, f1);
        let anaglyph: bool = self.gameSettings().m_anaglyph.get();
        for k in 0..2
        {
            if k == 0
            {
                GL11::glColorMask(false, false, false, false);
            }
            else
            {
                colorMaskForAnaglyph(anaglyph);
            }
            for l in (-b1 + 1)..=b1
            {
                for i1 in (-b1 + 1)..=b1
                {
                    tessellator.startDrawingQuads();
                    let f13: f32 = l as f32 * b0;
                    let f14: f32 = i1 as f32 * b0;
                    let f15: f32 = f13 - f10;
                    let f16: f32 = f14 - f11;
                    let vertex = |x: f32, y: f32, z: f32, u: f32, v: f32| {
                        tessellator.addVertexWithUV(
                            x as f64,
                            y as f64,
                            z as f64,
                            (u * f19 + f17) as f64,
                            (v * f19 + f18) as f64,
                        );
                    };
                    if f3 > -f2 - 1.0
                    {
                        tessellator.setColorRGBA_F(f4 * 0.7, f5 * 0.7, f6 * 0.7, 0.8);
                        tessellator.setNormal(0.0, -1.0, 0.0);
                        vertex(f15 + 0.0, f3 + 0.0, f16 + b0, f13 + 0.0, f14 + b0);
                        vertex(f15 + b0, f3 + 0.0, f16 + b0, f13 + b0, f14 + b0);
                        vertex(f15 + b0, f3 + 0.0, f16 + 0.0, f13 + b0, f14 + 0.0);
                        vertex(f15 + 0.0, f3 + 0.0, f16 + 0.0, f13 + 0.0, f14 + 0.0);
                    }
                    if f3 <= f2 + 1.0
                    {
                        tessellator.setColorRGBA_F(f4, f5, f6, 0.8);
                        tessellator.setNormal(0.0, 1.0, 0.0);
                        vertex(f15 + 0.0, f3 + f2 - f12, f16 + b0, f13 + 0.0, f14 + b0);
                        vertex(f15 + b0, f3 + f2 - f12, f16 + b0, f13 + b0, f14 + b0);
                        vertex(f15 + b0, f3 + f2 - f12, f16 + 0.0, f13 + b0, f14 + 0.0);
                        vertex(f15 + 0.0, f3 + f2 - f12, f16 + 0.0, f13 + 0.0, f14 + 0.0);
                    }
                    tessellator.setColorRGBA_F(f4 * 0.9, f5 * 0.9, f6 * 0.9, 0.8);
                    if l > -1
                    {
                        tessellator.setNormal(-1.0, 0.0, 0.0);
                        for j1 in 0..8
                        {
                            let x: f32 = f15 + j1 as f32 + 0.0;
                            let u: f32 = f13 + j1 as f32 + 0.5;
                            vertex(x, f3 + 0.0, f16 + b0, u, f14 + b0);
                            vertex(x, f3 + f2, f16 + b0, u, f14 + b0);
                            vertex(x, f3 + f2, f16 + 0.0, u, f14 + 0.0);
                            vertex(x, f3 + 0.0, f16 + 0.0, u, f14 + 0.0);
                        }
                    }
                    if l <= 1
                    {
                        tessellator.setNormal(1.0, 0.0, 0.0);
                        for k1 in 0..8
                        {
                            let x: f32 = f15 + k1 as f32 + 1.0 - f12;
                            let u: f32 = f13 + k1 as f32 + 0.5;
                            vertex(x, f3 + 0.0, f16 + b0, u, f14 + b0);
                            vertex(x, f3 + f2, f16 + b0, u, f14 + b0);
                            vertex(x, f3 + f2, f16 + 0.0, u, f14 + 0.0);
                            vertex(x, f3 + 0.0, f16 + 0.0, u, f14 + 0.0);
                        }
                    }
                    tessellator.setColorRGBA_F(f4 * 0.8, f5 * 0.8, f6 * 0.8, 0.8);
                    if i1 > -1
                    {
                        tessellator.setNormal(0.0, 0.0, -1.0);
                        for l1 in 0..8
                        {
                            let z: f32 = f16 + l1 as f32 + 0.0;
                            let v: f32 = f14 + l1 as f32 + 0.5;
                            vertex(f15 + 0.0, f3 + f2, z, f13 + 0.0, v);
                            vertex(f15 + b0, f3 + f2, z, f13 + b0, v);
                            vertex(f15 + b0, f3 + 0.0, z, f13 + b0, v);
                            vertex(f15 + 0.0, f3 + 0.0, z, f13 + 0.0, v);
                        }
                    }
                    if i1 <= 1
                    {
                        tessellator.setNormal(0.0, 0.0, 1.0);
                        for i2 in 0..8
                        {
                            let z: f32 = f16 + i2 as f32 + 1.0 - f12;
                            let v: f32 = f14 + i2 as f32 + 0.5;
                            vertex(f15 + 0.0, f3 + f2, z, f13 + 0.0, v);
                            vertex(f15 + b0, f3 + f2, z, f13 + b0, v);
                            vertex(f15 + b0, f3 + 0.0, z, f13 + b0, v);
                            vertex(f15 + 0.0, f3 + 0.0, z, f13 + 0.0, v);
                        }
                    }
                    tessellator.draw();
                }
            }
        }
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glDisable(3042);
        GL11::glEnable(2884);
    }
}
