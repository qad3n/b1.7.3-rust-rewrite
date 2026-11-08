use crate::world::world_client::WorldClientExt;
use game::world::World;
use std::rc::Rc;

pub fn releaseDiscardedWorld(previous: Option<Rc<World>>, current: Option<Rc<World>>)
{
    let Some(previous) = previous
    else
    {
        return;
    };
    if current.is_some_and(|current| Rc::ptr_eq(&current, &previous))
    {
        return;
    }
    if let Some(ext) = previous.m_ext.downcast_ref::<WorldClientExt>()
    {
        ext.m_blockResets.clear();
        ext.m_entitiesByNetworkId.borrow_mut().clearMap();
        ext.m_forcedEntities.update(|set| set.clear());
        ext.m_pendingEntities.update(|set| set.clear());
    }
    game::port::releaseWorld(&previous);
}
