use crate::client::Minecraft;
use crate::controller::player_controller_sp::{playDigSound, setDamagePartialTime};
use crate::controller::{PLAYER_CONTROLLER, PlayerController, PlayerControllerClass};
use crate::entity::entity_client_player_mp;
use crate::network::net_client_handler::NetClientHandler;
use game::block::Block;
use game::container::Container;
use game::entity::{Entity, entity_player};
use game::item::ItemStack;
use game::network::packets::{
    Packet7UseEntity, Packet14BlockDig, Packet15Place, Packet16BlockItemSwitch, Packet102WindowClick,
};
use game::world::World;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct ControllerMPExt
{
    m_currentBlockX: Cell<i32>,
    m_currentBlockY: Cell<i32>,
    m_currentblockZ: Cell<i32>,
    m_curBlockDamageMP: Cell<f32>,
    m_prevBlockDamageMP: Cell<f32>,
    m_miningTicks: Cell<f32>,
    m_blockHitDelay: Cell<i32>,
    m_isHittingBlock: Cell<bool>,
    m_netClientHandler: Rc<NetClientHandler>,
    m_currentPlayerItem: Cell<i32>,
}

fn fields(this: &PlayerController) -> &ControllerMPExt { this.ext::<ControllerMPExt>() }

fn sendBlockRemoved(this: &Rc<PlayerController>, x: i32, y: i32, z: i32, face: i32) -> bool
{
    let n: i32 = this.theWorld().getBlockId(x, y, z);
    let bl: bool = (PLAYER_CONTROLLER.sendBlockRemoved)(this, x, y, z, face);
    let thePlayer: Rc<Entity> = this.thePlayer();
    if let Some(itemStack) = entity_player::getCurrentEquippedItem(&thePlayer)
    {
        itemStack.onDestroyBlock(n, x, y, z, &thePlayer);
        if itemStack.m_stackSize.get() == 0
        {
            itemStack.onRemoved(&thePlayer);
            entity_player::destroyCurrentEquippedItem(&thePlayer);
        }
    }
    bl
}

fn clickBlock(this: &Rc<PlayerController>, x: i32, y: i32, z: i32, face: i32)
{
    let ext: &ControllerMPExt = fields(this);
    if !ext.m_isHittingBlock.get()
        || x != ext.m_currentBlockX.get()
        || y != ext.m_currentBlockY.get()
        || z != ext.m_currentblockZ.get()
    {
        ext.m_netClientHandler.addToSendQueue(Box::new(Packet14BlockDig::new(0, x, y, z, face)));
        let n: i32 = this.theWorld().getBlockId(x, y, z);
        if n > 0 && ext.m_curBlockDamageMP.get() == 0.0
        {
            Block::byId(n).orNpe().onBlockClicked(&this.theWorld(), x, y, z, &this.thePlayer());
        }
        if n > 0 && Block::byId(n).orNpe().blockStrength(&this.thePlayer()) >= 1.0
        {
            this.sendBlockRemoved(x, y, z, face);
        }
        else
        {
            ext.m_isHittingBlock.set(true);
            ext.m_currentBlockX.set(x);
            ext.m_currentBlockY.set(y);
            ext.m_currentblockZ.set(z);
            ext.m_curBlockDamageMP.set(0.0);
            ext.m_prevBlockDamageMP.set(0.0);
            ext.m_miningTicks.set(0.0);
        }
    }
}

fn sendBlockRemoving(this: &Rc<PlayerController>, x: i32, y: i32, z: i32, face: i32)
{
    let ext: &ControllerMPExt = fields(this);
    if !ext.m_isHittingBlock.get()
    {
        return;
    }
    syncCurrentPlayItem(this);
    if ext.m_blockHitDelay.get() > 0
    {
        ext.m_blockHitDelay.set(ext.m_blockHitDelay.get() - 1);
        return;
    }
    if x == ext.m_currentBlockX.get() && y == ext.m_currentBlockY.get() && z == ext.m_currentblockZ.get()
    {
        let n: i32 = this.theWorld().getBlockId(x, y, z);
        if n == 0
        {
            ext.m_isHittingBlock.set(false);
            return;
        }
        let block: Rc<Block> = Block::byId(n).orNpe();
        ext.m_curBlockDamageMP.set(ext.m_curBlockDamageMP.get() + block.blockStrength(&this.thePlayer()));
        if ext.m_miningTicks.get() % 4.0 == 0.0
        {
            playDigSound(&this.m_mc, &block, x, y, z);
        }
        ext.m_miningTicks.set(ext.m_miningTicks.get() + 1.0);
        if ext.m_curBlockDamageMP.get() >= 1.0
        {
            ext.m_isHittingBlock.set(false);
            ext.m_netClientHandler.addToSendQueue(Box::new(Packet14BlockDig::new(2, x, y, z, face)));
            this.sendBlockRemoved(x, y, z, face);
            ext.m_curBlockDamageMP.set(0.0);
            ext.m_prevBlockDamageMP.set(0.0);
            ext.m_miningTicks.set(0.0);
            ext.m_blockHitDelay.set(5);
        }
    }
    else
    {
        this.clickBlock(x, y, z, face);
    }
}

fn syncCurrentPlayItem(this: &PlayerController)
{
    let ext: &ControllerMPExt = fields(this);
    let n: i32 = entity_player::inventory(&this.thePlayer()).m_currentItem.get();
    if n != ext.m_currentPlayerItem.get()
    {
        ext.m_currentPlayerItem.set(n);
        ext.m_netClientHandler.addToSendQueue(Box::new(Packet16BlockItemSwitch::new(n)));
    }
}

