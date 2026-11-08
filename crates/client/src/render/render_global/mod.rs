mod chunks;
mod sky;
mod world_access;

use crate::client::Minecraft;
use crate::render::{
    EntitySorter, GLAllocation, RenderBlocks, RenderEngine, RenderList, RenderManager, Tessellator, WorldRenderer,
};
use crate::settings::GameSettings;
use game::block::{Block, block_leaves};
use game::entity::Entity;
use game::math::math_helper;
use game::tileentity::TileEntity;
use game::world::{IWorldAccess, World};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::buffers::IntBuffer;
use lwjgl::opengl::{ARBOcclusionQuery, GL11};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub struct RenderGlobal
{
    m_self: Weak<RenderGlobal>,
    pub m_tileEntities: Rc<JArrayList<Rc<TileEntity>>>,
    m_worldObj: Field<Option<Rc<World>>>,
    m_renderEngine: Rc<RenderEngine>,
    m_worldRenderersToUpdate: JArrayList<Option<Rc<WorldRenderer>>>,
    m_sortedWorldRenderers: RefCell<Vec<Rc<WorldRenderer>>>,
    m_worldRenderers: RefCell<Vec<Rc<WorldRenderer>>>,
    m_renderChunksWide: Cell<i32>,
    m_renderChunksTall: Cell<i32>,
    m_renderChunksDeep: Cell<i32>,
    m_glRenderListBase: i32,
    m_mc: Rc<Minecraft>,
    m_globalRenderBlocks: RefCell<Option<RenderBlocks>>,
    m_glOcclusionQueryBase: Option<IntBuffer>,
    m_occlusionEnabled: bool,
    m_cloudOffsetX: Cell<i32>,
    m_starGLCallList: i32,
    m_glSkyList: i32,
    m_glSkyList2: i32,
    m_minBlockX: Cell<i32>,
    m_minBlockY: Cell<i32>,
    m_minBlockZ: Cell<i32>,
    m_maxBlockX: Cell<i32>,
    m_maxBlockY: Cell<i32>,
    m_maxBlockZ: Cell<i32>,
    m_renderDistance: Cell<i32>,
    m_renderEntitiesStartupCounter: Cell<i32>,
    m_countEntitiesTotal: Cell<i32>,
    m_countEntitiesRendered: Cell<i32>,
    m_countEntitiesHidden: Cell<i32>,
    m_occlusionResult: IntBuffer,
    m_renderersLoaded: Cell<i32>,
    m_renderersBeingClipped: Cell<i32>,
    m_renderersBeingOccluded: Cell<i32>,
    m_renderersBeingRendered: Cell<i32>,
    m_renderersSkippingRenderPass: Cell<i32>,
    m_worldRenderersCheckIndex: Cell<i32>,
    m_glRenderLists: JArrayList<Rc<WorldRenderer>>,
    m_allRenderLists: [RenderList; 4],
    pub m_glDummyList: i32,
    m_prevSortX: Cell<f64>,
    m_prevSortY: Cell<f64>,
    m_prevSortZ: Cell<f64>,
    pub m_damagePartialTime: Cell<f32>,
    m_frustrumCheckOffset: Cell<i32>,
}

