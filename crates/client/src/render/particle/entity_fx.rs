use crate::render::Tessellator;
use game::entity::{ENTITY, Entity, EntityClass, EntityExt};
use game::math::math_helper;
use game::world::World;
use jvm::math;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static INTERP_POS_X: Cell<f64> = const { Cell::new(0.0) };
    static INTERP_POS_Y: Cell<f64> = const { Cell::new(0.0) };
    static INTERP_POS_Z: Cell<f64> = const { Cell::new(0.0) };
}

pub fn interpPosX() -> f64 { INTERP_POS_X.with(Cell::get) }

pub fn interpPosY() -> f64 { INTERP_POS_Y.with(Cell::get) }

pub fn interpPosZ() -> f64 { INTERP_POS_Z.with(Cell::get) }

pub fn setInterpPos(x: f64, y: f64, z: f64)
{
    INTERP_POS_X.with(|cell| cell.set(x));
    INTERP_POS_Y.with(|cell| cell.set(y));
    INTERP_POS_Z.with(|cell| cell.set(z));
}

#[derive(Default)]
pub struct FXFields
{
    pub m_particleTextureIndex: Cell<i32>,
    pub m_particleTextureJitterX: Cell<f32>,
    pub m_particleTextureJitterY: Cell<f32>,
    pub m_particleAge: Cell<i32>,
    pub m_particleMaxAge: Cell<i32>,
    pub m_particleScale: Cell<f32>,
    pub m_particleGravity: Cell<f32>,
    pub m_particleRed: Cell<f32>,
    pub m_particleGreen: Cell<f32>,
    pub m_particleBlue: Cell<f32>,
}

pub struct FXExt
{
    pub m_fx: FXFields,
    pub m_extra: Box<dyn Any>,
}

impl EntityExt for FXExt
{
    fn asAny(&self) -> &dyn Any { self }
}

pub fn fx(this: &Entity) -> &FXFields { &this.ext::<FXExt>().m_fx }

pub fn extra<T: 'static>(this: &Entity) -> &T
{
    this.ext::<FXExt>().m_extra.downcast_ref::<T>().expect("particle ext matches its class")
}

fn randomJitter(scale: f32) -> f64 { ((math::random() * 2.0 - 1.0) as f32 * scale) as f64 }

pub fn construct(
    class: &'static EntityClass,
    extra: Box<dyn Any>,
    world: &Rc<World>,
    x: f64,
    y: f64,
    z: f64,
    velocityX: f64,
    velocityY: f64,
    velocityZ: f64,
) -> Rc<Entity>
{
    let this: Rc<Entity> = Entity::alloc(class, Box::new(FXExt { m_fx: FXFields::default(), m_extra: extra }));
    Entity::init(&this, Some(Rc::clone(world)));
    this.setSize(0.2, 0.2);
    this.m_yOffset.set(this.m_height.get() / 2.0);
    this.setPosition(x, y, z);
    let fields: &FXFields = fx(&this);
    fields.m_particleRed.set(1.0);
    fields.m_particleGreen.set(1.0);
    fields.m_particleBlue.set(1.0);
    this.m_motionX.set(velocityX + randomJitter(0.4));
    this.m_motionY.set(velocityY + randomJitter(0.4));
    this.m_motionZ.set(velocityZ + randomJitter(0.4));
    let f: f32 = (math::random() + math::random() + 1.0) as f32 * 0.15;
    let f1: f32 = math_helper::sqrt_double(
        this.m_motionX.get() * this.m_motionX.get()
            + this.m_motionY.get() * this.m_motionY.get()
            + this.m_motionZ.get() * this.m_motionZ.get(),
    );
    this.m_motionX.set(this.m_motionX.get() / f1 as f64 * f as f64 * 0.4f32 as f64);
    this.m_motionY.set(this.m_motionY.get() / f1 as f64 * f as f64 * 0.4f32 as f64 + 0.1f32 as f64);
    this.m_motionZ.set(this.m_motionZ.get() / f1 as f64 * f as f64 * 0.4f32 as f64);
    fields.m_particleTextureJitterX.set(this.m_rand.nextFloat() * 3.0);
    fields.m_particleTextureJitterY.set(this.m_rand.nextFloat() * 3.0);
    fields.m_particleScale.set((this.m_rand.nextFloat() * 0.5 + 0.5) * 2.0);
    fields.m_particleMaxAge.set((4.0 / (this.m_rand.nextFloat() * 0.9 + 0.1)) as i32);
    fields.m_particleAge.set(0);
    this
}

