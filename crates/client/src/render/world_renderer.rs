use crate::render::{ICamera, RenderBlocks, RenderItem, Tessellator, TileEntityRenderer};
use game::block::Block;
use game::entity::Entity;
use game::math::{AxisAlignedBB, math_helper};
use game::tileentity::TileEntity;
use game::world::{ChunkCache, IBlockAccess, IS_LIT, World};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicI32, Ordering};

pub static CHUNKS_UPDATED: AtomicI32 = AtomicI32::new(0);

pub struct WorldRenderer
{
    pub m_worldObj: Field<Option<Rc<World>>>,
    m_glRenderList: i32,
    pub m_posX: Cell<i32>,
    pub m_posY: Cell<i32>,
    pub m_posZ: Cell<i32>,
    pub m_sizeWidth: i32,
    pub m_sizeHeight: i32,
    pub m_sizeDepth: i32,
    pub m_posXMinus: Cell<i32>,
    pub m_posYMinus: Cell<i32>,
    pub m_posZMinus: Cell<i32>,
    pub m_posXClip: Cell<i32>,
    pub m_posYClip: Cell<i32>,
    pub m_posZClip: Cell<i32>,
    pub m_isInFrustum: Cell<bool>,
    pub m_skipRenderPass: [Cell<bool>; 2],
    pub m_posXPlus: Cell<i32>,
    pub m_posYPlus: Cell<i32>,
    pub m_posZPlus: Cell<i32>,
    pub m_rendererRadius: f32,
    pub m_needsUpdate: Cell<bool>,
    pub m_rendererBoundingBox: Field<Option<Rc<AxisAlignedBB>>>,
    pub m_chunkIndex: Cell<i32>,
    pub m_isVisible: Cell<bool>,
    pub m_isWaitingOnOcclusionQuery: Cell<bool>,
    pub m_glOcclusionQuery: Cell<i32>,
    pub m_isChunkLit: Cell<bool>,
    m_isInitialized: Cell<bool>,
    pub m_tileEntityRenderers: JArrayList<Rc<TileEntity>>,
    m_tileEntities: Rc<JArrayList<Rc<TileEntity>>>,
}

fn identitySet(entities: &[Rc<TileEntity>]) -> Vec<Rc<TileEntity>>
{
    let mut set: Vec<Rc<TileEntity>> = Vec::new();
    for entity in entities
    {
        if !set.iter().any(|existing| Rc::ptr_eq(existing, entity))
        {
            set.push(Rc::clone(entity));
        }
    }
    set
}

impl WorldRenderer
{
    pub fn new(
        world: Option<Rc<World>>,
        blockEntitiesToBeRendered: &Rc<JArrayList<Rc<TileEntity>>>,
        originX: i32,
        originY: i32,
        originZ: i32,
        size: i32,
        glList: i32,
    ) -> WorldRenderer
    {
        let renderer: WorldRenderer = WorldRenderer {
            m_worldObj: Field::new(world),
            m_glRenderList: glList,
            m_posX: Cell::new(-999),
            m_posY: Cell::new(0),
            m_posZ: Cell::new(0),
            m_sizeWidth: size,
            m_sizeHeight: size,
            m_sizeDepth: size,
            m_posXMinus: Cell::new(0),
            m_posYMinus: Cell::new(0),
            m_posZMinus: Cell::new(0),
            m_posXClip: Cell::new(0),
            m_posYClip: Cell::new(0),
            m_posZClip: Cell::new(0),
            m_isInFrustum: Cell::new(false),
            m_skipRenderPass: [Cell::new(false), Cell::new(false)],
            m_posXPlus: Cell::new(0),
            m_posYPlus: Cell::new(0),
            m_posZPlus: Cell::new(0),
            m_rendererRadius: math_helper::sqrt_float((size * size + size * size + size * size) as f32) / 2.0,
            m_needsUpdate: Cell::new(false),
            m_rendererBoundingBox: Field::new(None),
            m_chunkIndex: Cell::new(0),
            m_isVisible: Cell::new(true),
            m_isWaitingOnOcclusionQuery: Cell::new(false),
            m_glOcclusionQuery: Cell::new(0),
            m_isChunkLit: Cell::new(false),
            m_isInitialized: Cell::new(false),
            m_tileEntityRenderers: JArrayList::new(),
            m_tileEntities: Rc::clone(blockEntitiesToBeRendered),
        };
        renderer.setPosition(originX, originY, originZ);
        renderer.m_needsUpdate.set(false);
        renderer
    }

