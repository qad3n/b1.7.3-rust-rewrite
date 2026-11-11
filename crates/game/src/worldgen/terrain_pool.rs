use crate::worldgen::{chunk_provider_generate, chunk_provider_hell};
use jvm::{debug, warning};
use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet, VecDeque};
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock};
use std::thread::JoinHandle;

mod limits
{
    // queued plus finished chunks, each finished one holds 32 KiB of blocks
    pub const SLOTS: usize = 512;
    // chunks remembered as handed out so prefetch never makes them twice
    pub const SEEN: usize = 1 << 20;
    pub const WORKERS: usize = 6;
    pub const PREFETCH_RADIUS: i32 = 2;
    // generators a worker keeps for the seeds it serves
    pub const SOURCES: usize = 4;
    pub const STACK_BYTES: usize = 16 * 1024 * 1024;
}

type Sources = HashMap<(TerrainKind, i64), Box<dyn TerrainSource>>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TerrainKind
{
    Surface,
    Nether,
}

// a chunk's blocks before population plus the generator random state after making them
pub(crate) struct Generated
{
    pub(crate) m_blocks: Vec<i8>,
    pub(crate) m_rand: i64,
}

pub(crate) trait TerrainSource
{
    fn generate(&self, chunkX: i32, chunkZ: i32) -> Generated;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Job
{
    m_kind: TerrainKind,
    m_seed: i64,
    m_x: i32,
    m_z: i32,
}

enum Slot
{
    Queued(u64),
    Running,
    Done(u64, Generated),
    Failed,
}

#[derive(Default)]
struct State
{
    m_queue: VecDeque<Job>,
    m_slots: HashMap<Job, Slot>,
    m_seen: HashSet<Job>,
    m_stamp: u64,
}

impl State
{
    fn stamp(&mut self) -> u64
    {
        self.m_stamp += 1;
        self.m_stamp
    }

    fn age(slot: &Slot) -> Option<u64>
    {
        match slot
        {
            Slot::Queued(stamp) | Slot::Done(stamp, _) => Some(*stamp),
            Slot::Running | Slot::Failed => None,
        }
    }

    // drop the oldest waiting or unclaimed chunk to make room
    fn evict(&mut self) -> bool
    {
        let oldest: Option<Job> = self
            .m_slots
            .iter()
            .filter_map(|(job, slot)| Self::age(slot).map(|age| (age, *job)))
            .min_by_key(|(age, _)| *age)
            .map(|(_, job)| job);

        let Some(job) = oldest
        else
        {
            return false;
        };

        if let Some(Slot::Queued(_)) = self.m_slots.remove(&job)
        {
            self.m_queue.retain(|queued| *queued != job);
        }

        self.m_seen.remove(&job);
        true
    }

    fn prefetch(&mut self, center: Job)
    {
        if self.m_seen.len() >= limits::SEEN
        {
            self.m_seen.clear();
        }

        let radius: i32 = limits::PREFETCH_RADIUS;
        let mut ring: Vec<Job> = Vec::new();

        for dx in -radius..=radius
        {
            for dz in -radius..=radius
            {
                let job: Job = Job { m_x: center.m_x + dx, m_z: center.m_z + dz, ..center };
                if !self.m_seen.contains(&job)
                {
                    ring.push(job);
                }
            }
        }
        // nearest first once pushed to the front in reverse
        ring.sort_by_key(|job| -((job.m_x - center.m_x).abs().max((job.m_z - center.m_z).abs())));

        for job in ring
        {
            if self.m_slots.len() >= limits::SLOTS && !self.evict()
            {
                return;
            }

            let stamp: u64 = self.stamp();

            self.m_seen.insert(job);
            self.m_slots.insert(job, Slot::Queued(stamp));
            self.m_queue.push_front(job);
        }
    }
}

struct Pool
{
    m_state: Mutex<State>,
    m_changed: Condvar,
}

impl Pool
{
    fn lock(&self) -> MutexGuard<'_, State> { self.m_state.lock().expect("terrain pool lock is never poisoned") }

