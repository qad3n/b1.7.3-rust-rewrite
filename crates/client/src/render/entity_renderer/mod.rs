mod camera;
mod fog;
mod weather;

use crate::client::Minecraft;
use crate::controller::PlayerController;
use crate::gui::ScaledResolution;
use crate::render::{
    ClippingHelperImpl, EffectRenderer, Frustrum, GLAllocation, ICamera, ItemRenderer, RenderGlobal, RenderHelper,
};
use crate::settings::{GameSettings, MouseFilter};
use game::entity::{Entity, entity_living};
use game::math::{AxisAlignedBB, MovingObjectPosition, Vec3D, math_helper};
use game::world::{ChunkProviderLoadOrGenerate, IChunkProvider, World};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::buffers::FloatBuffer;
use lwjgl::input::Mouse;
use lwjgl::opengl::{Display, GL11};
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static ANAGLYPH_ENABLED: Cell<bool> = const { Cell::new(false) };
    static ANAGLYPH_FIELD: Cell<i32> = const { Cell::new(0) };
}

pub struct EntityRenderer
{
    m_mc: Rc<Minecraft>,
    m_farPlaneDistance: Cell<f32>,
    pub m_itemRenderer: Rc<ItemRenderer>,
    m_rendererUpdateCount: Cell<i32>,
    m_pointedEntity: jvm::field::Field<Option<Rc<Entity>>>,
    m_mouseFilterXAxis: MouseFilter,
    m_mouseFilterYAxis: MouseFilter,
    pub m_mouseFilterDummy1: MouseFilter,
    pub m_mouseFilterDummy2: MouseFilter,
    pub m_mouseFilterDummy3: MouseFilter,
    pub m_mouseFilterDummy4: MouseFilter,
    m_oldCamPos: Cell<f32>,
    m_newCamPos: Cell<f32>,
    m_oldCamPitch: Cell<f32>,
    m_newCamPitch: Cell<f32>,
    m_oldCamYaw: Cell<f32>,
    m_newCamYaw: Cell<f32>,
    m_oldFov: Cell<f32>,
    m_newFov: Cell<f32>,
    m_oldCamRoll: Cell<f32>,
    m_newCamRoll: Cell<f32>,
    m_cloudFog: Cell<bool>,
    m_cameraZoom: Cell<f64>,
    m_cameraYaw: Cell<f64>,
    m_cameraPitch: Cell<f64>,
    m_prevFrameTime: Cell<i64>,
    m_lastWorldRenderTime: Cell<i64>,
    m_random: Random,
    m_rainSoundCounter: Cell<i32>,
    pub m_field_1394_b: Cell<i32>,
    pub m_field_1393_c: Cell<i32>,
    m_fogColorBuffer: FloatBuffer,
    m_fogColorRed: Cell<f32>,
    m_fogColorGreen: Cell<f32>,
    m_fogColorBlue: Cell<f32>,
    m_fogColor2: Cell<f32>,
    m_fogColor1: Cell<f32>,
}

impl EntityRenderer
{
    pub fn anaglyphEnabled() -> bool { ANAGLYPH_ENABLED.with(Cell::get) }

    pub fn anaglyphField() -> i32 { ANAGLYPH_FIELD.with(Cell::get) }

    fn setAnaglyphField(value: i32) { ANAGLYPH_FIELD.with(|field| field.set(value)); }

    pub fn new(minecraft: &Rc<Minecraft>) -> EntityRenderer
    {
        EntityRenderer {
            m_mc: Rc::clone(minecraft),
            m_farPlaneDistance: Cell::new(0.0),
            m_itemRenderer: Rc::new(ItemRenderer::new(Rc::clone(minecraft))),
            m_rendererUpdateCount: Cell::new(0),
            m_pointedEntity: jvm::field::Field::new(None),
            m_mouseFilterXAxis: MouseFilter::new(),
            m_mouseFilterYAxis: MouseFilter::new(),
            m_mouseFilterDummy1: MouseFilter::new(),
            m_mouseFilterDummy2: MouseFilter::new(),
            m_mouseFilterDummy3: MouseFilter::new(),
            m_mouseFilterDummy4: MouseFilter::new(),
            m_oldCamPos: Cell::new(4.0),
            m_newCamPos: Cell::new(4.0),
            m_oldCamPitch: Cell::new(0.0),
            m_newCamPitch: Cell::new(0.0),
            m_oldCamYaw: Cell::new(0.0),
            m_newCamYaw: Cell::new(0.0),
            m_oldFov: Cell::new(0.0),
            m_newFov: Cell::new(0.0),
            m_oldCamRoll: Cell::new(0.0),
            m_newCamRoll: Cell::new(0.0),
            m_cloudFog: Cell::new(false),
            m_cameraZoom: Cell::new(1.0),
            m_cameraYaw: Cell::new(0.0),
            m_cameraPitch: Cell::new(0.0),
            m_prevFrameTime: Cell::new(jvm::system::currentTimeMillis()),
            m_lastWorldRenderTime: Cell::new(0),
            m_random: Random::new(),
            m_rainSoundCounter: Cell::new(0),
            m_field_1394_b: Cell::new(0),
            m_field_1393_c: Cell::new(0),
            m_fogColorBuffer: GLAllocation::createDirectFloatBuffer(16),
            m_fogColorRed: Cell::new(0.0),
            m_fogColorGreen: Cell::new(0.0),
            m_fogColorBlue: Cell::new(0.0),
            m_fogColor2: Cell::new(0.0),
            m_fogColor1: Cell::new(0.0),
        }
    }

