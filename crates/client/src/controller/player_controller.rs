use crate::client::Minecraft;
use crate::entity::entity_player_sp;
use game::block::Block;
use game::container::Container;
use game::entity::{Entity, entity_player};
use game::inventory::InventoryPlayer;
use game::item::ItemStack;
use game::world::World;
use jvm::throwable::OrNpe;
use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

game::vtable! {
    PlayerControllerClass for PlayerController {
        fn adjustPlayer(player: &Rc<Entity>);
        fn attackEntity(player: &Rc<Entity>, target: &Rc<Entity>);
        fn clickBlock(x: i32, y: i32, z: i32, face: i32);
        fn clickSlot(menuId: i32, slot: i32, button: i32, quickMove: bool, player: &Rc<Entity>) -> Option<Rc<ItemStack>>;
        fn closeMenu(menuId: i32, player: &Rc<Entity>);
        fn createPlayer(world: &Rc<World>) -> Rc<Entity>;
        fn flipPlayer(player: &Rc<Entity>);
        fn getBlockReachDistance() -> f32;
        fn initWorld(world: &Rc<World>);
        fn interactWithEntity(player: &Rc<Entity>, target: &Rc<Entity>);
        fn resetBlockRemoving();
        fn sendBlockRemoved(x: i32, y: i32, z: i32, face: i32) -> bool;
        fn sendBlockRemoving(x: i32, y: i32, z: i32, face: i32);
        fn sendPlaceBlock(player: &Rc<Entity>, world: &Rc<World>, itemInHand: Option<&Rc<ItemStack>>, x: i32, y: i32, z: i32, face: i32) -> bool;
        fn sendUseItem(player: &Rc<Entity>, world: &Rc<World>, itemInHand: &Rc<ItemStack>) -> bool;
        fn setPartialTime(tickDelta: f32);
        fn shouldDrawHUD() -> bool;
        fn updateController();
    }
}

fn sendBlockRemoved(this: &Rc<PlayerController>, x: i32, y: i32, z: i32, _face: i32) -> bool
{
    let world: Rc<World> = this.theWorld();
    let block: Option<Rc<Block>> = Block::byId(world.getBlockId(x, y, z));
    world.doEvent(2001, x, y, z, block.as_ref().orNpe().m_blockID + world.getBlockMetadata(x, y, z) * 256);
    let n: i32 = world.getBlockMetadata(x, y, z);
    let bl: bool = world.setBlockWithNotify(x, y, z, 0);
    if let Some(block) = block
        && bl
    {
        block.onBlockDestroyedByPlayer(&world, x, y, z, n);
    }
    bl
}

fn sendUseItem(_this: &Rc<PlayerController>, player: &Rc<Entity>, world: &Rc<World>, itemInHand: &Rc<ItemStack>)
-> bool
{
    let n: i32 = itemInHand.m_stackSize.get();
    let itemStack: Option<Rc<ItemStack>> = itemInHand.useItemRightClick(world, player);
    let changed: bool = match &itemStack
    {
        Some(stack) => !Rc::ptr_eq(stack, itemInHand) || stack.m_stackSize.get() != n,
        None => true,
    };
    if changed
    {
        let inventory: Rc<InventoryPlayer> = entity_player::inventory(player);
        let currentItem: usize = inventory.m_currentItem.get() as usize;
        inventory.m_mainInventory.borrow_mut()[currentItem] = itemStack.clone();
        if itemStack.orNpe().m_stackSize.get() == 0
        {
            inventory.m_mainInventory.borrow_mut()[currentItem] = None;
        }
        return true;
    }
    false
}

fn sendPlaceBlock(
    _this: &Rc<PlayerController>,
    player: &Rc<Entity>,
    world: &Rc<World>,
    itemInHand: Option<&Rc<ItemStack>>,
    x: i32,
    y: i32,
    z: i32,
    face: i32,
) -> bool
{
    let n: i32 = world.getBlockId(x, y, z);
    if n > 0 && Block::byId(n).orNpe().blockActivated(world, x, y, z, player)
    {
        return true;
    }
    match itemInHand
    {
        None => false,
        Some(itemInHand) => itemInHand.useItem(player, world, x, y, z, face),
    }
}

fn clickSlot(
    _this: &Rc<PlayerController>,
    _menuId: i32,
    slot: i32,
    button: i32,
    quickMove: bool,
    player: &Rc<Entity>,
) -> Option<Rc<ItemStack>>
{
    let craftingInventory: Rc<Container> = entity_player::fields(player).m_craftingInventory.get().orNpe();
    craftingInventory.onClickSlot(slot, button, quickMove, player)
}

fn closeMenu(_this: &Rc<PlayerController>, _menuId: i32, player: &Rc<Entity>)
{
    let fields: &entity_player::PlayerFields = entity_player::fields(player);
    fields.m_craftingInventory.get().orNpe().onCraftGuiClosed(player);
    fields.m_craftingInventory.set(fields.m_inventorySlots.get());
}

pub const PLAYER_CONTROLLER: PlayerControllerClass = PlayerControllerClass {
    m_name: "PlayerController",
    m_parent: None,
    m_interfaces: &[],
    adjustPlayer: |_, _| {},
    attackEntity: |_, player, target| entity_player::attackTargetEntityWithCurrentItem(player, target),
    clickBlock: |this, x, y, z, face| {
        this.theWorld().onBlockHit(Some(&this.thePlayer()), x, y, z, face);
        this.sendBlockRemoved(x, y, z, face);
    },
    clickSlot,
    closeMenu,
    createPlayer: |this, world| {
        let mc: &Rc<Minecraft> = &this.m_mc;
        entity_player_sp::new(mc, world, mc.m_session.get(), world.worldProvider().m_worldType.get())
    },
    flipPlayer: |_, _| {},
    getBlockReachDistance: |_| 5.0,
    initWorld: |_, _| {},
    interactWithEntity: |_, player, target| entity_player::useCurrentItemOnEntity(player, target),
    resetBlockRemoving: |_| {},
    sendBlockRemoved,
    sendBlockRemoving: |_, _, _, _, _| {},
    sendPlaceBlock,
    sendUseItem,
    setPartialTime: |_, _| {},
    shouldDrawHUD: |_| true,
    updateController: |_| {},
};

pub struct PlayerController
{
    pub m_class: &'static PlayerControllerClass,
    m_ext: Box<dyn Any>,
    pub m_mc: Rc<Minecraft>,
    pub m_creative: Cell<bool>,
}

impl PlayerController
{
    pub fn new(minecraft: &Rc<Minecraft>) -> Rc<PlayerController>
    {
        PlayerController::construct(&PLAYER_CONTROLLER, Box::new(()), minecraft)
    }

    pub fn construct(
        class: &'static PlayerControllerClass,
        ext: Box<dyn Any>,
        minecraft: &Rc<Minecraft>,
    ) -> Rc<PlayerController>
    {
        Rc::new(PlayerController {
            m_class: class,
            m_ext: ext,
            m_mc: Rc::clone(minecraft),
            m_creative: Cell::new(false),
        })
    }

    pub fn ext<T: 'static>(&self) -> &T
    {
        self.m_ext.downcast_ref::<T>().expect("player controller ext matches its class")
    }

    pub fn theWorld(&self) -> Rc<World> { self.m_mc.m_theWorld.get().orNpe() }

    pub fn thePlayer(&self) -> Rc<Entity> { self.m_mc.m_thePlayer.get().orNpe() }
}
