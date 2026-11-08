use crate::gui::FontRenderer;
use crate::render::{
    EntityRenderer, EntitySorter, ICamera, RenderGlobal, RenderHelper, RenderManager, RenderSorter, Tessellator,
    TileEntityRenderer, WorldRenderer,
};
use game::block::Block;
use game::entity::Entity;
use game::item::ItemStack;
use game::math::{AxisAlignedBB, EnumMovingObjectType, MovingObjectPosition, Vec3D, math_helper};
use game::world::World;
use jvm::array_list::JArrayList;
use jvm::throwable::OrNpe;
use lwjgl::opengl::{ARBOcclusionQuery, GL11};
use std::rc::Rc;

fn interpolate(last: f64, current: f64, tickDelta: f64) -> f64 { last + (current - last) * tickDelta }

impl RenderGlobal
{
    pub fn renderEntities(&self, cameraPos: &Vec3D, culler: &dyn ICamera, tickDelta: f32)
    {
        if self.m_renderEntitiesStartupCounter.get() > 0
        {
            self.m_renderEntitiesStartupCounter.set(self.m_renderEntitiesStartupCounter.get() - 1);
            return;
        }
        let world: Rc<World> = self.worldObj();
        let fontRenderer: Option<Rc<FontRenderer>> = self.m_mc.m_fontRenderer.get();
        let viewEntity: Option<Rc<Entity>> = self.m_mc.m_renderViewEntity.get();
        TileEntityRenderer::instance().cacheActiveRenderInfo(
            &world,
            &self.m_renderEngine,
            fontRenderer.clone(),
            viewEntity.clone(),
            tickDelta,
        );
        RenderManager::instance().cacheActiveRenderInfo(
            &world,
            &self.m_renderEngine,
            fontRenderer,
            viewEntity,
            &self.gameSettings(),
            tickDelta,
        );
        self.m_countEntitiesTotal.set(0);
        self.m_countEntitiesRendered.set(0);
        self.m_countEntitiesHidden.set(0);
        let entityliving: Rc<Entity> = self.renderViewEntity();
        let delta: f64 = tickDelta as f64;
        let x: f64 = interpolate(entityliving.m_lastTickPosX.get(), entityliving.posX(), delta);
        let y: f64 = interpolate(entityliving.m_lastTickPosY.get(), entityliving.posY(), delta);
        let z: f64 = interpolate(entityliving.m_lastTickPosZ.get(), entityliving.posZ(), delta);
        RenderManager::setRenderPos(x, y, z);
        TileEntityRenderer::setStaticPlayerPos(x, y, z);
        let list: Rc<JArrayList<Rc<Entity>>> = world.getLoadedEntityList();
        self.m_countEntitiesTotal.set(list.size());
        let mut i: i32 = 0;
        while i < world.m_weatherEffects.size()
        {
            let entity: Rc<Entity> = world.m_weatherEffects.get(i);
            self.m_countEntitiesRendered.set(self.m_countEntitiesRendered.get() + 1);
            if entity.isInRangeToRenderVec3D(cameraPos)
            {
                RenderManager::instance().renderEntity(&entity, tickDelta);
            }
            i += 1;
        }
        let mut k: i32 = 0;
        while k < list.size()
        {
            let entity1: Rc<Entity> = list.get(k);
            let viewEntity: Rc<Entity> = self.renderViewEntity();
            if entity1.isInRangeToRenderVec3D(cameraPos)
                && (entity1.m_ignoreFrustumCheck.get() || culler.isBoundingBoxInFrustum(&entity1.m_boundingBox))
                && (!Rc::ptr_eq(&entity1, &viewEntity)
                    || self.gameSettings().m_thirdPersonView.get()
                    || viewEntity.isPlayerSleeping())
            {
                let j: i32 = math_helper::floor_double(entity1.posY()).clamp(0, 127);
                if world.blockExists(
                    math_helper::floor_double(entity1.posX()),
                    j,
                    math_helper::floor_double(entity1.posZ()),
                )
                {
                    self.m_countEntitiesRendered.set(self.m_countEntitiesRendered.get() + 1);
                    RenderManager::instance().renderEntity(&entity1, tickDelta);
                }
            }
            k += 1;
        }
        let mut l: i32 = 0;
        while l < self.m_tileEntities.size()
        {
            TileEntityRenderer::instance().renderTileEntity(&self.m_tileEntities.get(l), tickDelta);
            l += 1;
        }
    }

