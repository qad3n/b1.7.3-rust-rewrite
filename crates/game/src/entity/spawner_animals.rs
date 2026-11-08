use crate::block::{Material, block_bed};
use crate::entity::{
    Entity, entity_chicken, entity_cow, entity_creeper, entity_ghast, entity_living, entity_pig, entity_pig_zombie,
    entity_sheep, entity_skeleton, entity_slime, entity_spider, entity_squid, entity_wolf, entity_zombie,
};
use crate::math::math_helper;
use crate::pathfinding::{PathEntity, PathPoint, Pathfinder};
use crate::world::{ChunkCoordinates, IBlockAccess, World};
use crate::worldgen::{BiomeGenBase, EnumCreatureType, SpawnListEntry};
use common::world::{ChunkCoordIntPair, ChunkPosition};
use jvm::array_list::JArrayList;
use jvm::hash_map::JHashSet;
use jvm::throwable::{self, OrNpe, Throwable};
use std::cell::RefCell;
use std::rc::Rc;

type Constructor = fn(Option<Rc<World>>) -> Rc<Entity>;

const SPAWNABLE: [(&str, Constructor); 13] = [
    ("EntitySpider", entity_spider::new),
    ("EntityZombie", entity_zombie::new),
    ("EntitySkeleton", entity_skeleton::new),
    ("EntityCreeper", entity_creeper::new),
    ("EntitySlime", entity_slime::new),
    ("EntitySheep", entity_sheep::new),
    ("EntityPig", entity_pig::new),
    ("EntityChicken", entity_chicken::new),
    ("EntityCow", entity_cow::new),
    ("EntitySquid", entity_squid::new),
    ("EntityWolf", entity_wolf::new),
    ("EntityGhast", entity_ghast::new),
    ("EntityPigZombie", entity_pig_zombie::new),
];

const NIGHT_SPAWN_ENTITIES: [Constructor; 3] = [entity_spider::new, entity_zombie::new, entity_skeleton::new];

const SPAWN_RADIUS_CHUNKS: i32 = 8;

thread_local! {
    static ELIGIBLE_CHUNKS_FOR_SPAWNING: RefCell<JHashSet<ChunkCoordIntPair>> = RefCell::new(JHashSet::new());
}

pub struct SpawnerAnimals;

fn instantiate(create: Constructor, world: &Rc<World>) -> Option<Rc<Entity>>
{
    let result: Result<Rc<Entity>, Throwable> =
        throwable::catchThrowable(&throwable::EXCEPTION, || create(Some(Rc::clone(world))));
    match result
    {
        Ok(entity) => Some(entity),
        Err(error) =>
        {
            error.printStackTrace();
            None
        }
    }
}

fn constructorFor(className: &str) -> Constructor
{
    match SPAWNABLE.iter().find(|(name, _)| *name == className)
    {
        Some((_, create)) => *create,
        None => throwable::throw(&throwable::ILLEGAL_STATE, &format!("no constructor for {className}")),
    }
}

impl SpawnerAnimals
{
    fn getRandomSpawningPointInChunk(world: &World, chunkX: i32, chunkZ: i32) -> ChunkPosition
    {
        let i: i32 = chunkX + world.m_rand.nextIntBounded(16);
        let j: i32 = world.m_rand.nextIntBounded(128);
        let k: i32 = chunkZ + world.m_rand.nextIntBounded(16);
        ChunkPosition::new(i, j, k)
    }

