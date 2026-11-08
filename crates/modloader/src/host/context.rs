use game::entity::{Entity, entity_living, entity_player};
use game::world::World;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockCtx, BlockPos, EntityInfo, StackData};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

thread_local! {
    static WORLDS: RefCell<HashMap<i32, Weak<World>>> = RefCell::new(HashMap::new());
    static CURRENT: RefCell<Vec<Rc<World>>> = const { RefCell::new(Vec::new()) };
}

pub fn dimension(world: &World) -> i32 { world.worldProvider().m_worldType.get() }

pub fn rememberWorld(world: &Rc<World>)
{
    WORLDS.with(|worlds| worlds.borrow_mut().insert(dimension(world), Rc::downgrade(world)));
}

pub fn forgetWorld(world: &World) { WORLDS.with(|worlds| worlds.borrow_mut().remove(&dimension(world))); }

pub fn worlds() -> Vec<Rc<World>> { WORLDS.with(|worlds| worlds.borrow().values().filter_map(Weak::upgrade).collect()) }

// guest calls name worlds by dimension, the world of the running callback wins over the remembered one
pub fn world(dimension: i32) -> Option<Rc<World>>
{
    let current: Option<Rc<World>> =
        CURRENT.with(|stack| stack.borrow().iter().rev().find(|world| self::dimension(world) == dimension).cloned());
    current.or_else(|| WORLDS.with(|worlds| worlds.borrow().get(&dimension).and_then(Weak::upgrade)))
}

// block access callbacks carry no world, the running callback or the only remembered world names it
pub fn currentDimension() -> i32
{
    let current: Option<i32> = CURRENT.with(|stack| stack.borrow().last().map(|world| dimension(world)));
    current.unwrap_or_else(|| WORLDS.with(|worlds| worlds.borrow().keys().next().copied().unwrap_or(0)))
}

pub fn withWorld<R>(world: &Rc<World>, body: impl FnOnce() -> R) -> R
{
    CURRENT.with(|stack| stack.borrow_mut().push(Rc::clone(world)));
    let result: R = body();
    CURRENT.with(|stack| stack.borrow_mut().pop());
    result
}

pub fn blockCtx(world: &Rc<World>, block: i32, x: i32, y: i32, z: i32) -> BlockCtx
{
    BlockCtx {
        world: dimension(world),
        pos: BlockPos::new(x, y, z),
        block,
        meta: world.getBlockMetadata(x, y, z),
        authority: crate::side::isAuthority(world),
    }
}

pub fn entityById(world: &World, id: i32) -> Option<Rc<Entity>>
{
    let list: Vec<Rc<Entity>> = world.m_loadedEntityList.toVec();
    list.into_iter().find(|entity| entity.m_entityId.get() == id)
}

pub fn entityIn(dimension: i32, id: i32) -> Option<Rc<Entity>>
{
    let world: Rc<World> = world(dimension)?;
    entityById(&world, id)
}

pub fn entityKind(entity: &Entity) -> Option<Identifier>
{
    let key: String = game::entity::EntityList::getEntityString(entity)?.to_string();
    crate::registry::with(|registries| {
        registries.m_entities.rows().find(|row| row.m_value.m_saveKey == key).map(|row| row.m_id.clone())
    })
}

pub fn entityInfo(entity: &Rc<Entity>) -> EntityInfo
{
    let isPlayer: bool = entity.instanceOf("EntityPlayer");
    EntityInfo {
        id: entity.m_entityId.get(),
        world: entity.m_worldObj.get().map_or(0, |world| dimension(&world)),
        kind: if isPlayer
        {
            Some(Identifier::minecraft("player"))
        }
        else
        {
            entityKind(entity)
        },
        pos: [entity.m_posX.get(), entity.m_posY.get(), entity.m_posZ.get()],
        motion: [entity.m_motionX.get(), entity.m_motionY.get(), entity.m_motionZ.get()],
        yaw: entity.m_rotationYaw.get(),
        pitch: entity.m_rotationPitch.get(),
        health: entity.instanceOf("EntityLiving").then(|| entity_living::fields(entity).m_health.get()),
        player: isPlayer
            .then(|| entity_player::fields(entity).m_username.get().map(|name| name.to_string()).unwrap_or_default()),
        alive: !entity.m_isDead.get(),
    }
}

pub fn stackData(stack: &game::item::ItemStack) -> StackData
{
    StackData { id: stack.m_itemID.get(), count: stack.m_stackSize.get(), damage: stack.getItemDamage() }
}

pub fn stackFrom(data: &StackData) -> Option<Rc<game::item::ItemStack>>
{
    (data.count > 0 && game::item::Item::byId(data.id).is_some())
        .then(|| game::item::ItemStack::withDamage(data.id, data.count, data.damage))
}