    fn gameSettings(&self) -> Rc<GameSettings> { self.m_mc.m_gameSettings.get().orNpe() }

    fn theWorld(&self) -> Rc<World> { self.m_mc.m_theWorld.get().orNpe() }

    fn renderViewEntity(&self) -> Rc<Entity> { self.m_mc.m_renderViewEntity.get().orNpe() }

    fn playerController(&self) -> Rc<PlayerController> { self.m_mc.m_playerController.get().orNpe() }

    pub fn updateRenderer(&self)
    {
        self.m_fogColor2.set(self.m_fogColor1.get());
        self.m_newCamPos.set(self.m_oldCamPos.get());
        self.m_newCamPitch.set(self.m_oldCamPitch.get());
        self.m_newCamYaw.set(self.m_oldCamYaw.get());
        self.m_newFov.set(self.m_oldFov.get());
        self.m_newCamRoll.set(self.m_oldCamRoll.get());
        if self.m_mc.m_renderViewEntity.get().is_none()
        {
            self.m_mc.m_renderViewEntity.set(self.m_mc.m_thePlayer.get());
        }
        let view: Rc<Entity> = self.renderViewEntity();
        let f: f32 = self.theWorld().getLightBrightness(
            math_helper::floor_double(view.posX()),
            math_helper::floor_double(view.posY()),
            math_helper::floor_double(view.posZ()),
        );
        let f2: f32 = (3 - self.gameSettings().m_renderDistance.get()) as f32 / 3.0;
        let f3: f32 = f * (1.0 - f2) + f2;
        self.m_fogColor1.set(self.m_fogColor1.get() + (f3 - self.m_fogColor1.get()) * 0.1);
        self.m_rendererUpdateCount.set(self.m_rendererUpdateCount.get() + 1);
        self.m_itemRenderer.updateEquippedItem();
        self.addRainParticles();
    }

    pub fn getMouseOver(&self, tickDelta: f32)
    {
        let Some(view) = self.m_mc.m_renderViewEntity.get()
        else
        {
            return;
        };
        let Some(world) = self.m_mc.m_theWorld.get()
        else
        {
            return;
        };
        let mut d: f64 = self.playerController().getBlockReachDistance() as f64;
        self.m_mc.m_objectMouseOver.set(entity_living::rayTrace(&view, d, tickDelta));
        let mut d2: f64 = d;
        let vec3D: Rc<Vec3D> = entity_living::getPosition(&view, tickDelta);
        if let Some(hit) = self.m_mc.m_objectMouseOver.get()
        {
            d2 = hit.m_hitVec.distanceTo(&vec3D);
        }
        let creative: bool = self.playerController().instanceOf("PlayerControllerTest");
        if creative
        {
            d = 32.0;
        }
        else
        {
            if d2 > 3.0
            {
                d2 = 3.0;
            }
            d = d2;
        }
        let vec3D2: Rc<Vec3D> = entity_living::getLook(&view, tickDelta);
        let (lookX, lookY, lookZ): (f64, f64, f64) =
            (vec3D2.m_xCoord.get() * d, vec3D2.m_yCoord.get() * d, vec3D2.m_zCoord.get() * d);
        let vec3D3: Rc<Vec3D> = vec3D.addVector(lookX, lookY, lookZ);
        self.m_pointedEntity.set(None);
        let f: f64 = 1.0f32 as f64;
        let list: Vec<Rc<Entity>> = world
            .getEntitiesWithinAABBExcludingEntity(
                Some(&view),
                &view.m_boundingBox.addCoord(lookX, lookY, lookZ).expand(f, f, f),
            )
            .toVec();
        let mut d3: f64 = 0.0;
        for entity in list
        {
            if !entity.canBeCollidedWith()
            {
                continue;
            }
            let f2: f64 = entity.getCollisionBorderSize() as f64;
            let axisAlignedBB: Rc<AxisAlignedBB> = entity.m_boundingBox.expand(f2, f2, f2);
            let movingObjectPosition: Option<MovingObjectPosition> = axisAlignedBB.clip(&vec3D, &vec3D3);
            if axisAlignedBB.isVecInside(&vec3D)
            {
                if !(0.0 < d3) && d3 != 0.0
                {
                    continue;
                }
                self.m_pointedEntity.set(Some(entity));
                d3 = 0.0;
                continue;
            }
            let Some(hit) = movingObjectPosition
            else
            {
                continue;
            };
            let d4: f64 = vec3D.distanceTo(&hit.m_hitVec);
            if !(d4 < d3) && d3 != 0.0
            {
                continue;
            }
            self.m_pointedEntity.set(Some(entity));
            d3 = d4;
        }
        if let Some(pointed) = self.m_pointedEntity.get()
            && !creative
        {
            self.m_mc.m_objectMouseOver.set(Some(MovingObjectPosition::newEntity(&pointed)));
        }
    }

