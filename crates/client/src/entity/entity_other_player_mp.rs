use game::entity::entity_living::LivingFields;
use game::entity::entity_player::PlayerFields;
use game::entity::{Entity, EntityClass, EntityExt, entity_living, entity_player};
use game::inventory::InventoryPlayer;
use game::item::ItemStack;
use game::math::math_helper;
use game::world::World;
use jvm::math;
use jvm::string::JString;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Default)]
pub struct OtherPlayerMPExt
{
    m_living: LivingFields,
    m_player: PlayerFields,
    m_lerpSteps: Cell<i32>,
    m_lerpX: Cell<f64>,
    m_lerpY: Cell<f64>,
    m_lerpZ: Cell<f64>,
    m_lerpYaw: Cell<f64>,
    m_lerpPitch: Cell<f64>,
    pub m_field_20924_a: Cell<f32>,
}

impl EntityExt for OtherPlayerMPExt
{
    fn asAny(&self) -> &dyn Any { self }

    fn living(&self) -> Option<&LivingFields> { Some(&self.m_living) }

    fn player(&self) -> Option<&PlayerFields> { Some(&self.m_player) }
}

fn fields(this: &Entity) -> &OtherPlayerMPExt { this.ext::<OtherPlayerMPExt>() }

fn onUpdate(this: &Rc<Entity>)
{
    entity_player::fields(this).m_sleepingCameraOffsetY.set(0.0);
    (entity_player::CLASS.onUpdate)(this);
    let living: &LivingFields = entity_living::fields(this);
    living.m_lastWalkAnimationSpeed.set(living.m_walkAnimationSpeed.get());
    let d: f64 = this.posX() - this.m_prevPosX.get();
    let d2: f64 = this.posZ() - this.m_prevPosZ.get();
    let mut f: f32 = math_helper::sqrt_double(d * d + d2 * d2) * 4.0;
    if f > 1.0
    {
        f = 1.0;
    }
    living.m_walkAnimationSpeed.set(living.m_walkAnimationSpeed.get() + (f - living.m_walkAnimationSpeed.get()) * 0.4);
    living.m_walkAnimationProgress.set(living.m_walkAnimationProgress.get() + living.m_walkAnimationSpeed.get());
}

fn onLivingUpdate(this: &Rc<Entity>)
{
    (entity_player::CLASS.updatePlayerActionState)(this);
    let ext: &OtherPlayerMPExt = fields(this);
    let lerpSteps: i32 = ext.m_lerpSteps.get();
    if lerpSteps > 0
    {
        let d2: f64 = this.posX() + (ext.m_lerpX.get() - this.posX()) / lerpSteps as f64;
        let d3: f64 = this.posY() + (ext.m_lerpY.get() - this.posY()) / lerpSteps as f64;
        let d4: f64 = this.posZ() + (ext.m_lerpZ.get() - this.posZ()) / lerpSteps as f64;
        let mut d: f64 = ext.m_lerpYaw.get() - this.m_rotationYaw.get() as f64;
        while d < -180.0
        {
            d += 360.0;
        }
        while d >= 180.0
        {
            d -= 360.0;
        }
        this.m_rotationYaw.set((this.m_rotationYaw.get() as f64 + d / lerpSteps as f64) as f32);
        this.m_rotationPitch.set(
            (this.m_rotationPitch.get() as f64
                + (ext.m_lerpPitch.get() - this.m_rotationPitch.get() as f64) / lerpSteps as f64) as f32,
        );
        ext.m_lerpSteps.set(lerpSteps - 1);
        this.setPosition(d2, d3, d4);
        this.setRotation(this.m_rotationYaw.get(), this.m_rotationPitch.get());
    }
    let player: &PlayerFields = entity_player::fields(this);
    let living: &LivingFields = entity_living::fields(this);
    player.m_lastBob.set(player.m_bob.get());
    let mut f: f32 = math_helper::sqrt_double(
        this.m_motionX.get() * this.m_motionX.get() + this.m_motionZ.get() * this.m_motionZ.get(),
    );
    let mut f2: f32 = math::atan(-this.m_motionY.get() * 0.2f32 as f64) as f32 * 15.0;
    if f > 0.1
    {
        f = 0.1;
    }
    if !this.m_onGround.get() || living.m_health.get() <= 0
    {
        f = 0.0;
    }
    if this.m_onGround.get() || living.m_health.get() <= 0
    {
        f2 = 0.0;
    }
    player.m_bob.set(player.m_bob.get() + (f - player.m_bob.get()) * 0.4);
    living.m_tilt.set(living.m_tilt.get() + (f2 - living.m_tilt.get()) * 0.8);
}

fn outfitWithItem(this: &Rc<Entity>, slot: i32, item: i32, metadata: i32)
{
    let mut itemStack: Option<Rc<ItemStack>> = None;
    if item >= 0
    {
        itemStack = Some(ItemStack::withDamage(item, 1, metadata));
    }
    let inventory: Rc<InventoryPlayer> = entity_player::inventory(this);
    if slot == 0
    {
        let currentItem: usize = inventory.m_currentItem.get() as usize;
        inventory.m_mainInventory.borrow_mut()[currentItem] = itemStack;
    }
    else
    {
        inventory.m_armorInventory.borrow_mut()[(slot - 1) as usize] = itemStack;
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityOtherPlayerMP",
    m_parent: Some(&entity_player::CLASS),
    resetHeight: |this| this.m_yOffset.set(0.0),
    attackEntityFrom: |_, _, _| true,
    setPositionAndRotation2: |this, x, y, z, yaw, pitch, steps| {
        let ext: &OtherPlayerMPExt = fields(this);
        ext.m_lerpX.set(x);
        ext.m_lerpY.set(y);
        ext.m_lerpZ.set(z);
        ext.m_lerpYaw.set(yaw as f64);
        ext.m_lerpPitch.set(pitch as f64);
        ext.m_lerpSteps.set(steps);
    },
    onUpdate,
    getShadowSize: |_| 0.0,
    onLivingUpdate,
    outfitWithItem,
    addRespawnParticles: |_| {},
    ..entity_player::CLASS
};

pub fn new(world: &Rc<World>, name: Option<JString>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(OtherPlayerMPExt::default()));
    entity_player::init(&entity, Rc::clone(world));
    let player: &PlayerFields = entity_player::fields(&entity);
    player.m_username.set(name.clone());
    entity.m_yOffset.set(0.0);
    entity.m_stepHeight.set(0.0);
    if let Some(name) = &name
        && name.length() > 0
    {
        entity
            .m_skinUrl
            .set(Some(JString::from(format!("http://s3.amazonaws.com/MinecraftSkins/{name}.png").as_str())));
    }
    entity.m_noClip.set(true);
    player.m_sleepingCameraOffsetY.set(0.25);
    entity.m_renderDistanceWeight.set(10.0);
    entity
}
