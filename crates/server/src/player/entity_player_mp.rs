use crate::network::NetServerHandler;
use crate::player::{EntityTracker, ItemInWorldManager};
use crate::server::MinecraftServer;
use crate::world::world_server;
use common::world::ChunkCoordIntPair;
use game::container::{
    Container, ICrafting, container_chest, container_dispenser, container_furnace, container_workbench,
};
use game::entity::entity_living::LivingFields;
use game::entity::entity_player::{EnumStatus, PlayerFields};
use game::entity::{Entity, EntityClass, EntityExt, entity_living, entity_player};
use game::inventory::{IInventory, InventoryPlayer};
use game::item::{Item, ItemStack};
use game::locale::StringTranslate;
use game::network::packet::Packet;
use game::network::packets::{
    Packet3Chat, Packet5PlayerInventory, Packet8UpdateHealth, Packet17Sleep, Packet18Animation, Packet22Collect,
    Packet39AttachEntity, Packet51MapChunk, Packet100OpenWindow, Packet101CloseWindow, Packet103SetSlot,
    Packet104WindowItems, Packet105UpdateProgressbar, Packet200Statistic,
};
use game::stats::StatBase;
use game::tileentity::TileEntity;
use game::world::{ChunkCoordinates, World};
use jvm::array_list::JArrayList;
use jvm::field::Field;
use jvm::hash_map::JHashSet;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct PlayerMPExt
{
    m_living: LivingFields,
    m_player: PlayerFields,
    pub m_playerNetServerHandler: Field<Option<Rc<NetServerHandler>>>,
    pub m_mcServer: Field<Option<Rc<MinecraftServer>>>,
    pub m_itemInWorldManager: Field<Option<Rc<ItemInWorldManager>>>,
    pub m_trackedX: Cell<f64>,
    pub m_trackedZ: Cell<f64>,
    pub m_loadedChunks: JArrayList<ChunkCoordIntPair>,
    pub m_allLoadedChunks: Field<JHashSet<ChunkCoordIntPair>>,
    m_lastHealth: Cell<i32>,
    m_ticksOfInvuln: Cell<i32>,
    m_playerInventory: RefCell<Vec<Option<Rc<ItemStack>>>>,
    m_currentWindowId: Cell<i32>,
    pub m_isChangingQuantityOnly: Cell<bool>,
}

impl EntityExt for PlayerMPExt
{
    fn asAny(&self) -> &dyn Any { self }

    fn living(&self) -> Option<&LivingFields> { Some(&self.m_living) }

    fn player(&self) -> Option<&PlayerFields> { Some(&self.m_player) }
}

pub fn fields(this: &Entity) -> &PlayerMPExt { this.ext::<PlayerMPExt>() }

pub fn netServerHandler(this: &Entity) -> Rc<NetServerHandler> { fields(this).m_playerNetServerHandler.get().orNpe() }

fn mcServer(this: &Entity) -> Rc<MinecraftServer> { fields(this).m_mcServer.get().orNpe() }

fn itemInWorldManager(this: &Entity) -> Rc<ItemInWorldManager> { fields(this).m_itemInWorldManager.get().orNpe() }

fn craftingInventory(this: &Entity) -> Rc<Container> { entity_player::fields(this).m_craftingInventory.get().orNpe() }

fn entityTracker(this: &Entity) -> Rc<EntityTracker>
{
    mcServer(this).getEntityTracker(entity_player::fields(this).m_dimension.get())
}

struct PlayerCrafting
{
    m_player: Rc<Entity>,
}

impl ICrafting for PlayerCrafting
{
    fn updateCraftingInventory(&self, container: &Rc<Container>, items: &[Option<Rc<ItemStack>>])
    {
        updateCraftingInventory(&self.m_player, container, items);
    }

    fn onSlotChanged(&self, container: &Rc<Container>, slot: i32, item: Option<Rc<ItemStack>>)
    {
        updateCraftingInventorySlot(&self.m_player, container, slot, item.as_ref());
    }

    fn onDataChanged(&self, container: &Rc<Container>, id: i32, value: i32)
    {
        updateCraftingInventoryInfo(&self.m_player, container, id, value);
    }