    pub fn updateCameraAndRender(&self, tickDelta: f32)
    {
        let mc: &Rc<Minecraft> = &self.m_mc;
        if !Display::isActive()
        {
            if jvm::system::currentTimeMillis() - self.m_prevFrameTime.get() > 500
            {
                mc.displayInGameMenu();
            }
        }
        else
        {
            self.m_prevFrameTime.set(jvm::system::currentTimeMillis());
        }
        let settings: Rc<GameSettings> = self.gameSettings();
        if mc.m_inGameHasFocus.get()
        {
            let mouseHelper = mc.m_mouseHelper.get().orNpe();
            mouseHelper.mouseXYChange();
            let n: f32 = settings.m_mouseSensitivity.get() * 0.6 + 0.2;
            let n2: f32 = n * n * n * 8.0;
            let mut smooth: f32 = mouseHelper.m_deltaX.get() as f32 * n2;
            let mut smooth2: f32 = mouseHelper.m_deltaY.get() as f32 * n2;
            let n3: i32 = if settings.m_invertMouse.get() { -1 } else { 1 };
            if settings.m_smoothCamera.get()
            {
                smooth = self.m_mouseFilterXAxis.smooth(smooth, 0.05 * n2);
                smooth2 = self.m_mouseFilterYAxis.smooth(smooth2, 0.05 * n2);
            }
            mc.m_thePlayer.get().orNpe().updateLocalPlayerCamera(smooth, smooth2 * n3 as f32);
        }
        if mc.m_skipRenderWorld.get()
        {
            return;
        }
        ANAGLYPH_ENABLED.with(|enabled| enabled.set(settings.m_anaglyph.get()));
        let scaledResolution: ScaledResolution =
            ScaledResolution::new(&settings, mc.m_displayWidth.get(), mc.m_displayHeight.get());
        let scaledWidth: i32 = scaledResolution.getScaledWidth();
        let scaledHeight: i32 = scaledResolution.getScaledHeight();
        let n4: i32 = Mouse::getX() * scaledWidth / mc.m_displayWidth.get();
        let n5: i32 = scaledHeight - Mouse::getY() * scaledHeight / mc.m_displayHeight.get() - 1;
        let n6: i64 = match settings.m_limitFramerate.get()
        {
            1 => 120,
            2 => 40,
            _ => 200,
        };
        let frameNanos: i64 = 1000000000 / n6;
        if mc.m_theWorld.get().is_some()
        {
            if settings.m_limitFramerate.get() == 0
            {
                self.renderWorld(tickDelta, 0);
            }
            else
            {
                self.renderWorld(tickDelta, self.m_lastWorldRenderTime.get() + frameNanos);
            }
            if settings.m_limitFramerate.get() == 2
            {
                let millis: i64 = (self.m_lastWorldRenderTime.get() + frameNanos - jvm::system::nanoTime()) / 1000000;
                if millis > 0 && millis < 500
                {
                    std::thread::sleep(std::time::Duration::from_millis(millis as u64));
                }
            }
            self.m_lastWorldRenderTime.set(jvm::system::nanoTime());
            if !settings.m_hideGUI.get() || mc.m_currentScreen.get().is_some()
            {
                mc.m_ingameGUI.get().orNpe().renderGameOverlay(tickDelta, mc.m_currentScreen.get().is_some(), n4, n5);
            }
        }
        else
        {
            GL11::glViewport(0, 0, mc.m_displayWidth.get(), mc.m_displayHeight.get());
            GL11::glMatrixMode(5889);
            GL11::glLoadIdentity();
            GL11::glMatrixMode(5888);
            GL11::glLoadIdentity();
            self.setupGuiState();
            if settings.m_limitFramerate.get() == 2
            {
                let mut millis2: i64 =
                    (self.m_lastWorldRenderTime.get() + frameNanos - jvm::system::nanoTime()) / 1000000;
                if millis2 < 0
                {
                    millis2 += 10;
                }
                if millis2 > 0 && millis2 < 500
                {
                    std::thread::sleep(std::time::Duration::from_millis(millis2 as u64));
                }
            }
            self.m_lastWorldRenderTime.set(jvm::system::nanoTime());
        }
        if let Some(screen) = mc.m_currentScreen.get()
        {
            GL11::glClear(256);
            screen.drawScreen(n4, n5, tickDelta);
            if let Some(screen) = mc.m_currentScreen.get()
            {
                screen.renderParticlesGui(tickDelta);
            }
        }
    }

