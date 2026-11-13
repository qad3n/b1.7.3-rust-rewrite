use crate::host::context;
use game::entity::Entity;
use game::tileentity::TileEntity;
use game::world::{Chunk, World};
use mod_protocol::Identifier;
use mod_protocol::abi::{AttachmentChange, BlockPos, Holder, attachmentEvent};
use mod_protocol::spec::AttachmentSpec;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

pub const ATTACHED: &str = "loader:attached";

const CHUNK: i32 = 16;

struct Declared
{
    m_owner: String,
    m_spec: AttachmentSpec,
}

#[derive(Default)]
struct Changes
{
    // in the order they were made, each holder and name once per tick
    m_order: Vec<(Holder, Identifier)>,
    m_seen: HashSet<(Holder, Identifier)>,
}

thread_local!
{
    static DECLARED: RefCell<BTreeMap<Identifier, Declared>> = const { RefCell::new(BTreeMap::new()) };
    // values of the world as a whole that are not saved, and a client's copy of the world's synced values
    static SCRATCH: RefCell<HashMap<(Holder, Identifier), Vec<u8>>> = RefCell::new(HashMap::new());
    static CHANGED: RefCell<Changes> = RefCell::new(Changes::default());
}

pub fn declare(owner: &str, name: Identifier, spec: AttachmentSpec) -> Result<(), String>
{
    if name.namespace() != owner
    {
        return Err(format!("{name} is outside the namespace {owner}"));
    }

    DECLARED.with(|declared|
    {
        let mut declared = declared.borrow_mut();
        if declared.contains_key(&name)
        {
            return Err(format!("attachment {name} is declared twice"));
        }

        declared.insert(name.clone(), Declared { m_owner: owner.to_owned(), m_spec: spec });
        Ok(())
    })?;

    if spec.synced
    {
        crate::events::define(attachmentEvent(&name));
    }

    Ok(())
}

// the owner of a synced value may listen for it changing on a side that draws the world
fn notify(change: AttachmentChange)
{
    let event: Identifier = attachmentEvent(&change.name);
    if let Err(error) = crate::events::postCustom(event, mod_protocol::wire::encode(&change))
    {
        crate::log::debug(crate::log::LOADER, &format!("no attachment event for {}: {error}", change.name));
    }
}

fn specOf(name: &Identifier) -> Result<AttachmentSpec, String>
{
    DECLARED
        .with(|declared| declared.borrow().get(name).map(|row| row.m_spec))
        .ok_or_else(|| format!("attachment {name} is not declared, declare it while the mod loads"))
}

fn synced(name: &str) -> bool
{
    Identifier::parse(name)
        .ok()
        .and_then(|id| DECLARED.with(|declared| declared.borrow().get(&id).map(|row| row.m_spec.synced)))
        .unwrap_or(false)
}

fn tileAt(dimension: i32, pos: BlockPos) -> Result<Rc<TileEntity>, String>
{
    context::world(dimension)
        .ok_or_else(|| format!("no world for dimension {dimension}"))?
        .getBlockTileEntity(pos.x, pos.y, pos.z)
        .ok_or_else(|| format!("no tile entity at {pos:?}"))
}

fn chunkAt(dimension: i32, x: i32, z: i32) -> Result<Rc<Chunk>, String>
{
    Ok(context::world(dimension)
        .ok_or_else(|| format!("no world for dimension {dimension}"))?
        .getChunkFromChunkCoords(x, z))
}

fn entityAt(dimension: i32, id: i32) -> Result<Rc<Entity>, String>
{
    context::entityIn(dimension, id).ok_or_else(|| format!("no entity {id}"))
}

fn scratch(holder: Holder, name: &Identifier) -> Option<Vec<u8>>
{
    SCRATCH.with(|values| values.borrow().get(&(holder, name.clone())).cloned())
}

fn setScratch(holder: Holder, name: &Identifier, data: Option<Vec<u8>>)
{
    SCRATCH.with(|values|
    {
        let mut values = values.borrow_mut();
        match data
        {
            Some(bytes) => values.insert((holder, name.clone()), bytes),
            None => values.remove(&(holder, name.clone())),
        };
    });
}

fn read(holder: Holder, name: &Identifier, spec: AttachmentSpec) -> Result<Option<Vec<u8>>, String>
{
    let key: String = name.to_string();
    match holder
    {
        Holder::Entity { world, id } => Ok(crate::entities::attached(&*entityAt(world, id)?, &key, spec)),
        Holder::Tile { world, pos } =>
        {
            let tile: Rc<TileEntity> = tileAt(world, pos)?;
            Ok(slots(&tile.m_attachments, &tile.m_unsaved, spec).borrow().get(&key).cloned())
        }

        Holder::Chunk { world, x, z } =>
        {
            let chunk: Rc<Chunk> = chunkAt(world, x, z)?;
            Ok(slots(&chunk.m_attachments, &chunk.m_unsaved, spec).borrow().get(&key).cloned())
        }

        Holder::World if !spec.saved => Ok(scratch(holder, name)),
        // a client has no save, its copy of the world's values is the one the server sent
        Holder::World if context::mirroring() => Ok(scratch(holder, name)),
        Holder::World => Ok(crate::persist::worldValue(&key)),
    }
}

