use crate::client::{Minecraft, Session};
use crate::controller::{PLAYER_CONTROLLER, PlayerController, PlayerControllerClass};
use game::entity::entity_player;
use game::inventory::InventoryPlayer;
use game::item::ItemStack;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub const CLASS: PlayerControllerClass = PlayerControllerClass {
    m_name: "PlayerControllerTest",
    m_parent: Some(&PLAYER_CONTROLLER),
    m_interfaces: &[],
    adjustPlayer: |this, player| {
        let playerInventory: Rc<InventoryPlayer> = entity_player::inventory(player);
        for i in 0..9
        {
            let thePlayerInventory: Rc<InventoryPlayer> = entity_player::inventory(&this.thePlayer());
            if playerInventory.m_mainInventory.borrow()[i].is_none()
            {
                thePlayerInventory.m_mainInventory.borrow_mut()[i] =
                    Some(ItemStack::withBlock(&Session::registeredBlocksList().get(i as i32)));
                continue;
            }
            thePlayerInventory.m_mainInventory.borrow()[i].clone().orNpe().m_stackSize.set(1);
        }
    },
    shouldDrawHUD: |_| false,
    initWorld: |this, world| (PLAYER_CONTROLLER.initWorld)(this, world),
    updateController: |_| {},
    ..PLAYER_CONTROLLER
};

pub fn new(minecraft: &Rc<Minecraft>) -> Rc<PlayerController>
{
    let this: Rc<PlayerController> = PlayerController::construct(&CLASS, Box::new(()), minecraft);
    this.m_creative.set(true);
    this
}
