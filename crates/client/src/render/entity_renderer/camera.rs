use crate::render::EntityRenderer;
use crate::settings::GameSettings;
use game::block::{Block, Material};
use game::entity::entity_living::{self, LivingFields};
use game::entity::{Entity, entity_player};
use game::math::{MovingObjectPosition, Vec3D, math_helper};
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use lwjgl::util::glu::GLU;
use std::f64::consts::PI;
use std::rc::Rc;

const PI_F: f32 = PI as f32;

impl EntityRenderer
{
    pub(super) fn getFOVModifier(&self, tickDelta: f32) -> f32
    {
        let entityLiving: Rc<Entity> = self.renderViewEntity();
        let mut f: f32 = 70.0;
        if entityLiving.isInsideOfMaterial(&Material::water())
        {
            f = 60.0;
        }
        let living: &LivingFields = entity_living::fields(&entityLiving);
        if living.m_health.get() <= 0
        {
            let f2: f32 = living.m_deathTime.get() as f32 + tickDelta;
            f /= (1.0 - 500.0 / (f2 + 500.0)) * 2.0 + 1.0;
        }
        f + self.m_newFov.get() + (self.m_oldFov.get() - self.m_newFov.get()) * tickDelta
    }

    pub(super) fn hurtCameraEffect(&self, tickDelta: f32)
    {
        let entityLiving: Rc<Entity> = self.renderViewEntity();
        let living: &LivingFields = entity_living::fields(&entityLiving);
        let mut f2: f32 = living.m_hurtTime.get() as f32 - tickDelta;
        if living.m_health.get() <= 0
        {
            let f: f32 = living.m_deathTime.get() as f32 + tickDelta;
            GL11::glRotatef(40.0 - 8000.0 / (f + 200.0), 0.0, 0.0, 1.0);
        }
        if f2 < 0.0
        {
            return;
        }
        f2 /= living.m_maxHurtTime.get() as f32;
        f2 = math_helper::sin(f2 * f2 * f2 * f2 * PI_F);
        let f: f32 = living.m_attackedAtYaw.get();
        GL11::glRotatef(-f, 0.0, 1.0, 0.0);
        GL11::glRotatef(-f2 * 14.0, 0.0, 0.0, 1.0);
        GL11::glRotatef(f, 0.0, 1.0, 0.0);
    }

    pub(super) fn setupViewBobbing(&self, tickDelta: f32)
    {
        let entityPlayer: Rc<Entity> = self.renderViewEntity();
        if !entityPlayer.instanceOf("EntityPlayer")
        {
            return;
        }
        let player: &entity_player::PlayerFields = entity_player::fields(&entityPlayer);
        let living: &LivingFields = entity_living::fields(&entityPlayer);
        let f: f32 = entityPlayer.m_distanceWalkedModified.get() - entityPlayer.m_prevDistanceWalkedModified.get();
        let f2: f32 = -(entityPlayer.m_distanceWalkedModified.get() + f * tickDelta);
        let f3: f32 = player.m_lastBob.get() + (player.m_bob.get() - player.m_lastBob.get()) * tickDelta;
        let f4: f32 = living.m_cameraPitch.get() + (living.m_tilt.get() - living.m_cameraPitch.get()) * tickDelta;
        GL11::glTranslatef(math_helper::sin(f2 * PI_F) * f3 * 0.5, -(math_helper::cos(f2 * PI_F) * f3).abs(), 0.0);
        GL11::glRotatef(math_helper::sin(f2 * PI_F) * f3 * 3.0, 0.0, 0.0, 1.0);
        GL11::glRotatef((math_helper::cos(f2 * PI_F - 0.2) * f3).abs() * 5.0, 1.0, 0.0, 0.0);
        GL11::glRotatef(f4, 1.0, 0.0, 0.0);
    }

