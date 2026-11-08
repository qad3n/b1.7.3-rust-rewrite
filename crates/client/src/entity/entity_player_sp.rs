use crate::client::{Minecraft, Session};
use crate::entity::entity_client_player_mp;
use crate::gui::{GuiChest, GuiCrafting, GuiDispenser, GuiEditSign, GuiFurnace};
use crate::render::particle::entity_pickup_fx;
use crate::settings::{MouseFilter, MovementInput};
use common::nbt::NBTTagCompound;
use game::entity::entity_living::LivingFields;
use game::entity::entity_player::PlayerFields;
use game::entity::{Entity, EntityClass, EntityExt, entity_living, entity_player};
use game::inventory::IInventory;
use game::math::math_helper;
use game::stats::{Achievement, AchievementList, StatBase};
use game::tileentity::TileEntity;
use game::world::World;
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::any::Any;
use std::rc::Rc;

pub struct PlayerSPFields
{
    pub m_movementInput: Field<Option<Rc<MovementInput>>>,
    pub m_mc: Rc<Minecraft>,
    pub m_field_21903_bJ: MouseFilter,
    pub m_field_21904_bK: MouseFilter,
    pub m_field_21902_bL: MouseFilter,
}

pub struct PlayerSPExt
{
    pub m_living: LivingFields,
    pub m_player: PlayerFields,
    pub m_sp: PlayerSPFields,
    pub m_mp: Option<entity_client_player_mp::ClientPlayerMPFields>,
}

impl EntityExt for PlayerSPExt
{
    fn asAny(&self) -> &dyn Any { self }

    fn living(&self) -> Option<&LivingFields> { Some(&self.m_living) }

    fn player(&self) -> Option<&PlayerFields> { Some(&self.m_player) }
}

pub fn fields(this: &Entity) -> &PlayerSPFields { &this.ext::<PlayerSPExt>().m_sp }

pub fn mc(this: &Entity) -> &Rc<Minecraft> { &fields(this).m_mc }

pub fn movementInput(this: &Entity) -> Rc<MovementInput> { fields(this).m_movementInput.get().orNpe() }

fn onLivingUpdate(this: &Rc<Entity>)
{
    let mc: &Rc<Minecraft> = mc(this);
    let openInventory: Rc<StatBase> = AchievementList::openInventory().orNpe();
    if !mc.m_statFileWriter.get().orNpe().hasAchievementUnlocked(&openInventory)
    {
        mc.m_guiAchievement.get().orNpe().queueAchievementInformation(&openInventory);
    }
    let player: &PlayerFields = entity_player::fields(this);
    player.m_prevTimeInPortal.set(player.m_timeInPortal.get());
    if player.m_inPortal.get()
    {
        if !this.world().m_multiplayerWorld.get() && this.m_ridingEntity.get().is_some()
        {
            this.mountEntity(None);
        }
        if mc.m_currentScreen.get().is_some()
        {
            mc.displayGuiScreen(None);
        }
        if player.m_timeInPortal.get() == 0.0
        {
            mc.m_sndManager.get().orNpe().playSoundFX("portal.trigger", 1.0, this.m_rand.nextFloat() * 0.4 + 0.8);
        }
        player.m_timeInPortal.set(player.m_timeInPortal.get() + 0.0125);
        if player.m_timeInPortal.get() >= 1.0
        {
            player.m_timeInPortal.set(1.0);
            if !this.world().m_multiplayerWorld.get()
            {
                player.m_timeUntilPortal.set(10);
                mc.m_sndManager.get().orNpe().playSoundFX("portal.travel", 1.0, this.m_rand.nextFloat() * 0.4 + 0.8);
                mc.usePortal();
            }
        }
        player.m_inPortal.set(false);
    }
    else
    {
        if player.m_timeInPortal.get() > 0.0
        {
            player.m_timeInPortal.set(player.m_timeInPortal.get() - 0.05);
        }
        if player.m_timeInPortal.get() < 0.0
        {
            player.m_timeInPortal.set(0.0);
        }
    }
    if player.m_timeUntilPortal.get() > 0
    {
        player.m_timeUntilPortal.set(player.m_timeUntilPortal.get() - 1);
    }
    let movementInput: Rc<MovementInput> = movementInput(this);
    movementInput.updatePlayerMoveState(this);
    if movementInput.m_sneak.get() && this.m_ySize.get() < 0.2
    {
        this.m_ySize.set(0.2);
    }
    let width: f64 = this.m_width.get() as f64;
    let minY: f64 = this.m_boundingBox.minY() + 0.5;
    this.pushOutOfBlocks(this.posX() - width * 0.35, minY, this.posZ() + width * 0.35);
    this.pushOutOfBlocks(this.posX() - width * 0.35, minY, this.posZ() - width * 0.35);
    this.pushOutOfBlocks(this.posX() + width * 0.35, minY, this.posZ() - width * 0.35);
    this.pushOutOfBlocks(this.posX() + width * 0.35, minY, this.posZ() + width * 0.35);
    (entity_player::CLASS.onLivingUpdate)(this);
}