impl RenderGlobal
{
    pub fn new(minecraft: &Rc<Minecraft>, textureManager: &Rc<RenderEngine>) -> Rc<RenderGlobal>
    {
        let occlusionResult: IntBuffer = GLAllocation::createDirectIntBuffer(64);
        let glDummyList: i32 = GLAllocation::generateDisplayLists(1);
        let n3: i32 = 64;
        let glRenderListBase: i32 = GLAllocation::generateDisplayLists(n3 * n3 * n3 * 3);
        let occlusionEnabled: bool = minecraft.m_glCapabilities.get().orNpe().checkARBOcclusion();
        let mut glOcclusionQueryBase: Option<IntBuffer> = None;
        if occlusionEnabled
        {
            occlusionResult.clear();
            let base: IntBuffer = GLAllocation::createDirectIntBuffer(n3 * n3 * n3);
            base.clear();
            base.setPosition(0);
            base.setLimit(n3 * n3 * n3);
            ARBOcclusionQuery::glGenQueriesARB(&base);
            glOcclusionQueryBase = Some(base);
        }
        let starGLCallList: i32 = GLAllocation::generateDisplayLists(3);
        GL11::glPushMatrix();
        GL11::glNewList(starGLCallList, 4864);
        Self::renderStars();
        GL11::glEndList();
        GL11::glPopMatrix();
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let glSkyList: i32 = starGLCallList + 1;
        GL11::glNewList(glSkyList, 4864);
        let n4: i32 = 64;
        let n5: i32 = 256 / n4 + 2;
        let mut f: f32 = 16.0;
        let mut n2: i32 = -n4 * n5;
        while n2 <= n4 * n5
        {
            let mut n: i32 = -n4 * n5;
            while n <= n4 * n5
            {
                tessellator.startDrawingQuads();
                tessellator.addVertex((n2 + 0) as f64, f as f64, (n + 0) as f64);
                tessellator.addVertex((n2 + n4) as f64, f as f64, (n + 0) as f64);
                tessellator.addVertex((n2 + n4) as f64, f as f64, (n + n4) as f64);
                tessellator.addVertex((n2 + 0) as f64, f as f64, (n + n4) as f64);
                tessellator.draw();
                n += n4;
            }
            n2 += n4;
        }
        GL11::glEndList();
        let glSkyList2: i32 = starGLCallList + 2;
        GL11::glNewList(glSkyList2, 4864);
        f = -16.0;
        tessellator.startDrawingQuads();
        n2 = -n4 * n5;
        while n2 <= n4 * n5
        {
            let mut n: i32 = -n4 * n5;
            while n <= n4 * n5
            {
                tessellator.addVertex((n2 + n4) as f64, f as f64, (n + 0) as f64);
                tessellator.addVertex((n2 + 0) as f64, f as f64, (n + 0) as f64);
                tessellator.addVertex((n2 + 0) as f64, f as f64, (n + n4) as f64);
                tessellator.addVertex((n2 + n4) as f64, f as f64, (n + n4) as f64);
                n += n4;
            }
            n2 += n4;
        }
        tessellator.draw();
        GL11::glEndList();
        Rc::new_cyclic(|this| RenderGlobal {
            m_self: Weak::clone(this),
            m_tileEntities: Rc::new(JArrayList::new()),
            m_worldObj: Field::new(None),
            m_renderEngine: Rc::clone(textureManager),
            m_worldRenderersToUpdate: JArrayList::new(),
            m_sortedWorldRenderers: RefCell::new(Vec::new()),
            m_worldRenderers: RefCell::new(Vec::new()),
            m_renderChunksWide: Cell::new(0),
            m_renderChunksTall: Cell::new(0),
            m_renderChunksDeep: Cell::new(0),
            m_glRenderListBase: glRenderListBase,
            m_mc: Rc::clone(minecraft),
            m_globalRenderBlocks: RefCell::new(None),
            m_glOcclusionQueryBase: glOcclusionQueryBase,
            m_occlusionEnabled: occlusionEnabled,
            m_cloudOffsetX: Cell::new(0),
            m_starGLCallList: starGLCallList,
            m_glSkyList: glSkyList,
            m_glSkyList2: glSkyList2,
            m_minBlockX: Cell::new(0),
            m_minBlockY: Cell::new(0),
            m_minBlockZ: Cell::new(0),
            m_maxBlockX: Cell::new(0),
            m_maxBlockY: Cell::new(0),
            m_maxBlockZ: Cell::new(0),
            m_renderDistance: Cell::new(-1),
            m_renderEntitiesStartupCounter: Cell::new(2),
            m_countEntitiesTotal: Cell::new(0),
            m_countEntitiesRendered: Cell::new(0),
            m_countEntitiesHidden: Cell::new(0),
            m_occlusionResult: occlusionResult,
            m_renderersLoaded: Cell::new(0),
            m_renderersBeingClipped: Cell::new(0),
            m_renderersBeingOccluded: Cell::new(0),
            m_renderersBeingRendered: Cell::new(0),
            m_renderersSkippingRenderPass: Cell::new(0),
            m_worldRenderersCheckIndex: Cell::new(0),
            m_glRenderLists: JArrayList::new(),
            m_allRenderLists: [RenderList::new(), RenderList::new(), RenderList::new(), RenderList::new()],
            m_glDummyList: glDummyList,
            m_prevSortX: Cell::new(-9999.0),
            m_prevSortY: Cell::new(-9999.0),
            m_prevSortZ: Cell::new(-9999.0),
            m_damagePartialTime: Cell::new(0.0),
            m_frustrumCheckOffset: Cell::new(0),
        })
    }

