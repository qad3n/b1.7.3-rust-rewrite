use crate::gui::FontRenderer;
use crate::render::RenderEngine;
use crate::render::tile::{
    TileEntityMobSpawnerRenderer, TileEntityRendererPiston, TileEntitySignRenderer, TileEntitySpecialRenderer,
};
use game::entity::Entity;
use game::tileentity::{TileEntity, TileEntityClass};
use game::world::World;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static INSTANCE: Rc<TileEntityRenderer> = Rc::new(TileEntityRenderer::new());
    static STATIC_PLAYER_POS: Cell<(f64, f64, f64)> = const { Cell::new((0.0, 0.0, 0.0)) };
}

const ROOT_CLASS: &str = "TileEntity";
const RENDER_DISTANCE_SQ: f64 = 4096.0;

pub struct TileEntityRenderer
{
    m_specialRendererMap: RefCell<HashMap<&'static str, Option<Rc<dyn TileEntitySpecialRenderer>>>>,
    m_fontRenderer: Field<Option<Rc<FontRenderer>>>,
    pub m_renderEngine: Field<Option<Rc<RenderEngine>>>,
    pub m_worldObj: Field<Option<Rc<World>>>,
    pub m_entityLivingPlayer: Field<Option<Rc<Entity>>>,
    pub m_playerYaw: Cell<f32>,
    pub m_playerPitch: Cell<f32>,
    pub m_playerX: Cell<f64>,
    pub m_playerY: Cell<f64>,
    pub m_playerZ: Cell<f64>,
}

impl TileEntityRenderer
{
    fn new() -> TileEntityRenderer
    {
        let renderers: [(&'static str, Rc<dyn TileEntitySpecialRenderer>); 3] = [
            ("TileEntitySign", Rc::new(TileEntitySignRenderer::new())),
            ("TileEntityMobSpawner", Rc::new(TileEntityMobSpawnerRenderer::new())),
            ("TileEntityPiston", Rc::new(TileEntityRendererPiston::new())),
        ];
        TileEntityRenderer {
            m_specialRendererMap: RefCell::new(
                renderers.into_iter().map(|(name, renderer)| (name, Some(renderer))).collect(),
            ),
            m_fontRenderer: Field::new(None),
            m_renderEngine: Field::new(None),
            m_worldObj: Field::new(None),
            m_entityLivingPlayer: Field::new(None),
            m_playerYaw: Cell::new(0.0),
            m_playerPitch: Cell::new(0.0),
            m_playerX: Cell::new(0.0),
            m_playerY: Cell::new(0.0),
            m_playerZ: Cell::new(0.0),
        }
    }

    pub fn instance() -> Rc<TileEntityRenderer> { INSTANCE.with(Rc::clone) }

    pub fn staticPlayerPos() -> (f64, f64, f64) { STATIC_PLAYER_POS.with(Cell::get) }

    pub fn setStaticPlayerPos(x: f64, y: f64, z: f64) { STATIC_PLAYER_POS.with(|pos| pos.set((x, y, z))); }

    pub fn getSpecialRendererForClass(
        &self,
        class: &'static TileEntityClass,
    ) -> Option<Rc<dyn TileEntitySpecialRenderer>>
    {
        if let Some(renderer) = self.m_specialRendererMap.borrow().get(class.m_name)
        {
            return renderer.clone();
        }
        if class.m_name == ROOT_CLASS
        {
            return None;
        }
        let renderer: Option<Rc<dyn TileEntitySpecialRenderer>> =
            class.m_parent.and_then(|parent| self.getSpecialRendererForClass(parent));
        self.m_specialRendererMap.borrow_mut().insert(class.m_name, renderer.clone());
        renderer
    }

    pub fn hasSpecialRenderer(&self, tileEntity: Option<&Rc<TileEntity>>) -> bool
    {
        self.getSpecialRendererForEntity(tileEntity).is_some()
    }

    pub fn getSpecialRendererForEntity(
        &self,
        tileEntity: Option<&Rc<TileEntity>>,
    ) -> Option<Rc<dyn TileEntitySpecialRenderer>>
    {
        tileEntity.and_then(|tileEntity| self.getSpecialRendererForClass(tileEntity.m_class))
    }

    pub fn cacheActiveRenderInfo(
        &self,
        world: &Rc<World>,
        textureManager: &Rc<RenderEngine>,
        fontRenderer: Option<Rc<FontRenderer>>,
        camera: Option<Rc<Entity>>,
        tickDelta: f32,
    )
    {
        if !self.m_worldObj.with(|current| current.as_ref().is_some_and(|current| Rc::ptr_eq(current, world)))
        {
            self.setWorld(Some(Rc::clone(world)));
        }
        self.m_renderEngine.set(Some(Rc::clone(textureManager)));
        self.m_entityLivingPlayer.set(camera.clone());
        self.m_fontRenderer.set(fontRenderer);
        let camera: Rc<Entity> = camera.orNpe();
        let lerp = |previous: f32, current: f32| -> f32 { previous + (current - previous) * tickDelta };
        self.m_playerYaw.set(lerp(camera.m_prevRotationYaw.get(), camera.m_rotationYaw.get()));
        self.m_playerPitch.set(lerp(camera.m_prevRotationPitch.get(), camera.m_rotationPitch.get()));
        let tick: f64 = tickDelta as f64;
        self.m_playerX.set(camera.m_lastTickPosX.get() + (camera.posX() - camera.m_lastTickPosX.get()) * tick);
        self.m_playerY.set(camera.m_lastTickPosY.get() + (camera.posY() - camera.m_lastTickPosY.get()) * tick);
        self.m_playerZ.set(camera.m_lastTickPosZ.get() + (camera.posZ() - camera.m_lastTickPosZ.get()) * tick);
    }

    pub fn renderTileEntity(&self, tileEntity: &Rc<TileEntity>, tickDelta: f32)
    {
        if tileEntity.getDistanceFrom(self.m_playerX.get(), self.m_playerY.get(), self.m_playerZ.get())
            < RENDER_DISTANCE_SQ
        {
            let (x, y, z): (i32, i32, i32) =
                (tileEntity.m_xCoord.get(), tileEntity.m_yCoord.get(), tileEntity.m_zCoord.get());
            let f: f32 = self.m_worldObj.get().orNpe().getLightBrightness(x, y, z);
            GL11::glColor3f(f, f, f);
            let (px, py, pz): (f64, f64, f64) = Self::staticPlayerPos();
            self.renderTileEntityAt(tileEntity, x as f64 - px, y as f64 - py, z as f64 - pz, tickDelta);
        }
    }

    pub fn renderTileEntityAt(&self, tileEntity: &Rc<TileEntity>, dx: f64, dy: f64, dz: f64, tickDelta: f32)
    {
        if let Some(renderer) = self.getSpecialRendererForEntity(Some(tileEntity))
        {
            renderer.renderTileEntityAt(tileEntity, dx, dy, dz, tickDelta);
        }
    }

    pub fn setWorld(&self, world: Option<Rc<World>>)
    {
        self.m_worldObj.set(world.clone());
        let renderers: Vec<Rc<dyn TileEntitySpecialRenderer>> =
            self.m_specialRendererMap.borrow().values().flatten().cloned().collect();
        for renderer in renderers
        {
            renderer.setWorld(world.clone());
        }
    }

    pub fn getFontRenderer(&self) -> Option<Rc<FontRenderer>> { self.m_fontRenderer.get() }
}
