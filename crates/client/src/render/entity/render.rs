use crate::gui::FontRenderer;
use crate::model::ModelBase;
use crate::render::{RenderEngine, RenderManager, Tessellator};
use game::block::Block;
use game::entity::Entity;
use game::math::{AxisAlignedBB, math_helper};
// abstract java methods throw through port
use game::port::abstractMethod;
use game::world::World;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

game::vtable! {
    RenderClass for Render {
        fn doRender(entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64, yaw: f32, tickDelta: f32);
        fn applyTranslation(entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64);
        fn rotateCorpse(entity: &Rc<Entity>, bob: f32, bodyYaw: f32, tickDelta: f32);
        fn getAttackAnimationProgress(entity: &Rc<Entity>, tickDelta: f32) -> f32;
        fn getBob(entity: &Rc<Entity>, tickDelta: f32) -> f32;
        fn renderEquippedItems(entity: &Rc<Entity>, tickDelta: f32);
        fn bindDecorationTexture(entity: &Rc<Entity>, layer: i32, tickDelta: f32) -> bool;
        fn shouldRenderPass(entity: &Rc<Entity>, layer: i32, tickDelta: f32) -> bool;
        fn getDeathMaxRotation(entity: &Rc<Entity>) -> f32;
        fn getColorMultiplier(entity: &Rc<Entity>, brightness: f32, tickDelta: f32) -> i32;
        fn preRenderCallback(entity: &Rc<Entity>, tickDelta: f32);
        fn passSpecialRender(entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64);
    }
}

pub struct LivingRenderFields
{
    pub m_mainModel: Rc<dyn ModelBase>,
    pub m_renderPassModel: Field<Option<Rc<dyn ModelBase>>>,
}

pub struct Render
{
    pub m_class: &'static RenderClass,
    pub m_shadowSize: Cell<f32>,
    pub m_shadowDarkness: Cell<f32>,
    pub m_living: Option<LivingRenderFields>,
    pub m_ext: Box<dyn Any>,
}

const SHADOW_LIFT: f64 = 0.015625;
const FIRE_FRAME: f32 = 15.99;