fn sendPlaceBlock(
    this: &Rc<PlayerController>,
    player: &Rc<Entity>,
    world: &Rc<World>,
    itemInHand: Option<&Rc<ItemStack>>,
    x: i32,
    y: i32,
    z: i32,
    face: i32,
) -> bool
{
    syncCurrentPlayItem(this);
    let currentItem: Option<Rc<ItemStack>> = entity_player::inventory(player).getCurrentItem();
    fields(this).m_netClientHandler.addToSendQueue(Box::new(Packet15Place::new(x, y, z, face, currentItem.as_ref())));
    (PLAYER_CONTROLLER.sendPlaceBlock)(this, player, world, itemInHand, x, y, z, face)
}

fn sendUseItem(this: &Rc<PlayerController>, player: &Rc<Entity>, world: &Rc<World>, itemInHand: &Rc<ItemStack>)
-> bool
{
    syncCurrentPlayItem(this);
    let currentItem: Option<Rc<ItemStack>> = entity_player::inventory(player).getCurrentItem();
    fields(this).m_netClientHandler.addToSendQueue(Box::new(Packet15Place::new(-1, -1, -1, 255, currentItem.as_ref())));
    (PLAYER_CONTROLLER.sendUseItem)(this, player, world, itemInHand)
}

fn clickSlot(
    this: &Rc<PlayerController>,
    menuId: i32,
    slot: i32,
    button: i32,
    quickMove: bool,
    player: &Rc<Entity>,
) -> Option<Rc<ItemStack>>
{
    let craftingInventory: Rc<Container> = entity_player::fields(player).m_craftingInventory.get().orNpe();
    let s: i16 = craftingInventory.nextInteractionId(&entity_player::inventory(player));
    let itemStack: Option<Rc<ItemStack>> = (PLAYER_CONTROLLER.clickSlot)(this, menuId, slot, button, quickMove, player);
    fields(this).m_netClientHandler.addToSendQueue(Box::new(Packet102WindowClick::new(
        menuId,
        slot,
        button,
        quickMove,
        itemStack.as_ref(),
        s,
    )));
    itemStack
}

fn useEntity(this: &PlayerController, player: &Rc<Entity>, target: &Rc<Entity>, action: i32)
{
    syncCurrentPlayItem(this);
    fields(this).m_netClientHandler.addToSendQueue(Box::new(Packet7UseEntity::new(
        player.m_entityId.get(),
        target.m_entityId.get(),
        action,
    )));
}

pub const CLASS: PlayerControllerClass = PlayerControllerClass {
    m_name: "PlayerControllerMP",
    m_parent: Some(&PLAYER_CONTROLLER),
    m_interfaces: &[],
    flipPlayer: |_, player| player.m_rotationYaw.set(-180.0),
    sendBlockRemoved,
    clickBlock,
    resetBlockRemoving: |this| {
        fields(this).m_curBlockDamageMP.set(0.0);
        fields(this).m_isHittingBlock.set(false);
    },
    sendBlockRemoving,
    setPartialTime: |this, tickDelta| {
        let ext: &ControllerMPExt = fields(this);
        setDamagePartialTime(&this.m_mc, ext.m_prevBlockDamageMP.get(), ext.m_curBlockDamageMP.get(), tickDelta);
    },
    getBlockReachDistance: |_| 4.0,
    initWorld: |this, world| (PLAYER_CONTROLLER.initWorld)(this, world),
    updateController: |this| {
        syncCurrentPlayItem(this);
        fields(this).m_prevBlockDamageMP.set(fields(this).m_curBlockDamageMP.get());
        this.m_mc.m_sndManager.get().orNpe().playRandomMusicIfReady();
    },
    sendPlaceBlock,
    sendUseItem,
    createPlayer: |this, world| {
        let mc: &Rc<Minecraft> = &this.m_mc;
        entity_client_player_mp::new(mc, world, mc.m_session.get(), &fields(this).m_netClientHandler)
    },
    attackEntity: |this, player, target| {
        useEntity(this, player, target, 1);
        entity_player::attackTargetEntityWithCurrentItem(player, target);
    },
    interactWithEntity: |this, player, target| {
        useEntity(this, player, target, 0);
        entity_player::useCurrentItemOnEntity(player, target);
    },
    clickSlot,
    closeMenu: |_, _, _| {},
    ..PLAYER_CONTROLLER
};

pub fn new(minecraft: &Rc<Minecraft>, networkHandler: &Rc<NetClientHandler>) -> Rc<PlayerController>
{
    let ext: ControllerMPExt = ControllerMPExt {
        m_currentBlockX: Cell::new(-1),
        m_currentBlockY: Cell::new(-1),
        m_currentblockZ: Cell::new(-1),
        m_curBlockDamageMP: Cell::new(0.0),
        m_prevBlockDamageMP: Cell::new(0.0),
        m_miningTicks: Cell::new(0.0),
        m_blockHitDelay: Cell::new(0),
        m_isHittingBlock: Cell::new(false),
        m_netClientHandler: Rc::clone(networkHandler),
        m_currentPlayerItem: Cell::new(0),
    };
    PlayerController::construct(&CLASS, Box::new(ext), minecraft)
}