    fn containsRenderer(&self, renderer: &Rc<WorldRenderer>) -> bool
    {
        self.m_worldRenderersToUpdate
            .toVec()
            .iter()
            .any(|entry| entry.as_ref().is_some_and(|entry| Rc::ptr_eq(entry, renderer)))
    }

    pub fn sortAndRender(&self, camera: &Rc<Entity>, layer: i32, tickDelta: f64) -> i32
    {
        for _ in 0..10
        {
            let length: i32 = self.m_worldRenderers.borrow().len() as i32;
            self.m_worldRenderersCheckIndex.set((self.m_worldRenderersCheckIndex.get() + 1) % length);
            let worldrenderer: Rc<WorldRenderer> =
                Rc::clone(&self.m_worldRenderers.borrow()[self.m_worldRenderersCheckIndex.get() as usize]);
            if worldrenderer.m_needsUpdate.get() && !self.containsRenderer(&worldrenderer)
            {
                self.m_worldRenderersToUpdate.add(Some(worldrenderer));
            }
        }
        if self.gameSettings().m_renderDistance.get() != self.m_renderDistance.get()
        {
            self.loadRenderers();
        }
        if layer == 0
        {
            self.m_renderersLoaded.set(0);
            self.m_renderersBeingClipped.set(0);
            self.m_renderersBeingOccluded.set(0);
            self.m_renderersBeingRendered.set(0);
            self.m_renderersSkippingRenderPass.set(0);
        }
        let d5: f64 = interpolate(camera.m_lastTickPosX.get(), camera.posX(), tickDelta);
        let d0: f64 = interpolate(camera.m_lastTickPosY.get(), camera.posY(), tickDelta);
        let d1: f64 = interpolate(camera.m_lastTickPosZ.get(), camera.posZ(), tickDelta);
        let d2: f64 = camera.posX() - self.m_prevSortX.get();
        let d3: f64 = camera.posY() - self.m_prevSortY.get();
        let d4: f64 = camera.posZ() - self.m_prevSortZ.get();
        if d2 * d2 + d3 * d3 + d4 * d4 > 16.0
        {
            self.m_prevSortX.set(camera.posX());
            self.m_prevSortY.set(camera.posY());
            self.m_prevSortZ.set(camera.posZ());
            self.markRenderersForNewPosition(
                math_helper::floor_double(camera.posX()),
                math_helper::floor_double(camera.posY()),
                math_helper::floor_double(camera.posZ()),
            );
            let sorter: EntitySorter = EntitySorter::new(camera);
            jvm::sort::sort(&mut self.m_sortedWorldRenderers.borrow_mut(), &mut |a, b| sorter.compare(a, b));
        }
        RenderHelper::disableStandardItemLighting();
        let mut j: i32 = 0;
        let settings = self.gameSettings();
        let sortedLength: i32 = self.m_sortedWorldRenderers.borrow().len() as i32;
        if self.m_occlusionEnabled && settings.m_advancedOpengl.get() && !settings.m_anaglyph.get() && layer == 0
        {
            let mut k: i32 = 0;
            let mut l: i32 = 16;
            self.checkOcclusionQueryResult(k, l);
            for i1 in k..l
            {
                self.sorted(i1).m_isVisible.set(true);
            }
            j += self.renderSortedRenderers(k, l, layer, tickDelta);
            loop
            {
                k = l;
                l *= 2;
                if l > sortedLength
                {
                    l = sortedLength;
                }
                GL11::glDisable(3553);
                GL11::glDisable(2896);
                GL11::glDisable(3008);
                GL11::glDisable(2912);
                GL11::glColorMask(false, false, false, false);
                GL11::glDepthMask(false);
                self.checkOcclusionQueryResult(k, l);
                GL11::glPushMatrix();
                let mut f9: f32 = 0.0;
                let mut f: f32 = 0.0;
                let mut f1: f32 = 0.0;
                for j1 in k..l
                {
                    let renderer: Rc<WorldRenderer> = self.sorted(j1);
                    if renderer.skipAllRenderPasses()
                    {
                        renderer.m_isInFrustum.set(false);
                        continue;
                    }
                    if !renderer.m_isInFrustum.get()
                    {
                        renderer.m_isVisible.set(true);
                    }
                    if renderer.m_isInFrustum.get() && !renderer.m_isWaitingOnOcclusionQuery.get()
                    {
                        let f2: f32 = math_helper::sqrt_float(renderer.distanceToEntitySquared(camera));
                        let k1: i32 = (1.0 + f2 / 128.0) as i32;
                        if self.m_cloudOffsetX.get() % k1 == j1 % k1
                        {
                            let f3: f32 = (renderer.m_posXMinus.get() as f64 - d5) as f32;
                            let f4: f32 = (renderer.m_posYMinus.get() as f64 - d0) as f32;
                            let f5: f32 = (renderer.m_posZMinus.get() as f64 - d1) as f32;
                            let f6: f32 = f3 - f9;
                            let f7: f32 = f4 - f;
                            let f8: f32 = f5 - f1;
                            if f6 != 0.0 || f7 != 0.0 || f8 != 0.0
                            {
                                GL11::glTranslatef(f6, f7, f8);
                                f9 += f6;
                                f += f7;
                                f1 += f8;
                            }
                            ARBOcclusionQuery::glBeginQueryARB(35092, renderer.m_glOcclusionQuery.get());
                            renderer.callOcclusionQueryList();
                            ARBOcclusionQuery::glEndQueryARB(35092);
                            renderer.m_isWaitingOnOcclusionQuery.set(true);
                        }
                    }
                }
                GL11::glPopMatrix();
                if settings.m_anaglyph.get()
                {
                    if EntityRenderer::anaglyphField() == 0
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
                GL11::glDepthMask(true);
                GL11::glEnable(3553);
                GL11::glEnable(3008);
                GL11::glEnable(2912);
                j += self.renderSortedRenderers(k, l, layer, tickDelta);
                if l >= sortedLength
                {
                    break;
                }
            }
        }
        else
        {
            j += self.renderSortedRenderers(0, sortedLength, layer, tickDelta);
        }
        j
    }

    fn sorted(&self, index: i32) -> Rc<WorldRenderer>
    {
        Rc::clone(&self.m_sortedWorldRenderers.borrow()[index as usize])
    }

    fn checkOcclusionQueryResult(&self, from: i32, to: i32)
    {
        for i in from..to
        {
            let renderer: Rc<WorldRenderer> = self.sorted(i);
            if renderer.m_isWaitingOnOcclusionQuery.get()
            {
                self.m_occlusionResult.clear();
                ARBOcclusionQuery::glGetQueryObjectuARB(
                    renderer.m_glOcclusionQuery.get(),
                    34919,
                    &self.m_occlusionResult,
                );
                if self.m_occlusionResult.getAt(0) != 0
                {
                    renderer.m_isWaitingOnOcclusionQuery.set(false);
                    self.m_occlusionResult.clear();
                    ARBOcclusionQuery::glGetQueryObjectuARB(
                        renderer.m_glOcclusionQuery.get(),
                        34918,
                        &self.m_occlusionResult,
                    );
                    renderer.m_isVisible.set(self.m_occlusionResult.getAt(0) != 0);
                }
            }
        }
    }

    fn renderSortedRenderers(&self, from: i32, to: i32, layer: i32, tickDelta: f64) -> i32
    {
        self.m_glRenderLists.clear();
        let mut i: i32 = 0;
        for j in from..to
        {
            let renderer: Rc<WorldRenderer> = self.sorted(j);
            let skip: bool = renderer.m_skipRenderPass[layer as usize].get();
            if layer == 0
            {
                self.m_renderersLoaded.set(self.m_renderersLoaded.get() + 1);
                if skip
                {
                    self.m_renderersSkippingRenderPass.set(self.m_renderersSkippingRenderPass.get() + 1);
                }
                else if !renderer.m_isInFrustum.get()
                {
                    self.m_renderersBeingClipped.set(self.m_renderersBeingClipped.get() + 1);
                }
                else if self.m_occlusionEnabled && !renderer.m_isVisible.get()
                {
                    self.m_renderersBeingOccluded.set(self.m_renderersBeingOccluded.get() + 1);
                }
                else
                {
                    self.m_renderersBeingRendered.set(self.m_renderersBeingRendered.get() + 1);
                }
            }
            if !skip
                && renderer.m_isInFrustum.get()
                && (!self.m_occlusionEnabled || renderer.m_isVisible.get())
                && renderer.getGLCallListForPass(layer) >= 0
            {
                self.m_glRenderLists.add(renderer);
                i += 1;
            }
        }
        let entityliving: Rc<Entity> = self.renderViewEntity();
        let d2: f64 = interpolate(entityliving.m_lastTickPosX.get(), entityliving.posX(), tickDelta);
        let d0: f64 = interpolate(entityliving.m_lastTickPosY.get(), entityliving.posY(), tickDelta);
        let d1: f64 = interpolate(entityliving.m_lastTickPosZ.get(), entityliving.posZ(), tickDelta);
        let mut l: usize = 0;
        for list in &self.m_allRenderLists
        {
            list.clear();
        }
        for l1 in 0..self.m_glRenderLists.size()
        {
            let worldrenderer: Rc<WorldRenderer> = self.m_glRenderLists.get(l1);
            let mut j1: Option<usize> = None;
            for k1 in 0..l
            {
                if self.m_allRenderLists[k1].isAt(
                    worldrenderer.m_posXMinus.get(),
                    worldrenderer.m_posYMinus.get(),
                    worldrenderer.m_posZMinus.get(),
                )
                {
                    j1 = Some(k1);
                }
            }
            let j1: usize = match j1
            {
                Some(index) => index,
                None =>
                {
                    let index: usize = l;
                    l += 1;
                    self.m_allRenderLists[index].init(
                        worldrenderer.m_posXMinus.get(),
                        worldrenderer.m_posYMinus.get(),
                        worldrenderer.m_posZMinus.get(),
                        d2,
                        d0,
                        d1,
                    );
                    index
                }
            };
            self.m_allRenderLists[j1].addGlList(worldrenderer.getGLCallListForPass(layer));
        }
        self.renderAllRenderLists(layer, tickDelta);
        i
    }

    pub fn renderAllRenderLists(&self, _layer: i32, _tickDelta: f64)
    {
        for list in &self.m_allRenderLists
        {
            list.render();
        }
    }

    pub fn updateClouds(&self) { self.m_cloudOffsetX.set(self.m_cloudOffsetX.get() + 1); }

    pub fn updateRenderers(&self, camera: &Rc<Entity>, force: bool) -> bool
    {
        let b0: usize = 2;
        let rendersorter: RenderSorter = RenderSorter::new(camera);
        let mut aworldrenderer: [Option<Rc<WorldRenderer>>; 2] = [None, None];
        let mut arraylist: Option<Vec<Rc<WorldRenderer>>> = None;
        let i: i32 = self.m_worldRenderersToUpdate.size();
        let mut j: i32 = 0;
        for k in 0..i
        {
            let worldrenderer: Rc<WorldRenderer> = self.m_worldRenderersToUpdate.get(k).orNpe();
            if !force
            {
                if worldrenderer.distanceToEntitySquared(camera) > 256.0
                {
                    let mut l: i32 = 0;
                    while (l as usize) < b0
                        && aworldrenderer[l as usize]
                            .as_ref()
                            .is_none_or(|slot| rendersorter.compare(slot, &worldrenderer) <= 0)
                    {
                        l += 1;
                    }
                    l -= 1;
                    if l <= 0
                    {
                        continue;
                    }
                    let mut i1: i32 = l;
                    loop
                    {
                        i1 -= 1;
                        if i1 == 0
                        {
                            break;
                        }
                        aworldrenderer[(i1 - 1) as usize] = aworldrenderer[i1 as usize].clone();
                    }
                    aworldrenderer[l as usize] = Some(worldrenderer);
                    continue;
                }
            }
            else if !worldrenderer.m_isInFrustum.get()
            {
                continue;
            }
            j += 1;
            arraylist.get_or_insert_with(Vec::new).push(worldrenderer);
            self.m_worldRenderersToUpdate.set(k, None);
        }
        if let Some(mut arraylist) = arraylist
        {
            if arraylist.len() > 1
            {
                jvm::sort::sort(&mut arraylist, &mut |a, b| rendersorter.compare(a, b));
            }
            for worldrenderer3 in arraylist.iter().rev()
            {
                worldrenderer3.updateRenderer();
                worldrenderer3.m_needsUpdate.set(false);
            }
        }
        let mut k2: i32 = 0;
        for l2 in (0..b0).rev()
        {
            if let Some(worldrenderer4) = aworldrenderer[l2].clone()
            {
                if !worldrenderer4.m_isInFrustum.get() && l2 != b0 - 1
                {
                    aworldrenderer[l2] = None;
                    aworldrenderer[0] = None;
                    break;
                }
                worldrenderer4.updateRenderer();
                worldrenderer4.m_needsUpdate.set(false);
                k2 += 1;
            }
        }
        let mut i3: i32 = 0;
        let mut j3: i32 = 0;
        let k3: i32 = self.m_worldRenderersToUpdate.size();
        while i3 != k3
        {
            if let Some(worldrenderer1) = self.m_worldRenderersToUpdate.get(i3)
            {
                let flag1: bool = aworldrenderer
                    .iter()
                    .any(|slot| slot.as_ref().is_some_and(|slot| Rc::ptr_eq(slot, &worldrenderer1)));
                if !flag1
                {
                    if j3 != i3
                    {
                        self.m_worldRenderersToUpdate.set(j3, Some(worldrenderer1));
                    }
                    j3 += 1;
                }
            }
            i3 += 1;
        }
        loop
        {
            i3 -= 1;
            if i3 < j3
            {
                break;
            }
            self.m_worldRenderersToUpdate.removeAt(i3);
        }
        i == j + k2
    }

    pub fn drawBlockBreaking(
        &self,
        camera: &Rc<Entity>,
        hit: &MovingObjectPosition,
        var3: i32,
        itemInHand: Option<&Rc<ItemStack>>,
        tickDelta: f32,
    )
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glEnable(3042);
        GL11::glEnable(3008);
        GL11::glBlendFunc(770, 1);
        let time: f32 = jvm::system::currentTimeMillis() as f32;
        GL11::glColor4f(1.0, 1.0, 1.0, (math_helper::sin(time / 100.0) * 0.2 + 0.4) * 0.5);
        if var3 == 0
        {
            if self.m_damagePartialTime.get() > 0.0
            {
                GL11::glBlendFunc(774, 768);
                let i: i32 = self.m_renderEngine.getTexture("/terrain.png");
                GL11::glBindTexture(3553, i);
                GL11::glColor4f(1.0, 1.0, 1.0, 0.5);
                GL11::glPushMatrix();
                let j: i32 = self.worldObj().getBlockId(hit.m_blockX, hit.m_blockY, hit.m_blockZ);
                let block: Option<Rc<Block>> = if j > 0 { Block::byId(j) } else { None };
                GL11::glDisable(3008);
                GL11::glPolygonOffset(-3.0, -3.0);
                GL11::glEnable(32823);
                let delta: f64 = tickDelta as f64;
                let d0: f64 = interpolate(camera.m_lastTickPosX.get(), camera.posX(), delta);
                let d1: f64 = interpolate(camera.m_lastTickPosY.get(), camera.posY(), delta);
                let d2: f64 = interpolate(camera.m_lastTickPosZ.get(), camera.posZ(), delta);
                let block: Rc<Block> = block.unwrap_or_else(Block::stone);
                GL11::glEnable(3008);
                tessellator.startDrawingQuads();
                tessellator.setTranslationD(-d0, -d1, -d2);
                tessellator.disableColor();
                self.m_globalRenderBlocks.borrow_mut().as_mut().orNpe().renderBlockUsingTexture(
                    &block,
                    hit.m_blockX,
                    hit.m_blockY,
                    hit.m_blockZ,
                    240 + (self.m_damagePartialTime.get() * 10.0) as i32,
                );
                tessellator.draw();
                tessellator.setTranslationD(0.0, 0.0, 0.0);
                GL11::glDisable(3008);
                GL11::glPolygonOffset(0.0, 0.0);
                GL11::glDisable(32823);
                GL11::glEnable(3008);
                GL11::glDepthMask(true);
                GL11::glPopMatrix();
            }
        }
        else if itemInHand.is_some()
        {
            GL11::glBlendFunc(770, 771);
            let f: f32 = math_helper::sin(jvm::system::currentTimeMillis() as f32 / 100.0) * 0.2 + 0.8;
            GL11::glColor4f(f, f, f, math_helper::sin(jvm::system::currentTimeMillis() as f32 / 200.0) * 0.2 + 0.5);
            let l: i32 = self.m_renderEngine.getTexture("/terrain.png");
            GL11::glBindTexture(3553, l);
        }
        GL11::glDisable(3042);
        GL11::glDisable(3008);
    }