impl Render
{
    pub fn alloc(class: &'static RenderClass, living: Option<LivingRenderFields>, ext: Box<dyn Any>) -> Rc<Render>
    {
        Rc::new(Render {
            m_class: class,
            m_shadowSize: Cell::new(0.0),
            m_shadowDarkness: Cell::new(1.0),
            m_living: living,
            m_ext: ext,
        })
    }

    pub fn ext<T: 'static>(&self) -> &T { self.m_ext.downcast_ref::<T>().expect("render ext matches its class") }

    pub fn living(&self) -> &LivingRenderFields
    {
        self.m_living.as_ref().expect("RenderLiving fields on a living renderer")
    }

    pub fn renderManager(&self) -> Rc<RenderManager> { RenderManager::instance() }

    fn renderEngine(&self) -> Rc<RenderEngine> { self.renderManager().m_renderEngine.get().orNpe() }

    pub fn loadTexture(&self, path: &str)
    {
        let renderengine: Rc<RenderEngine> = self.renderEngine();
        renderengine.bindTexture(renderengine.getTexture(path));
    }

    pub fn loadDownloadableImageTexture(&self, url: Option<&str>, backup: Option<&str>) -> bool
    {
        let renderengine: Rc<RenderEngine> = self.renderEngine();
        let i: i32 = match url
        {
            Some(url) => renderengine.getTextureForDownloadableImage(url, backup),
            None => backup.map_or(-1, |backup| renderengine.getTexture(backup)),
        };
        if i >= 0
        {
            renderengine.bindTexture(i);
            return true;
        }
        false
    }

    fn renderEntityOnFire(&self, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64)
    {
        GL11::glDisable(GL11::GL_LIGHTING);
        let i: i32 = Block::fire().m_blockIndexInTexture.get();
        let j: i32 = (i & 15) << 4;
        let k: i32 = i & 240;
        GL11::glPushMatrix();
        GL11::glTranslatef(dx as f32, dy as f32, dz as f32);
        let f4: f32 = entity.m_width.get() * 1.4;
        GL11::glScalef(f4, f4, f4);
        self.loadTexture("/terrain.png");
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let mut f5: f32 = 0.5;
        let f6: f32 = 0.0;
        let mut f7: f32 = entity.m_height.get() / f4;
        let mut f8: f32 = (entity.posY() - entity.m_boundingBox.minY()) as f32;
        GL11::glRotatef(-self.renderManager().m_playerViewY.get(), 0.0, 1.0, 0.0);
        GL11::glTranslatef(0.0, 0.0, -0.3 + f7 as i32 as f32 * 0.02);
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        let mut f9: f32 = 0.0;
        let mut l: i32 = 0;
        tessellator.startDrawingQuads();
        while f7 > 0.0
        {
            let row: i32 = if l % 2 == 0 { k } else { k + 16 };
            let mut f: f32 = j as f32 / 256.0;
            let mut f1: f32 = (j as f32 + FIRE_FRAME) / 256.0;
            let f2: f32 = row as f32 / 256.0;
            let f3: f32 = (row as f32 + FIRE_FRAME) / 256.0;
            if l / 2 % 2 == 0
            {
                std::mem::swap(&mut f, &mut f1);
            }
            tessellator.addVertexWithUV((f5 - f6) as f64, (0.0 - f8) as f64, f9 as f64, f1 as f64, f3 as f64);
            tessellator.addVertexWithUV((-f5 - f6) as f64, (0.0 - f8) as f64, f9 as f64, f as f64, f3 as f64);
            tessellator.addVertexWithUV((-f5 - f6) as f64, (1.4 - f8) as f64, f9 as f64, f as f64, f2 as f64);
            tessellator.addVertexWithUV((f5 - f6) as f64, (1.4 - f8) as f64, f9 as f64, f1 as f64, f2 as f64);
            f7 -= 0.45;
            f8 -= 0.45;
            f5 *= 0.9;
            f9 += 0.03;
            l += 1;
        }
        tessellator.draw();
        GL11::glPopMatrix();
        GL11::glEnable(GL11::GL_LIGHTING);
    }

    fn renderShadow(&self, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64, yaw: f32, tickDelta: f32)
    {
        GL11::glEnable(GL11::GL_BLEND);
        GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
        let renderengine: Rc<RenderEngine> = self.renderEngine();
        renderengine.bindTexture(renderengine.getTexture("%clamp%/misc/shadow.png"));
        let world: Rc<World> = self.renderManager().m_worldObj.get().orNpe();
        GL11::glDepthMask(false);
        let f: f32 = self.m_shadowSize.get();
        let tick: f64 = tickDelta as f64;
        let d0: f64 = entity.m_lastTickPosX.get() + (entity.posX() - entity.m_lastTickPosX.get()) * tick;
        let d1: f64 = entity.m_lastTickPosY.get()
            + (entity.posY() - entity.m_lastTickPosY.get()) * tick
            + entity.getShadowSize() as f64;
        let d2: f64 = entity.m_lastTickPosZ.get() + (entity.posZ() - entity.m_lastTickPosZ.get()) * tick;
        let i: i32 = math_helper::floor_double(d0 - f as f64);
        let j: i32 = math_helper::floor_double(d0 + f as f64);
        let k: i32 = math_helper::floor_double(d1 - f as f64);
        let l: i32 = math_helper::floor_double(d1);
        let i1: i32 = math_helper::floor_double(d2 - f as f64);
        let j1: i32 = math_helper::floor_double(d2 + f as f64);
        let d3: f64 = dx - d0;
        let d4: f64 = dy - d1;
        let d5: f64 = dz - d2;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        for k1 in i..=j
        {
            for l1 in k..=l
            {
                for i2 in i1..=j1
                {
                    let j2: i32 = world.getBlockId(k1, l1 - 1, i2);
                    if j2 > 0 && world.getBlockLightValue(k1, l1, i2) > 3
                    {
                        let shadow: f64 = entity.getShadowSize() as f64;
                        self.renderShadowOnBlock(
                            &world,
                            &Block::byId(j2).orNpe(),
                            dx,
                            dy + shadow,
                            dz,
                            (k1, l1, i2),
                            yaw,
                            f,
                            (d3, d4 + shadow, d5),
                        );
                    }
                }
            }
        }
        tessellator.draw();
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glDisable(GL11::GL_BLEND);
        GL11::glDepthMask(true);
    }

    fn renderShadowOnBlock(
        &self,
        world: &World,
        block: &Rc<Block>,
        dx: f64,
        dy: f64,
        dz: f64,
        (x, y, z): (i32, i32, i32),
        yaw: f32,
        shadowSize: f32,
        (cx, cy, cz): (f64, f64, f64),
    )
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        if !block.renderAsNormalBlock()
        {
            return;
        }
        let mut d: f64 = (yaw as f64 - (dy - (y as f64 + cy)) / 2.0) * 0.5 * world.getLightBrightness(x, y, z) as f64;
        if d < 0.0
        {
            return;
        }
        if d > 1.0
        {
            d = 1.0;
        }
        tessellator.setColorRGBA_F(1.0, 1.0, 1.0, d as f32);
        let d2: f64 = x as f64 + block.m_minX.get() + cx;
        let d3: f64 = x as f64 + block.m_maxX.get() + cx;
        let d4: f64 = y as f64 + block.m_minY.get() + cy + SHADOW_LIFT;
        let d5: f64 = z as f64 + block.m_minZ.get() + cz;
        let d6: f64 = z as f64 + block.m_maxZ.get() + cz;
        let f: f32 = ((dx - d2) / 2.0 / shadowSize as f64 + 0.5) as f32;
        let f2: f32 = ((dx - d3) / 2.0 / shadowSize as f64 + 0.5) as f32;
        let f3: f32 = ((dz - d5) / 2.0 / shadowSize as f64 + 0.5) as f32;
        let f4: f32 = ((dz - d6) / 2.0 / shadowSize as f64 + 0.5) as f32;
        tessellator.addVertexWithUV(d2, d4, d5, f as f64, f3 as f64);
        tessellator.addVertexWithUV(d2, d4, d6, f as f64, f4 as f64);
        tessellator.addVertexWithUV(d3, d4, d6, f2 as f64, f4 as f64);
        tessellator.addVertexWithUV(d3, d4, d5, f2 as f64, f3 as f64);
    }

