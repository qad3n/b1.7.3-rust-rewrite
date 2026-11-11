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

thread_local!
{
    // identifier and numeric id of every placeholder, kept in the snapshot so reinstalling restores content
    static MISSING_ROWS: RefCell<BTreeMap<String, BTreeMap<Identifier, i32>>> = const { RefCell::new(BTreeMap::new()) };
    static PLACEHOLDERS: RefCell<BTreeSet<(String, i32)>> = const { RefCell::new(BTreeSet::new()) };
    // the map a dedicated server already applied before its worlds existed, the overworld load then skips it
    static PRELOADED: RefCell<Option<RegistrySnapshot>> = const { RefCell::new(None) };
}

pub fn snapshot() -> RegistrySnapshot
{
    let mut maps: BTreeMap<String, BTreeMap<Identifier, i32>> = registry::with(|registries|
    {
        registries.synced().iter().map(|registry| (registry.name().to_owned(), registry.snapshot())).collect()
    });
    maps.insert(crate::states::SNAPSHOT_KEY.to_owned(), crate::states::counts());

    MISSING_ROWS.with(|missing|
    {
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
    registry::with(|registries| registries.synced().iter().any(|registry| registry.ownsAny(owner)))
}

// the identifier a placeholder id stands for, from the saved map of a removed mod
pub fn missingName(registry: &str, numeric: i32) -> Option<Identifier>
{
    MISSING_ROWS
        .with(|rows| rows.borrow().get(registry)?.iter().find(|(_, id)| **id == numeric).map(|(name, _)| name.clone()))
}

fn modded(map: &BTreeMap<Identifier, i32>) -> BTreeMap<Identifier, i32>
{
    map.iter().filter(|(id, _)| !id.isVanilla()).map(|(id, numeric)| (id.clone(), *numeric)).collect()
}

// rows the snapshot names but no mod registered any more, the content those ids hold is kept as placeholders
fn missingRows(registries: &Registries, snapshot: &RegistrySnapshot) -> BTreeMap<String, BTreeMap<Identifier, i32>>
{
    let mut missing: BTreeMap<String, BTreeMap<Identifier, i32>> = BTreeMap::new();

    for registry in registries.synced()
    {
        let Some(saved) = snapshot.registries.get(registry.name())
        else
        {
            continue;
        };

        for (id, numeric) in modded(saved).into_iter().filter(|(id, _)| !registry.contains(id))
        {
            missing.entry(registry.name().to_owned()).or_default().insert(id, numeric);
        }
    }

    missing
}

fn unregisterModded()
{
    let (blocks, items): (Vec<i32>, Vec<i32>) = registry::with(|registries|
    {
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
            // a removed mod's tile entity stays on its placeholder, the chunk refuses tiles on non container blocks
            Block::setIsBlockContainerEntry(*numeric, true);
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

    registry::with(|registries|
    {
        // a block's item form shares its id, so a moved block may not land on a vanilla item
        let vanillaItems: BTreeSet<i32> =
            registries.m_items.rows().filter(|row| row.m_id.isVanilla()).map(|row| row.m_numeric).collect();
        let reserved = |name: &str| -> BTreeSet<i32>
        {
            let mut taken: BTreeSet<i32> =
                snapshot.registries.get(name).map(|map| map.values().copied().collect()).unwrap_or_default();

            if name == "block"
            {
                taken.extend(vanillaItems.iter().copied());
            }

            taken
        };

        let saved =
            |name: &str| -> BTreeMap<Identifier, i32> { snapshot.registries.get(name).map(modded).unwrap_or_default() };
        let mut blocks: BTreeMap<Identifier, i32> = BTreeMap::new();
        let blockItems: BTreeSet<Identifier> = registries
            .m_items
            .modded()
            .filter(|row| row.m_value.m_block.is_some())
            .map(|row| row.m_id.clone())
            .collect();

        for registry in registries.syncedMut()
        {
            let mut map: BTreeMap<Identifier, i32> = saved(registry.name());

            // a block item follows its block, which SYNCED_ORDER reassigns first
            if registry.name() == "item"
            {
                map.extend(blocks.iter().filter(|(id, _)| blockItems.contains(*id)).map(|(id, n)| (id.clone(), *n)));
            }

            registry.reassign(&map, &reserved(registry.name()));

            if registry.name() == "block"
            {
                blocks = registry.snapshot();
            }
        }
    });

    placeholders(&missing);
    MISSING_ROWS.with(|rows| *rows.borrow_mut() = missing.clone());
    crate::lifecycle::rebuild();
    missing.iter().flat_map(|(name, rows)| rows.keys().map(move |id| format!("{name} {id}"))).collect()
}

// a dedicated server builds a tracker and player manager per dimension before any world exists, so the level's
// saved map applies first and mod dimensions keep their saved numbers when another mod was added
pub fn preload(worldDir: &std::path::Path)
{
    // a world without saved mod content in a game without mods stays vanilla, even after keeping an earlier one
    if !crate::state::hasMods()
    {
        crate::state::setKeeper(false);
    }

    let path: std::path::PathBuf = worldDir.join(persist::LOADER_DIR).join(REGISTRY_FILE);
    let Some(saved) =
        std::fs::read(&path).ok().and_then(|bytes| persist::decode::<RegistrySnapshot>(REGISTRY_FILE, &bytes))
    else
    {
        return;
    };

    let modded: bool = saved.registries.values().flat_map(BTreeMap::keys).any(|id| !id.isVanilla());
    if !crate::state::hasMods() && modded
    {
        crate::lifecycle::keep();
    }

    if !crate::play::active()
    {
        return;
    }

    if saved != snapshot()
    {
        crate::log::debug(crate::log::LOADER, &format!("applying the id map saved in {}", worldDir.display()));
        applySaved(Some(worldDir), &saved);
    }

    PRELOADED.with(|preloaded| *preloaded.borrow_mut() = Some(saved));
}

// the first load that meets content of a removed mod backs the world up once and names what became placeholders
fn applySaved(worldDir: Option<&std::path::Path>, saved: &RegistrySnapshot)
{
    let missing: Vec<String> = registry::with(|registries| missingRows(registries, saved))
        .iter()
        .flat_map(|(name, rows)| rows.keys().map(move |id| format!("{name} {id}")))
        .collect();

    if !missing.is_empty()
    {
        if let Some(dir) = worldDir
        {
            backup(dir);
        }

        crate::log::warn(
            crate::log::LOADER,
            &format!("this world has content from removed mods, kept as placeholders: {}", missing.join(", ")),
        );
    }

    apply(saved);
}

// every authoritative world load applies its saved map and writes the merged one back
pub fn onWorldLoad(world: &Rc<World>)
{
    crate::palette::clear();

    let saved: Option<RegistrySnapshot> =
        persist::read(world, REGISTRY_FILE).and_then(|bytes| persist::decode(REGISTRY_FILE, &bytes));
    // the same map, as on a nether trip, needs no rebuild
    let preloaded: Option<RegistrySnapshot> = PRELOADED.with(|preloaded| preloaded.borrow_mut().take());
    if let Some(saved) = saved.as_ref().filter(|saved| **saved != snapshot() && Some(*saved) != preloaded.as_ref())
    {
        applySaved(persist::worldDir(world).as_deref(), saved);
    }

    if let Err(error) = persist::write(world, REGISTRY_FILE, &wire::encode(&snapshot()))
    {
        crate::log::error(crate::log::LOADER, &format!("could not save the registry snapshot: {error}"));
    }
}

// one copy of the world before the first load that holds content of a removed mod
fn backup(dir: &std::path::Path)
{
    let marker: std::path::PathBuf = dir.join(persist::LOADER_DIR).join("backed_up");
    if marker.exists()
    {
        return;
    }

    match game::storage::world_format::backup(dir)
    {
        Ok(target) =>
        {
            crate::log::warn(crate::log::LOADER, &format!("world backed up to {}", target.display()));

            let marked: std::io::Result<()> = std::fs::create_dir_all(dir.join(persist::LOADER_DIR))
                .and_then(|()| std::fs::write(&marker, jvm::system::currentTimeMillis().to_string()));

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

thread_local!
{
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

    let class: &'static TileEntityClass = MISSING_TILE_CLASS.with(|cell|
    {
        *cell.get_or_init(||
        {
            crate::classes::deriveTileEntityClass(&TILE_ENTITY, MISSING, |class|
            {
                class.readFromNBT = |this, nbt|
                {
                    (TILE_ENTITY.readFromNBT)(this, nbt);

                    let raw: Rc<NBTBase> = NBTTagCompound::create();
                    copyInto(nbt, raw.compound());
                    *this.ext::<MissingTile>().m_raw.borrow_mut() = Some(raw);
                };

                class.writeToNBT = |this, nbt|
                {
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

    let class: &'static EntityClass = MISSING_ENTITY_CLASS.with(|cell|
    {
        *cell.get_or_init(||
        {
            crate::classes::deriveEntityClass(&ENTITY, MISSING, |class|
            {
                class.onUpdate = |_| {};
                class.entityInit = |_| {};

                class.readEntityFromNBT = |this, nbt|
                {
                    let raw: Rc<NBTBase> = NBTTagCompound::create();
                    copyInto(nbt, raw.compound());
                    *this.ext::<MissingEntity>().m_raw.borrow_mut() = Some(raw);
                };

                class.writeEntityToNBT = |this, nbt|
                {
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
    let raw: Rc<NBTBase> = entity.m_ext.asAny().downcast_ref::<MissingEntity>()?.m_raw.borrow().clone()?;
    raw.compound().getString("id").map(|id| id.to_string())
}