    fn entity(&self) -> Rc<Entity> { Rc::clone(&self.m_player) }
}

pub fn asCrafting(this: &Rc<Entity>) -> Rc<dyn ICrafting> { Rc::new(PlayerCrafting { m_player: Rc::clone(this) }) }

pub fn new(
    server: &Rc<MinecraftServer>,
    world: &Rc<World>,
    name: Option<JString>,
    interactionManager: &Rc<ItemInWorldManager>,
) -> Rc<Entity>
{
    let ext: PlayerMPExt = PlayerMPExt {
        m_living: LivingFields::default(),
        m_player: PlayerFields::default(),
        m_playerNetServerHandler: Field::new(None),
        m_mcServer: Field::new(None),
        m_itemInWorldManager: Field::new(None),
        m_trackedX: Cell::new(0.0),
        m_trackedZ: Cell::new(0.0),
        m_loadedChunks: JArrayList::new(),
        m_allLoadedChunks: Field::new(JHashSet::new()),
        m_lastHealth: Cell::new(-99999999),
        m_ticksOfInvuln: Cell::new(60),
        m_playerInventory: RefCell::new(vec![None, None, None, None, None]),
        m_currentWindowId: Cell::new(0),
        m_isChangingQuantityOnly: Cell::new(false),
    };
    let this: Rc<Entity> = Entity::alloc(&CLASS, Box::new(ext));
    entity_player::init(&this, Rc::clone(world));
    interactionManager.m_thisPlayer.set(Some(Rc::clone(&this)));
    fields(&this).m_itemInWorldManager.set(Some(Rc::clone(interactionManager)));
    let chunkcoordinates: ChunkCoordinates = world.getSpawnPoint();
    let mut i: i32 = chunkcoordinates.m_x;
    let mut j: i32 = chunkcoordinates.m_z;
    let mut k: i32 = chunkcoordinates.m_y;
    if !world.worldProvider().m_hasNoSky.get()
    {
        i += this.m_rand.nextIntBounded(20) - 10;
        k = world.findTopSolidBlockServer(i, j);
        j += this.m_rand.nextIntBounded(20) - 10;
    }
    this.setLocationAndAngles(i as f64 + 0.5, k as f64, j as f64 + 0.5, 0.0, 0.0);
    fields(&this).m_mcServer.set(Some(Rc::clone(server)));
    this.m_stepHeight.set(0.0);
    entity_player::fields(&this).m_username.set(name);
    this.m_yOffset.set(0.0);
    this
}

fn setWorld(this: &Rc<Entity>, world: Option<Rc<World>>)
{
    (entity_player::CLASS.setWorld)(this, world.clone());
    let manager: Rc<ItemInWorldManager> = ItemInWorldManager::new(&world.orNpe());
    fields(this).m_itemInWorldManager.set(Some(Rc::clone(&manager)));
    manager.m_thisPlayer.set(Some(Rc::clone(this)));
}

pub fn initMenu(this: &Rc<Entity>) { craftingInventory(this).onCraftGuiOpened(&asCrafting(this)); }

fn onUpdate(this: &Rc<Entity>)
{
    itemInWorldManager(this).tick();
    let ext: &PlayerMPExt = fields(this);
    ext.m_ticksOfInvuln.set(ext.m_ticksOfInvuln.get() - 1);
    craftingInventory(this).updateCraftingResults();
    for i in 0..5
    {
        let itemstack: Option<Rc<ItemStack>> = getEquipmentInSlot(this, i);
        let previous: Option<Rc<ItemStack>> = ext.m_playerInventory.borrow()[i as usize].clone();
        if !game::port::sameObject(itemstack.as_ref(), previous.as_ref())
        {
            entityTracker(this).sendPacketToTrackedPlayers(
                this,
                Box::new(Packet5PlayerInventory::new(this.m_entityId.get(), i, itemstack.as_ref())),
            );
            ext.m_playerInventory.borrow_mut()[i as usize] = itemstack;
        }
    }
}

pub fn getEquipmentInSlot(this: &Entity, slot: i32) -> Option<Rc<ItemStack>>
{
    let inventory: Rc<InventoryPlayer> = entity_player::inventory(this);
    if slot == 0
    {
        return inventory.getCurrentItem();
    }
    inventory.m_armorInventory.borrow()[(slot - 1) as usize].clone()
}

