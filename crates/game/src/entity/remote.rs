use crate::entity::Entity;
use crate::math::{AxisAlignedBB, math_helper};
use crate::world::World;
use jvm::array_list::JArrayList;
use std::cell::Cell;
use std::rc::Rc;

mod playback
{
    pub const CAPACITY: usize = 4;
    // spacing assumed until two moving samples show the real send interval
    pub const DEFAULT_INTERVAL: i32 = 3;
    // extra ticks of buffer for packets that arrive a tick late or early
    pub const STAMPED_JITTER: i32 = 1;
    pub const ARRIVAL_JITTER: i32 = 2;
    // clock drift is nudged away slowly and jumped once it gets this far
    pub const RESYNC: f64 = 8.0;
    pub const DRIFT_RATE: f64 = 0.1;
    pub const DRIFT_LIMIT: f64 = 0.2;
}

mod correction
{
    pub const BLEND_TICKS: i32 = 3;
    // a predicted entity this far off snaps instead of gliding back
    pub const SNAP_DISTANCE: f64 = 4.0;
    // knockback strong enough to show right away instead of after the buffer delay
    pub const KICK_SPEED: f64 = 0.3;
}

mod placement
{
    // slabs stairs and rounding lift a box onto the block below it but walls never do
    pub const STEP_LIMIT: f64 = 0.6;
    // matches the horizontal slack vanilla gave received positions
    pub const BORDER: f64 = 0.03125;
    pub const SETTLE: f64 = 0.01;
}

thread_local! {
    static CLIENT_TICK: Cell<i32> = const { Cell::new(0) };
    static SERVER_TICK: Cell<Option<i32>> = const { Cell::new(None) };
}

pub fn tick() { CLIENT_TICK.with(|tick| tick.set(tick.get() + 1)); }

// set by the loader tick channel so samples carry the server tick they were sent on
pub fn setServerTick(tick: Option<i32>) { SERVER_TICK.with(|cell| cell.set(tick)); }

fn clientTick() -> i32 { CLIENT_TICK.with(Cell::get) }