pub fn resetPlayerKeyState(this: &Entity) { movementInput(this).resetKeyState(); }

pub fn handleKeyPress(this: &Entity, key: i32, state: bool)
{
    movementInput(this).checkKeyForMovementInput(key, state);
}

pub fn getPlayerArmorValue(this: &Entity) -> i32 { entity_player::inventory(this).getTotalArmorValue() }

pub fn setHealth(this: &Rc<Entity>, health: i32)
{
    let living: &LivingFields = entity_living::fields(this);
    let i: i32 = living.m_health.get() - health;
    if i <= 0
    {
        living.m_health.set(health);
        if i < 0
        {
            this.m_heartsLife.set(living.m_heartsHalvesLife.get() / 2);
        }
    }
    else
    {
        living.m_prevDamageTaken.set(i);
        living.m_prevHealth.set(living.m_health.get());
        this.m_heartsLife.set(living.m_heartsHalvesLife.get());
        this.damageEntity(i);
        living.m_maxHurtTime.set(10);
        living.m_hurtTime.set(10);
    }
}

pub fn addStat(this: &Rc<Entity>, stat: Option<&Rc<StatBase>>, amount: i32)
{
    let Some(stat) = stat
    else
    {
        return;
    };
    let mc: &Rc<Minecraft> = mc(this);
    if stat.isAchievement()
    {
        let achievement: &Achievement = Achievement::of(stat);
        let unlocked: bool = match &achievement.m_parentAchievement
        {
            None => true,
            Some(parent) => mc.m_statFileWriter.get().orNpe().hasAchievementUnlocked(parent),
        };
        if unlocked
        {
            if !mc.m_statFileWriter.get().orNpe().hasAchievementUnlocked(stat)
            {
                mc.m_guiAchievement.get().orNpe().queueTakenAchievement(stat);
            }
            mc.m_statFileWriter.get().orNpe().readStat(stat, amount);
        }
    }
    else
    {
        mc.m_statFileWriter.get().orNpe().readStat(stat, amount);
    }
}

fn isBlockTranslucent(this: &Entity, x: i32, y: i32, z: i32) -> bool { this.world().isBlockNormalCube(x, y, z) }

