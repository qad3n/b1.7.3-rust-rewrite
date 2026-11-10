use crate::host::context;
use game::world::World;
use mod_protocol::Identifier;
use mod_protocol::abi::{
    BlockPos, GuestCall, GuestReply, Handle, Located, StructurePiece, StructureSpec, StructureStart,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

const FILE: &str = "structures.dat";

mod limits
{
    // starts are indexed in square cells of 1 << CELL_SHIFT blocks
    pub const CELL_SHIFT: i32 = 8;
    // locate asks a finder about every chunk out to this many chunks
    pub const LOCATE_RADIUS: i32 = 128;
    pub const CHUNK_SHIFT: i32 = 4;
    pub const CHUNK_CENTER: i32 = 8;
}

// the structures the game itself builds, named by the key its structure hook passes
struct VanillaRow
{
    m_key: &'static str,
    m_name: &'static str,
    m_dimension: &'static str,
}

const VANILLA: [VanillaRow; 1] = [VanillaRow { m_key: "dungeon", m_name: "Dungeon", m_dimension: "overworld" }];

struct Kind
{
    m_id: Identifier,
    m_spec: StructureSpec,
    m_owner: Option<usize>,
    m_finder: Option<Handle>,
}

struct Record
{
    m_dimension: i32,
    m_start: StructureStart,
}

#[derive(Serialize, Deserialize)]
struct Saved
{
    dimension: Identifier,
    start: StructureStart,
}

#[derive(Default)]
struct Built
{
    m_records: Vec<Record>,
    m_cells: HashMap<(i32, i32, i32), Vec<usize>>,
    m_keys: HashSet<(Identifier, i32, BlockPos)>,
    // starts in dimensions no registry names now, written back untouched so removing a mod loses nothing
    m_orphans: Vec<Saved>,
    m_warned: HashSet<(Identifier, Identifier)>,
}

thread_local! {
    static KINDS: RefCell<Vec<Kind>> = RefCell::new(VANILLA.iter().map(vanillaKind).collect());
    static BUILT: RefCell<Built> = RefCell::new(Built::default());
}

fn vanillaKind(row: &VanillaRow) -> Kind
{
    Kind {
        m_id: Identifier::minecraft(row.m_key),
        m_spec: StructureSpec { name: row.m_name.to_owned(), dimension: Identifier::minecraft(row.m_dimension) },
        m_owner: None,
        m_finder: None,
    }
}

pub fn register(modIndex: usize, id: Identifier, spec: StructureSpec, finder: Option<Handle>) -> Result<(), String>
{
    KINDS.with(|kinds| {
        let mut kinds = kinds.borrow_mut();
        if kinds.iter().any(|kind| kind.m_id == id)
        {
            return Err(format!("structure {id} is already registered"));
        }

        kinds.push(Kind { m_id: id, m_spec: spec, m_owner: Some(modIndex), m_finder: finder });
        Ok(())
    })
}

fn dimensionId(numeric: i32) -> Option<Identifier>
{
    crate::registry::with(|registries| registries.m_dimensions.byNumeric(numeric).map(|row| row.m_id.clone()))
}

fn dimensionNumeric(id: &Identifier) -> Option<i32>
{
    crate::registry::with(|registries| registries.m_dimensions.numeric(id))
}

// the owner, dimension and finder of a kind
fn kindOf(id: &Identifier) -> Option<(Option<usize>, Identifier, Option<Handle>)>
{
    KINDS.with(|kinds| {
        kinds
            .borrow()
            .iter()
            .find(|kind| kind.m_id == *id)
            .map(|kind| (kind.m_owner, kind.m_spec.dimension.clone(), kind.m_finder))
    })
}

fn ordered(min: [i32; 3], max: [i32; 3]) -> bool { (0..3).all(|axis| min[axis] <= max[axis]) }

fn intersects(min: [i32; 3], max: [i32; 3], otherMin: [i32; 3], otherMax: [i32; 3]) -> bool
{
    (0..3).all(|axis| max[axis] >= otherMin[axis] && min[axis] <= otherMax[axis])
}

// the boxes that stand for a start, its pieces or its whole box when it has none
fn boxes(start: &StructureStart) -> Vec<([i32; 3], [i32; 3])>
{
    match start.pieces.is_empty()
    {
        true => vec![(start.min, start.max)],
        false => start.pieces.iter().map(|piece| (piece.min, piece.max)).collect(),
    }
}

fn touches(start: &StructureStart, min: [i32; 3], max: [i32; 3]) -> bool
{
    intersects(start.min, start.max, min, max)
        && boxes(start).iter().any(|(pieceMin, pieceMax)| intersects(*pieceMin, *pieceMax, min, max))
}

fn cells(dimension: i32, min: [i32; 3], max: [i32; 3]) -> impl Iterator<Item = (i32, i32, i32)>
{
    let shift: i32 = limits::CELL_SHIFT;
    (min[0] >> shift..=max[0] >> shift)
        .flat_map(move |x| (min[2] >> shift..=max[2] >> shift).map(move |z| (dimension, x, z)))
}

impl Built
{
    fn insert(&mut self, dimension: i32, start: StructureStart)
    {
        let index: usize = self.m_records.len();
        for cell in cells(dimension, start.min, start.max)
        {
            self.m_cells.entry(cell).or_default().push(index);
        }
        self.m_keys.insert((start.structure.clone(), dimension, start.origin));
        self.m_records.push(Record { m_dimension: dimension, m_start: start });
    }

    fn near(&self, dimension: i32, min: [i32; 3], max: [i32; 3]) -> Vec<usize>
    {
        let mut found: Vec<usize> =
            cells(dimension, min, max).filter_map(|cell| self.m_cells.get(&cell)).flatten().copied().collect();
        found.sort_unstable();
        found.dedup();
        found
    }

    // a structure of one mod built into another mod's is logged once per pair of kinds, generation stays as it is
    fn warnOverlaps(&mut self, dimension: i32, start: &StructureStart)
    {
        for index in self.near(dimension, start.min, start.max)
        {
            let other: &StructureStart = &self.m_records[index].m_start;
            let foreign: bool = other.structure.namespace() != start.structure.namespace();
            let crossed: bool = foreign && boxes(start).iter().any(|(min, max)| touches(other, *min, *max));
            let pair: (Identifier, Identifier) = (start.structure.clone(), other.structure.clone());
            if !crossed || self.m_warned.contains(&pair)
            {
                continue;
            }

            let BlockPos { x, y, z } = start.origin;
            let BlockPos { x: otherX, y: otherY, z: otherZ } = other.origin;
            crate::log::warn(
                crate::log::LOADER,
                &format!(
                    "structure {} at {x} {y} {z} overlaps {} at {otherX} {otherY} {otherZ}",
                    start.structure, other.structure
                ),
            );
            self.m_warned.insert(pair);
        }
    }
}

// a mod records a start of its own kind, a vanilla start comes from the game with no owner
fn add(owner: Option<usize>, world: &World, start: StructureStart) -> Result<(), String>
{
    if !crate::side::isAuthority(world)
    {
        return Err("structures are recorded only on the side that owns the world".to_owned());
    }

    let (kindOwner, dimension, _): (Option<usize>, Identifier, Option<Handle>) =
        kindOf(&start.structure).ok_or_else(|| format!("structure {} is not registered", start.structure))?;
    if kindOwner != owner
    {
        return Err(format!("structure {} is not registered by this mod", start.structure));
    }

    let numeric: i32 = context::dimension(world);
    if dimensionNumeric(&dimension) != Some(numeric)
    {
        return Err(format!("structure {} belongs in {dimension}", start.structure));
    }

    let boxesOrdered: bool =
        ordered(start.min, start.max) && start.pieces.iter().all(|piece| ordered(piece.min, piece.max));
    if !boxesOrdered
    {
        return Err(format!("structure {} has a box whose min lies above its max", start.structure));
    }

    BUILT.with(|built| {
        let mut built = built.borrow_mut();
        if built.m_keys.contains(&(start.structure.clone(), numeric, start.origin))
        {
            return;
        }

        built.warnOverlaps(numeric, &start);
        built.insert(numeric, start);
    });
    Ok(())
}

pub fn addModded(modIndex: usize, world: &World, start: StructureStart) -> Result<(), String>
{
    add(Some(modIndex), world, start)
}

// the game's structure hook, recorded only while mods are loaded so a vanilla world gains no files
pub fn addVanilla(world: &World, key: &str, min: [i32; 3], max: [i32; 3])
{
    if !crate::play::active() || !crate::side::isAuthority(world)
    {
        return;
    }

    let Some(row) = VANILLA.iter().find(|row| row.m_key == key)
    else
    {
        crate::log::warn(crate::log::LOADER, &format!("the game built an unknown structure {key}"));
        return;
    };

    let origin: BlockPos = BlockPos::new((min[0] + max[0]) / 2, min[1], (min[2] + max[2]) / 2);
    let piece: StructurePiece = StructurePiece { min, max, label: String::new() };
    let start: StructureStart =
        StructureStart { structure: Identifier::minecraft(row.m_key), origin, min, max, pieces: vec![piece] };
    if let Err(error) = add(None, world, start)
    {
        crate::log::warn(crate::log::LOADER, &format!("could not record {key}: {error}"));
    }
}

// the recorded starts crossing a box, each with only the pieces that cross it
pub fn find(world: &World, min: [i32; 3], max: [i32; 3], structure: Option<&Identifier>) -> Vec<StructureStart>
{
    let numeric: i32 = context::dimension(world);
    BUILT.with(|built| {
        let built = built.borrow();
        built
            .near(numeric, min, max)
            .into_iter()
            .map(|index| &built.m_records[index])
            .filter(|record| record.m_dimension == numeric)
            .map(|record| &record.m_start)
            .filter(|start| structure.is_none_or(|kind| start.structure == *kind) && touches(start, min, max))
            .map(|start| StructureStart {
                pieces: start
                    .pieces
                    .iter()
                    .filter(|piece| intersects(piece.min, piece.max, min, max))
                    .cloned()
                    .collect(),
                ..start.clone()
            })
            .collect()
    })
}

fn distance(from: BlockPos, x: i32, z: i32) -> i64
{
    let (dx, dz): (i64, i64) = ((x - from.x) as i64, (z - from.z) as i64);
    dx * dx + dz * dz
}

fn recordedNear(numeric: i32, structure: &Identifier, from: BlockPos, radius: i32) -> Option<Located>
{
    let reach: i32 = radius << limits::CHUNK_SHIFT;
    BUILT.with(|built| {
        built
            .borrow()
            .m_records
            .iter()
            .filter(|record| record.m_dimension == numeric && record.m_start.structure == *structure)
            .map(|record| record.m_start.origin)
            .filter(|origin| (origin.x - from.x).abs() <= reach && (origin.z - from.z).abs() <= reach)
            .min_by_key(|origin| distance(from, origin.x, origin.z))
            .map(|origin| Located { x: origin.x, z: origin.z, generated: true })
    })
}

fn finds(world: &World, owner: usize, handler: Handle, chunkX: i32, chunkZ: i32) -> bool
{
    let call: GuestCall =
        GuestCall::StructureFinder { handler, seed: world.getRandomSeed(), chunk_x: chunkX, chunk_z: chunkZ };
    let reply: Option<GuestReply> = crate::state::modAt(owner).and_then(|runtime| runtime.call(&call));
    matches!(reply, Some(GuestReply::Bool(true)))
}

// rings of chunks around the start, stopping once a ring lies farther than the best hit
fn foundNear(world: &World, owner: usize, handler: Handle, from: BlockPos, radius: i32) -> Option<Located>
{
    let (centerX, centerZ): (i32, i32) = (from.x >> limits::CHUNK_SHIFT, from.z >> limits::CHUNK_SHIFT);
    let mut best: Option<(i64, Located)> = None;
    for ring in 0..=radius
    {
        let inner: i64 = (((ring - 1).max(0) as i64) << limits::CHUNK_SHIFT).pow(2);
        if best.is_some_and(|(nearest, _)| nearest < inner)
        {
            break;
        }

        for chunkX in centerX - ring..=centerX + ring
        {
            for chunkZ in centerZ - ring..=centerZ + ring
            {
                let onRing: bool = (chunkX - centerX).abs() == ring || (chunkZ - centerZ).abs() == ring;
                if !onRing || !finds(world, owner, handler, chunkX, chunkZ)
                {
                    continue;
                }

                let x: i32 = (chunkX << limits::CHUNK_SHIFT) + limits::CHUNK_CENTER;
                let z: i32 = (chunkZ << limits::CHUNK_SHIFT) + limits::CHUNK_CENTER;
                let away: i64 = distance(from, x, z);
                if best.is_none_or(|(nearest, _)| away < nearest)
                {
                    best = Some((away, Located { x, z, generated: false }));
                }
            }
        }
    }
    best.map(|(_, found)| found)
}

// the nearest start of a kind, recorded ones and the kind's finder both count and the closer wins
pub fn locate(world: &Rc<World>, structure: &Identifier, from: BlockPos, radius: i32)
-> Result<Option<Located>, String>
{
    if !(0..=limits::LOCATE_RADIUS).contains(&radius)
    {
        return Err(format!("locate radius {radius} must lie in 0..={}", limits::LOCATE_RADIUS));
    }

    let (owner, dimension, finder): (Option<usize>, Identifier, Option<Handle>) =
        kindOf(structure).ok_or_else(|| format!("structure {structure} is not registered"))?;
    let numeric: i32 = context::dimension(world);
    if dimensionNumeric(&dimension) != Some(numeric)
    {
        return Ok(None);
    }

    let recorded: Option<Located> = recordedNear(numeric, structure, from, radius);
    let found: Option<Located> = match (owner, finder)
    {
        (Some(owner), Some(handler)) => context::withWorld(world, || foundNear(world, owner, handler, from, radius)),
        _ => None,
    };
    let nearest: Option<Located> = [recorded, found]
        .into_iter()
        .flatten()
        .min_by_key(|located| (distance(from, located.x, located.z), !located.generated));
    Ok(nearest)
}

pub fn load(world: &World)
{
    let saved: Vec<Saved> =
        crate::persist::read(world, FILE).and_then(|bytes| crate::persist::decode(FILE, &bytes)).unwrap_or_default();
    let mut built: Built = Built::default();
    for entry in saved
    {
        match dimensionNumeric(&entry.dimension)
        {
            Some(numeric) => built.insert(numeric, entry.start),
            None => built.m_orphans.push(entry),
        }
    }
    crate::log::debug(crate::log::LOADER, &format!("loaded {} structure starts", built.m_records.len()));
    BUILT.with(|slot| *slot.borrow_mut() = built);
}

pub fn save(world: &World)
{
    let saved: Vec<Saved> = BUILT.with(|built| {
        let built = built.borrow();
        let recorded = built.m_records.iter().filter_map(|record| {
            let dimension: Identifier = dimensionId(record.m_dimension)?;
            Some(Saved { dimension, start: record.m_start.clone() })
        });
        let kept = built
            .m_orphans
            .iter()
            .map(|orphan| Saved { dimension: orphan.dimension.clone(), start: orphan.start.clone() });
        recorded.chain(kept).collect()
    });
    if saved.is_empty()
    {
        return;
    }

    if let Err(error) = crate::persist::write(world, FILE, &mod_protocol::wire::encode(&saved))
    {
        crate::log::error(crate::log::LOADER, &format!("could not save structure starts: {error}"));
    }
}

// every kind with its owner and how many starts this world has recorded of it
pub fn summary() -> String
{
    let counts: BTreeMap<Identifier, usize> = BUILT.with(|built| {
        let mut counts: BTreeMap<Identifier, usize> = BTreeMap::new();
        for record in &built.borrow().m_records
        {
            *counts.entry(record.m_start.structure.clone()).or_default() += 1;
        }
        counts
    });
    let lines: Vec<String> = KINDS.with(|kinds| {
        kinds
            .borrow()
            .iter()
            .map(|kind| {
                let count: usize = counts.get(&kind.m_id).copied().unwrap_or(0);
                let finder: &str = if kind.m_finder.is_some()
                {
                    ", locatable"
                }
                else
                {
                    ""
                };
                format!("{} \"{}\" in {}: {count} built{finder}", kind.m_id, kind.m_spec.name, kind.m_spec.dimension)
            })
            .collect()
    });
    lines.join("; ")
}

pub fn describe(start: &StructureStart) -> String
{
    let BlockPos { x, y, z } = start.origin;
    let labels: Vec<&str> =
        start.pieces.iter().map(|piece| piece.label.as_str()).filter(|label| !label.is_empty()).collect();
    format!(
        "{} at {x} {y} {z} spanning {:?} to {:?} with {} pieces here [{}]",
        start.structure,
        start.min,
        start.max,
        start.pieces.len(),
        labels.join(", ")
    )
}
