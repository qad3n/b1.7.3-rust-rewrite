use crate::persist;
use crate::registry::{self, Registries};
use common::nbt::{NBTBase, NBTTagCompound};
use game::block::{Block, Material};
use game::entity::{ENTITY, Entity, EntityClass, EntityExt};
use game::item::Item;
use game::tileentity::{TILE_ENTITY, TileEntity, TileEntityClass};
use game::world::World;
use mod_protocol::handshake::RegistrySnapshot;
use mod_protocol::{Identifier, wire};
use std::any::Any;
use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

pub const REGISTRY_FILE: &str = "registry.dat";
pub const MISSING: &str = "loader:missing";
const MISSING_SPRITE: i32 = 30;
const SYNCED: [&str; 6] = ["block", "item", "entity", "tile_entity", "dimension", "biome"];

thread_local! {
    // identifier and numeric id of every placeholder, kept in the snapshot so reinstalling restores content
    static MISSING_ROWS: RefCell<BTreeMap<String, BTreeMap<Identifier, i32>>> = const { RefCell::new(BTreeMap::new()) };
    static PLACEHOLDERS: RefCell<BTreeSet<(String, i32)>> = const { RefCell::new(BTreeSet::new()) };
}

pub fn snapshot() -> RegistrySnapshot
{
    let mut maps: BTreeMap<String, BTreeMap<Identifier, i32>> = registry::with(|registries| {
        BTreeMap::from([
            ("block".to_owned(), registries.m_blocks.snapshot()),
            ("item".to_owned(), registries.m_items.snapshot()),
            ("entity".to_owned(), registries.m_entities.snapshot()),
            ("tile_entity".to_owned(), registries.m_tileEntities.snapshot()),
            ("dimension".to_owned(), registries.m_dimensions.snapshot()),
            ("biome".to_owned(), registries.m_biomes.snapshot()),
        ])
    });
    MISSING_ROWS.with(|missing| {
        for (name, rows) in missing.borrow().iter()
        {
            maps.entry(name.clone()).or_default().extend(rows.iter().map(|(id, numeric)| (id.clone(), *numeric)));
        }
    });
    RegistrySnapshot { registries: maps }
}

// the registries in the snapshot, a mod owning rows in any of them cannot be missing on a client
pub fn ownsSyncedRows(owner: &str) -> bool
{
    registry::with(|registries| {
        registries.m_blocks.modded().any(|row| row.m_owner == owner)
            || registries.m_items.modded().any(|row| row.m_owner == owner)
            || registries.m_entities.modded().any(|row| row.m_owner == owner)
            || registries.m_tileEntities.modded().any(|row| row.m_owner == owner)
            || registries.m_dimensions.modded().any(|row| row.m_owner == owner)
            || registries.m_biomes.modded().any(|row| row.m_owner == owner)
    })
}

fn modded(map: &BTreeMap<Identifier, i32>) -> BTreeMap<Identifier, i32>
{
    map.iter().filter(|(id, _)| !id.isVanilla()).map(|(id, numeric)| (id.clone(), *numeric)).collect()
}

// rows the snapshot names but no mod registered any more, the content those ids hold is kept as placeholders
fn missingRows(registries: &Registries, snapshot: &RegistrySnapshot) -> BTreeMap<String, BTreeMap<Identifier, i32>>
{
    let mut missing: BTreeMap<String, BTreeMap<Identifier, i32>> = BTreeMap::new();
    for name in SYNCED
    {
        let Some(saved) = snapshot.registries.get(name)
        else
        {
            continue;
        };
        for (id, numeric) in modded(saved)
        {
            let known: bool = match name
            {
                "block" => registries.m_blocks.get(&id).is_some(),
                "item" => registries.m_items.get(&id).is_some(),
                "entity" => registries.m_entities.get(&id).is_some(),
                "tile_entity" => registries.m_tileEntities.get(&id).is_some(),
                "dimension" => registries.m_dimensions.get(&id).is_some(),
                _ => registries.m_biomes.get(&id).is_some(),
            };
            if !known
            {
                missing.entry(name.to_owned()).or_default().insert(id, numeric);
            }
        }
    }
    missing
}