    pub fn setPosition(&self, x: i32, y: i32, z: i32)
    {
        if x == self.m_posX.get() && y == self.m_posY.get() && z == self.m_posZ.get()
        {
            return;
        }
        self.setDontDraw();
        self.m_posX.set(x);
        self.m_posY.set(y);
        self.m_posZ.set(z);
        self.m_posXPlus.set(x + self.m_sizeWidth / 2);
        self.m_posYPlus.set(y + self.m_sizeHeight / 2);
        self.m_posZPlus.set(z + self.m_sizeDepth / 2);
        self.m_posXClip.set(x & 0x3FF);
        self.m_posYClip.set(y);
        self.m_posZClip.set(z & 0x3FF);
        self.m_posXMinus.set(x - self.m_posXClip.get());
        self.m_posYMinus.set(y - self.m_posYClip.get());
        self.m_posZMinus.set(z - self.m_posZClip.get());
        let f: f32 = 6.0;
        self.m_rendererBoundingBox.set(Some(AxisAlignedBB::getBoundingBox(
            (x as f32 - f) as f64,
            (y as f32 - f) as f64,
            (z as f32 - f) as f64,
            ((x + self.m_sizeWidth) as f32 + f) as f64,
            ((y + self.m_sizeHeight) as f32 + f) as f64,
            ((z + self.m_sizeDepth) as f32 + f) as f64,
        )));
        GL11::glNewList(self.m_glRenderList + 2, 4864);
        RenderItem::renderAABB(&AxisAlignedBB::getBoundingBoxFromPool(
            (self.m_posXClip.get() as f32 - f) as f64,
            (self.m_posYClip.get() as f32 - f) as f64,
            (self.m_posZClip.get() as f32 - f) as f64,
            ((self.m_posXClip.get() + self.m_sizeWidth) as f32 + f) as f64,
            ((self.m_posYClip.get() + self.m_sizeHeight) as f32 + f) as f64,
            ((self.m_posZClip.get() + self.m_sizeDepth) as f32 + f) as f64,
        ));
        GL11::glEndList();
        self.markDirty();
    }

    fn setupGLTranslation(&self)
    {
        GL11::glTranslatef(self.m_posXClip.get() as f32, self.m_posYClip.get() as f32, self.m_posZClip.get() as f32);
    }