fn attackEntityFrom(this: &Rc<Entity>, source: Option<&Rc<Entity>>, amount: i32) -> bool
{
    if fields(this).m_ticksOfInvuln.get() > 0
    {
        return false;
    }
    if !mcServer(this).m_pvpOn.get()
    {
        if let Some(source) = source
        {
            if source.instanceOf("EntityPlayer")
            {
                return false;
            }
            if source.instanceOf("EntityArrow")
            {
                let owner: Option<Rc<Entity>> = game::entity::entity_arrow::fields(source).m_owner.get();
                if owner.is_some_and(|owner| owner.instanceOf("EntityPlayer"))
                {
                    return false;
                }
            }
        }
    }
    (entity_player::CLASS.attackEntityFrom)(this, source, amount)
}

pub fn onUpdateEntity(this: &Rc<Entity>, syncChunks: bool)
{
    (entity_player::CLASS.onUpdate)(this);
    let inventory: Rc<InventoryPlayer> = entity_player::inventory(this);
    let mut i: i32 = 0;
    while i < inventory.getSizeInventory()
    {
        let itemstack: Option<Rc<ItemStack>> = inventory.getStackInSlot(i);
        if let Some(itemstack) = itemstack
            && Item::byIdOrNpe(itemstack.m_itemID.get()).isNetworkSynced()
            && netServerHandler(this).getNumChunkDataPackets() <= 2
        {
            let packet: Option<Box<dyn Packet>> =
                Item::byIdOrNpe(itemstack.m_itemID.get()).getUpdatePacket(&itemstack, &this.world(), this);
            if let Some(packet) = packet
            {
                netServerHandler(this).sendPacket(packet);
            }
        }
        i += 1;
    }
    let ext: &PlayerMPExt = fields(this);
    if syncChunks && !ext.m_loadedChunks.isEmpty()
    {
        let chunkcoordintpair: ChunkCoordIntPair = ext.m_loadedChunks.get(0);
        let mut flag: bool = false;
        if netServerHandler(this).getNumChunkDataPackets() < 4
        {
            flag = true;
        }
        if flag
        {
            let worldserver: Rc<World> = mcServer(this).getWorldManager(entity_player::fields(this).m_dimension.get());
            ext.m_loadedChunks.remove(&chunkcoordintpair);
            netServerHandler(this).sendPacket(Box::new(Packet51MapChunk::new(
                chunkcoordintpair.m_chunkXPos * 16,
                0,
                chunkcoordintpair.m_chunkZPos * 16,
                16,
                128,
                16,
                &worldserver,
            )));
            let list: Vec<Rc<TileEntity>> = world_server::getTileEntityList(
                &worldserver,
                chunkcoordintpair.m_chunkXPos * 16,
                0,
                chunkcoordintpair.m_chunkZPos * 16,
                chunkcoordintpair.m_chunkXPos * 16 + 16,
                128,
                chunkcoordintpair.m_chunkZPos * 16 + 16,
            );
            for tileentity in &list
            {
                getTileEntityInfo(this, Some(tileentity));
            }
        }
    }
    let player: &PlayerFields = entity_player::fields(this);
    if player.m_inPortal.get()
    {
        if mcServer(this).propertyManager().getBooleanProperty("allow-nether", true)
        {
            if !game::port::sameObject(
                player.m_craftingInventory.get().as_ref(),
                player.m_inventorySlots.get().as_ref(),
            )
            {
                this.closeScreen();
            }
            if let Some(riding) = this.m_ridingEntity.get()
            {
                this.mountEntity(Some(&riding));
            }
            else
            {
                player.m_timeInPortal.set(player.m_timeInPortal.get() + 0.0125);
                if player.m_timeInPortal.get() >= 1.0
                {
                    player.m_timeInPortal.set(1.0);
                    player.m_timeUntilPortal.set(10);
                    mcServer(this).configManager().sendPlayerToOtherDimension(this);
                }
            }
            player.m_inPortal.set(false);
        }
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
    let health: i32 = entity_living::fields(this).m_health.get();
    if health != ext.m_lastHealth.get()
    {
        netServerHandler(this).sendPacket(Box::new(Packet8UpdateHealth::new(health)));
        ext.m_lastHealth.set(health);
    }
}

fn getTileEntityInfo(this: &Entity, blockEntity: Option<&Rc<TileEntity>>)
{
    if let Some(blockEntity) = blockEntity
    {
        let packet: Option<Box<dyn Packet>> = blockEntity.getDescriptionPacket();
        if let Some(packet) = packet
        {
            netServerHandler(this).sendPacket(packet);
        }
    }
}

fn onItemPickup(this: &Rc<Entity>, item: &Rc<Entity>, count: i32)
{
    if !item.m_isDead.get()
    {
        let entitytracker: Rc<EntityTracker> = entityTracker(this);
        if item.instanceOf("EntityItem")
        {
            entitytracker.sendPacketToTrackedPlayers(
                item,
                Box::new(Packet22Collect::new(item.m_entityId.get(), this.m_entityId.get())),
            );
        }
        if item.instanceOf("EntityArrow")
        {
            entitytracker.sendPacketToTrackedPlayers(
                item,
                Box::new(Packet22Collect::new(item.m_entityId.get(), this.m_entityId.get())),
            );
        }
    }
    (entity_player::CLASS.onItemPickup)(this, item, count);
    craftingInventory(this).updateCraftingResults();
}

fn swingItem(this: &Rc<Entity>)
{
    let player: &PlayerFields = entity_player::fields(this);
    if !player.m_isSwinging.get()
    {
        player.m_swingProgressInt.set(-1);
        player.m_isSwinging.set(true);
        let entitytracker: Rc<EntityTracker> = entityTracker(this);
        entitytracker.sendPacketToTrackedPlayers(this, Box::new(Packet18Animation::new(this, 1)));
    }
}

pub fn afterRespawn(_this: &Rc<Entity>) {}

fn sleepInBedAt(this: &Rc<Entity>, x: i32, y: i32, z: i32) -> EnumStatus
{
    let entityplayer_enumstatus: EnumStatus = (entity_player::CLASS.sleepInBedAt)(this, x, y, z);
    if entityplayer_enumstatus == EnumStatus::Ok
    {
        let entitytracker: Rc<EntityTracker> = entityTracker(this);
        entitytracker.sendPacketToTrackedPlayers(this, Box::new(Packet17Sleep::new(this, 0, x, y, z)));
        netServerHandler(this).teleportTo(
            this.m_posX.get(),
            this.m_posY.get(),
            this.m_posZ.get(),
            this.m_rotationYaw.get(),
            this.m_rotationPitch.get(),
        );
        netServerHandler(this).sendPacket(Box::new(Packet17Sleep::new(this, 0, x, y, z)));
    }
    entityplayer_enumstatus
}

fn wakeUpPlayer(this: &Rc<Entity>, resetSleepTimer: bool, updateAllPlayersSleeping: bool, setSpawnPoint: bool)
{
    if this.isPlayerSleeping()
    {
        let entitytracker: Rc<EntityTracker> = entityTracker(this);
        entitytracker.sendPacketToTrackedPlayersAndTrackedEntity(this, Box::new(Packet18Animation::new(this, 3)));
    }
    (entity_player::CLASS.wakeUpPlayer)(this, resetSleepTimer, updateAllPlayersSleeping, setSpawnPoint);
    if let Some(handler) = fields(this).m_playerNetServerHandler.get()
    {
        handler.teleportTo(
            this.m_posX.get(),
            this.m_posY.get(),
            this.m_posZ.get(),
            this.m_rotationYaw.get(),
            this.m_rotationPitch.get(),
        );
    }
}

fn mountEntity(this: &Rc<Entity>, entity: Option<&Rc<Entity>>)
{
    (entity_player::CLASS.mountEntity)(this, entity);
    netServerHandler(this).sendPacket(Box::new(Packet39AttachEntity::new(this, this.m_ridingEntity.get().as_ref())));
    netServerHandler(this).teleportTo(
        this.m_posX.get(),
        this.m_posY.get(),
        this.m_posZ.get(),
        this.m_rotationYaw.get(),
        this.m_rotationPitch.get(),
    );
}

pub fn handleFalling(this: &Rc<Entity>, distance: f64, onGround: bool)
{
    (entity_player::CLASS.updateFallState)(this, distance, onGround);
}

fn getNextWidowId(this: &Entity)
{
    let ext: &PlayerMPExt = fields(this);
    ext.m_currentWindowId.set(ext.m_currentWindowId.get() % 100 + 1);
}

fn openContainer(this: &Rc<Entity>, container: Rc<Container>)
{
    entity_player::fields(this).m_craftingInventory.set(Some(Rc::clone(&container)));
    container.m_windowId.set(fields(this).m_currentWindowId.get());
    container.onCraftGuiOpened(&asCrafting(this));
}

fn displayWorkbenchGUI(this: &Rc<Entity>, x: i32, y: i32, z: i32)
{
    getNextWidowId(this);
    netServerHandler(this).sendPacket(Box::new(Packet100OpenWindow::new(
        fields(this).m_currentWindowId.get(),
        1,
        Some(JString::from("Crafting")),
        9,
    )));
    openContainer(this, container_workbench::create(&entity_player::inventory(this), &this.world(), x, y, z));
}

fn displayGUIChest(this: &Rc<Entity>, inventory: Option<&Rc<dyn IInventory>>)
{
    getNextWidowId(this);
    let inventory: &Rc<dyn IInventory> = inventory.orNpe();
    netServerHandler(this).sendPacket(Box::new(Packet100OpenWindow::new(
        fields(this).m_currentWindowId.get(),
        0,
        Some(JString::from(inventory.getInvName().as_str())),
        inventory.getSizeInventory(),
    )));
    openContainer(this, container_chest::create(entity_player::inventory(this), Rc::clone(inventory)));
}

fn displayGUIFurnace(this: &Rc<Entity>, furnace: Option<&Rc<TileEntity>>)
{
    getNextWidowId(this);
    let furnace: &Rc<TileEntity> = furnace.orNpe();
    netServerHandler(this).sendPacket(Box::new(Packet100OpenWindow::new(
        fields(this).m_currentWindowId.get(),
        2,
        Some(JString::from(furnace.getInvName())),
        furnace.getSizeInventory(),
    )));
    openContainer(this, container_furnace::create(&entity_player::inventory(this), furnace));
}

fn displayGUIDispenser(this: &Rc<Entity>, dispenser: Option<&Rc<TileEntity>>)
{
    getNextWidowId(this);
    let dispenser: &Rc<TileEntity> = dispenser.orNpe();
    netServerHandler(this).sendPacket(Box::new(Packet100OpenWindow::new(
        fields(this).m_currentWindowId.get(),
        3,
        Some(JString::from(dispenser.getInvName())),
        dispenser.getSizeInventory(),
    )));
    openContainer(this, container_dispenser::create(entity_player::inventory(this), dispenser));
}

fn updateCraftingInventorySlot(this: &Rc<Entity>, menu: &Rc<Container>, slot: i32, item: Option<&Rc<ItemStack>>)
{
    if menu.getSlot(slot).instanceOf("SlotCrafting")
    {
        return;
    }
    if fields(this).m_isChangingQuantityOnly.get()
    {
        return;
    }
    netServerHandler(this).sendPacket(Box::new(Packet103SetSlot::new(menu.m_windowId.get(), slot, item)));
}

pub fn setMenu(this: &Rc<Entity>, menu: &Rc<Container>) { updateCraftingInventory(this, menu, &menu.getItems()); }

pub fn updateCraftingInventory(this: &Rc<Entity>, menu: &Rc<Container>, items: &[Option<Rc<ItemStack>>])
{
    netServerHandler(this).sendPacket(Box::new(Packet104WindowItems::new(menu.m_windowId.get(), items)));
    netServerHandler(this).sendPacket(Box::new(Packet103SetSlot::new(
        -1,
        -1,
        entity_player::inventory(this).getItemStack().as_ref(),
    )));
}

fn updateCraftingInventoryInfo(this: &Rc<Entity>, menu: &Rc<Container>, id: i32, value: i32)
{
    netServerHandler(this).sendPacket(Box::new(Packet105UpdateProgressbar::new(menu.m_windowId.get(), id, value)));
}

fn closeScreen(this: &Rc<Entity>)
{
    netServerHandler(this).sendPacket(Box::new(Packet101CloseWindow::new(craftingInventory(this).m_windowId.get())));
    closeCraftingGui(this);
}

pub fn updateHeldItem(this: &Rc<Entity>)
{
    if fields(this).m_isChangingQuantityOnly.get()
    {
        return;
    }
    netServerHandler(this).sendPacket(Box::new(Packet103SetSlot::new(
        -1,
        -1,
        entity_player::inventory(this).getItemStack().as_ref(),
    )));
}

pub fn closeCraftingGui(this: &Rc<Entity>)
{
    craftingInventory(this).onCraftGuiClosed(this);
    let player: &PlayerFields = entity_player::fields(this);
    player.m_craftingInventory.set(player.m_inventorySlots.get());
}

pub fn setMovementType(
    this: &Rc<Entity>,
    sidewaysSpeed: f32,
    forwardSpeed: f32,
    jumping: bool,
    sneaking: bool,
    pitch: f32,
    yaw: f32,
)
{
    let living: &LivingFields = entity_living::fields(this);
    living.m_moveStrafing.set(sidewaysSpeed);
    living.m_moveForward.set(forwardSpeed);
    living.m_isJumping.set(jumping);
    this.setSneaking(sneaking);
    this.m_rotationPitch.set(pitch);
    this.m_rotationYaw.set(yaw);
}

fn addStat(this: &Rc<Entity>, stat: Option<&Rc<StatBase>>, amount: i32)
{
    let Some(stat) = stat
    else
    {
        return;
    };
    let mut amount: i32 = amount;
    if !stat.m_local.get()
    {
        while amount > 100
        {
            netServerHandler(this).sendPacket(Box::new(Packet200Statistic::new(stat.m_statId, 100)));
            amount -= 100;
        }
        netServerHandler(this).sendPacket(Box::new(Packet200Statistic::new(stat.m_statId, amount)));
    }
}

pub fn onDisconnect(this: &Rc<Entity>)
{
    if let Some(riding) = this.m_ridingEntity.get()
    {
        this.mountEntity(Some(&riding));
    }
    if let Some(rider) = this.m_riddenByEntity.get()
    {
        rider.mountEntity(Some(this));
    }
    if entity_player::fields(this).m_sleeping.get()
    {
        this.wakeUpPlayer(true, false, false);
    }
}

pub fn markHealthDirty(this: &Entity) { fields(this).m_lastHealth.set(-99999999); }

fn addChatMessage(this: &Rc<Entity>, message: &str)
{
    let stringtranslate: Rc<StringTranslate> = StringTranslate::getInstance();
    let s: String = stringtranslate.translateKey(message);
    netServerHandler(this).sendPacket(Box::new(Packet3Chat::new(JString::from(s.as_str()))));
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityPlayerMP",
    m_parent: Some(&entity_player::CLASS),
    m_interfaces: &["ICrafting"],
    setWorld,
    getInventory: |this| Some(fields(this).m_playerInventory.borrow().clone()),
    resetHeight: |this| this.m_yOffset.set(0.0),
    getEyeHeight: |_| 1.62,
    onUpdate,
    onDeath: |this, _| entity_player::inventory(this).dropAllItems(),
    attackEntityFrom,
    pvpEnabled: |this| mcServer(this).m_pvpOn.get(),
    heal: |this, amount| (entity_player::CLASS.heal)(this, amount),
    onLivingUpdate: |this| (entity_player::CLASS.onLivingUpdate)(this),
    onItemPickup,
    swingItem,
    sleepInBedAt,
    wakeUpPlayer,
    mountEntity,
    updateFallState: |_, _, _| {},
    displayWorkbenchGUI,
    displayGUIChest,
    displayGUIFurnace,
    displayGUIDispenser,
    onItemStackChanged: |_, _| {},
    closeScreen,
    addStat,
    addChatMessage,
    ..entity_player::CLASS
};