    fn drawBlockHighlight(&self, renderGlobal: &RenderGlobal, entityLiving: &Rc<Entity>, tickDelta: f32)
    {
        let Some(hit) = self.m_mc.m_objectMouseOver.get()
        else
        {
            return;
        };
        let currentItem = game::entity::entity_player::inventory(entityLiving).getCurrentItem();
        GL11::glDisable(3008);
        renderGlobal.drawBlockBreaking(entityLiving, &hit, 0, currentItem.as_ref(), tickDelta);
        renderGlobal.drawSelectionBox(entityLiving, &hit, 0, currentItem.as_ref(), tickDelta);
        GL11::glEnable(3008);
    }

    pub fn renderWorld(&self, tickDelta: f32, renderTimeLimit: i64)
    {
        let mc: &Rc<Minecraft> = &self.m_mc;
        GL11::glEnable(2884);
        GL11::glEnable(2929);
        if mc.m_renderViewEntity.get().is_none()
        {
            mc.m_renderViewEntity.set(mc.m_thePlayer.get());
        }
        self.getMouseOver(tickDelta);
        let entityLiving: Rc<Entity> = self.renderViewEntity();
        let renderGlobal: Rc<RenderGlobal> = mc.m_renderGlobal.get().orNpe();
        let effectRenderer: Rc<EffectRenderer> = mc.m_effectRenderer.get().orNpe();
        let delta: f64 = tickDelta as f64;
        let d: f64 =
            entityLiving.m_lastTickPosX.get() + (entityLiving.posX() - entityLiving.m_lastTickPosX.get()) * delta;
        let d2: f64 =
            entityLiving.m_lastTickPosY.get() + (entityLiving.posY() - entityLiving.m_lastTickPosY.get()) * delta;
        let d3: f64 =
            entityLiving.m_lastTickPosZ.get() + (entityLiving.posZ() - entityLiving.m_lastTickPosZ.get()) * delta;
        let iChunkProvider: Rc<dyn IChunkProvider> = self.theWorld().getIChunkProvider();
        if let Some(chunkProviderLoadOrGenerate) = iChunkProvider.asLoadOrGenerate()
        {
            let chunkProvider: Rc<ChunkProviderLoadOrGenerate> = chunkProviderLoadOrGenerate;
            let n2: i32 = math_helper::floor_float(d as i32 as f32) >> 4;
            let n: i32 = math_helper::floor_float(d3 as i32 as f32) >> 4;
            chunkProvider.setCurrentChunkOver(n2, n);
        }
        let settings: Rc<GameSettings> = self.gameSettings();
        let isPlayer: bool = entityLiving.instanceOf("EntityPlayer");
        let water = game::block::Material::water();
        for i in 0..2
        {
            if settings.m_anaglyph.get()
            {
                Self::setAnaglyphField(i);
                if i == 0
                {
                    GL11::glColorMask(false, true, true, false);
                }
                else
                {
                    GL11::glColorMask(true, false, false, false);
                }
            }
            GL11::glViewport(0, 0, mc.m_displayWidth.get(), mc.m_displayHeight.get());
            self.updateFogColor(tickDelta);
            GL11::glClear(16640);
            GL11::glEnable(2884);
            self.setupCameraTransform(tickDelta, i);
            ClippingHelperImpl::getInstance();
            if settings.m_renderDistance.get() < 2
            {
                self.setupFog(-1, tickDelta);
                renderGlobal.renderSky(tickDelta);
            }
            GL11::glEnable(2912);
            self.setupFog(1, tickDelta);
            if settings.m_ambientOcclusion.get()
            {
                GL11::glShadeModel(7425);
            }
            let frustrum: Frustrum = Frustrum::new();
            frustrum.setPosition(d, d2, d3);
            renderGlobal.clipRenderersByFrustrum(&frustrum, tickDelta);
            if i == 0
            {
                while !renderGlobal.updateRenderers(&entityLiving, false) && renderTimeLimit != 0
                {
                    let l: i64 = renderTimeLimit - jvm::system::nanoTime();
                    if !(l >= 0 && l <= 1000000000)
                    {
                        break;
                    }
                }
            }
            self.setupFog(0, tickDelta);
            GL11::glEnable(2912);
            GL11::glBindTexture(3553, mc.m_renderEngine.get().orNpe().getTexture("/terrain.png"));
            RenderHelper::disableStandardItemLighting();
            renderGlobal.sortAndRender(&entityLiving, 0, delta);
            GL11::glShadeModel(7424);
            RenderHelper::enableStandardItemLighting();
            renderGlobal.renderEntities(&entity_living::getPosition(&entityLiving, tickDelta), &frustrum, tickDelta);
            effectRenderer.renderLit(&entityLiving, tickDelta);
            RenderHelper::disableStandardItemLighting();
            self.setupFog(0, tickDelta);
            effectRenderer.renderParticles(&entityLiving, tickDelta);
            if mc.m_objectMouseOver.get().is_some() && entityLiving.isInsideOfMaterial(&water) && isPlayer
            {
                self.drawBlockHighlight(&renderGlobal, &entityLiving, tickDelta);
            }
            GL11::glBlendFunc(770, 771);
            self.setupFog(0, tickDelta);
            GL11::glEnable(3042);
            GL11::glDisable(2884);
            GL11::glBindTexture(3553, mc.m_renderEngine.get().orNpe().getTexture("/terrain.png"));
            if settings.m_fancyGraphics.get()
            {
                if settings.m_ambientOcclusion.get()
                {
                    GL11::glShadeModel(7425);
                }
                GL11::glColorMask(false, false, false, false);
                let n: i32 = renderGlobal.sortAndRender(&entityLiving, 1, delta);
                if settings.m_anaglyph.get()
                {
                    if Self::anaglyphField() == 0
                    {
                        GL11::glColorMask(false, true, true, true);
                    }
                    else
                    {
                        GL11::glColorMask(true, false, false, true);
                    }
                }
                else
                {
                    GL11::glColorMask(true, true, true, true);
                }
                if n > 0
                {
                    renderGlobal.renderAllRenderLists(1, delta);
                }
                GL11::glShadeModel(7424);
            }
            else
            {
                renderGlobal.sortAndRender(&entityLiving, 1, delta);
            }
            GL11::glDepthMask(true);
            GL11::glEnable(2884);
            GL11::glDisable(3042);
            if self.m_cameraZoom.get() == 1.0
                && isPlayer
                && mc.m_objectMouseOver.get().is_some()
                && !entityLiving.isInsideOfMaterial(&water)
            {
                self.drawBlockHighlight(&renderGlobal, &entityLiving, tickDelta);
            }
            self.renderRainSnow(tickDelta);
            GL11::glDisable(2912);
            self.setupFog(0, tickDelta);
            GL11::glEnable(2912);
            renderGlobal.renderClouds(tickDelta);
            GL11::glDisable(2912);
            self.setupFog(1, tickDelta);
            if self.m_cameraZoom.get() == 1.0
            {
                GL11::glClear(256);
                self.renderItemInHand(tickDelta, i);
            }
            if !settings.m_anaglyph.get()
            {
                return;
            }
        }
        GL11::glColorMask(true, true, true, false);
    }

    pub fn setupGuiState(&self)
    {
        let mc: &Rc<Minecraft> = &self.m_mc;
        let scaledResolution: ScaledResolution =
            ScaledResolution::new(&self.gameSettings(), mc.m_displayWidth.get(), mc.m_displayHeight.get());
        GL11::glClear(256);
        GL11::glMatrixMode(5889);
        GL11::glLoadIdentity();
        GL11::glOrtho(0.0, scaledResolution.scaledWidthD(), scaledResolution.scaledHeightD(), 0.0, 1000.0, 3000.0);
        GL11::glMatrixMode(5888);
        GL11::glLoadIdentity();
        GL11::glTranslatef(0.0, 0.0, -2000.0);
    }
}
