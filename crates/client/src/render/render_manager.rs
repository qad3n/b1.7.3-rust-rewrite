use crate::gui::FontRenderer;
use crate::model::{
    ModelBiped, ModelChicken, ModelCow, ModelPig, ModelSheep1, ModelSheep2, ModelSkeleton, ModelSlime, ModelSquid,
    ModelWolf, ModelZombie,
};
use crate::render::entity::{Render, render_biped, render_living, render_mobs, render_objects, render_player};
use crate::render::{ItemRenderer, RenderEngine, RenderItem};
use crate::settings::GameSettings;
use game::block::Block;
use game::entity::{Entity, EntityClass};
use game::item::Item;
use game::math::math_helper;
use game::world::World;
use jvm::field::Field;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static INSTANCE: Rc<RenderManager> = Rc::new(RenderManager::new());
    static RENDER_POS: Cell<(f64, f64, f64)> = const { Cell::new((0.0, 0.0, 0.0)) };
}

const ROOT_CLASS: &str = "Entity";

pub struct RenderManager
{
    m_entityRenderMap: RefCell<HashMap<&'static str, Option<Rc<Render>>>>,
    m_fontRenderer: Field<Option<Rc<FontRenderer>>>,
    pub m_renderEngine: Field<Option<Rc<RenderEngine>>>,
    pub m_itemRenderer: Field<Option<Rc<ItemRenderer>>>,
    pub m_worldObj: Field<Option<Rc<World>>>,
    pub m_livingPlayer: Field<Option<Rc<Entity>>>,
    pub m_playerViewY: Cell<f32>,
    pub m_playerViewX: Cell<f32>,
    pub m_options: Field<Option<Rc<GameSettings>>>,
    pub m_cameraX: Cell<f64>,
    pub m_cameraY: Cell<f64>,
    pub m_cameraZ: Cell<f64>,
}

impl RenderManager
{
    fn new() -> RenderManager
    {
        let renders: Vec<(&'static str, Rc<Render>)> = vec![
            ("EntitySpider", render_mobs::newSpider()),
            ("EntityPig", render_mobs::newPig(Rc::new(ModelPig::new(0.0)), Rc::new(ModelPig::new(0.5)), 0.7)),
            ("EntitySheep", render_mobs::newSheep(Rc::new(ModelSheep2::new()), Rc::new(ModelSheep1::new()), 0.7)),
            ("EntityCow", render_mobs::newCow(Rc::new(ModelCow::new()), 0.7)),
            ("EntityWolf", render_mobs::newWolf(Rc::new(ModelWolf::new()), 0.5)),
            ("EntityChicken", render_mobs::newChicken(Rc::new(ModelChicken::new()), 0.3)),
            ("EntityCreeper", render_mobs::newCreeper()),
            ("EntitySkeleton", render_biped::new(Rc::new(ModelSkeleton::new()), 0.5)),
            ("EntityZombie", render_biped::new(Rc::new(ModelZombie::new()), 0.5)),
            ("EntitySlime", render_mobs::newSlime(Rc::new(ModelSlime::new(16)), Rc::new(ModelSlime::new(0)), 0.25)),
            ("EntityPlayer", render_player::new()),
            ("EntityGiantZombie", render_mobs::newGiantZombie(Rc::new(ModelZombie::new()), 0.5, 6.0)),
            ("EntityGhast", render_mobs::newGhast()),
            ("EntitySquid", render_mobs::newSquid(Rc::new(ModelSquid::new()), 0.7)),
            ("EntityLiving", render_living::new(Rc::new(ModelBiped::new(0.0, 0.0)), 0.5)),
            (ROOT_CLASS, render_objects::newEntity()),
            ("EntityPainting", render_objects::newPainting()),
            ("EntityArrow", render_objects::newArrow()),
            ("EntitySnowball", render_objects::newSnowball(Item::snowball().getIconFromDamage(0))),
            ("EntityEgg", render_objects::newSnowball(Item::egg().getIconFromDamage(0))),
            ("EntityFireball", render_objects::newFireball()),
            ("EntityItem", RenderItem::createRender()),
            ("EntityTNTPrimed", render_objects::newTNTPrimed()),
            ("EntityFallingSand", render_objects::newFallingSand()),
            ("EntityMinecart", render_objects::newMinecart()),
            ("EntityBoat", render_objects::newBoat()),
            ("EntityFish", render_objects::newFish()),
            ("EntityLightningBolt", render_objects::newLightningBolt()),
        ];
        RenderManager {
            m_entityRenderMap: RefCell::new(renders.into_iter().map(|(name, render)| (name, Some(render))).collect()),
            m_fontRenderer: Field::new(None),
            m_renderEngine: Field::new(None),
            m_itemRenderer: Field::new(None),
            m_worldObj: Field::new(None),
            m_livingPlayer: Field::new(None),
            m_playerViewY: Cell::new(0.0),
            m_playerViewX: Cell::new(0.0),
            m_options: Field::new(None),
            m_cameraX: Cell::new(0.0),
            m_cameraY: Cell::new(0.0),
            m_cameraZ: Cell::new(0.0),
        }
    }

    pub fn instance() -> Rc<RenderManager> { INSTANCE.with(Rc::clone) }