    pub fn performSpawning(world: &Rc<World>, spawnHostileMobs: bool, spawnPeacefulMobs: bool) -> i32
    {
        if !spawnHostileMobs && !spawnPeacefulMobs
        {
            return 0;
        }
        let eligible: Vec<ChunkCoordIntPair> = ELIGIBLE_CHUNKS_FOR_SPAWNING.with(|set| {
            let mut set = set.borrow_mut();
            set.clear();
            for i in 0..world.m_playerEntities.size()
            {
                let entityplayer: Rc<Entity> = world.m_playerEntities.get(i);
                let j: i32 = math_helper::floor_double(entityplayer.posX() / 16.0);
                let k: i32 = math_helper::floor_double(entityplayer.posZ() / 16.0);
                for l in -SPAWN_RADIUS_CHUNKS..=SPAWN_RADIUS_CHUNKS
                {
                    for i1 in -SPAWN_RADIUS_CHUNKS..=SPAWN_RADIUS_CHUNKS
                    {
                        set.add(ChunkCoordIntPair::new(l + j, i1 + k));
                    }
                }
            }
            set.items()
        });
        let mut k3: i32 = 0;
        let chunkcoordinates: ChunkCoordinates = world.getSpawnPoint();
        for enumcreaturetype in EnumCreatureType::VALUES
        {
            let peaceful: bool = enumcreaturetype.getPeacefulCreature();
            if !((!peaceful || spawnPeacefulMobs)
                && (peaceful || spawnHostileMobs)
                && world.countEntities(enumcreaturetype.getCreatureClass())
                    <= enumcreaturetype.getMaxNumberOfCreature() * eligible.len() as i32 / 256)
            {
                continue;
            }
            'chunks: for chunkcoordintpair in &eligible
            {
                let biomegenbase: Rc<BiomeGenBase> =
                    world.getWorldChunkManager().getBiomeGenAtChunkCoord(*chunkcoordintpair);
                let list: Rc<JArrayList<SpawnListEntry>> = biomegenbase.getSpawnableList(enumcreaturetype);
                if list.isEmpty()
                {
                    continue;
                }
                let entries: Vec<SpawnListEntry> = list.toVec();
                let j1: i32 = entries.iter().map(|entry| entry.m_spawnRarityRate).sum();
                let mut l3: i32 = world.m_rand.nextIntBounded(j1);
                let mut spawnEntry: &SpawnListEntry = &entries[0];
                for entry in &entries
                {
                    l3 -= entry.m_spawnRarityRate;
                    if l3 < 0
                    {
                        spawnEntry = entry;
                        break;
                    }
                }
                let chunkposition: ChunkPosition = Self::getRandomSpawningPointInChunk(
                    world,
                    chunkcoordintpair.m_chunkXPos * 16,
                    chunkcoordintpair.m_chunkZPos * 16,
                );
                let i4: i32 = chunkposition.m_x;
                let k1: i32 = chunkposition.m_y;
                let l1: i32 = chunkposition.m_z;
                if world.isBlockNormalCube(i4, k1, l1)
                    || !Rc::ptr_eq(&world.getBlockMaterial(i4, k1, l1), &enumcreaturetype.getCreatureMaterial())
                {
                    continue;
                }
                let mut i2: i32 = 0;
                for _ in 0..3
                {
                    let mut k2: i32 = i4;
                    let mut l2: i32 = k1;
                    let mut i3: i32 = l1;
                    let b1: i32 = 6;
                    for _ in 0..4
                    {
                        k2 += world.m_rand.nextIntBounded(b1) - world.m_rand.nextIntBounded(b1);
                        l2 += world.m_rand.nextIntBounded(1) - world.m_rand.nextIntBounded(1);
                        i3 += world.m_rand.nextIntBounded(b1) - world.m_rand.nextIntBounded(b1);
                        if !Self::canCreatureTypeSpawnAtLocation(enumcreaturetype, world, k2, l2, i3)
                        {
                            continue;
                        }
                        let f: f32 = k2 as f32 + 0.5;
                        let f1: f32 = l2 as f32;
                        let f2: f32 = i3 as f32 + 0.5;
                        if world.getClosestPlayer(f as f64, f1 as f64, f2 as f64, 24.0).is_some()
                        {
                            continue;
                        }
                        let f3: f32 = f - chunkcoordinates.m_x as f32;
                        let f4: f32 = f1 - chunkcoordinates.m_y as f32;
                        let f5: f32 = f2 - chunkcoordinates.m_z as f32;
                        let f6: f32 = f3 * f3 + f4 * f4 + f5 * f5;
                        if f6 < 576.0
                        {
                            continue;
                        }
                        let Some(entityliving) = instantiate(constructorFor(spawnEntry.m_entityClass), world)
                        else
                        {
                            return k3;
                        };
                        entityliving.setLocationAndAngles(
                            f as f64,
                            f1 as f64,
                            f2 as f64,
                            world.m_rand.nextFloat() * 360.0,
                            0.0,
                        );
                        if entityliving.getCanSpawnHere()
                        {
                            i2 += 1;
                            world.entityJoinedWorld(&entityliving);
                            Self::creatureSpecificInit(&entityliving, world, f, f1, f2);
                            if i2 >= entityliving.getMaxSpawnedInChunk()
                            {
                                continue 'chunks;
                            }
                        }
                        k3 += i2;
                    }
                }
            }
        }
        k3
    }

    fn canCreatureTypeSpawnAtLocation(category: EnumCreatureType, world: &World, x: i32, y: i32, z: i32) -> bool
    {
        if Rc::ptr_eq(&category.getCreatureMaterial(), &Material::water())
        {
            return world.getBlockMaterial(x, y, z).getIsLiquid() && !world.isBlockNormalCube(x, y + 1, z);
        }
        world.isBlockNormalCube(x, y - 1, z)
            && !world.isBlockNormalCube(x, y, z)
            && !world.getBlockMaterial(x, y, z).getIsLiquid()
            && !world.isBlockNormalCube(x, y + 1, z)
    }