    pub fn renderOffsetAABB(shape: &AxisAlignedBB, dx: f64, dy: f64, dz: f64)
    {
        GL11::glDisable(GL11::GL_TEXTURE_2D);
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        tessellator.startDrawingQuads();
        tessellator.setTranslationD(dx, dy, dz);
        let normals: [(f32, f32, f32); 6] =
            [(0.0, 0.0, -1.0), (0.0, 0.0, 1.0), (0.0, -1.0, 0.0), (0.0, 1.0, 0.0), (-1.0, 0.0, 0.0), (1.0, 0.0, 0.0)];
        for (face, (nx, ny, nz)) in boxFaces(shape).iter().zip(normals)
        {
            tessellator.setNormal(nx, ny, nz);
            for &(x, y, z) in face
            {
                tessellator.addVertex(x, y, z);
            }
        }
        tessellator.setTranslationD(0.0, 0.0, 0.0);
        tessellator.draw();
        GL11::glEnable(GL11::GL_TEXTURE_2D);
    }

    pub fn renderAABB(shape: &AxisAlignedBB)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        for face in boxFaces(shape)
        {
            for (x, y, z) in face
            {
                tessellator.addVertex(x, y, z);
            }
        }
        tessellator.draw();
    }

    pub fn doRenderShadowAndFire(&self, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64, _yaw: f32, tickDelta: f32)
    {
        let manager: Rc<RenderManager> = self.renderManager();
        if manager.m_options.get().orNpe().m_fancyGraphics.get() && self.m_shadowSize.get() > 0.0
        {
            let d0: f64 = manager.squaredDistanceToCamera(entity.posX(), entity.posY(), entity.posZ());
            let f: f32 = ((1.0 - d0 / 256.0) * self.m_shadowDarkness.get() as f64) as f32;
            if f > 0.0
            {
                self.renderShadow(entity, dx, dy, dz, f, tickDelta);
            }
        }
        if entity.isBurning()
        {
            self.renderEntityOnFire(entity, dx, dy, dz);
        }
    }

    pub fn getFontRendererFromRenderManager(&self) -> Option<Rc<FontRenderer>>
    {
        self.renderManager().getFontRenderer()
    }
}

fn boxFaces(shape: &AxisAlignedBB) -> [[(f64, f64, f64); 4]; 6]
{
    let (x0, y0, z0) = (shape.minX(), shape.minY(), shape.minZ());
    let (x1, y1, z1) = (shape.maxX(), shape.maxY(), shape.maxZ());
    [
        [(x0, y1, z0), (x1, y1, z0), (x1, y0, z0), (x0, y0, z0)],
        [(x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)],
        [(x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)],
        [(x0, y1, z1), (x1, y1, z1), (x1, y1, z0), (x0, y1, z0)],
        [(x0, y0, z1), (x0, y1, z1), (x0, y1, z0), (x0, y0, z0)],
        [(x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1)],
    ]
}

pub const RENDER: RenderClass = RenderClass {
    m_name: "Render",
    m_parent: None,
    m_interfaces: &[],
    doRender: |_, _, _, _, _, _, _| abstractMethod("doRender"),
    applyTranslation: |_, _, _, _, _| abstractMethod("applyTranslation"),
    rotateCorpse: |_, _, _, _, _| abstractMethod("rotateCorpse"),
    getAttackAnimationProgress: |_, _, _| abstractMethod("getAttackAnimationProgress"),
    getBob: |_, _, _| abstractMethod("getBob"),
    renderEquippedItems: |_, _, _| abstractMethod("renderEquippedItems"),
    bindDecorationTexture: |_, _, _, _| abstractMethod("bindDecorationTexture"),
    shouldRenderPass: |_, _, _, _| abstractMethod("shouldRenderPass"),
    getDeathMaxRotation: |_, _| abstractMethod("getDeathMaxRotation"),
    getColorMultiplier: |_, _, _, _| abstractMethod("getColorMultiplier"),
    preRenderCallback: |_, _, _| abstractMethod("preRenderCallback"),
    passSpecialRender: |_, _, _, _, _| abstractMethod("passSpecialRender"),
};
