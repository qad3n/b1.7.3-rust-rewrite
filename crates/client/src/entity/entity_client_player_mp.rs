use crate::client::{Minecraft, Session};
use crate::entity::entity_player_sp::{self, PlayerSPExt};
use crate::network::net_client_handler::NetClientHandler;
use game::entity::entity_living::LivingFields;
use game::entity::entity_player::PlayerFields;
use game::entity::{Entity, EntityClass, entity_living, entity_player};
use game::math::math_helper;
use game::network::packets::{
    Packet3Chat, Packet9Respawn, Packet10Flying, Packet11PlayerPosition, Packet12PlayerLook, Packet13PlayerLookMove,
    Packet14BlockDig, Packet18Animation, Packet19EntityAction, Packet101CloseWindow,
};
use game::stats::StatBase;
use game::world::World;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct ClientPlayerMPFields
{
    pub m_sendQueue: Rc<NetClientHandler>,
    m_ticksSinceSendInventory: Cell<i32>,
    m_healthInitialized: Cell<bool>,
    m_oldPosX: Cell<f64>,
    m_sentMinY: Cell<f64>,
    m_oldPosY: Cell<f64>,
    m_oldPosZ: Cell<f64>,
    m_oldRotationYaw: Cell<f32>,
    m_oldRotationPitch: Cell<f32>,
    m_sentOnGround: Cell<bool>,
    m_wasSneaking: Cell<bool>,
    m_ticksSinceSendOnGround: Cell<i32>,
}

pub fn fields(this: &Entity) -> &ClientPlayerMPFields
{
    match &this.ext::<PlayerSPExt>().m_mp
    {
        Some(fields) => fields,
        None => jvm::throwable::throw(
            &jvm::throwable::CLASS_CAST,
            &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.EntityClientPlayerMP", this.className()),
        ),
    }
}

fn sendQueue(this: &Entity) -> &Rc<NetClientHandler> { &fields(this).m_sendQueue }

fn onUpdate(this: &Rc<Entity>)
{
    if !this.world().blockExists(math_helper::floor_double(this.posX()), 64, math_helper::floor_double(this.posZ()))
    {
        return;
    }
    (entity_player_sp::CLASS.onUpdate)(this);
    sendMovement(this);
}

pub fn sendMovement(this: &Rc<Entity>)
{
    let mp: &ClientPlayerMPFields = fields(this);
    let ticksSinceSendInventory: i32 = mp.m_ticksSinceSendInventory.get();
    mp.m_ticksSinceSendInventory.set(ticksSinceSendInventory + 1);
    if ticksSinceSendInventory == 20
    {
        sendInventoryChanged(this);
        mp.m_ticksSinceSendInventory.set(0);
    }
    let sneaking: bool = this.isSneaking();
    if sneaking != mp.m_wasSneaking.get()
    {
        if sneaking
        {
            mp.m_sendQueue.addToSendQueue(Box::new(Packet19EntityAction::new(this, 1)));
        }
        else
        {
            mp.m_sendQueue.addToSendQueue(Box::new(Packet19EntityAction::new(this, 2)));
        }
        mp.m_wasSneaking.set(sneaking);
    }
    let minY: f64 = this.m_boundingBox.minY();
    let n: f64 = this.posX() - mp.m_oldPosX.get();
    let n2: f64 = minY - mp.m_sentMinY.get();
    let n3: f64 = this.posY() - mp.m_oldPosY.get();
    let n4: f64 = this.posZ() - mp.m_oldPosZ.get();
    let n5: f64 = (this.m_rotationYaw.get() - mp.m_oldRotationYaw.get()) as f64;
    let n6: f64 = (this.m_rotationPitch.get() - mp.m_oldRotationPitch.get()) as f64;
    let mut b: bool = n2 != 0.0 || n3 != 0.0 || n != 0.0 || n4 != 0.0;
    let b2: bool = n5 != 0.0 || n6 != 0.0;
    let yaw: f32 = this.m_rotationYaw.get();
    let pitch: f32 = this.m_rotationPitch.get();
    let onGround: bool = this.m_onGround.get();
    if this.m_ridingEntity.get().is_some()
    {
        let motionX: f64 = this.m_motionX.get();
        let motionZ: f64 = this.m_motionZ.get();
        if b2
        {
            mp.m_sendQueue
                .addToSendQueue(Box::new(Packet11PlayerPosition::new(motionX, -999.0, -999.0, motionZ, onGround)));
        }
        else
        {
            mp.m_sendQueue.addToSendQueue(Box::new(Packet13PlayerLookMove::new(
                motionX, -999.0, -999.0, motionZ, yaw, pitch, onGround,
            )));
        }
        b = false;
    }
    else if b && b2
    {
        mp.m_sendQueue.addToSendQueue(Box::new(Packet13PlayerLookMove::new(
            this.posX(),
            minY,
            this.posY(),
            this.posZ(),
            yaw,
            pitch,
            onGround,
        )));
        mp.m_ticksSinceSendOnGround.set(0);
    }
    else if b
    {
        mp.m_sendQueue.addToSendQueue(Box::new(Packet11PlayerPosition::new(
            this.posX(),
            minY,
            this.posY(),
            this.posZ(),
            onGround,
        )));
        mp.m_ticksSinceSendOnGround.set(0);
    }
    else if b2
    {
        mp.m_sendQueue.addToSendQueue(Box::new(Packet12PlayerLook::new(yaw, pitch, onGround)));
        mp.m_ticksSinceSendOnGround.set(0);
    }
    else
    {
        mp.m_sendQueue.addToSendQueue(Box::new(Packet10Flying::new(onGround)));
        if mp.m_sentOnGround.get() != onGround || mp.m_ticksSinceSendOnGround.get() > 200
        {
            mp.m_ticksSinceSendOnGround.set(0);
        }
        else
        {
            mp.m_ticksSinceSendOnGround.set(mp.m_ticksSinceSendOnGround.get() + 1);
        }
    }
    mp.m_sentOnGround.set(onGround);
    if b
    {
        mp.m_oldPosX.set(this.posX());
        mp.m_sentMinY.set(minY);
        mp.m_oldPosY.set(this.posY());
        mp.m_oldPosZ.set(this.posZ());
    }
    if b2
    {
        mp.m_oldRotationYaw.set(yaw);
        mp.m_oldRotationPitch.set(pitch);
    }
}