fn write(holder: Holder, name: &Identifier, spec: AttachmentSpec, data: Option<Vec<u8>>) -> Result<(), String>
{
    let key: String = name.to_string();
    match holder
    {
        Holder::Entity { world, id } => crate::entities::attach(&*entityAt(world, id)?, &key, spec, data),
        Holder::Tile { world, pos } =>
        {
            let tile: Rc<TileEntity> = tileAt(world, pos)?;
            put(slots(&tile.m_attachments, &tile.m_unsaved, spec), key, data);

            if spec.saved
                && let Some(world) = tile.m_worldObj.get()
            {
                world.getChunkFromBlockCoords(pos.x, pos.z).setChunkModified();
            }
        }

        Holder::Chunk { world, x, z } =>
        {
            let chunk: Rc<Chunk> = chunkAt(world, x, z)?;
            put(slots(&chunk.m_attachments, &chunk.m_unsaved, spec), key, data);

            if spec.saved
            {
                chunk.setChunkModified();
            }
        }

        Holder::World if !spec.saved => setScratch(holder, name, data),

        Holder::World if context::mirroring() => setScratch(holder, name, data),
        Holder::World => crate::persist::setWorldValue(&key, data),
    }

    Ok(())
}

// a tile or chunk keeps its saved values and the ones that only last as long as it does in two maps
fn slots<'a>(
    saved: &'a RefCell<common::nbt::attachments::Attachments>,
    unsaved: &'a RefCell<common::nbt::attachments::Attachments>,
    spec: AttachmentSpec,
) -> &'a RefCell<common::nbt::attachments::Attachments>
{
    if spec.saved { saved } else { unsaved }
}

fn put(slots: &RefCell<common::nbt::attachments::Attachments>, key: String, data: Option<Vec<u8>>)
{
    let mut slots = slots.borrow_mut();
    match data
    {
        Some(bytes) => slots.insert(key, bytes),
        None => slots.remove(&key),
    };
}

pub fn fetch(holder: Holder, name: &Identifier) -> Result<Option<Vec<u8>>, String> { read(holder, name, specOf(name)?) }

// a store that changes nothing is dropped here, so a mod may write the same value every tick at no cost to the network
pub fn store(owner: &str, holder: Holder, name: &Identifier, data: Option<Vec<u8>>) -> Result<(), String>
{
    let spec: AttachmentSpec = specOf(name)?;
    if name.namespace() != owner
    {
        return Err(format!("{name} belongs to another mod, {owner} may read it but not change it"));
    }

    if context::mirroring() && (spec.saved || spec.synced)
    {
        return Err(format!(
            "attachment {name} is saved or synced and only changes where the world is owned: this client mirrors a \
             server's world, so send a payload to the server and store it there"
        ));
    }

    if read(holder, name, spec)? == data
    {
        return Ok(());
    }

    write(holder, name, spec, data)?;

    // a server sends the change to clients, single player tells the mod itself, both once a tick
    if spec.synced
    {
        queue(holder, name.clone());
    }

    Ok(())
}

// the main data blob of a synced mod tile shares the queue under its channel name, so it too goes out once a tick
fn tileData() -> Identifier { crate::network::sync::tileChannel() }

pub fn tileDataChanged(dimension: i32, pos: BlockPos)
{
    if crate::side::isServer()
    {
        queue(Holder::Tile { world: dimension, pos }, tileData());
    }
}

fn queue(holder: Holder, name: Identifier)
{
    CHANGED.with(|changed|
    {
        let mut changed = changed.borrow_mut();
        if changed.m_seen.insert((holder, name.clone()))
        {
            changed.m_order.push((holder, name));
        }
    });
}

fn channel() -> Identifier { crate::network::loader::id(ATTACHED) }

fn encode(holder: Holder, name: Identifier, data: Option<Vec<u8>>) -> Vec<u8>
{
    mod_protocol::wire::encode(&AttachmentChange { holder, name, data })
}

fn middle(x: i32, z: i32) -> [f64; 3] { [f64::from(x * CHUNK + CHUNK / 2), 0.0, f64::from(z * CHUNK + CHUNK / 2)] }

