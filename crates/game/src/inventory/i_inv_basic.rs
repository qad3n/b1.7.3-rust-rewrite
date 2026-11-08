use crate::inventory::InventoryBasic;

pub trait IInvBasic
{
    fn onInventoryChanged(&self, inventory: &InventoryBasic);
}