    fn rc(&self) -> Rc<RenderGlobal> { self.m_self.upgrade().expect("a live render global is owned by an Rc") }

    fn worldObj(&self) -> Rc<World> { self.m_worldObj.get().orNpe() }

    fn gameSettings(&self) -> Rc<GameSettings> { self.m_mc.m_gameSettings.get().orNpe() }

    fn renderViewEntity(&self) -> Rc<Entity> { self.m_mc.m_renderViewEntity.get().orNpe() }

    fn renderStars()
    {
        let random: Random = Random::withSeed(10842);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        for _ in 0..1500
        {
            let mut d0: f64 = (random.nextFloat() * 2.0 - 1.0) as f64;
            let mut d1: f64 = (random.nextFloat() * 2.0 - 1.0) as f64;
            let mut d2: f64 = (random.nextFloat() * 2.0 - 1.0) as f64;
            let d3: f64 = (0.25 + random.nextFloat() * 0.25) as f64;
            let mut d4: f64 = d0 * d0 + d1 * d1 + d2 * d2;
            if d4 < 1.0 && d4 > 0.01
            {
                d4 = 1.0 / jvm::math::sqrt(d4);
                d0 *= d4;
                d1 *= d4;
                d2 *= d4;
                let d5: f64 = d0 * 100.0;
                let d6: f64 = d1 * 100.0;
                let d7: f64 = d2 * 100.0;
                let d8: f64 = jvm::math::atan2(d0, d2);
                let d9: f64 = jvm::math::sin(d8);
                let d10: f64 = jvm::math::cos(d8);
                let d11: f64 = jvm::math::atan2(jvm::math::sqrt(d0 * d0 + d2 * d2), d1);
                let d12: f64 = jvm::math::sin(d11);
                let d13: f64 = jvm::math::cos(d11);
                let d14: f64 = random.nextDouble() * std::f64::consts::PI * 2.0;
                let d15: f64 = jvm::math::sin(d14);
                let d16: f64 = jvm::math::cos(d14);
                for j in 0..4
                {
                    let d17: f64 = 0.0;
                    let d18: f64 = ((j & 2) - 1) as f64 * d3;
                    let d19: f64 = ((j + 1 & 2) - 1) as f64 * d3;
                    let d20: f64 = d17;
                    let d21: f64 = d18 * d16 - d19 * d15;
                    let d22: f64 = d19 * d16 + d18 * d15;
                    let d23: f64 = d22;
                    let d24: f64 = d21 * d12 + d20 * d13;
                    let d25: f64 = d20 * d12 - d21 * d13;
                    let d26: f64 = d25 * d9 - d23 * d10;
                    let d27: f64 = d24;
                    let d28: f64 = d23 * d9 + d25 * d10;
                    tessellator.addVertex(d5 + d26, d6 + d27, d7 + d28);
                }
            }
        }
        tessellator.draw();
    }

    pub fn changeWorld(&self, world: Option<Rc<World>>)
    {
        let listener: Rc<dyn IWorldAccess> = self.rc();
        if let Some(old) = self.m_worldObj.get()
        {
            old.removeWorldAccess(&listener);
        }
        self.m_prevSortX.set(-9999.0);
        self.m_prevSortY.set(-9999.0);
        self.m_prevSortZ.set(-9999.0);
        RenderManager::instance().setWorld(world.clone());
        self.m_worldObj.set(world.clone());
        *self.m_globalRenderBlocks.borrow_mut() = Some(match world.clone()
        {
            Some(world) => RenderBlocks::new(world),
            None => RenderBlocks::empty(),
        });
        if let Some(world) = world
        {
            world.addWorldAccess(listener);
            self.loadRenderers();
        }
    }

    fn rendererIndex(&self, x: i32, y: i32, z: i32) -> usize
    {
        ((z * self.m_renderChunksTall.get() + y) * self.m_renderChunksWide.get() + x) as usize
    }