    fn creatureSpecificInit(entity: &Rc<Entity>, world: &Rc<World>, x: f32, y: f32, z: f32)
    {
        if entity.instanceOf("EntitySpider") && world.m_rand.nextIntBounded(100) == 0
        {
            let entityskeleton: Rc<Entity> = entity_skeleton::new(Some(Rc::clone(world)));
            entityskeleton.setLocationAndAngles(x as f64, y as f64, z as f64, entity.m_rotationYaw.get(), 0.0);
            world.entityJoinedWorld(&entityskeleton);
            entityskeleton.mountEntity(Some(entity));
        }
        else if entity.instanceOf("EntitySheep")
        {
            entity_sheep::setFleeceColor(entity, entity_sheep::getRandomFleeceColor(&world.m_rand));
        }
    }

    pub fn performSleepSpawning(world: &Rc<World>, players: &[Rc<Entity>]) -> bool
    {
        let mut flag: bool = false;
        let pathfinder: Pathfinder = Pathfinder::new(Rc::clone(world) as Rc<dyn IBlockAccess>);
        for entityplayer in players
        {
            let aclass: &[Constructor] = &NIGHT_SPAWN_ENTITIES;
            let mut flag1: bool = false;
            let mut i: i32 = 0;
            while i < 20 && !flag1
            {
                i += 1;
                let j: i32 = math_helper::floor_double(entityplayer.posX()) + world.m_rand.nextIntBounded(32)
                    - world.m_rand.nextIntBounded(32);
                let k: i32 = math_helper::floor_double(entityplayer.posZ()) + world.m_rand.nextIntBounded(32)
                    - world.m_rand.nextIntBounded(32);
                let mut l: i32 = math_helper::floor_double(entityplayer.posY()) + world.m_rand.nextIntBounded(16)
                    - world.m_rand.nextIntBounded(16);
                if l < 1
                {
                    l = 1;
                }
                else if l > 128
                {
                    l = 128;
                }
                let i1: i32 = world.m_rand.nextIntBounded(aclass.len() as i32);
                let mut j1: i32 = l;
                while j1 > 2 && !world.isBlockNormalCube(j, j1 - 1, k)
                {
                    j1 -= 1;
                }
                while !Self::canCreatureTypeSpawnAtLocation(EnumCreatureType::Monster, world, j, j1, k)
                    && j1 < l + 16
                    && j1 < 128
                {
                    j1 += 1;
                }
                if j1 >= l + 16 || j1 >= 128
                {
                    continue;
                }
                let f: f32 = j as f32 + 0.5;
                let f1: f32 = j1 as f32;
                let f2: f32 = k as f32 + 0.5;
                let Some(entityliving) = instantiate(aclass[i1 as usize], world)
                else
                {
                    return flag;
                };
                entityliving.setLocationAndAngles(
                    f as f64,
                    f1 as f64,
                    f2 as f64,
                    world.m_rand.nextFloat() * 360.0,
                    0.0,
                );
                if !entityliving.getCanSpawnHere()
                {
                    continue;
                }
                let pathentity: Option<Rc<PathEntity>> =
                    pathfinder.createEntityPathTo(&entityliving, entityplayer, 32.0);
                let Some(pathentity) = pathentity.filter(|path| path.m_pathLength > 1)
                else
                {
                    continue;
                };
                let pathpoint: Rc<PathPoint> = pathentity.getLastNode().orNpe();
                if (pathpoint.m_xCoord as f64 - entityplayer.posX()).abs() < 1.5
                    && (pathpoint.m_zCoord as f64 - entityplayer.posZ()).abs() < 1.5
                    && (pathpoint.m_yCoord as f64 - entityplayer.posY()).abs() < 1.5
                {
                    let chunkcoordinates: ChunkCoordinates = block_bed::getNearestEmptyChunkCoordinates(
                        world,
                        math_helper::floor_double(entityplayer.posX()),
                        math_helper::floor_double(entityplayer.posY()),
                        math_helper::floor_double(entityplayer.posZ()),
                        1,
                    )
                    .unwrap_or_else(|| ChunkCoordinates::new(j, j1 + 1, k));
                    let x: f32 = chunkcoordinates.m_x as f32 + 0.5;
                    let y: f32 = chunkcoordinates.m_y as f32;
                    let z: f32 = chunkcoordinates.m_z as f32 + 0.5;
                    entityliving.setLocationAndAngles(x as f64, y as f64, z as f64, 0.0, 0.0);
                    world.entityJoinedWorld(&entityliving);
                    Self::creatureSpecificInit(&entityliving, world, x, y, z);
                    entityplayer.wakeUpPlayer(true, false, false);
                    entity_living::playLivingSound(&entityliving);
                    flag = true;
                    flag1 = true;
                }
            }
        }
        flag
    }
}