    fn orientCamera(&self, tickDelta: f32)
    {
        let entityliving: Rc<Entity> = self.renderViewEntity();
        let settings: Rc<GameSettings> = self.gameSettings();
        let delta: f64 = tickDelta as f64;
        let mut f: f32 = entityliving.m_yOffset.get() - 1.62;
        let d0: f64 = entityliving.m_prevPosX.get() + (entityliving.posX() - entityliving.m_prevPosX.get()) * delta;
        let d1: f64 =
            entityliving.m_prevPosY.get() + (entityliving.posY() - entityliving.m_prevPosY.get()) * delta - f as f64;
        let d2: f64 = entityliving.m_prevPosZ.get() + (entityliving.posZ() - entityliving.m_prevPosZ.get()) * delta;
        GL11::glRotatef(
            self.m_newCamRoll.get() + (self.m_oldCamRoll.get() - self.m_newCamRoll.get()) * tickDelta,
            0.0,
            0.0,
            1.0,
        );
        let yaw: f32 = entityliving.m_prevRotationYaw.get()
            + (entityliving.m_rotationYaw.get() - entityliving.m_prevRotationYaw.get()) * tickDelta;
        let pitch: f32 = entityliving.m_prevRotationPitch.get()
            + (entityliving.m_rotationPitch.get() - entityliving.m_prevRotationPitch.get()) * tickDelta;
        if entityliving.isPlayerSleeping()
        {
            f = (f as f64 + 1.0) as f32;
            GL11::glTranslatef(0.0, 0.3, 0.0);
            if !settings.m_debugCamera.get()
            {
                let world = self.theWorld();
                let x: i32 = math_helper::floor_double(entityliving.posX());
                let y: i32 = math_helper::floor_double(entityliving.posY());
                let z: i32 = math_helper::floor_double(entityliving.posZ());
                if world.getBlockId(x, y, z) == Block::blockBed().m_blockID
                {
                    let k: i32 = world.getBlockMetadata(x, y, z) & 3;
                    GL11::glRotatef((k * 90) as f32, 0.0, 1.0, 0.0);
                }
                GL11::glRotatef(yaw + 180.0, 0.0, -1.0, 0.0);
                GL11::glRotatef(pitch, -1.0, 0.0, 0.0);
            }
        }
        else if settings.m_thirdPersonView.get()
        {
            let mut d7: f64 =
                (self.m_newCamPos.get() + (self.m_oldCamPos.get() - self.m_newCamPos.get()) * tickDelta) as f64;
            if settings.m_debugCamera.get()
            {
                let f5: f32 =
                    self.m_newCamPitch.get() + (self.m_oldCamPitch.get() - self.m_newCamPitch.get()) * tickDelta;
                let f1: f32 = self.m_newCamYaw.get() + (self.m_oldCamYaw.get() - self.m_newCamYaw.get()) * tickDelta;
                GL11::glTranslatef(0.0, 0.0, (-d7) as f32);
                GL11::glRotatef(f1, 1.0, 0.0, 0.0);
                GL11::glRotatef(f5, 0.0, 1.0, 0.0);
            }
            else
            {
                let f6: f32 = entityliving.m_rotationYaw.get();
                let f7: f32 = entityliving.m_rotationPitch.get();
                let d3: f64 = (-math_helper::sin(f6 / 180.0 * PI_F) * math_helper::cos(f7 / 180.0 * PI_F)) as f64 * d7;
                let d4: f64 = (math_helper::cos(f6 / 180.0 * PI_F) * math_helper::cos(f7 / 180.0 * PI_F)) as f64 * d7;
                let d5: f64 = (-math_helper::sin(f7 / 180.0 * PI_F)) as f64 * d7;
                for l in 0..8
                {
                    let f2: f32 = ((l & 1) * 2 - 1) as f32 * 0.1;
                    let f3: f32 = ((l >> 1 & 1) * 2 - 1) as f32 * 0.1;
                    let f4: f32 = ((l >> 2 & 1) * 2 - 1) as f32 * 0.1;
                    let (f2, f3, f4): (f64, f64, f64) = (f2 as f64, f3 as f64, f4 as f64);
                    let hit: Option<MovingObjectPosition> = self.theWorld().rayTraceBlocks(
                        &Vec3D::createVector(d0 + f2, d1 + f3, d2 + f4),
                        &Vec3D::createVector(d0 - d3 + f2 + f4, d1 - d5 + f3, d2 - d4 + f4),
                    );
                    if let Some(hit) = hit
                    {
                        let d6: f64 = hit.m_hitVec.distanceTo(&Vec3D::createVector(d0, d1, d2));
                        if d6 < d7
                        {
                            d7 = d6;
                        }
                    }
                }
                GL11::glRotatef(entityliving.m_rotationPitch.get() - f7, 1.0, 0.0, 0.0);
                GL11::glRotatef(entityliving.m_rotationYaw.get() - f6, 0.0, 1.0, 0.0);
                GL11::glTranslatef(0.0, 0.0, (-d7) as f32);
                GL11::glRotatef(f6 - entityliving.m_rotationYaw.get(), 0.0, 1.0, 0.0);
                GL11::glRotatef(f7 - entityliving.m_rotationPitch.get(), 1.0, 0.0, 0.0);
            }
        }
        else
        {
            GL11::glTranslatef(0.0, 0.0, -0.1);
        }
        if !settings.m_debugCamera.get()
        {
            GL11::glRotatef(pitch, 1.0, 0.0, 0.0);
            GL11::glRotatef(yaw + 180.0, 0.0, 1.0, 0.0);
        }
        GL11::glTranslatef(0.0, f, 0.0);
        let d0: f64 = entityliving.m_prevPosX.get() + (entityliving.posX() - entityliving.m_prevPosX.get()) * delta;
        let d1: f64 =
            entityliving.m_prevPosY.get() + (entityliving.posY() - entityliving.m_prevPosY.get()) * delta - f as f64;
        let d2: f64 = entityliving.m_prevPosZ.get() + (entityliving.posZ() - entityliving.m_prevPosZ.get()) * delta;
        self.m_cloudFog.set(self.m_mc.m_renderGlobal.get().orNpe().hasThiccFog(d0, d1, d2, tickDelta));
    }