// once a server tick: every synced value that changed goes, as it stands now, to the players who can see its holder
pub fn flush()
{
    let changes: Vec<(Holder, Identifier)> = CHANGED.with(|changed|
    {
        let mut changed = changed.borrow_mut();
        changed.m_seen.clear();
        std::mem::take(&mut changed.m_order)
    });

    let tileData: Identifier = tileData();
    for (holder, name) in changes
    {
        if name == tileData
        {
            if let Holder::Tile { world, pos } = holder
                && let Ok(tile) = tileAt(world, pos)
            {
                crate::network::behindChunks(|| crate::network::sync::tileChanged(&tile));
            }

            continue;
        }

        // a holder that went away since the change has nobody left to tell
        let Ok(data) = fetch(holder, &name)
        else
        {
            continue;
        };

        if !crate::side::isServer()
        {
            notify(AttachmentChange { holder, name, data });
            continue;
        }

        let bytes: Vec<u8> = encode(holder, name, data);
        match holder
        {
            Holder::Tile { world, pos } =>
            {
                if let Some(world) = context::world(world)
                {
                    let at: [f64; 3] = [f64::from(pos.x), f64::from(pos.y), f64::from(pos.z)];
                    crate::network::behindChunks(||
                    {
                        crate::network::sendNear(&channel(), &world, at, crate::network::Reach::Column, &bytes);
                    });
                }
            }

            Holder::Chunk { world, x, z } =>
            {
                if let Some(world) = context::world(world)
                {
                    crate::network::behindChunks(||
                    {
                        crate::network::sendNear(
                            &channel(),
                            &world,
                            middle(x, z),
                            crate::network::Reach::Column,
                            &bytes,
                        );
                    });
                }
            }

            Holder::Entity { world, id } =>
            {
                if let Some(entity) = context::entityIn(world, id)
                    && let Some(world) = entity.m_worldObj.get()
                {
                    let at: [f64; 3] = [entity.m_posX.get(), entity.m_posY.get(), entity.m_posZ.get()];
                    crate::network::sendNear(&channel(), &world, at, crate::network::Reach::Sphere, &bytes);
                }
            }

            Holder::World => crate::network::broadcast(&channel(), &bytes),
        }
    }
}

fn send(player: &Rc<Entity>, holder: Holder, name: &str, data: Vec<u8>)
{
    if let Ok(name) = Identifier::parse(name)
    {
        crate::network::sendToPlayer(&channel(), player, encode(holder, name, Some(data)));
    }
}

fn sendSynced(player: &Rc<Entity>, holder: Holder, slots: &RefCell<common::nbt::attachments::Attachments>)
{
    for (name, data) in slots.borrow().iter().filter(|(name, _)| synced(name))
    {
        send(player, holder, name, data.clone());
    }
}

// a player who starts seeing a chunk gets the synced values on it and on its tile entities
pub fn chunkSight(player: &Rc<Entity>, world: &World, chunk: &Chunk)
{
    let dimension: i32 = context::dimension(world);
    let onChunk: Holder = Holder::Chunk { world: dimension, x: chunk.m_xPosition, z: chunk.m_zPosition };
    sendSynced(player, onChunk, &chunk.m_attachments);
    sendSynced(player, onChunk, &chunk.m_unsaved);

    for tile in chunk.m_chunkTileEntityMap.get().values()
    {
        let pos: BlockPos = BlockPos::new(tile.m_xCoord.get(), tile.m_yCoord.get(), tile.m_zCoord.get());
        let onTile: Holder = Holder::Tile { world: dimension, pos };
        sendSynced(player, onTile, &tile.m_attachments);
        sendSynced(player, onTile, &tile.m_unsaved);
    }
}

// a player who starts seeing an entity, or who just joined and needs its own, gets the entity's synced values
pub fn entitySight(entity: &Entity, player: &Rc<Entity>)
{
    let Some(world) = entity.m_worldObj.get()
    else
    {
        return;
    };

    let holder: Holder = Holder::Entity { world: context::dimension(&world), id: entity.m_entityId.get() };
    for (name, data) in crate::entities::attachedAll(entity).into_iter().filter(|(name, _)| synced(name))
    {
        send(player, holder, &name, data);
    }
}

// on login, respawn and dimension change a player's client gets the world's synced values and the player's own
pub fn playerSight(player: &Rc<Entity>)
{
    if !crate::side::isServer()
    {
        return;
    }

    for (name, data) in crate::persist::worldValues().into_iter().filter(|(name, _)| synced(name))
    {
        send(player, Holder::World, &name, data);
    }

    entitySight(player, player);
}

// a client takes the server's value as it is, the rules about who may change what are the server's
pub fn received(message: AttachmentChange)
{
    let Ok(spec) = specOf(&message.name)
    else
    {
        crate::log::warn(crate::log::LOADER, &format!("the server sent undeclared attachment {}", message.name));
        return;
    };

    // a tile or entity the client has not built yet is told again when it comes into sight
    if write(message.holder, &message.name, spec, message.data.clone()).is_err()
    {
        crate::log::debug(crate::log::LOADER, &format!("no holder yet for attachment {}", message.name));
        return;
    }

    notify(message);
}

// who declared what, for the registry report
pub fn declared() -> Vec<(Identifier, String, AttachmentSpec)>
{
    DECLARED.with(|declared|
    {
        declared.borrow().iter().map(|(name, row)| (name.clone(), row.m_owner.clone(), row.m_spec)).collect()
    })
}

// leaving a world or a server drops what was not saved
pub fn reset()
{
    SCRATCH.with(|values| values.borrow_mut().clear());
    CHANGED.with(|changed| *changed.borrow_mut() = Changes::default());
}
