use game::world::World;
use game::worldgen::{WorldGenFlowers, WorldGenMinable, WorldGenerator};
use jvm::random::Random;
use mod_protocol::spec::{WorldgenKind, WorldgenSpec};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Feature
{
    m_kind: WorldgenKind,
    m_block: i32,
    m_dimension: i32,
    m_veinSize: i32,
    m_perChunk: i32,
    m_minY: i32,
    m_maxY: i32,
}

thread_local! {
    static FEATURES: RefCell<Vec<Feature>> = const { RefCell::new(Vec::new()) };
}

pub fn add(spec: &WorldgenSpec) -> Result<(), String>
{
    let (block, dimension): (Option<i32>, Option<i32>) = crate::registry::with(|registries| {
        (registries.blockId(&spec.block), registries.m_dimensions.numeric(&spec.dimension))
    });
    let block: i32 = block.ok_or_else(|| format!("worldgen block {} is not registered", spec.block))?;
    let dimension: i32 = dimension.ok_or_else(|| format!("worldgen dimension {} is not registered", spec.dimension))?;
    let [minY, maxY] = spec.y;
    if minY < 0 || maxY > 128 || minY >= maxY
    {
        return Err(format!("worldgen y range {minY}..{maxY} must lie in 0..128"));
    }
    FEATURES.with(|features| {
        features.borrow_mut().push(Feature {
            m_kind: spec.kind,
            m_block: block,
            m_dimension: dimension,
            m_veinSize: spec.vein_size,
            m_perChunk: spec.per_chunk,
            m_minY: minY,
            m_maxY: maxY,
        })
    });
    Ok(())
}

pub fn clear() { FEATURES.with(|features| features.borrow_mut().clear()); }

pub fn hasFeatures() -> bool { FEATURES.with(|features| !features.borrow().is_empty()) }

// each feature seeds its own random from the world seed and chunk, vanilla populate randoms stay untouched
pub fn populate(world: &Rc<World>, chunkX: i32, chunkZ: i32)
{
    let dimension: i32 = world.worldProvider().m_worldType.get();
    FEATURES.with(|features| {
        for (index, feature) in
            features.borrow().iter().enumerate().filter(|(_, feature)| feature.m_dimension == dimension)
        {
            let seed: i64 = world.getRandomSeed()
                ^ (chunkX as i64).wrapping_mul(341_873_128_712)
                ^ (chunkZ as i64).wrapping_mul(132_897_987_541)
                ^ (index as i64 + 1).wrapping_mul(0x5DEECE66D);
            let random: Random = Random::withSeed(seed);
            for _ in 0..feature.m_perChunk
            {
                let x: i32 = chunkX * 16 + random.nextIntBounded(16) + 8;
                let y: i32 = feature.m_minY + random.nextIntBounded(feature.m_maxY - feature.m_minY);
                let z: i32 = chunkZ * 16 + random.nextIntBounded(16) + 8;
                match feature.m_kind
                {
                    WorldgenKind::Ore =>
                    {
                        WorldGenMinable::new(feature.m_block, feature.m_veinSize).generate(world, &random, x, y, z);
                    }
                    WorldgenKind::Plant =>
                    {
                        WorldGenFlowers::new(feature.m_block).generate(world, &random, x, y, z);
                    }
                }
            }
        }
    });
}
