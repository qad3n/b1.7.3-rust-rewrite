use crate::render::RenderManager;
use crate::render::tile::TileEntitySpecialRenderer;
use game::entity::{Entity, EntityList};
use game::tileentity::{TileEntity, tile_entity_mob_spawner};
use jvm::string::JString;
use lwjgl::opengl::GL11;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct TileEntityMobSpawnerRenderer
{
    m_entityHashMap: RefCell<HashMap<Option<JString>, Option<Rc<Entity>>>>,
}

impl TileEntityMobSpawnerRenderer
{
    pub fn new() -> TileEntityMobSpawnerRenderer
    {
        TileEntityMobSpawnerRenderer { m_entityHashMap: RefCell::new(HashMap::new()) }
    }
}

impl TileEntitySpecialRenderer for TileEntityMobSpawnerRenderer
{
    fn renderTileEntityAt(&self, x: &Rc<TileEntity>, d: f64, d2: f64, d3: f64, f: f32)
    {
        GL11::glPushMatrix();
        GL11::glTranslatef(d as f32 + 0.5, d2 as f32, d3 as f32 + 0.5);
        let mobID: Option<JString> = tile_entity_mob_spawner::getMobID(x);
        let cached: Option<Option<Rc<Entity>>> = self.m_entityHashMap.borrow().get(&mobID).cloned();
        let entity: Option<Rc<Entity>> = match cached.flatten()
        {
            Some(entity) => Some(entity),
            None =>
            {
                let created: Option<Rc<Entity>> = EntityList::createEntityInWorld(mobID.as_ref(), &x.world());
                self.m_entityHashMap.borrow_mut().insert(mobID, created.clone());
                created
            }
        };
        if let Some(entity) = entity
        {
            entity.setWorld(x.m_worldObj.get());
            let f2: f32 = 0.4375;
            let spawner: &tile_entity_mob_spawner::SpawnerExt = tile_entity_mob_spawner::fields(x);
            GL11::glTranslatef(0.0, 0.4, 0.0);
            GL11::glRotatef(
                (spawner.m_yaw2.get() + (spawner.m_yaw.get() - spawner.m_yaw2.get()) * f as f64) as f32 * 10.0,
                0.0,
                1.0,
                0.0,
            );
            GL11::glRotatef(-30.0, 1.0, 0.0, 0.0);
            GL11::glTranslatef(0.0, -0.4, 0.0);
            GL11::glScalef(f2, f2, f2);
            entity.setLocationAndAngles(d, d2, d3, 0.0, 0.0);
            RenderManager::instance().renderEntityWithPosYaw(&entity, 0.0, 0.0, 0.0, 0.0, f);
        }
        GL11::glPopMatrix();
    }
}