fn pushOutOfBlocks(this: &Rc<Entity>, x: f64, y: f64, z: f64) -> bool
{
    let n: i32 = math_helper::floor_double(x);
    let n2: i32 = math_helper::floor_double(y);
    let n3: i32 = math_helper::floor_double(z);
    let d: f64 = x - n as f64;
    let d2: f64 = z - n3 as f64;
    if isBlockTranslucent(this, n, n2, n3) || isBlockTranslucent(this, n, n2 + 1, n3)
    {
        let bl: bool = !isBlockTranslucent(this, n - 1, n2, n3) && !isBlockTranslucent(this, n - 1, n2 + 1, n3);
        let bl2: bool = !isBlockTranslucent(this, n + 1, n2, n3) && !isBlockTranslucent(this, n + 1, n2 + 1, n3);
        let bl3: bool = !isBlockTranslucent(this, n, n2, n3 - 1) && !isBlockTranslucent(this, n, n2 + 1, n3 - 1);
        let bl4: bool = !isBlockTranslucent(this, n, n2, n3 + 1) && !isBlockTranslucent(this, n, n2 + 1, n3 + 1);
        let mut n4: i32 = -1;
        let mut d3: f64 = 9999.0;
        if bl && d < d3
        {
            d3 = d;
            n4 = 0;
        }
        if bl2 && 1.0 - d < d3
        {
            d3 = 1.0 - d;
            n4 = 1;
        }
        if bl3 && d2 < d3
        {
            d3 = d2;
            n4 = 4;
        }
        if bl4 && 1.0 - d2 < d3
        {
            n4 = 5;
        }
        let f: f32 = 0.1;
        match n4
        {
            0 => this.m_motionX.set(-f as f64),
            1 => this.m_motionX.set(f as f64),
            4 => this.m_motionZ.set(-f as f64),
            5 => this.m_motionZ.set(f as f64),
            _ =>
            {}
        }
    }
    false
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityPlayerSP",
    m_parent: Some(&entity_player::CLASS),
    moveEntity: |this, dx, dy, dz| (entity_player::CLASS.moveEntity)(this, dx, dy, dz),
    updatePlayerActionState: |this| {
        (entity_player::CLASS.updatePlayerActionState)(this);
        let movementInput: Rc<MovementInput> = movementInput(this);
        let living: &LivingFields = entity_living::fields(this);
        living.m_moveStrafing.set(movementInput.m_moveStrafe.get());
        living.m_moveForward.set(movementInput.m_moveForward.get());
        living.m_isJumping.set(movementInput.m_jump.get());
    },
    onLivingUpdate,
    writeEntityToNBT: |this, nbt: &NBTTagCompound| {
        (entity_player::CLASS.writeEntityToNBT)(this, nbt);
        nbt.setInteger("Score", entity_player::fields(this).m_score.get());
    },
    readEntityFromNBT: |this, nbt| {
        (entity_player::CLASS.readEntityFromNBT)(this, nbt);
        entity_player::fields(this).m_score.set(nbt.getInteger("Score"));
    },
    closeScreen: |this| {
        (entity_player::CLASS.closeScreen)(this);
        mc(this).displayGuiScreen(None);
    },
    displayGUIEditSign: |this, sign: &Rc<TileEntity>| mc(this).displayGuiScreen(Some(GuiEditSign::new(sign))),
    displayGUIChest: |this, inventory: Option<&Rc<dyn IInventory>>| {
        let playerInventory: Rc<dyn IInventory> = entity_player::inventory(this);
        mc(this).displayGuiScreen(Some(GuiChest::new(playerInventory, inventory.cloned())));
    },
    displayWorkbenchGUI: |this, x, y, z| {
        mc(this).displayGuiScreen(Some(GuiCrafting::new(&entity_player::inventory(this), &this.world(), x, y, z)));
    },
    displayGUIFurnace: |this, furnace| {
        mc(this).displayGuiScreen(Some(GuiFurnace::new(&entity_player::inventory(this), furnace.cloned())));
    },
    displayGUIDispenser: |this, dispenser| {
        mc(this).displayGuiScreen(Some(GuiDispenser::new(&entity_player::inventory(this), dispenser.cloned())));
    },
    onItemPickup: |this, item, _count| {
        let mc: &Rc<Minecraft> = mc(this);
        let fx: Rc<Entity> = entity_pickup_fx::new(&mc.m_theWorld.get().orNpe(), item, this, -0.5);
        mc.m_effectRenderer.get().orNpe().addEffect(fx);
    },
    sendChatMessage: |_, _| {},
    isSneaking: |this| movementInput(this).m_sneak.get() && !entity_player::fields(this).m_sleeping.get(),
    setHealth,
    respawnPlayer: |this| mc(this).respawn(false, 0),
    addRespawnParticles: |_| {},
    addChatMessage: |this, message| mc(this).m_ingameGUI.get().orNpe().addChatMessageTranslate(message),
    addStat,
    pushOutOfBlocks,
    ..entity_player::CLASS
};

pub fn init(this: &Rc<Entity>, world: &Rc<World>, session: Option<Rc<Session>>, dimension: i32)
{
    entity_player::init(this, Rc::clone(world));
    entity_player::fields(this).m_dimension.set(dimension);
    if let Some(session) = &session
        && let Some(username) = session.m_username.get()
        && !username.is_empty()
    {
        this.m_skinUrl
            .set(Some(JString::from(format!("http://s3.amazonaws.com/MinecraftSkins/{username}.png").as_str())));
    }
    let username: Option<String> = session.orNpe().m_username.get();
    entity_player::fields(this).m_username.set(username.map(|name| JString::from(name.as_str())));
}

pub fn spFields(minecraft: &Rc<Minecraft>) -> PlayerSPFields
{
    PlayerSPFields {
        m_movementInput: Field::new(None),
        m_mc: Rc::clone(minecraft),
        m_field_21903_bJ: MouseFilter::new(),
        m_field_21904_bK: MouseFilter::new(),
        m_field_21902_bL: MouseFilter::new(),
    }
}

pub fn new(minecraft: &Rc<Minecraft>, world: &Rc<World>, session: Option<Rc<Session>>, dimension: i32) -> Rc<Entity>
{
    let ext: PlayerSPExt = PlayerSPExt {
        m_living: LivingFields::default(),
        m_player: PlayerFields::default(),
        m_sp: spFields(minecraft),
        m_mp: None,
    };
    let entity: Rc<Entity> = Entity::alloc(&CLASS, Box::new(ext));
    init(&entity, world, session, dimension);
    entity
}
