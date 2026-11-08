use crate::client::Minecraft;
use crate::controller::{PLAYER_CONTROLLER, PlayerController, PlayerControllerClass};
use game::block::{Block, StepSound};
use game::entity::{Entity, entity_player};
use game::item::ItemStack;
use game::world::World;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct ControllerSPExt
{
    m_targetBlockX: Cell<i32>,
    m_targetBlockY: Cell<i32>,
    m_targetBlockZ: Cell<i32>,
    m_curBlockDamage: Cell<f32>,
    m_prevBlockDamage: Cell<f32>,
    m_miningTicks: Cell<f32>,
    m_blockHitWait: Cell<i32>,
}

fn fields(this: &PlayerController) -> &ControllerSPExt { this.ext::<ControllerSPExt>() }

fn sendBlockRemoved(this: &Rc<PlayerController>, x: i32, y: i32, z: i32, face: i32) -> bool
{
    let blockId: i32 = this.theWorld().getBlockId(x, y, z);
    let blockMetadata: i32 = this.theWorld().getBlockMetadata(x, y, z);
    let sendBlockRemoved: bool = (PLAYER_CONTROLLER.sendBlockRemoved)(this, x, y, z, face);
    let thePlayer: Rc<Entity> = this.thePlayer();
    let currentEquippedItem: Option<Rc<ItemStack>> = entity_player::getCurrentEquippedItem(&thePlayer);
    let canHarvestBlock: bool = entity_player::canHarvestBlock(&thePlayer, &Block::byId(blockId).orNpe());
    if let Some(currentEquippedItem) = currentEquippedItem
    {
        currentEquippedItem.onDestroyBlock(blockId, x, y, z, &thePlayer);
        if currentEquippedItem.m_stackSize.get() == 0
        {
            currentEquippedItem.onRemoved(&thePlayer);
            entity_player::destroyCurrentEquippedItem(&thePlayer);
        }
    }
    if sendBlockRemoved && canHarvestBlock
    {
        Block::byId(blockId).orNpe().harvestBlock(&this.theWorld(), &thePlayer, x, y, z, blockMetadata);
    }
    sendBlockRemoved
}

fn clickBlock(this: &Rc<PlayerController>, x: i32, y: i32, z: i32, face: i32)
{
    this.theWorld().onBlockHit(Some(&this.thePlayer()), x, y, z, face);
    let blockId: i32 = this.theWorld().getBlockId(x, y, z);
    if blockId > 0 && fields(this).m_curBlockDamage.get() == 0.0
    {
        Block::byId(blockId).orNpe().onBlockClicked(&this.theWorld(), x, y, z, &this.thePlayer());
    }
    if blockId > 0 && Block::byId(blockId).orNpe().blockStrength(&this.thePlayer()) >= 1.0
    {
        this.sendBlockRemoved(x, y, z, face);
    }
}

pub fn playDigSound(mc: &Minecraft, block: &Block, x: i32, y: i32, z: i32)
{
    let stepSound: Rc<StepSound> = block.m_stepSound.get();
    mc.m_sndManager.get().orNpe().playSound(
        &stepSound.getStepping(),
        x as f32 + 0.5,
        y as f32 + 0.5,
        z as f32 + 0.5,
        (stepSound.getVolume() + 1.0) / 8.0,
        stepSound.getPitch() * 0.5,
    );
}

fn sendBlockRemoving(this: &Rc<PlayerController>, x: i32, y: i32, z: i32, face: i32)
{
    let ext: &ControllerSPExt = fields(this);
    if ext.m_blockHitWait.get() > 0
    {
        ext.m_blockHitWait.set(ext.m_blockHitWait.get() - 1);
        return;
    }
    if x == ext.m_targetBlockX.get() && y == ext.m_targetBlockY.get() && z == ext.m_targetBlockZ.get()
    {
        let blockId: i32 = this.theWorld().getBlockId(x, y, z);
        if blockId == 0
        {
            return;
        }
        let block: Rc<Block> = Block::byId(blockId).orNpe();
        ext.m_curBlockDamage.set(ext.m_curBlockDamage.get() + block.blockStrength(&this.thePlayer()));
        if ext.m_miningTicks.get() % 4.0 == 0.0
        {
            playDigSound(&this.m_mc, &block, x, y, z);
        }
        ext.m_miningTicks.set(ext.m_miningTicks.get() + 1.0);
        if ext.m_curBlockDamage.get() >= 1.0
        {
            this.sendBlockRemoved(x, y, z, face);
            ext.m_curBlockDamage.set(0.0);
            ext.m_prevBlockDamage.set(0.0);
            ext.m_miningTicks.set(0.0);
            ext.m_blockHitWait.set(5);
        }
    }
    else
    {
        ext.m_curBlockDamage.set(0.0);
        ext.m_prevBlockDamage.set(0.0);
        ext.m_miningTicks.set(0.0);
        ext.m_targetBlockX.set(x);
        ext.m_targetBlockY.set(y);
        ext.m_targetBlockZ.set(z);
    }
}

pub fn setDamagePartialTime(mc: &Minecraft, prevBlockDamage: f32, curBlockDamage: f32, tickDelta: f32)
{
    let damage: f32 = if curBlockDamage <= 0.0
    {
        0.0
    }
    else
    {
        prevBlockDamage + (curBlockDamage - prevBlockDamage) * tickDelta
    };
    mc.m_ingameGUI.get().orNpe().m_damageGuiPartialTime.set(damage);
    mc.m_renderGlobal.get().orNpe().m_damagePartialTime.set(damage);
}

pub const CLASS: PlayerControllerClass = PlayerControllerClass {
    m_name: "PlayerControllerSP",
    m_parent: Some(&PLAYER_CONTROLLER),
    m_interfaces: &[],
    flipPlayer: |_, player| player.m_rotationYaw.set(-180.0),
    sendBlockRemoved,
    clickBlock,
    resetBlockRemoving: |this| {
        fields(this).m_curBlockDamage.set(0.0);
        fields(this).m_blockHitWait.set(0);
    },
    sendBlockRemoving,
    setPartialTime: |this, tickDelta| {
        let ext: &ControllerSPExt = fields(this);
        setDamagePartialTime(&this.m_mc, ext.m_prevBlockDamage.get(), ext.m_curBlockDamage.get(), tickDelta);
    },
    getBlockReachDistance: |_| 4.0,
    initWorld: |this, world: &Rc<World>| (PLAYER_CONTROLLER.initWorld)(this, world),
    updateController: |this| {
        fields(this).m_prevBlockDamage.set(fields(this).m_curBlockDamage.get());
        this.m_mc.m_sndManager.get().orNpe().playRandomMusicIfReady();
    },
    ..PLAYER_CONTROLLER
};

pub fn new(minecraft: &Rc<Minecraft>) -> Rc<PlayerController>
{
    let ext: ControllerSPExt = ControllerSPExt {
        m_targetBlockX: Cell::new(-1),
        m_targetBlockY: Cell::new(-1),
        m_targetBlockZ: Cell::new(-1),
        m_curBlockDamage: Cell::new(0.0),
        m_prevBlockDamage: Cell::new(0.0),
        m_miningTicks: Cell::new(0.0),
        m_blockHitWait: Cell::new(0),
    };
    PlayerController::construct(&CLASS, Box::new(ext), minecraft)
}