pub fn multiplyVelocity(this: &Rc<Entity>, value: f32) -> Rc<Entity>
{
    this.m_motionX.set(this.m_motionX.get() * value as f64);
    this.m_motionY.set((this.m_motionY.get() - 0.1f32 as f64) * value as f64 + 0.1f32 as f64);
    this.m_motionZ.set(this.m_motionZ.get() * value as f64);
    Rc::clone(this)
}

pub fn multiplySize(this: &Rc<Entity>, value: f32) -> Rc<Entity>
{
    this.setSize(0.2 * value, 0.2 * value);
    let fields: &FXFields = fx(this);
    fields.m_particleScale.set(fields.m_particleScale.get() * value);
    Rc::clone(this)
}

pub fn setPrevPosition(this: &Entity)
{
    this.m_prevPosX.set(this.posX());
    this.m_prevPosY.set(this.posY());
    this.m_prevPosZ.set(this.posZ());
}

pub fn ageParticle(this: &Rc<Entity>)
{
    let fields: &FXFields = fx(this);
    let age: i32 = fields.m_particleAge.get();
    fields.m_particleAge.set(age + 1);
    if age >= fields.m_particleMaxAge.get()
    {
        this.setEntityDead();
    }
}

pub fn scaleMotion(this: &Entity, factor: f64)
{
    this.m_motionX.set(this.m_motionX.get() * factor);
    this.m_motionY.set(this.m_motionY.get() * factor);
    this.m_motionZ.set(this.m_motionZ.get() * factor);
}

pub fn groundFriction(this: &Entity)
{
    if this.m_onGround.get()
    {
        this.m_motionX.set(this.m_motionX.get() * 0.7f32 as f64);
        this.m_motionZ.set(this.m_motionZ.get() * 0.7f32 as f64);
    }
}

pub fn moveByMotion(this: &Rc<Entity>)
{
    this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
}

pub fn growScale(this: &Entity, initialSize: f32, tickDelta: f32)
{
    let fields: &FXFields = fx(this);
    let mut f: f32 = (fields.m_particleAge.get() as f32 + tickDelta) / fields.m_particleMaxAge.get() as f32 * 32.0;
    if f < 0.0
    {
        f = 0.0;
    }
    if f > 1.0
    {
        f = 1.0;
    }
    fields.m_particleScale.set(initialSize * f);
}

pub fn lifeFraction(this: &Entity, tickDelta: f32) -> f32
{
    let fields: &FXFields = fx(this);
    (fields.m_particleAge.get() as f32 + tickDelta) / fields.m_particleMaxAge.get() as f32
}

fn interpolated(this: &Entity, tickDelta: f32) -> (f32, f32, f32)
{
    let delta: f64 = tickDelta as f64;
    (
        (this.m_prevPosX.get() + (this.posX() - this.m_prevPosX.get()) * delta - interpPosX()) as f32,
        (this.m_prevPosY.get() + (this.posY() - this.m_prevPosY.get()) * delta - interpPosY()) as f32,
        (this.m_prevPosZ.get() + (this.posZ() - this.m_prevPosZ.get()) * delta - interpPosZ()) as f32,
    )
}

