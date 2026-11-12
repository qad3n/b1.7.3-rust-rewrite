use game::world::World;
use game::worldgen::{WorldGenFlowers, WorldGenMinable, WorldGenerator};
use jvm::random::Random;
use mod_protocol::Identifier;
use mod_protocol::spec::{WorldgenKind, WorldgenSpec};
use std::cell::{RefCell, RefMut};
use std::collections::HashSet;
use std::rc::Rc;

pub struct Feature
{
    m_key: i64,
    m_kind: WorldgenKind,
    m_block: i32,
    m_dimension: i32,
    m_veinSize: i32,
    m_perChunk: i32,
    m_minY: i32,
    m_maxY: i32,
    // a replaces block or block tag, beta's stone when absent
    m_replaces: Option<HashSet<i32>>,
    // biomes the feature is limited to, every biome when empty
    m_biomes: Vec<Identifier>,
}

// the blocks a replaces entry names, a tag first, then a single block
fn replaceable(id: &Identifier) -> Result<HashSet<i32>, String>
{
    let tagged: std::collections::BTreeSet<i32> = super::tags::blockIds(id);
    if !tagged.is_empty()
    {
        return Ok(tagged.into_iter().collect());
    }

    crate::registry::with(|registries| registries.blockId(id))
        .map(|block| HashSet::from([block]))
        .ok_or_else(|| format!("worldgen replaces {id}, which is neither a block nor a block tag"))
}

thread_local!
{
    static FEATURES: RefCell<Vec<Feature>> = const { RefCell::new(Vec::new()) };
}

// FNV-1a over the identifier, the same on every machine and every run
pub(crate) fn stableKey(id: &Identifier) -> i64
{
    const OFFSET: u64 = 0xCBF29CE484222325;
    const PRIME: u64 = 0x0100000001B3;
    id.to_string().bytes().fold(OFFSET, |hash, byte| (hash ^ byte as u64).wrapping_mul(PRIME)) as i64
}

pub fn add(id: &Identifier, spec: &WorldgenSpec) -> Result<(), String>
{
    let (block, dimension): (Option<i32>, Option<i32>) = crate::registry::with(|registries|
    {
        (registries.blockId(&spec.block), registries.m_dimensions.numeric(&spec.dimension))
    });
    let block: i32 = block.ok_or_else(|| format!("worldgen block {} is not registered", spec.block))?;
    let dimension: i32 = dimension.ok_or_else(|| format!("worldgen dimension {} is not registered", spec.dimension))?;
    let [minY, maxY] = spec.y;
    let top: i32 = crate::dimensions::height(&spec.dimension).m_world;
    if minY < 0 || maxY > top || minY >= maxY
    {
        return Err(format!("worldgen y range {minY}..{maxY} must lie in 0..{top}"));
    }

    let key: i64 = stableKey(id);
    let replaces: Option<HashSet<i32>> = spec.replaces.as_ref().map(replaceable).transpose()?;

    // features run in key order, so the order mods load in never changes what a chunk gets
    FEATURES.with(|features|
    {
        let mut features: RefMut<Vec<Feature>> = features.borrow_mut();
        let at: usize = features.partition_point(|feature| feature.m_key < key);
        features.insert(
            at,
            Feature
            {
                m_key: key,
                m_kind: spec.kind,
                m_block: block,
                m_dimension: dimension,
                m_veinSize: spec.vein_size,
                m_perChunk: spec.per_chunk,
                m_minY: minY,
                m_maxY: maxY,
                m_replaces: replaces,
                m_biomes: spec.biomes.clone(),
            },
        )
    });

    Ok(())
}

pub fn clear() { FEATURES.with(|features| features.borrow_mut().clear()); }

pub fn hasFeatures() -> bool { FEATURES.with(|features| !features.borrow().is_empty()) }

// each feature seeds its own random from the world seed and chunk, vanilla populate randoms stay untouched
pub fn populate(world: &Rc<World>, chunkX: i32, chunkZ: i32)
{
    let dimension: i32 = world.worldProvider().m_worldType.get();

    FEATURES.with(|features|
    {
        for feature in features.borrow().iter().filter(|feature| feature.m_dimension == dimension)
        {
            let seed: i64 = world.getRandomSeed()
                ^ (chunkX as i64).wrapping_mul(341873128712)
                ^ (chunkZ as i64).wrapping_mul(132897987541)
                ^ feature.m_key;
            let random: Random = Random::withSeed(seed);

            for _ in 0..feature.m_perChunk
            {
                let x: i32 = chunkX * 16 + random.nextIntBounded(16) + 8;
                let y: i32 = feature.m_minY + random.nextIntBounded(feature.m_maxY - feature.m_minY);
                let z: i32 = chunkZ * 16 + random.nextIntBounded(16) + 8;
                let outside: bool = !feature.m_biomes.is_empty()
                    && crate::biomes::biomeAt(world, x, z).is_none_or(|biome| !feature.m_biomes.contains(&biome));
                if outside
                {
                    continue;
                }

                match (feature.m_kind, &feature.m_replaces)
                {
                    (WorldgenKind::Ore, None) =>
                    {
                        WorldGenMinable::new(feature.m_block, feature.m_veinSize).generate(world, &random, x, y, z);
                    }

                    (WorldgenKind::Ore, Some(replaces)) =>
                    {
                        let state: (i32, i32) = (feature.m_block, 0);
                        super::features::ore(world, &random, state, feature.m_veinSize, [x, y, z], replaces);
                    }

                    (WorldgenKind::Plant, _) =>
                    {
                        WorldGenFlowers::new(feature.m_block).generate(world, &random, x, y, z);
                    }
                }
            }
        }
    });
}
