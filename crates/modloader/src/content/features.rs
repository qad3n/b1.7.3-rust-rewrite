use crate::unify::{OreRow, Unified};
use game::block::Block;
use game::world::World;
use jvm::random::Random;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, FeatureCall, FeatureInfo, GuestCall, GuestReply, Handle, RegistryKind, StateData};
use mod_protocol::spec::{FeatureKind, FeatureModifySpec, FeatureSpec, HeightSpec, OreFeatureSpec, TreeFeatureSpec};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::rc::Rc;

const CONFIG_FILE: &str = "loader.toml";

mod chunk
{
    pub const SIZE: i32 = 16;
    // decorations land half a chunk over so they spill into loaded neighbours, ore veins add it themselves
    pub const EDGE: i32 = 8;
}

// WorldGenMinable's vein shape
mod vein
{
    pub const CENTER: i32 = 8;
    pub const REACH: f32 = 8.0;
    pub const RISE_SPREAD: i32 = 3;
    pub const RISE: i32 = 2;
    pub const WIDTH: f64 = 16.0;
    pub const HALF: f64 = 0.5;
}

// WorldGenTrees' canopy
mod canopy
{
    pub const LAYERS: i32 = 3;
    pub const WIDE: i32 = 2;
}

// a feature's numeric state, block tags and dimension under the current id map
enum Placer
{
    Ore
    {
        state: (i32, i32),
        spec: OreFeatureSpec,
        replaces: HashSet<i32>,
    },
    Tree
    {
        log: (i32, i32),
        leaves: (i32, i32),
        spec: TreeFeatureSpec,
        soil: HashSet<i32>,
        foliage: HashSet<i32>,
    },
    Custom(usize, Handle),
}

struct Row
{
    m_id: Identifier,
    m_owner: String,
    m_mod: Option<usize>,
    m_key: i64,
    m_spec: FeatureSpec,
    m_handler: Option<Handle>,
    m_enabled: bool,
    m_dimension: Option<i32>,
    m_placer: Option<Rc<Placer>>,
}