    pub fn drawSelectionBox(
        &self,
        camera: &Rc<Entity>,
        hit: &MovingObjectPosition,
        var3: i32,
        _itemInHand: Option<&Rc<ItemStack>>,
        tickDelta: f32,
    )
    {
        if var3 != 0 || hit.m_typeOfHit != EnumMovingObjectType::TILE
        {
            return;
        }
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        GL11::glColor4f(0.0, 0.0, 0.0, 0.4);
        GL11::glLineWidth(2.0);
        GL11::glDisable(3553);
        GL11::glDepthMask(false);
        let f: f32 = 0.002;
        let world: Rc<World> = self.worldObj();
        let i: i32 = world.getBlockId(hit.m_blockX, hit.m_blockY, hit.m_blockZ);
        if i > 0
        {
            let block: Rc<Block> = Block::byId(i).orNpe();
            block.setBlockBoundsBasedOnState(&*world, hit.m_blockX, hit.m_blockY, hit.m_blockZ);
            let delta: f64 = tickDelta as f64;
            let d0: f64 = interpolate(camera.m_lastTickPosX.get(), camera.posX(), delta);
            let d1: f64 = interpolate(camera.m_lastTickPosY.get(), camera.posY(), delta);
            let d2: f64 = interpolate(camera.m_lastTickPosZ.get(), camera.posZ(), delta);
            let shape: Rc<AxisAlignedBB> = block
                .getSelectedBoundingBoxFromPool(&world, hit.m_blockX, hit.m_blockY, hit.m_blockZ)
                .expand(f as f64, f as f64, f as f64)
                .getOffsetBoundingBox(-d0, -d1, -d2);
            Self::drawOutlinedBoundingBox(&shape);
        }
        GL11::glDepthMask(true);
        GL11::glEnable(3553);
        GL11::glDisable(3042);
    }