fn stamp() -> (i32, bool)
{
    match SERVER_TICK.with(Cell::get)
    {
        Some(tick) => (tick, true),
        None => (clientTick(), false),
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Pose
{
    pub m_pos: [f64; 3],
    pub m_yaw: f32,
    pub m_pitch: f32,
}

impl Pose
{
    pub fn new(x: f64, y: f64, z: f64, yaw: f32, pitch: f32) -> Pose
    {
        Pose { m_pos: [x, y, z], m_yaw: yaw, m_pitch: pitch }
    }

    pub fn of(entity: &Entity) -> Pose
    {
        Pose::new(entity.posX(), entity.posY(), entity.posZ(), entity.m_rotationYaw.get(), entity.m_rotationPitch.get())
    }

    fn lerp(&self, to: &Pose, t: f64) -> Pose
    {
        Pose {
            m_pos: math_helper::lerp3(self.m_pos, to.m_pos, t),
            m_yaw: math_helper::lerpDegrees(self.m_yaw, to.m_yaw, t),
            m_pitch: math_helper::lerpDegrees(self.m_pitch, to.m_pitch, t),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Sample
{
    m_tick: i32,
    m_pose: Pose,
}

// positions the server sent for an entity this client does not simulate, played back a few ticks late
#[derive(Default)]
pub struct Track
{
    m_samples: Cell<[Sample; playback::CAPACITY]>,
    m_len: Cell<usize>,
    m_clock: Cell<f64>,
    m_arrival: Cell<i32>,
    m_interval: Cell<i32>,
    m_stamped: Cell<bool>,
    m_velocity: Cell<[f64; 3]>,
    m_correction: Cell<[f64; 3]>,
    m_blend: Cell<i32>,
    m_impulse: Cell<i32>,
    m_kicked: Cell<bool>,
    m_drift: Cell<[f64; 3]>,
    m_driftTicks: Cell<i32>,
}

impl Track
{
    pub fn new() -> Track { Track::default() }

    fn interval(&self) -> i32
    {
        if self.m_interval.get() > 0
        {
            self.m_interval.get()
        }
        else
        {
            playback::DEFAULT_INTERVAL
        }
    }

    fn delay(&self) -> i32
    {
        let jitter: i32 = if self.m_stamped.get()
        {
            playback::STAMPED_JITTER
        }
        else
        {
            playback::ARRIVAL_JITTER
        };
        self.interval() + jitter
    }

    pub fn clear(&self) { self.m_len.set(0); }

    pub fn push(&self, current: Pose, pose: Pose)
    {
        let (tick, stamped): (i32, bool) = stamp();
        let mut samples: [Sample; playback::CAPACITY] = self.m_samples.get();
        let mut len: usize = self.m_len.get();
        if stamped != self.m_stamped.get()
        {
            len = 0;
            self.m_stamped.set(stamped);
        }

        self.m_arrival.set(clientTick());
        // once playback ran out the entity coasted or rested on its own, so the next leg starts where it is now
        let finished: bool = len == 0 || self.m_clock.get() >= samples[len - 1].m_tick as f64;
        if finished
        {
            // starting one buffer delay back makes the new leg play at once with no pause
            samples[0] = Sample { m_tick: tick - self.delay(), m_pose: current };
            len = 1;
            self.m_clock.set(samples[0].m_tick as f64);
        }

        let last: Sample = samples[len - 1];
        let gap: i32 = tick - last.m_tick;
        if gap <= 0
        {
            samples[len - 1].m_pose = pose;
            self.m_samples.set(samples);
            self.m_len.set(len);
            return;
        }

        let interval: i32 = self.interval();
        self.m_interval.set(
            if gap < interval
            {
                gap
            }
            else
            {
                interval + (gap > interval) as i32
            },
        );

        if len == playback::CAPACITY
        {
            samples.copy_within(1.., 0);
            len -= 1;
        }

        samples[len] = Sample { m_tick: tick, m_pose: pose };
        self.m_samples.set(samples);
        self.m_len.set(len + 1);
    }

    pub fn latest(&self) -> Option<Pose>
    {
        let len: usize = self.m_len.get();
        if len == 0
        {
            return None;
        }

        Some(self.m_samples.get()[len - 1].m_pose)
    }

    // vehicles and hooks coast on their own physics once playback runs out, as vanilla did
    pub fn playing(&self) -> bool
    {
        let len: usize = self.m_len.get();
        len > 0 && self.m_clock.get() < self.m_samples.get()[len - 1].m_tick as f64
    }

    // advances the playback clock one tick and returns where the entity should be
    pub fn next(&self) -> Option<Pose>
    {
        let len: usize = self.m_len.get();
        if len == 0
        {
            return None;
        }

        let samples: [Sample; playback::CAPACITY] = self.m_samples.get();
        let newest: i32 = samples[len - 1].m_tick;
        let now: i32 = newest + clientTick() - self.m_arrival.get();
        let target: f64 = (now - self.delay()) as f64;
        let error: f64 = target - self.m_clock.get();
        let clock: f64 = if error.abs() > playback::RESYNC
        {
            target
        }
        else
        {
            self.m_clock.get()
                + 1.0
                + (error * playback::DRIFT_RATE).clamp(-playback::DRIFT_LIMIT, playback::DRIFT_LIMIT)
        };
        let clock: f64 = clock.clamp(samples[0].m_tick as f64, newest as f64);
        self.m_clock.set(clock);

        Some(sampleAt(&samples[..len], clock))
    }

    pub fn step(&self, this: &Entity)
    {
        if let Some(pose) = self.next()
        {
            let [x, y, z]: [f64; 3] = pose.m_pos;
            this.setPosition(x, y, z);
            this.setRotation(pose.m_yaw, pose.m_pitch);
        }
    }

    // boats carts and hooks dead reckon with the last velocity once playback runs out
    pub fn setVelocity(&self, this: &Entity, x: f64, y: f64, z: f64)
    {
        self.m_velocity.set([x, y, z]);
        self.restoreVelocity(this);
    }

    pub fn restoreVelocity(&self, this: &Entity)
    {
        let [x, y, z]: [f64; 3] = self.m_velocity.get();
        this.m_motionX.set(x);
        this.m_motionY.set(y);
        this.m_motionZ.set(z);
    }

    // a predicted entity glides to the server position over a few ticks, false when it is too far off
    pub fn correct(&self, current: [f64; 3], target: [f64; 3]) -> bool
    {
        let error: [f64; 3] = std::array::from_fn(|i| target[i] - current[i]);
        if error.iter().map(|axis| axis * axis).sum::<f64>() > correction::SNAP_DISTANCE * correction::SNAP_DISTANCE
        {
            self.m_blend.set(0);
            return false;
        }

        self.m_correction.set(error.map(|axis| axis / correction::BLEND_TICKS as f64));
        self.m_blend.set(correction::BLEND_TICKS);
        true
    }

    pub fn blend(&self, this: &Entity)
    {
        if self.m_blend.get() <= 0
        {
            return;
        }

        self.m_blend.set(self.m_blend.get() - 1);
        let [dx, dy, dz]: [f64; 3] = self.m_correction.get();
        this.setPosition(this.posX() + dx, this.posY() + dy, this.posZ() + dz);
    }

    pub fn rein(&self, this: &Rc<Entity>)
    {
        let Some(latest) = self.latest()
        else
        {
            return;
        };

        let [x, y, z]: [f64; 3] = latest.m_pos;
        let (dx, dy, dz): (f64, f64, f64) = (x - this.posX(), y - this.posY(), z - this.posZ());
        if dx * dx + dy * dy + dz * dz > correction::SNAP_DISTANCE * correction::SNAP_DISTANCE
        {
            place(this, &latest);
        }
    }

    // knockback runs local physics for one buffer delay so a hit shows at once
    pub fn kick(&self, motion: [f64; 3])
    {
        let horizontal: f64 = (motion[0] * motion[0] + motion[2] * motion[2]).sqrt();
        if horizontal < correction::KICK_SPEED && motion[1] < correction::KICK_SPEED
        {
            return;
        }

        self.m_impulse.set(self.delay());
    }

    pub fn takeImpulse(&self) -> bool
    {
        if self.m_impulse.get() <= 0
        {
            return false;
        }

        self.m_impulse.set(self.m_impulse.get() - 1);
        self.m_kicked.set(true);
        true
    }

    // after a hit the buffer still shows the moment before it, so the predicted lead fades out while the buffer catches up
    pub fn ease(&self, this: &Entity, pose: Pose) -> Pose
    {
        let delay: i32 = self.delay();
        if self.m_kicked.replace(false)
        {
            let current: [f64; 3] = [this.posX(), this.posY(), this.posZ()];
            self.m_drift.set(std::array::from_fn(|i| current[i] - pose.m_pos[i]));
            self.m_driftTicks.set(delay);
        }

        let ticks: i32 = self.m_driftTicks.get();
        if ticks <= 0
        {
            return pose;
        }

        self.m_driftTicks.set(ticks - 1);
        let share: f64 = ticks as f64 / delay as f64;
        let drift: [f64; 3] = self.m_drift.get();
        Pose { m_pos: std::array::from_fn(|i| pose.m_pos[i] + drift[i] * share), ..pose }
    }
}

fn sampleAt(samples: &[Sample], clock: f64) -> Pose
{
    for pair in samples.windows(2)
    {
        let (from, to): (Sample, Sample) = (pair[0], pair[1]);
        if clock <= to.m_tick as f64
        {
            let span: f64 = (to.m_tick - from.m_tick) as f64;
            let t: f64 = ((clock - from.m_tick as f64) / span).clamp(0.0, 1.0);
            return from.m_pose.lerp(&to.m_pose, t);
        }
    }

    samples[samples.len() - 1].m_pose
}

// vehicles and hooks play received poses back as they are and restore the velocity the server sent
pub fn setTarget(this: &Rc<Entity>, x: f64, y: f64, z: f64, yaw: f32, pitch: f32, _steps: i32)
{
    this.m_remote.push(Pose::of(this), Pose::new(x, y, z, yaw, pitch));
    this.m_remote.restoreVelocity(this);
}

// puts an entity at a received pose without letting it sit inside walls
pub fn place(this: &Rc<Entity>, pose: &Pose)
{
    let world: Rc<World> = this.world();
    let [x, y, z]: [f64; 3] = pose.m_pos;
    let (dx, dy, dz): (f64, f64, f64) = (x - this.posX(), y - this.posY(), z - this.posZ());
    let moved: Rc<AxisAlignedBB> = this.m_boundingBox.getOffsetBoundingBox(dx, dy, dz);
    let lift: f64 = clearance(this, &world, &moved);
    if lift <= placement::STEP_LIMIT
    {
        this.setPosition(x, y + lift, z);
    }
    else
    {
        let [sx, sy, sz]: [f64; 3] = sweep(this, &world, dx, dy, dz);
        this.setPosition(this.posX() + sx, this.posY() + sy, this.posZ() + sz);
    }

    this.setRotation(pose.m_yaw, pose.m_pitch);
}

// moves a mob through its own collision so steps sounds and ground state stay as before
pub fn follow(this: &Rc<Entity>, pose: &Pose)
{
    let [x, y, z]: [f64; 3] = pose.m_pos;
    this.moveEntity(x - this.posX(), y - this.posY(), z - this.posZ());
    this.setRotation(pose.m_yaw, pose.m_pitch);

    let off: f64 = (x - this.posX()).abs() + (y - this.posY()).abs() + (z - this.posZ()).abs();
    if off <= placement::SETTLE
    {
        return;
    }

    // the server path went where this client world disagrees so the server wins once the spot is free
    let world: Rc<World> = this.world();
    let target: Rc<AxisAlignedBB> =
        this.m_boundingBox.getOffsetBoundingBox(x - this.posX(), y - this.posY(), z - this.posZ());
    if clearance(this, &world, &target) == 0.0
    {
        this.setPosition(x, y, z);
    }
}

// how far a box must rise to clear the blocks it overlaps
fn clearance(this: &Rc<Entity>, world: &Rc<World>, shape: &AxisAlignedBB) -> f64
{
    let boxes: Rc<JArrayList<Rc<AxisAlignedBB>>> =
        world.getCollidingBoundingBoxes(this, &shape.contract(placement::BORDER, 0.0, placement::BORDER));
    let mut top: f64 = shape.minY();
    for i in 0..boxes.size()
    {
        top = top.max(boxes.get(i).maxY());
    }

    top - shape.minY()
}

fn sweep(this: &Rc<Entity>, world: &Rc<World>, dx: f64, dy: f64, dz: f64) -> [f64; 3]
{
    let start: Rc<AxisAlignedBB> = this.m_boundingBox.copy();
    let boxes: Rc<JArrayList<Rc<AxisAlignedBB>>> = world.getCollidingBoundingBoxes(this, &start.addCoord(dx, dy, dz));
    let solids: Vec<Rc<AxisAlignedBB>> = boxes.toVec();

    let mut dy: f64 = dy;
    for solid in &solids
    {
        dy = solid.calculateYOffset(&start, dy);
    }
    let shape: Rc<AxisAlignedBB> = start.getOffsetBoundingBox(0.0, dy, 0.0);

    let mut dx: f64 = dx;
    for solid in &solids
    {
        dx = solid.calculateXOffset(&shape, dx);
    }
    let shape: Rc<AxisAlignedBB> = shape.getOffsetBoundingBox(dx, 0.0, 0.0);

    let mut dz: f64 = dz;
    for solid in &solids
    {
        dz = solid.calculateZOffset(&shape, dz);
    }

    [dx, dy, dz]
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn pose(x: f64) -> Pose { Pose::new(x, 64.0, 0.0, 0.0, 0.0) }

    fn advance(ticks: i32)
    {
        for _ in 0..ticks
        {
            tick();
        }
    }

    #[test]
    fn playbackMovesEvenlyBetweenSamples()
    {
        setServerTick(None);
        let track: Track = Track::new();
        track.push(pose(0.0), pose(3.0));
        let mut seen: Vec<f64> = Vec::new();
        for step in 1..=12
        {
            advance(1);
            if step % 3 == 0
            {
                track.push(pose(0.0), pose(3.0 + step as f64));
            }
            seen.push(track.next().map(|pose| pose.m_pos[0]).unwrap_or_default());
        }

        let steps: Vec<f64> = seen.windows(2).map(|pair| pair[1] - pair[0]).collect();
        assert!(steps.iter().all(|step| *step >= 0.0), "never moves backwards {seen:?}");
        assert!(steps.iter().skip(4).all(|step| *step <= 1.5), "no jumps once buffered {seen:?}");
    }

    #[test]
    fn starvedPlaybackHoldsAtNewest()
    {
        setServerTick(None);
        let track: Track = Track::new();
        track.push(pose(0.0), pose(2.0));
        advance(30);
        let held: Option<Pose> = (0..5).map(|_| track.next()).last().flatten();
        assert_eq!(held, Some(pose(2.0)));
        assert!(!track.playing());
    }

    #[test]
    fn nextLegStartsWhereCoastingEnded()
    {
        setServerTick(None);
        let track: Track = Track::new();
        track.push(pose(0.0), pose(2.0));
        advance(30);
        track.next();
        track.push(pose(5.0), pose(6.0));
        let first: Pose = track.next().unwrap_or_default();
        assert!((5.0..=6.0).contains(&first.m_pos[0]), "no snap back to the old sample {first:?}");
    }

    #[test]
    fn restartAfterRestStartsFromRestingPose()
    {
        setServerTick(None);
        let track: Track = Track::new();
        track.push(pose(0.0), pose(1.0));
        advance(100);
        track.next();
        track.push(pose(1.0), pose(2.0));
        let first: Pose = track.next().unwrap_or_default();
        assert!(first.m_pos[0] <= 1.5, "eases out of rest instead of jumping {first:?}");
    }

    #[test]
    fn stampedSamplesUseServerTicks()
    {
        let track: Track = Track::new();
        setServerTick(Some(1000));
        track.push(pose(0.0), pose(1.0));
        setServerTick(Some(1003));
        track.push(pose(0.0), pose(2.0));
        assert_eq!(track.interval(), 3);
        setServerTick(None);
    }

    #[test]
    fn correctionSnapsWhenFarOff()
    {
        let track: Track = Track::new();
        assert!(track.correct([0.0; 3], [1.0, 0.0, 0.0]));
        assert!(!track.correct([0.0; 3], [10.0, 0.0, 0.0]));
    }

    #[test]
    fn onlyStrongHitsKick()
    {
        let track: Track = Track::new();
        track.kick([0.05, 0.0, 0.05]);
        assert!(!track.takeImpulse());
        track.kick([0.4, 0.4, 0.0]);
        assert!(track.takeImpulse());
    }
}
