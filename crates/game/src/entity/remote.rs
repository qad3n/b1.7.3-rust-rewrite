use crate::entity::Entity;
use crate::math::{AxisAlignedBB, math_helper};
use crate::world::World;
use jvm::array_list::JArrayList;
use std::cell::Cell;
use std::rc::Rc;

mod playback
{
    // vanilla's catch up length, used until two updates show how often the server sends this entity
    pub const DEFAULT_INTERVAL: i32 = 3;
    // the tracker sends nothing while an entity rests so a longer gap says nothing about its send rate
    pub const REST_GAP: i32 = 20;
}

mod correction
{
    pub const BLEND_TICKS: i32 = 3;
    // a predicted entity this far off snaps instead of gliding back
    pub const SNAP_DISTANCE: f64 = 4.0;
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

// the latest pose the server sent for an entity this client does not simulate and the ticks left to reach it
#[derive(Default)]
pub struct Track
{
    m_target: Cell<Option<Pose>>,
    m_steps: Cell<i32>,
    m_last: Cell<i32>,
    m_stamped: Cell<bool>,
    m_interval: Cell<i32>,
    m_velocity: Cell<[f64; 3]>,
    m_correction: Cell<[f64; 3]>,
    m_blend: Cell<i32>,
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

    // the entity reaches the new pose just as the server's next update is due, so it never stops or runs ahead
    pub fn push(&self, pose: Pose)
    {
        let (tick, stamped): (i32, bool) = stamp();
        let gap: i32 = tick - self.m_last.get();
        let known: bool = self.m_target.get().is_some() && stamped == self.m_stamped.get();
        if known && gap > 0 && gap <= playback::REST_GAP
        {
            self.m_interval.set(gap);
        }

        self.m_last.set(tick);
        self.m_stamped.set(stamped);
        self.m_target.set(Some(pose));
        if !known || gap > 0
        {
            self.m_steps.set(self.interval());
        }
    }

    pub fn latest(&self) -> Option<Pose> { self.m_target.get() }

    // vehicles and hooks coast on their own physics once they reach the target, as vanilla did
    pub fn playing(&self) -> bool { self.m_target.get().is_some() && self.m_steps.get() > 0 }

    // one tick closer to the target, starting from wherever the entity is now
    pub fn next(&self, this: &Entity) -> Option<Pose> { self.advance(Pose::of(this)) }

    fn advance(&self, from: Pose) -> Option<Pose>
    {
        let target: Pose = self.m_target.get()?;
        let steps: i32 = self.m_steps.get();
        if steps <= 0
        {
            return Some(target);
        }

        self.m_steps.set(steps - 1);
        Some(from.lerp(&target, 1.0 / steps as f64))
    }

    pub fn step(&self, this: &Entity)
    {
        if let Some(pose) = self.next(this)
        {
            let [x, y, z]: [f64; 3] = pose.m_pos;
            this.setPosition(x, y, z);
            this.setRotation(pose.m_yaw, pose.m_pitch);
        }
    }

    // boats carts and hooks dead reckon with the last velocity once they reach the target
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
}

// vehicles and hooks play received poses back as they are and restore the velocity the server sent
pub fn setTarget(this: &Rc<Entity>, x: f64, y: f64, z: f64, yaw: f32, pitch: f32, _steps: i32)
{
    this.m_remote.push(Pose::new(x, y, z, yaw, pitch));
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