    pub(super) fn setupCameraTransform(&self, tickDelta: f32, anaglyphRenderPass: i32)
    {
        let settings: Rc<GameSettings> = self.gameSettings();
        self.m_farPlaneDistance.set((256 >> settings.m_renderDistance.get()) as f32);
        GL11::glMatrixMode(5889);
        GL11::glLoadIdentity();
        let f: f32 = 0.07;
        if settings.m_anaglyph.get()
        {
            GL11::glTranslatef(-(anaglyphRenderPass * 2 - 1) as f32 * f, 0.0, 0.0);
        }
        let aspect: f32 = self.m_mc.m_displayWidth.get() as f32 / self.m_mc.m_displayHeight.get() as f32;
        if self.m_cameraZoom.get() != 1.0
        {
            GL11::glTranslatef(self.m_cameraYaw.get() as f32, (-self.m_cameraPitch.get()) as f32, 0.0);
            GL11::glScaled(self.m_cameraZoom.get(), self.m_cameraZoom.get(), 1.0);
        }
        GLU::gluPerspective(self.getFOVModifier(tickDelta), aspect, 0.05, self.m_farPlaneDistance.get() * 2.0);
        GL11::glMatrixMode(5888);
        GL11::glLoadIdentity();
        if settings.m_anaglyph.get()
        {
            GL11::glTranslatef((anaglyphRenderPass * 2 - 1) as f32 * 0.1, 0.0, 0.0);
        }
        self.hurtCameraEffect(tickDelta);
        if settings.m_viewBobbing.get()
        {
            self.setupViewBobbing(tickDelta);
        }
        let player: Rc<Entity> = self.m_mc.m_thePlayer.get().orNpe();
        let fields: &entity_player::PlayerFields = entity_player::fields(&player);
        let f1: f32 = fields.m_prevTimeInPortal.get()
            + (fields.m_timeInPortal.get() - fields.m_prevTimeInPortal.get()) * tickDelta;
        if f1 > 0.0
        {
            let mut f2: f32 = 5.0 / (f1 * f1 + 5.0) - f1 * 0.04;
            f2 *= f2;
            let angle: f32 = (self.m_rendererUpdateCount.get() as f32 + tickDelta) * 20.0;
            GL11::glRotatef(angle, 0.0, 1.0, 1.0);
            GL11::glScalef(1.0 / f2, 1.0, 1.0);
            GL11::glRotatef(-angle, 0.0, 1.0, 1.0);
        }
        self.orientCamera(tickDelta);
    }

    pub(super) fn renderItemInHand(&self, tickDelta: f32, anaglyphRenderPass: i32)
    {
        let settings: Rc<GameSettings> = self.gameSettings();
        GL11::glLoadIdentity();
        if settings.m_anaglyph.get()
        {
            GL11::glTranslatef((anaglyphRenderPass * 2 - 1) as f32 * 0.1, 0.0, 0.0);
        }
        GL11::glPushMatrix();
        self.hurtCameraEffect(tickDelta);
        if settings.m_viewBobbing.get()
        {
            self.setupViewBobbing(tickDelta);
        }
        let sleeping: bool = self.renderViewEntity().isPlayerSleeping();
        if !(settings.m_thirdPersonView.get() || sleeping || settings.m_hideGUI.get())
        {
            self.m_itemRenderer.renderItemInFirstPerson(tickDelta);
        }
        GL11::glPopMatrix();
        if !settings.m_thirdPersonView.get() && !self.renderViewEntity().isPlayerSleeping()
        {
            self.m_itemRenderer.renderOverlays(tickDelta);
            self.hurtCameraEffect(tickDelta);
        }
        if settings.m_viewBobbing.get()
        {
            self.setupViewBobbing(tickDelta);
        }
    }
}