    pub fn updateRenderer(&self)
    {
        if !self.m_needsUpdate.get()
        {
            return;
        }
        CHUNKS_UPDATED.fetch_add(1, Ordering::Relaxed);
        let n: i32 = self.m_posX.get();
        let n2: i32 = self.m_posY.get();
        let n3: i32 = self.m_posZ.get();
        let n4: i32 = self.m_posX.get() + self.m_sizeWidth;
        let n5: i32 = self.m_posY.get() + self.m_sizeHeight;
        let n6: i32 = self.m_posZ.get() + self.m_sizeDepth;
        for pass in &self.m_skipRenderPass
        {
            pass.set(true);
        }
        IS_LIT.with(|lit| lit.set(false));
        let mut hashSet: Vec<Rc<TileEntity>> = identitySet(&self.m_tileEntityRenderers.toVec());
        self.m_tileEntityRenderers.clear();
        let n7: i32 = 1;
        let chunkCache: Rc<dyn IBlockAccess> =
            ChunkCache::new(&self.m_worldObj.get().orNpe(), n - n7, n2 - n7, n3 - n7, n4 + n7, n5 + n7, n6 + n7);
        let mut renderBlocks: RenderBlocks = RenderBlocks::new(Rc::clone(&chunkCache));
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        for i in 0..2
        {
            let mut bl: bool = false;
            let mut bl2: bool = false;
            let mut bl3: bool = false;
            for j in n2..n5
            {
                for k in n3..n6
                {
                    for i2 in n..n4
                    {
                        let n9: i32 = chunkCache.getBlockId(i2, j, k);
                        if n9 <= 0
                        {
                            continue;
                        }
                        if !bl3
                        {
                            bl3 = true;
                            GL11::glNewList(self.m_glRenderList + i, 4864);
                            GL11::glPushMatrix();
                            self.setupGLTranslation();
                            let f: f32 = 1.000001;
                            GL11::glTranslatef(
                                -self.m_sizeDepth as f32 / 2.0,
                                -self.m_sizeHeight as f32 / 2.0,
                                -self.m_sizeDepth as f32 / 2.0,
                            );
                            GL11::glScalef(f, f, f);
                            GL11::glTranslatef(
                                self.m_sizeDepth as f32 / 2.0,
                                self.m_sizeHeight as f32 / 2.0,
                                self.m_sizeDepth as f32 / 2.0,
                            );
                            tessellator.startDrawingQuads();
                            tessellator.setTranslationD(
                                -self.m_posX.get() as f64,
                                -self.m_posY.get() as f64,
                                -self.m_posZ.get() as f64,
                            );
                        }
                        if i == 0 && Block::isBlockContainer(n9)
                        {
                            let tileEntity: Option<Rc<TileEntity>> = chunkCache.getBlockTileEntity(i2, j, k);
                            if TileEntityRenderer::instance().hasSpecialRenderer(tileEntity.as_ref())
                            {
                                self.m_tileEntityRenderers.add(tileEntity.orNpe());
                            }
                        }
                        let block: Rc<Block> = Block::byId(n9).orNpe();
                        let n8: i32 = block.getRenderBlockPass();
                        if n8 != i
                        {
                            bl = true;
                            continue;
                        }
                        bl2 |= renderBlocks.renderBlockByRenderType(&block, i2, j, k);
                    }
                }
            }
            if bl3
            {
                tessellator.draw();
                GL11::glPopMatrix();
                GL11::glEndList();
                tessellator.setTranslationD(0.0, 0.0, 0.0);
            }
            else
            {
                bl2 = false;
            }
            if bl2
            {
                self.m_skipRenderPass[i as usize].set(false);
            }
            if !bl
            {
                break;
            }
        }
        let renderers: Vec<Rc<TileEntity>> = self.m_tileEntityRenderers.toVec();
        let hashSet2: Vec<Rc<TileEntity>> = identitySet(&renderers)
            .into_iter()
            .filter(|entity| !hashSet.iter().any(|old| Rc::ptr_eq(old, entity)))
            .collect();
        self.m_tileEntities.addAll(&hashSet2);
        hashSet.retain(|old| !renderers.iter().any(|entity| Rc::ptr_eq(old, entity)));
        for removed in &hashSet
        {
            while self.m_tileEntities.removeIdentity(removed)
            {}
        }
        self.m_isChunkLit.set(IS_LIT.with(Cell::get));
        self.m_isInitialized.set(true);
    }

    pub fn distanceToEntitySquared(&self, camera: &Entity) -> f32
    {
        let f: f32 = (camera.posX() - self.m_posXPlus.get() as f64) as f32;
        let f2: f32 = (camera.posY() - self.m_posYPlus.get() as f64) as f32;
        let f3: f32 = (camera.posZ() - self.m_posZPlus.get() as f64) as f32;
        f * f + f2 * f2 + f3 * f3
    }

    pub fn setDontDraw(&self)
    {
        for pass in &self.m_skipRenderPass
        {
            pass.set(true);
        }
        self.m_isInFrustum.set(false);
        self.m_isInitialized.set(false);
    }

    pub fn delete(&self)
    {
        self.setDontDraw();
        self.m_worldObj.set(None);
    }

    pub fn getGLCallListForPass(&self, layer: i32) -> i32
    {
        if !self.m_isInFrustum.get()
        {
            return -1;
        }
        if !self.m_skipRenderPass[layer as usize].get()
        {
            return self.m_glRenderList + layer;
        }
        -1
    }

    pub fn updateInFrustrum(&self, culler: &dyn ICamera)
    {
        self.m_isInFrustum.set(culler.isBoundingBoxInFrustum(&self.m_rendererBoundingBox.get().orNpe()));
    }

    pub fn callOcclusionQueryList(&self) { GL11::glCallList(self.m_glRenderList + 2); }

    pub fn skipAllRenderPasses(&self) -> bool
    {
        if !self.m_isInitialized.get()
        {
            return false;
        }
        self.m_skipRenderPass[0].get() && self.m_skipRenderPass[1].get()
    }

    pub fn markDirty(&self) { self.m_needsUpdate.set(true); }
}