    pub fn loadRenderers(&self)
    {
        let settings: Rc<GameSettings> = self.gameSettings();
        block_leaves::setGraphicsLevel(&Block::leaves(), settings.m_fancyGraphics.get());
        self.m_renderDistance.set(settings.m_renderDistance.get());
        for renderer in self.m_worldRenderers.borrow().iter()
        {
            renderer.delete();
        }
        let mut j1: i32 = 64 << (3 - self.m_renderDistance.get());
        if j1 > 400
        {
            j1 = 400;
        }
        self.m_renderChunksWide.set(j1 / 16 + 1);
        self.m_renderChunksTall.set(8);
        self.m_renderChunksDeep.set(j1 / 16 + 1);
        let wide: i32 = self.m_renderChunksWide.get();
        let tall: i32 = self.m_renderChunksTall.get();
        let deep: i32 = self.m_renderChunksDeep.get();
        let count: usize = (wide * tall * deep) as usize;
        let mut worldRenderers: Vec<Option<Rc<WorldRenderer>>> = vec![None; count];
        let mut sortedWorldRenderers: Vec<Option<Rc<WorldRenderer>>> = vec![None; count];
        let mut b0: i32 = 0;
        let mut j: i32 = 0;
        self.m_minBlockX.set(0);
        self.m_minBlockY.set(0);
        self.m_minBlockZ.set(0);
        self.m_maxBlockX.set(wide);
        self.m_maxBlockY.set(tall);
        self.m_maxBlockZ.set(deep);
        for k in 0..self.m_worldRenderersToUpdate.size()
        {
            self.m_worldRenderersToUpdate.get(k).orNpe().m_needsUpdate.set(false);
        }
        self.m_worldRenderersToUpdate.clear();
        self.m_tileEntities.clear();
        let world: Option<Rc<World>> = self.m_worldObj.get();
        for k1 in 0..wide
        {
            for l in 0..tall
            {
                for i1 in 0..deep
                {
                    let index: usize = self.rendererIndex(k1, l, i1);
                    let renderer: Rc<WorldRenderer> = Rc::new(WorldRenderer::new(
                        world.clone(),
                        &self.m_tileEntities,
                        k1 * 16,
                        l * 16,
                        i1 * 16,
                        16,
                        self.m_glRenderListBase + b0,
                    ));
                    if let Some(base) = &self.m_glOcclusionQueryBase
                    {
                        renderer.m_glOcclusionQuery.set(base.getAt(j));
                    }
                    renderer.m_isWaitingOnOcclusionQuery.set(false);
                    renderer.m_isVisible.set(true);
                    renderer.m_isInFrustum.set(true);
                    renderer.m_chunkIndex.set(j);
                    j += 1;
                    renderer.markDirty();
                    worldRenderers[index] = Some(Rc::clone(&renderer));
                    sortedWorldRenderers[index] = Some(Rc::clone(&renderer));
                    self.m_worldRenderersToUpdate.add(Some(renderer));
                    b0 += 3;
                }
            }
        }
        *self.m_worldRenderers.borrow_mut() = worldRenderers.into_iter().map(OrNpe::orNpe).collect();
        *self.m_sortedWorldRenderers.borrow_mut() = sortedWorldRenderers.into_iter().map(OrNpe::orNpe).collect();
        if self.m_worldObj.get().is_some()
            && let Some(entityliving) = self.m_mc.m_renderViewEntity.get()
        {
            self.markRenderersForNewPosition(
                math_helper::floor_double(entityliving.posX()),
                math_helper::floor_double(entityliving.posY()),
                math_helper::floor_double(entityliving.posZ()),
            );
            let sorter: EntitySorter = EntitySorter::new(&entityliving);
            jvm::sort::sort(&mut self.m_sortedWorldRenderers.borrow_mut(), &mut |a, b| sorter.compare(a, b));
        }
        self.m_renderEntitiesStartupCounter.set(2);
    }

    pub fn getDebugInfoRenders(&self) -> String
    {
        format!(
            "C: {}/{}. F: {}, O: {}, E: {}",
            self.m_renderersBeingRendered.get(),
            self.m_renderersLoaded.get(),
            self.m_renderersBeingClipped.get(),
            self.m_renderersBeingOccluded.get(),
            self.m_renderersSkippingRenderPass.get()
        )
    }

    pub fn getDebugInfoEntities(&self) -> String
    {
        format!(
            "E: {}/{}. B: {}, I: {}",
            self.m_countEntitiesRendered.get(),
            self.m_countEntitiesTotal.get(),
            self.m_countEntitiesHidden.get(),
            self.m_countEntitiesTotal.get() - self.m_countEntitiesHidden.get() - self.m_countEntitiesRendered.get()
        )
    }