    pub fn renderPos() -> (f64, f64, f64) { RENDER_POS.with(Cell::get) }

    pub fn setRenderPos(x: f64, y: f64, z: f64) { RENDER_POS.with(|pos| pos.set((x, y, z))); }

    pub fn getEntityClassRenderObject(&self, class: &'static EntityClass) -> Option<Rc<Render>>
    {
        if let Some(render) = self.m_entityRenderMap.borrow().get(class.m_name)
        {
            return render.clone();
        }
        if class.m_name == ROOT_CLASS
        {
            return None;
        }
        let render: Option<Rc<Render>> = class.m_parent.and_then(|parent| self.getEntityClassRenderObject(parent));
        self.m_entityRenderMap.borrow_mut().insert(class.m_name, render.clone());
        render
    }

    pub fn getEntityRenderObject(&self, entity: &Entity) -> Option<Rc<Render>>
    {
        self.getEntityClassRenderObject(entity.m_class)
    }

    pub fn cacheActiveRenderInfo(
        &self,
        world: &Rc<World>,
        textureManager: &Rc<RenderEngine>,
        fontRenderer: Option<Rc<FontRenderer>>,
        camera: Option<Rc<Entity>>,
        settings: &Rc<GameSettings>,
        tickDelta: f32,
    )
    {
        self.m_worldObj.set(Some(Rc::clone(world)));
        self.m_renderEngine.set(Some(Rc::clone(textureManager)));
        self.m_options.set(Some(Rc::clone(settings)));
        self.m_livingPlayer.set(camera.clone());
        self.m_fontRenderer.set(fontRenderer);
        let camera: Rc<Entity> = jvm::throwable::OrNpe::orNpe(camera);
        let x: i32 = math_helper::floor_double(camera.posX());
        let y: i32 = math_helper::floor_double(camera.posY());
        let z: i32 = math_helper::floor_double(camera.posZ());
        if camera.isPlayerSleeping()
        {
            if world.getBlockId(x, y, z) == Block::blockBed().m_blockID
            {
                let k: i32 = world.getBlockMetadata(x, y, z) & 3;
                self.m_playerViewY.set((k * 90 + 180) as f32);
                self.m_playerViewX.set(0.0);
            }
        }
        else
        {
            let lerp = |previous: f32, current: f32| -> f32 { previous + (current - previous) * tickDelta };
            self.m_playerViewY.set(lerp(camera.m_prevRotationYaw.get(), camera.m_rotationYaw.get()));
            self.m_playerViewX.set(lerp(camera.m_prevRotationPitch.get(), camera.m_rotationPitch.get()));
        }
        let tick: f64 = tickDelta as f64;
        self.m_cameraX.set(camera.m_lastTickPosX.get() + (camera.posX() - camera.m_lastTickPosX.get()) * tick);
        self.m_cameraY.set(camera.m_lastTickPosY.get() + (camera.posY() - camera.m_lastTickPosY.get()) * tick);
        self.m_cameraZ.set(camera.m_lastTickPosZ.get() + (camera.posZ() - camera.m_lastTickPosZ.get()) * tick);
    }

    pub fn renderEntity(&self, entity: &Rc<Entity>, tickDelta: f32)
    {
        let tick: f64 = tickDelta as f64;
        let d0: f64 = entity.m_lastTickPosX.get() + (entity.posX() - entity.m_lastTickPosX.get()) * tick;
        let d1: f64 = entity.m_lastTickPosY.get() + (entity.posY() - entity.m_lastTickPosY.get()) * tick;
        let d2: f64 = entity.m_lastTickPosZ.get() + (entity.posZ() - entity.m_lastTickPosZ.get()) * tick;
        let f: f32 =
            entity.m_prevRotationYaw.get() + (entity.m_rotationYaw.get() - entity.m_prevRotationYaw.get()) * tickDelta;
        let f1: f32 = entity.getEntityBrightness(tickDelta);
        lwjgl::opengl::GL11::glColor3f(f1, f1, f1);
        let (x, y, z): (f64, f64, f64) = Self::renderPos();
        self.renderEntityWithPosYaw(entity, d0 - x, d1 - y, d2 - z, f, tickDelta);
    }

    pub fn renderEntityWithPosYaw(&self, entity: &Rc<Entity>, dx: f64, dy: f64, dz: f64, yaw: f32, tickDelta: f32)
    {
        if let Some(render) = self.getEntityRenderObject(entity)
        {
            render.doRender(entity, dx, dy, dz, yaw, tickDelta);
            render.doRenderShadowAndFire(entity, dx, dy, dz, yaw, tickDelta);
        }
    }

    pub fn setWorld(&self, world: Option<Rc<World>>) { self.m_worldObj.set(world); }

    pub fn squaredDistanceToCamera(&self, x: f64, y: f64, z: f64) -> f64
    {
        let d0: f64 = x - self.m_cameraX.get();
        let d1: f64 = y - self.m_cameraY.get();
        let d2: f64 = z - self.m_cameraZ.get();
        d0 * d0 + d1 * d1 + d2 * d2
    }

    pub fn getFontRenderer(&self) -> Option<Rc<FontRenderer>> { self.m_fontRenderer.get() }
}