fn drawQuad(
    this: &Entity,
    target: &dyn std::any::Any,
    tickDelta: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    forwards: f32,
    sideways: f32,
    uv: [f32; 4],
)
{
    // the game crate entity vtable passes the tessellator as Any so port downcasts it
    let tessellator: &Tessellator = crate::port::tessellator(target);
    let [u0, u1, v0, v1]: [f32; 4] = uv;
    let f4: f32 = 0.1 * fx(this).m_particleScale.get();
    let (f5, f6, f7): (f32, f32, f32) = interpolated(this, tickDelta);
    let vertex = |x: f32, y: f32, z: f32, u: f32, v: f32| {
        tessellator.addVertexWithUV(x as f64, y as f64, z as f64, u as f64, v as f64);
    };
    vertex(f5 - dx * f4 - forwards * f4, f6 - dy * f4, f7 - dz * f4 - sideways * f4, u0, v1);
    vertex(f5 - dx * f4 + forwards * f4, f6 + dy * f4, f7 - dz * f4 + sideways * f4, u0, v0);
    vertex(f5 + dx * f4 + forwards * f4, f6 + dy * f4, f7 + dz * f4 + sideways * f4, u1, v0);
    vertex(f5 + dx * f4 - forwards * f4, f6 - dy * f4, f7 + dz * f4 - sideways * f4, u1, v1);
}

pub fn renderParticle(
    this: &Rc<Entity>,
    target: &dyn std::any::Any,
    tickDelta: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    forwards: f32,
    sideways: f32,
)
{
    // the game crate entity vtable passes the tessellator as Any so port downcasts it
    let tessellator: &Tessellator = crate::port::tessellator(target);
    let fields: &FXFields = fx(this);
    let index: i32 = fields.m_particleTextureIndex.get();
    let f: f32 = (index % 16) as f32 / 16.0;
    let f1: f32 = f + 0.0624375;
    let f2: f32 = (index / 16) as f32 / 16.0;
    let f3: f32 = f2 + 0.0624375;
    let f8: f32 = this.getEntityBrightness(tickDelta);
    tessellator.setColorOpaque_F(
        fields.m_particleRed.get() * f8,
        fields.m_particleGreen.get() * f8,
        fields.m_particleBlue.get() * f8,
    );
    drawQuad(this, tessellator, tickDelta, dx, dy, dz, forwards, sideways, [f1, f, f2, f3]);
}

pub fn renderJitteredParticle(
    this: &Rc<Entity>,
    target: &dyn std::any::Any,
    tickDelta: f32,
    dx: f32,
    dy: f32,
    dz: f32,
    forwards: f32,
    sideways: f32,
)
{
    // the game crate entity vtable passes the tessellator as Any so port downcasts it
    let tessellator: &Tessellator = crate::port::tessellator(target);
    let fields: &FXFields = fx(this);
    let index: i32 = fields.m_particleTextureIndex.get();
    let f: f32 = ((index % 16) as f32 + fields.m_particleTextureJitterX.get() / 4.0) / 16.0;
    let f1: f32 = f + 0.015609375;
    let f2: f32 = ((index / 16) as f32 + fields.m_particleTextureJitterY.get() / 4.0) / 16.0;
    let f3: f32 = f2 + 0.015609375;
    let f8: f32 = this.getEntityBrightness(tickDelta);
    tessellator.setColorOpaque_F(
        f8 * fields.m_particleRed.get(),
        f8 * fields.m_particleGreen.get(),
        f8 * fields.m_particleBlue.get(),
    );
    drawQuad(this, tessellator, tickDelta, dx, dy, dz, forwards, sideways, [f, f1, f2, f3]);
}

fn onUpdate(this: &Rc<Entity>)
{
    setPrevPosition(this);
    ageParticle(this);
    this.m_motionY.set(this.m_motionY.get() - 0.04 * fx(this).m_particleGravity.get() as f64);
    moveByMotion(this);
    scaleMotion(this, 0.98f32 as f64);
    groundFriction(this);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityFX",
    m_parent: Some(&ENTITY),
    canTriggerWalking: |_| false,
    entityInit: |_| {},
    onUpdate,
    renderParticle,
    getFXLayer: |_| 0,
    writeEntityToNBT: |_, _| {},
    readEntityFromNBT: |_, _| {},
    ..ENTITY
};