    fn markRenderersForNewPosition(&self, mut cameraX: i32, _cameraY: i32, mut cameraZ: i32)
    {
        cameraX -= 8;
        cameraZ -= 8;
        self.m_minBlockX.set(i32::MAX);
        self.m_minBlockY.set(i32::MAX);
        self.m_minBlockZ.set(i32::MAX);
        self.m_maxBlockX.set(i32::MIN);
        self.m_maxBlockY.set(i32::MIN);
        self.m_maxBlockZ.set(i32::MIN);
        let i: i32 = self.m_renderChunksWide.get() * 16;
        let j: i32 = i / 2;
        let worldRenderers: std::cell::Ref<'_, Vec<Rc<WorldRenderer>>> = self.m_worldRenderers.borrow();
        for k in 0..self.m_renderChunksWide.get()
        {
            let mut l: i32 = k * 16;
            let mut i1: i32 = l + j - cameraX;
            if i1 < 0
            {
                i1 -= i - 1;
            }
            i1 /= i;
            l -= i1 * i;
            self.m_minBlockX.set(self.m_minBlockX.get().min(l));
            self.m_maxBlockX.set(self.m_maxBlockX.get().max(l));
            for j1 in 0..self.m_renderChunksDeep.get()
            {
                let mut k1: i32 = j1 * 16;
                let mut l1: i32 = k1 + j - cameraZ;
                if l1 < 0
                {
                    l1 -= i - 1;
                }
                l1 /= i;
                k1 -= l1 * i;
                self.m_minBlockZ.set(self.m_minBlockZ.get().min(k1));
                self.m_maxBlockZ.set(self.m_maxBlockZ.get().max(k1));
                for i2 in 0..self.m_renderChunksTall.get()
                {
                    let j2: i32 = i2 * 16;
                    self.m_minBlockY.set(self.m_minBlockY.get().min(j2));
                    self.m_maxBlockY.set(self.m_maxBlockY.get().max(j2));
                    let worldrenderer: &Rc<WorldRenderer> = &worldRenderers[self.rendererIndex(k, i2, j1)];
                    let flag: bool = worldrenderer.m_needsUpdate.get();
                    worldrenderer.setPosition(l, j2, k1);
                    if !flag && worldrenderer.m_needsUpdate.get()
                    {
                        self.m_worldRenderersToUpdate.add(Some(Rc::clone(worldrenderer)));
                    }
                }
            }
        }
    }

    pub fn markDirty(&self, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32)
    {
        let i: i32 = math_helper::bucketInt(minX, 16);
        let j: i32 = math_helper::bucketInt(minY, 16);
        let k: i32 = math_helper::bucketInt(minZ, 16);
        let l: i32 = math_helper::bucketInt(maxX, 16);
        let i1: i32 = math_helper::bucketInt(maxY, 16);
        let j1: i32 = math_helper::bucketInt(maxZ, 16);
        let wide: i32 = self.m_renderChunksWide.get();
        let tall: i32 = self.m_renderChunksTall.get();
        let deep: i32 = self.m_renderChunksDeep.get();
        let worldRenderers: std::cell::Ref<'_, Vec<Rc<WorldRenderer>>> = self.m_worldRenderers.borrow();
        for k1 in i..=l
        {
            let mut l1: i32 = k1 % wide;
            if l1 < 0
            {
                l1 += wide;
            }
            for i2 in j..=i1
            {
                let mut j2: i32 = i2 % tall;
                if j2 < 0
                {
                    j2 += tall;
                }
                for k2 in k..=j1
                {
                    let mut l2: i32 = k2 % deep;
                    if l2 < 0
                    {
                        l2 += deep;
                    }
                    let worldrenderer: &Rc<WorldRenderer> = &worldRenderers[self.rendererIndex(l1, j2, l2)];
                    if !worldrenderer.m_needsUpdate.get()
                    {
                        self.m_worldRenderersToUpdate.add(Some(Rc::clone(worldrenderer)));
                        worldrenderer.markDirty();
                    }
                }
            }
        }
    }

    pub fn clipRenderersByFrustrum(&self, culler: &dyn crate::render::ICamera, _tickDelta: f32)
    {
        for (i, renderer) in self.m_worldRenderers.borrow().iter().enumerate()
        {
            if !renderer.skipAllRenderPasses()
                && (!renderer.m_isInFrustum.get() || (i as i32 + self.m_frustrumCheckOffset.get() & 15) == 0)
            {
                renderer.updateInFrustrum(culler);
            }
        }
        self.m_frustrumCheckOffset.set(self.m_frustrumCheckOffset.get() + 1);
    }

    pub fn releaseGlLists(&self) { GLAllocation::releaseList(self.m_glRenderListBase); }
}