    fn drawOutlinedBoundingBox(shape: &AxisAlignedBB)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let (minX, minY, minZ): (f64, f64, f64) = (shape.m_minX.get(), shape.m_minY.get(), shape.m_minZ.get());
        let (maxX, maxY, maxZ): (f64, f64, f64) = (shape.m_maxX.get(), shape.m_maxY.get(), shape.m_maxZ.get());
        for y in [minY, maxY]
        {
            tessellator.startDrawing(3);
            tessellator.addVertex(minX, y, minZ);
            tessellator.addVertex(maxX, y, minZ);
            tessellator.addVertex(maxX, y, maxZ);
            tessellator.addVertex(minX, y, maxZ);
            tessellator.addVertex(minX, y, minZ);
            tessellator.draw();
        }
        tessellator.startDrawing(1);
        for (x, z) in [(minX, minZ), (maxX, minZ), (maxX, maxZ), (minX, maxZ)]
        {
            tessellator.addVertex(x, minY, z);
            tessellator.addVertex(x, maxY, z);
        }
        tessellator.draw();
    }

    pub fn updateAllRenderers(&self)
    {
        for renderer in self.m_worldRenderers.borrow().iter()
        {
            if renderer.m_isChunkLit.get() && !renderer.m_needsUpdate.get()
            {
                self.m_worldRenderersToUpdate.add(Some(Rc::clone(renderer)));
                renderer.markDirty();
            }
        }
    }
}