fn unregisterModded()
{
    let (blocks, items): (Vec<i32>, Vec<i32>) = registry::with(|registries| {
        (
            registries.m_blocks.modded().map(|row| row.m_numeric).collect(),
            registries.m_items.modded().map(|row| row.m_numeric).collect(),
        )
    });
    for id in blocks
    {
        Block::unregister(id);
        Item::unregister(id);
    }
    for id in items
    {
        Item::unregister(id);
    }
    for (kind, id) in PLACEHOLDERS.with(|set| std::mem::take(&mut *set.borrow_mut()))
    {
        if kind == "block"
        {
            Block::unregister(id);
        }
        Item::unregister(id);
    }
}

fn placeholders(missing: &BTreeMap<String, BTreeMap<Identifier, i32>>)
{
    for numeric in missing.get("block").into_iter().flat_map(|rows| rows.values())
    {
        if Block::byId(*numeric).is_none()
        {
            Block::basic(*numeric, MISSING_SPRITE, Material::rock()).setHardness(1.0).setBlockName("loader.missing");
            game::item::item_block::create(numeric - 256);
            PLACEHOLDERS.with(|set| set.borrow_mut().insert(("block".to_owned(), *numeric)));
        }
    }
    for numeric in missing.get("item").into_iter().flat_map(|rows| rows.values())
    {
        if Item::byId(*numeric).is_none()
        {
            Item::basic(numeric - 256).setItemName("loader.missing");
            PLACEHOLDERS.with(|set| set.borrow_mut().insert(("item".to_owned(), *numeric)));
        }
    }
}

// a mod renames its own content through [remap] old = "new" in mod.toml
pub fn rename(id: &Identifier) -> Identifier
{
    let Some(runtime) = crate::state::byId(id.namespace())
    else
    {
        return id.clone();
    };
    match runtime.m_manifest.remap.get(id.path()).and_then(|path| Identifier::new(id.namespace(), path).ok())
    {
        Some(renamed) => renamed,
        None => id.clone(),
    }
}

fn renamed(snapshot: &RegistrySnapshot) -> RegistrySnapshot
{
    let registries = snapshot
        .registries
        .iter()
        .map(|(name, rows)| (name.clone(), rows.iter().map(|(id, numeric)| (rename(id), *numeric)).collect()))
        .collect();
    RegistrySnapshot { registries }
}

// saved ids win, new content takes ids nothing saved uses, then every mod object is rebuilt at its id
pub fn apply(snapshot: &RegistrySnapshot) -> Vec<String>
{
    let snapshot: &RegistrySnapshot = &renamed(snapshot);
    let missing: BTreeMap<String, BTreeMap<Identifier, i32>> =
        registry::with(|registries| missingRows(registries, snapshot));
    unregisterModded();
    crate::content::reset();
    registry::with(|registries| {
        let reserved = |name: &str| -> BTreeSet<i32> {
            snapshot.registries.get(name).map(|map| map.values().copied().collect()).unwrap_or_default()
        };
        let saved =
            |name: &str| -> BTreeMap<Identifier, i32> { snapshot.registries.get(name).map(modded).unwrap_or_default() };
        reassign(&mut registries.m_blocks, &saved("block"), &reserved("block"));
        let blocks: BTreeMap<Identifier, i32> = registries.m_blocks.snapshot();
        let mut items: BTreeMap<Identifier, i32> = saved("item");
        for row in registries.m_items.modded().filter(|row| row.m_value.m_block.is_some())
        {
            if let Some(numeric) = blocks.get(&row.m_id)
            {
                items.insert(row.m_id.clone(), *numeric);
            }
        }
        reassign(&mut registries.m_items, &items, &reserved("item"));
        reassign(&mut registries.m_entities, &saved("entity"), &reserved("entity"));
        reassign(&mut registries.m_tileEntities, &saved("tile_entity"), &reserved("tile_entity"));
        reassign(&mut registries.m_dimensions, &saved("dimension"), &reserved("dimension"));
        reassign(&mut registries.m_biomes, &saved("biome"), &reserved("biome"));
    });
    placeholders(&missing);
    MISSING_ROWS.with(|rows| *rows.borrow_mut() = missing.clone());
    crate::lifecycle::rebuild();
    missing.iter().flat_map(|(name, rows)| rows.keys().map(move |id| format!("{name} {id}"))).collect()
}