#[derive(Default)]
struct Table
{
    // run order, step first and stable key within a step
    m_rows: Vec<Row>,
    m_disabled: BTreeSet<Identifier>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LoaderConfig
{
    worldgen: FeatureModifySpec,
}

thread_local!
{
    static TABLE: RefCell<Table> = RefCell::new(Table::default());
}

pub fn register(
    owner: &str,
    modIndex: Option<usize>,
    id: Identifier,
    spec: FeatureSpec,
    handler: Option<Handle>,
) -> Result<(), String>
{
    let custom: bool = matches!(spec.kind, FeatureKind::Custom);
    if custom != handler.is_some()
    {
        return Err(format!("feature {id} needs a handler exactly when it is custom"));
    }

    if let FeatureKind::Ore(ore) = &spec.kind
    {
        heightRange(&ore.height)?;
    }

    TABLE.with(|table|
    {
        crate::registry::claim(RegistryKind::Feature, &id)?;
        let mut table = table.borrow_mut();

        let key: i64 = super::worldgen::stableKey(&id);
        let at: usize = table.m_rows.partition_point(|row| (row.m_spec.step, row.m_key) < (spec.step, key));
        let row: Row = Row
        {
            m_id: id,
            m_owner: owner.to_owned(),
            m_mod: modIndex,
            m_key: key,
            m_spec: spec,
            m_handler: handler,
            m_enabled: true,
            m_dimension: None,
            m_placer: None,
        };
        table.m_rows.insert(at, row);
        Ok(())
    })
}

pub fn disable(spec: FeatureModifySpec) { TABLE.with(|table| table.borrow_mut().m_disabled.extend(spec.disable)); }

fn heightRange(height: &HeightSpec) -> Result<(), String>
{
    let valid: bool = match height
    {
        HeightSpec::Uniform { min, max } => min < max,
        HeightSpec::Sum { draws, .. } => !draws.is_empty() && draws.iter().all(|bound| *bound > 0),
    };
    if !valid
    {
        return Err(format!("height {height:?} draws from an empty range"));
    }

    Ok(())
}

pub fn oreRows() -> Vec<OreRow>
{
    TABLE.with(|table|
    {
        table
            .borrow()
            .m_rows
            .iter()
            .filter_map(|row| match &row.m_spec.kind
            {
                FeatureKind::Ore(ore) => Some(OreRow
                {
                    m_owner: row.m_owner.clone(),
                    m_id: row.m_id.clone(),
                    m_block: ore.block.clone(),
                    m_dimension: row.m_spec.dimension.clone(),
                    m_ore: true,
                }),
                _ => None,
            })
            .collect()
    })
}

// config/loader.toml [worldgen] disable = ["<mod>:<feature>"]
fn configDisabled() -> Vec<Identifier>
{
    crate::paths()
        .and_then(|paths| std::fs::read_to_string(paths.m_config.join(CONFIG_FILE)).ok())
        .and_then(|text| toml::from_str::<LoaderConfig>(&text).ok())
        .map(|config| config.worldgen.disable)
        .unwrap_or_default()
}

// numeric ids change with every id map, so each content build resolves the features again
pub fn resolve(decisions: &BTreeMap<Identifier, Unified>) -> Vec<(String, String)>
{
    let mut problems: Vec<(String, String)> = Vec::new();
    let config: Vec<Identifier> = configDisabled();

    TABLE.with(|table|
    {
        let mut table = table.borrow_mut();
        let disabled: BTreeSet<Identifier> = table.m_disabled.iter().cloned().chain(config).collect();

        for row in &mut table.m_rows
        {
            row.m_enabled = !disabled.contains(&row.m_id) && !matches!(decisions.get(&row.m_id), Some(Unified::Skip));
            row.m_dimension =
                crate::registry::with(|registries| registries.m_dimensions.numeric(&row.m_spec.dimension));
            let target: Option<&Identifier> = match decisions.get(&row.m_id)
            {
                Some(Unified::Place(target)) => Some(target),
                _ => None,
            };

            match placer(row, target)
            {
                Ok(placer) => row.m_placer = Some(Rc::new(placer)),
                Err(reason) =>
                {
                    row.m_placer = None;
                    problems.push((row.m_owner.clone(), format!("feature {}: {reason}", row.m_id)));
                }
            }

            if row.m_dimension.is_none()
            {
                problems.push((row.m_owner.clone(), format!("feature {} names no dimension", row.m_id)));
            }
        }
    });

    problems
}

fn state(block: &Identifier, properties: &BTreeMap<String, String>) -> Result<(i32, i32), String>
{
    let data: StateData = StateData { block: block.clone(), properties: properties.clone() };
    crate::states::resolve(&data).ok_or_else(|| format!("{block} {properties:?} is not a registered block state"))
}

fn blocks(tag: &Identifier) -> HashSet<i32> { super::tags::blockIds(tag).into_iter().collect() }

fn placer(row: &Row, target: Option<&Identifier>) -> Result<Placer, String>
{
    match &row.m_spec.kind
    {
        FeatureKind::Ore(ore) =>
        {
            let block: &Identifier = target.unwrap_or(&ore.block);
            Ok(Placer::Ore
            {
                state: state(block, &ore.properties)?,
                spec: ore.clone(),
                replaces: blocks(&ore.replaces),
            })
        }

        FeatureKind::Tree(tree) => Ok(Placer::Tree
        {
            log: state(&tree.log, &tree.log_properties)?,
            leaves: state(&tree.leaves, &tree.leaves_properties)?,
            spec: tree.clone(),
            soil: blocks(&tree.soil),
            foliage: blocks(&Identifier::minecraft("leaves")),
        }),

        FeatureKind::Custom => match (row.m_mod, row.m_handler)
        {
            (Some(modIndex), Some(handler)) => Ok(Placer::Custom(modIndex, handler)),
            _ => Err("a custom feature needs the mod that registered it".to_owned()),
        },
    }
}

// placers hold numeric ids, so they go with the id map they were resolved under
pub fn clear() { TABLE.with(|table| table.borrow_mut().m_rows.iter_mut().for_each(|row| row.m_placer = None)); }

pub fn list() -> Vec<FeatureInfo>
{
    TABLE.with(|table|
    {
        table
            .borrow()
            .m_rows
            .iter()
            .map(|row| FeatureInfo
            {
                id: row.m_id.clone(),
                owner: row.m_owner.clone(),
                spec: row.m_spec.clone(),
                enabled: row.m_enabled,
            })
            .collect()
    })
}

// the serde name data files use
fn label<T: serde::Serialize>(value: &T) -> String
{
    toml::Value::try_from(value).ok().and_then(|value| value.as_str().map(str::to_owned)).unwrap_or_default()
}

// one line per feature in run order for the loader console
pub fn summary() -> String
{
    let lines: Vec<String> = list()
        .iter()
        .map(|feature|
        {
            let kind: &str = match feature.spec.kind
            {
                FeatureKind::Ore(_) => "ore",
                FeatureKind::Tree(_) => "tree",
                FeatureKind::Custom => "custom",
            };
            let state: &str = if feature.enabled { "" } else { " off" };
            format!(
                "{} {} {kind} {} in {}{state}",
                feature.id,
                feature.owner,
                label(&feature.spec.step),
                feature.spec.dimension
            )
        })
        .collect();
    format!("{} features\n{}", lines.len(), lines.join("\n"))
}

fn seed(world: &World, chunkX: i32, chunkZ: i32, key: i64) -> i64
{
    const SPREAD_X: i64 = 341873128712;
    const SPREAD_Z: i64 = 132897987541;

    world.getRandomSeed() ^ (chunkX as i64).wrapping_mul(SPREAD_X) ^ (chunkZ as i64).wrapping_mul(SPREAD_Z) ^ key
}

fn biomeFits(world: &World, tag: Option<&Identifier>, x: i32, z: i32) -> bool
{
    let Some(tag) = tag
    else
    {
        return true;
    };

    crate::biomes::biomeAt(world, x, z).is_some_and(|biome| super::tags::contains(RegistryKind::Biome, tag, &biome))
}

// every enabled feature of the world's dimension in run order, each from its own seeded random
pub fn populate(world: &Rc<World>, chunkX: i32, chunkZ: i32)
{
    let dimension: i32 = crate::host::context::dimension(world);
    let runs: Vec<(i64, Option<Identifier>, Rc<Placer>)> = TABLE.with(|table|
    {
        table
            .borrow()
            .m_rows
            .iter()
            .filter(|row| row.m_enabled && !row.m_spec.placed && row.m_dimension == Some(dimension))
            .filter_map(|row| Some((row.m_key, row.m_spec.biomes.clone(), row.m_placer.clone()?)))
            .collect()
    });
    let (x, z): (i32, i32) = (chunkX * chunk::SIZE, chunkZ * chunk::SIZE);

    for (key, biomes, placer) in runs
    {
        if !biomeFits(world, biomes.as_ref(), x + chunk::SIZE, z + chunk::SIZE)
        {
            continue;
        }

        let random: Random = Random::withSeed(seed(world, chunkX, chunkZ, key));
        run(world, &placer, &random, BlockPos { x, y: 0, z }, true, dimension);
    }
}

// one placement by name, as a sapling grows its tree
pub fn place(world: &Rc<World>, id: &Identifier, pos: BlockPos, seed: i64) -> Result<bool, String>
{
    let placer: Option<Rc<Placer>> = TABLE
        .with(|table|
        {
            let table = table.borrow();
            let row: &Row = table.m_rows.iter().find(|row| row.m_id == *id)?;
            Some(row.m_enabled.then(|| row.m_placer.clone()).flatten())
        })
        .ok_or_else(|| format!("no feature {id}"))?;

    let Some(placer) = placer
    else
    {
        return Ok(false);
    };

    let dimension: i32 = crate::host::context::dimension(world);
    Ok(run(world, &placer, &Random::withSeed(seed), pos, false, dimension))
}

fn run(world: &Rc<World>, placer: &Placer, random: &Random, origin: BlockPos, chunk: bool, dimension: i32) -> bool
{
    match placer
    {
        Placer::Ore { state, spec, replaces } =>
        {
            let tries: i32 = if chunk { spec.per_chunk } else { 1 };
            let mut placed: bool = false;

            for _ in 0..tries
            {
                let at: [i32; 3] = if chunk
                {
                    let x: i32 = origin.x + random.nextIntBounded(chunk::SIZE);
                    let y: i32 = height(&spec.height, random);
                    [x, y, origin.z + random.nextIntBounded(chunk::SIZE)]
                }
                else
                {
                    [origin.x, origin.y, origin.z]
                };
                placed |= ore(world, random, *state, spec.vein_size, at, replaces);
            }

            placed
        }

        Placer::Tree { log, leaves, spec, soil, foliage } =>
        {
            let parts: Tree = Tree
            {
                m_log: *log,
                m_leaves: *leaves,
                m_spec: spec,
                m_soil: soil,
                m_foliage: foliage,
                m_top: top(dimension),
            };
            if !chunk
            {
                return parts.grow(world, random, origin.x, origin.y, origin.z);
            }

            if random.nextIntBounded(spec.chance.max(1)) != 0
            {
                return false;
            }

            let mut placed: bool = false;
            for _ in 0..spec.per_chunk
            {
                let x: i32 = origin.x + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
                let z: i32 = origin.z + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
                placed |= parts.grow(world, random, x, world.getHeightValue(x, z), z);
            }

            placed
        }

        Placer::Custom(modIndex, handler) =>
        {
            let Some(runtime) = crate::state::modAt(*modIndex)
            else
            {
                return false;
            };

            let call: GuestCall = GuestCall::Feature
            {
                handler: *handler,
                call: FeatureCall { world: dimension, origin, seed: random.nextLong(), chunk },
            };
            matches!(crate::host::context::withWorld(world, || runtime.call(&call)), Some(GuestReply::Bool(true)))
        }
    }
}

// a mod dimension may be taller than beta's 128
fn top(dimension: i32) -> i32
{
    crate::dimensions::dimension(dimension)
        .map_or(game::world::HEIGHT, |modded| crate::dimensions::height(&modded.m_id).m_world)
}

fn height(spec: &HeightSpec, random: &Random) -> i32
{
    match spec
    {
        HeightSpec::Uniform { min, max } => min + random.nextIntBounded(max - min),
        HeightSpec::Sum { draws, offset } =>
        {
            draws.iter().map(|bound| random.nextIntBounded(*bound)).sum::<i32>() + offset
        }
    }
}

// WorldGenMinable with any block state and any replaceable tag, the same random draws as beta's ores
fn ore(world: &Rc<World>, random: &Random, state: (i32, i32), size: i32, at: [i32; 3], replaces: &HashSet<i32>)
-> bool
{
    let [x, y, z] = at;
    let angle: f32 = random.nextFloat() * jvm::math::PI as f32;
    let spread: f32 = size as f32 / vein::REACH;
    let sin: f32 = game::math::math_helper::sin(angle) * spread;
    let cos: f32 = game::math::math_helper::cos(angle) * spread;
    let x0: f64 = ((x + vein::CENTER) as f32 + sin) as f64;
    let x1: f64 = ((x + vein::CENTER) as f32 - sin) as f64;
    let z0: f64 = ((z + vein::CENTER) as f32 + cos) as f64;
    let z1: f64 = ((z + vein::CENTER) as f32 - cos) as f64;
    let y0: f64 = (y + random.nextIntBounded(vein::RISE_SPREAD) + vein::RISE) as f64;
    let y1: f64 = (y + random.nextIntBounded(vein::RISE_SPREAD) + vein::RISE) as f64;
    let mut placed: bool = false;

    for i in 0..=size
    {
        let part: f64 = i as f64 / size as f64;
        let cx: f64 = x0 + (x1 - x0) * part;
        let cy: f64 = y0 + (y1 - y0) * part;
        let cz: f64 = z0 + (z1 - z0) * part;
        let scale: f64 = random.nextDouble() * size as f64 / vein::WIDTH;
        let bulge: f64 = (game::math::math_helper::sin(i as f32 * jvm::math::PI as f32 / size as f32) + 1.0) as f64;
        let wide: f64 = bulge * scale + 1.0;
        let tall: f64 = bulge * scale + 1.0;

        for bx in floor(cx - wide / 2.0)..=floor(cx + wide / 2.0)
        {
            let dx: f64 = (bx as f64 + vein::HALF - cx) / (wide / 2.0);
            if dx * dx >= 1.0
            {
                continue;
            }

            for by in floor(cy - tall / 2.0)..=floor(cy + tall / 2.0)
            {
                let dy: f64 = (by as f64 + vein::HALF - cy) / (tall / 2.0);
                if dx * dx + dy * dy >= 1.0
                {
                    continue;
                }

                for bz in floor(cz - wide / 2.0)..=floor(cz + wide / 2.0)
                {
                    let dz: f64 = (bz as f64 + vein::HALF - cz) / (wide / 2.0);
                    let inside: bool = dx * dx + dy * dy + dz * dz < 1.0;
                    if inside && replaces.contains(&world.getBlockId(bx, by, bz))
                    {
                        placed |= world.setBlockAndMetadata(bx, by, bz, state.0, state.1);
                    }
                }
            }
        }
    }

    placed
}

fn floor(value: f64) -> i32 { game::math::math_helper::floor_double(value) }

// the parts of one tree feature, shaped as WorldGenTrees
struct Tree<'a>
{
    m_log: (i32, i32),
    m_leaves: (i32, i32),
    m_spec: &'a TreeFeatureSpec,
    m_soil: &'a HashSet<i32>,
    m_foliage: &'a HashSet<i32>,
    m_top: i32,
}

impl Tree<'_>
{
    fn open(&self, block: i32) -> bool { block == 0 || self.m_foliage.contains(&block) || block == self.m_leaves.0 }

