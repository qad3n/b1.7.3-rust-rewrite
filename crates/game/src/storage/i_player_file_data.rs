use crate::entity::Entity;
use std::rc::Rc;

pub trait IPlayerFileData
{
    fn writePlayerData(&self, player: &Rc<Entity>);

    fn readPlayerData(&self, player: &Rc<Entity>);
}