fn reassign<T>(registry: &mut registry::Registry<T>, saved: &BTreeMap<Identifier, i32>, reserved: &BTreeSet<i32>)
{
    let free: Vec<(Identifier, i32)> = registry
        .modded()
        .filter(|row| !saved.contains_key(&row.m_id))
        .filter(|row| reserved.contains(&row.m_numeric))
        .map(|row| (row.m_id.clone(), row.m_numeric))
        .collect();
    let mut map: BTreeMap<Identifier, i32> = saved.clone();
    let used: BTreeSet<i32> = registry
        .rows()
        .filter(|row| !free.iter().any(|(id, _)| *id == row.m_id))
        .map(|row| row.m_numeric)
        .chain(reserved.iter().copied())
        .collect();
    let mut next: i32 = free.iter().map(|(_, numeric)| *numeric).min().unwrap_or(0);
    for (id, _) in free
    {
        while used.contains(&next)
        {
            next += 1;
        }
        map.insert(id, next);
        next += 1;
    }
    registry.remap(&map);
}

// every authoritative world load applies its saved map and writes the merged one back
pub fn onWorldLoad(world: &Rc<World>)
{
    let saved: Option<RegistrySnapshot> =
        persist::read(world, REGISTRY_FILE).and_then(|bytes| wire::decode(&bytes).ok());
    // the same map, as on a nether trip, needs no rebuild
    if let Some(saved) = saved.as_ref().filter(|saved| **saved != snapshot())
    {
        let missing: Vec<String> = registry::with(|registries| missingRows(registries, saved))
            .iter()
            .flat_map(|(name, rows)| rows.keys().map(move |id| format!("{name} {id}")))
            .collect();
        if !missing.is_empty()
        {
            backup(world);
            crate::log::warn(
                crate::log::LOADER,
                &format!("this world has content from removed mods, kept as placeholders: {}", missing.join(", ")),
            );
        }
        apply(saved);
    }
    if let Err(error) = persist::write(world, REGISTRY_FILE, &wire::encode(&snapshot()))
    {
        crate::log::error(crate::log::LOADER, &format!("could not save the registry snapshot: {error}"));
    }
}