    // the trunk and canopy space is free of anything but air and leaves
    fn room(&self, world: &Rc<World>, x: i32, y: i32, z: i32, height: i32, top: i32) -> bool
    {
        for level in y..=y + 1 + height
        {
            let reach: i32 = match level
            {
                _ if level >= y + 1 + height - canopy::WIDE => canopy::WIDE,
                _ if level == y => 0,
                _ => 1,
            };

            for a in x - reach..=x + reach
            {
                for b in z - reach..=z + reach
                {
                    if !(0..top).contains(&level) || !self.open(world.getBlockId(a, level, b))
                    {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn grow(&self, world: &Rc<World>, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let height: i32 = random.nextIntBounded(self.m_spec.spread.max(1)) + self.m_spec.min_height;
        let top: i32 = self.m_top;
        if y < 1 || y + height + 1 > top || !self.room(world, x, y, z, height, top)
        {
            return false;
        }

        if !self.m_soil.contains(&world.getBlockId(x, y - 1, z)) || y >= top - height - 1
        {
            return false;
        }

        world.setBlock(x, y - 1, z, Block::DIRT);

        for leafY in y - canopy::LAYERS + height..=y + height
        {
            let fromTop: i32 = leafY - (y + height);
            let radius: i32 = 1 - fromTop / 2;

            for leafX in x - radius..=x + radius
            {
                for leafZ in z - radius..=z + radius
                {
                    let corner: bool = (leafX - x).abs() == radius && (leafZ - z).abs() == radius;
                    // beta draws for every corner before it skips the top layer's corners
                    if corner && (random.nextIntBounded(2) == 0 || fromTop == 0)
                    {
                        continue;
                    }

                    if !Block::opaqueCubeLookup(world.getBlockId(leafX, leafY, leafZ))
                    {
                        world.setBlockAndMetadata(leafX, leafY, leafZ, self.m_leaves.0, self.m_leaves.1);
                    }
                }
            }
        }

        for level in 0..height
        {
            if self.open(world.getBlockId(x, y + level, z))
            {
                world.setBlockAndMetadata(x, y + level, z, self.m_log.0, self.m_log.1);
            }
        }

        true
    }
}