    fn work(&self)
    {
        let sources: RefCell<Sources> = RefCell::new(HashMap::new());

        loop
        {
            let job: Job = self.next();
            let result: Result<Generated, Box<dyn std::any::Any + Send>> =
                panic::catch_unwind(AssertUnwindSafe(||
                {
                    let mut sources: RefMut<Sources> = sources.borrow_mut();
                    if sources.len() >= limits::SOURCES && !sources.contains_key(&(job.m_kind, job.m_seed))
                    {
                        sources.clear();
                    }

                    sources
                        .entry((job.m_kind, job.m_seed))
                        .or_insert_with(|| source(job.m_kind, job.m_seed))
                        .generate(job.m_x, job.m_z)
                }));

            let mut state: MutexGuard<State> = self.lock();

            let slot: Slot = match result
            {
                Ok(generated) => Slot::Done(state.stamp(), generated),

                Err(_) =>
                {
                    warning!(
                        "Terrain worker failed on chunk {},{}, the main thread makes it instead",
                        job.m_x,
                        job.m_z
                    );
                    sources.borrow_mut().clear();
                    Slot::Failed
                }
            };
            state.m_slots.insert(job, slot);
            drop(state);
            self.m_changed.notify_all();
        }
    }

    fn next(&self) -> Job
    {
        let mut state: MutexGuard<State> = self.lock();

        loop
        {
            if let Some(job) = state.m_queue.pop_front()
            {
                state.m_slots.insert(job, Slot::Running);
                return job;
            }

            state = self.m_changed.wait(state).expect("terrain pool lock is never poisoned");
        }
    }

    fn take(&self, job: Job) -> Option<Generated>
    {
        let mut state: MutexGuard<State> = self.lock();

        let generated: Option<Generated> = loop
        {
            match state.m_slots.remove(&job)
            {
                Some(Slot::Done(_, generated)) => break Some(generated),

                Some(Slot::Running) =>
                {
                    state.m_slots.insert(job, Slot::Running);
                    state = self.m_changed.wait(state).expect("terrain pool lock is never poisoned");
                }

                Some(Slot::Queued(_)) =>
                {
                    state.m_queue.retain(|queued| *queued != job);
                    break None;
                }

                Some(Slot::Failed) | None => break None,
            }
        };

        state.m_seen.insert(job);
        state.prefetch(job);
        drop(state);
        self.m_changed.notify_all();
        generated
    }
}

fn source(kind: TerrainKind, seed: i64) -> Box<dyn TerrainSource>
{
    match kind
    {
        TerrainKind::Surface => chunk_provider_generate::source(seed),
        TerrainKind::Nether => chunk_provider_hell::source(seed),
    }
}

fn workers() -> usize
{
    let threads: usize = std::thread::available_parallelism().map_or(1, usize::from);
    (threads - 1).min(limits::WORKERS)
}

fn start() -> Option<Arc<Pool>>
{
    let workers: usize = workers();
    if workers == 0
    {
        return None;
    }

    debug!("Starting {workers} terrain workers");

    let pool: Arc<Pool> = Arc::new(Pool { m_state: Mutex::new(State::default()), m_changed: Condvar::new() });

    for i in 0..workers
    {
        let worker: Arc<Pool> = Arc::clone(&pool);
        let spawned: std::io::Result<JoinHandle<()>> = std::thread::Builder::new()
            .name(format!("Terrain worker {i}"))
            .stack_size(limits::STACK_BYTES)
            .spawn(move || worker.work());

        if let Err(error) = spawned
        {
            warning!("Could not start terrain worker {i}: {error}");
        }
    }

    Some(pool)
}

fn pool() -> Option<&'static Arc<Pool>>
{
    static POOL: OnceLock<Option<Arc<Pool>>> = OnceLock::new();
    POOL.get_or_init(start).as_ref()
}

// terrain made ahead on a worker, or none when the caller has to make it itself
pub(crate) fn take(kind: TerrainKind, seed: i64, chunkX: i32, chunkZ: i32) -> Option<Generated>
{
    pool()?.take(Job { m_kind: kind, m_seed: seed, m_x: chunkX, m_z: chunkZ })
}