fn copyTree(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()>
{
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)?.flatten()
    {
        let target: std::path::PathBuf = to.join(entry.file_name());
        if entry.path().is_dir()
        {
            copyTree(&entry.path(), &target)?;
        }
        else
        {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

// one copy of the world before the first load that holds content of a removed mod
fn backup(world: &World)
{
    let Some(dir) = persist::worldDir(world)
    else
    {
        return;
    };
    let marker: std::path::PathBuf = dir.join(persist::LOADER_DIR).join("backed_up");
    if marker.exists()
    {
        return;
    }
    let stamp: i64 = jvm::system::currentTimeMillis();
    let target: std::path::PathBuf = dir.with_file_name(format!(
        "{}_backup_{stamp}",
        dir.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
    ));
    match copyTree(&dir, &target)
    {
        Ok(()) =>
        {
            crate::log::warn(crate::log::LOADER, &format!("world backed up to {}", target.display()));
            let marked: std::io::Result<()> = std::fs::create_dir_all(dir.join(persist::LOADER_DIR))
                .and_then(|()| std::fs::write(&marker, stamp.to_string()));
            if let Err(error) = marked
            {
                crate::log::warn(crate::log::LOADER, &format!("could not mark the backup: {error}"));
            }
        }
        Err(error) => crate::log::error(crate::log::LOADER, &format!("world backup failed: {error}")),
    }
}

struct MissingTile
{
    m_raw: RefCell<Option<Rc<NBTBase>>>,
}

fn copyInto(from: &NBTTagCompound, to: &NBTTagCompound)
{
    for tag in from.getValues()
    {
        to.setTag(&tag.getKey().to_string(), &tag);
    }
}

thread_local! {
    static MISSING_TILE_CLASS: OnceCell<&'static TileEntityClass> = const { OnceCell::new() };
    static MISSING_ENTITY_CLASS: OnceCell<&'static EntityClass> = const { OnceCell::new() };
}

// a tile entity of a removed mod keeps its whole compound and writes it back unchanged
pub fn missingTile(id: &str) -> Option<Rc<TileEntity>>
{
    let id: Identifier = Identifier::parse(id).ok()?;
    if id.isVanilla()
    {
        return None;
    }
    let class: &'static TileEntityClass = MISSING_TILE_CLASS.with(|cell| {
        *cell.get_or_init(|| {
            crate::classes::deriveTileEntityClass(&TILE_ENTITY, MISSING, |class| {
                class.readFromNBT = |this, nbt| {
                    (TILE_ENTITY.readFromNBT)(this, nbt);
                    let raw: Rc<NBTBase> = NBTTagCompound::create();
                    copyInto(nbt, raw.compound());
                    *this.ext::<MissingTile>().m_raw.borrow_mut() = Some(raw);
                };
                class.writeToNBT = |this, nbt| {
                    if let Some(raw) = this.ext::<MissingTile>().m_raw.borrow().as_ref()
                    {
                        copyInto(raw.compound(), nbt);
                    }
                    nbt.setInteger("x", this.m_xCoord.get());
                    nbt.setInteger("y", this.m_yCoord.get());
                    nbt.setInteger("z", this.m_zCoord.get());
                };
            })
        })
    });
    Some(TileEntity::new(class, Box::new(MissingTile { m_raw: RefCell::new(None) })))
}

struct MissingEntity
{
    m_raw: RefCell<Option<Rc<NBTBase>>>,
}

impl EntityExt for MissingEntity
{
    fn asAny(&self) -> &dyn Any { self }
}

// an entity of a removed mod stands still and saves its original compound, key included
pub fn missingEntity(key: &str, world: &Rc<World>) -> Option<Rc<Entity>>
{
    let id: Identifier = Identifier::parse(key).ok()?;
    if id.isVanilla()
    {
        return None;
    }
    let class: &'static EntityClass = MISSING_ENTITY_CLASS.with(|cell| {
        *cell.get_or_init(|| {
            crate::classes::deriveEntityClass(&ENTITY, MISSING, |class| {
                class.onUpdate = |_| {};
                class.entityInit = |_| {};
                class.readEntityFromNBT = |this, nbt| {
                    let raw: Rc<NBTBase> = NBTTagCompound::create();
                    copyInto(nbt, raw.compound());
                    *this.ext::<MissingEntity>().m_raw.borrow_mut() = Some(raw);
                };
                class.writeEntityToNBT = |this, nbt| {
                    if let Some(raw) = this.ext::<MissingEntity>().m_raw.borrow().as_ref()
                    {
                        for tag in raw.compound().getValues()
                        {
                            let name: String = tag.getKey().to_string();
                            if !matches!(
                                name.as_str(),
                                "Pos" | "Motion" | "Rotation" | "FallDistance" | "Fire" | "Air" | "OnGround"
                            )
                            {
                                nbt.setTag(&name, &tag);
                            }
                        }
                    }
                };
            })
        })
    });
    let entity: Rc<Entity> = Entity::alloc(class, Box::new(MissingEntity { m_raw: RefCell::new(None) }));
    Entity::init(&entity, Some(Rc::clone(world)));
    Some(entity)
}

pub fn missingEntityKey(entity: &Entity) -> Option<String>
{
    let raw = entity.m_ext.asAny().downcast_ref::<MissingEntity>()?.m_raw.borrow().clone()?;
    raw.compound().getString("id").map(|id| id.to_string())
}