fn sendInventoryChanged(_this: &Entity) {}

pub fn incrementRemoteStat(this: &Rc<Entity>, stat: Option<&Rc<StatBase>>, amount: i32)
{
    let Some(stat) = stat
    else
    {
        return;
    };
    if !stat.m_local.get()
    {
        entity_player_sp::addStat(this, Some(stat), amount);
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityClientPlayerMP",
    m_parent: Some(&entity_player_sp::CLASS),
    attackEntityFrom: |_, _, _| false,
    heal: |_, _| {},
    onUpdate,
    dropCurrentItem: |this| sendQueue(this).addToSendQueue(Box::new(Packet14BlockDig::new(4, 0, 0, 0, 0))),
    joinEntityItemWithWorld: |_, _| {},
    sendChatMessage: |this, content| {
        sendQueue(this).addToSendQueue(Box::new(Packet3Chat::new(JString::from(content))));
    },
    swingItem: |this| {
        (entity_player_sp::CLASS.swingItem)(this);
        sendQueue(this).addToSendQueue(Box::new(Packet18Animation::new(this, 1)));
    },
    respawnPlayer: |this| {
        sendInventoryChanged(this);
        let dimension: i8 = entity_player::fields(this).m_dimension.get() as i8;
        sendQueue(this).addToSendQueue(Box::new(Packet9Respawn::new(dimension)));
    },
    damageEntity: |this, amount| {
        let living: &LivingFields = entity_living::fields(this);
        living.m_health.set(living.m_health.get() - amount);
    },
    closeScreen: |this| {
        let windowId: i32 = entity_player::fields(this).m_craftingInventory.get().orNpe().m_windowId.get();
        sendQueue(this).addToSendQueue(Box::new(Packet101CloseWindow::new(windowId)));
        entity_player::inventory(this).setItemStack(None);
        (entity_player_sp::CLASS.closeScreen)(this);
    },
    setHealth: |this, health| {
        let mp: &ClientPlayerMPFields = fields(this);
        if mp.m_healthInitialized.get()
        {
            entity_player_sp::setHealth(this, health);
        }
        else
        {
            entity_living::fields(this).m_health.set(health);
            mp.m_healthInitialized.set(true);
        }
    },
    addStat: |this, stat, amount| {
        let Some(stat) = stat
        else
        {
            return;
        };
        if stat.m_local.get()
        {
            entity_player_sp::addStat(this, Some(stat), amount);
        }
    },
    ..entity_player_sp::CLASS
};

pub fn new(
    minecraft: &Rc<Minecraft>,
    world: &Rc<World>,
    session: Option<Rc<Session>>,
    networkHandler: &Rc<NetClientHandler>,
) -> Rc<Entity>
{
    let mp: ClientPlayerMPFields = ClientPlayerMPFields {
        m_sendQueue: Rc::clone(networkHandler),
        m_ticksSinceSendInventory: Cell::new(0),
        m_healthInitialized: Cell::new(false),
        m_oldPosX: Cell::new(0.0),
        m_sentMinY: Cell::new(0.0),
        m_oldPosY: Cell::new(0.0),
        m_oldPosZ: Cell::new(0.0),
        m_oldRotationYaw: Cell::new(0.0),
        m_oldRotationPitch: Cell::new(0.0),
        m_sentOnGround: Cell::new(false),
        m_wasSneaking: Cell::new(false),
        m_ticksSinceSendOnGround: Cell::new(0),
    };
    let ext: PlayerSPExt = PlayerSPExt {
        m_living: LivingFields::default(),
        m_player: PlayerFields::default(),
        m_sp: entity_player_sp::spFields(minecraft),
        m_mp: Some(mp),
    };
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(ext));
    entity_player_sp::init(&entity, world, session, 0);
    entity
}
