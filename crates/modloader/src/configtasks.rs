use mod_protocol::Identifier;
use mod_protocol::abi::{GuestCall, GuestReply, Handle};
use mod_protocol::handshake::Hello;
use std::cell::RefCell;

// a configuration step a mod runs with each joining client that has the mod, before play
struct Task
{
    m_id: Identifier,
    m_mod: usize,
    m_handler: Handle,
}

thread_local!
{
    static TASKS: RefCell<Vec<Task>> = const { RefCell::new(Vec::new()) };
}

pub fn register(modIndex: usize, id: Identifier, handler: Handle) -> Result<(), String>
{
    TASKS.with(|tasks|
    {
        let mut tasks = tasks.borrow_mut();
        if tasks.iter().any(|task| task.m_id == id)
        {
            return Err(format!("configuration task {id} already exists"));
        }

        tasks.push(Task { m_id: id, m_mod: modIndex, m_handler: handler });
        Ok(())
    })
}

// the tasks of mods the joining client also runs, in load order
pub fn forPeer(peer: &Hello) -> Vec<Identifier>
{
    TASKS.with(|tasks|
    {
        tasks
            .borrow()
            .iter()
            .filter(|task| peer.mods.iter().any(|entry| entry.id == task.m_id.namespace()))
            .map(|task| task.m_id.clone())
            .collect()
    })
}

fn find(id: &Identifier) -> Option<(usize, Handle)>
{
    TASKS.with(|tasks| tasks.borrow().iter().find(|task| task.m_id == *id).map(|task| (task.m_mod, task.m_handler)))
}

// what the server sends this player for the task, None to skip it
pub fn payload(id: &Identifier, player: &str) -> Option<Vec<u8>>
{
    let (modIndex, handler) = find(id)?;
    let call: GuestCall = GuestCall::ConfigTask { handler, player: player.to_owned() };
    match crate::state::modAt(modIndex)?.call(&call)
    {
        Some(GuestReply::Bytes(bytes)) if !bytes.is_empty() => Some(bytes),
        Some(GuestReply::Bytes(_)) => None,
        other =>
        {
            crate::log::warn(id.namespace(), &format!("configuration task {id} answered {other:?}, skipped"));
            None
        }
    }
}

// the client side of a task, an error refuses the join with its reason
pub fn receive(id: &Identifier, data: Vec<u8>) -> Result<(), String>
{
    let (modIndex, handler) = find(id).ok_or_else(|| format!("no configuration task {id} here"))?;
    let runtime = crate::state::modAt(modIndex).ok_or_else(|| format!("{} is not loaded", id.namespace()))?;
    match runtime.call(&GuestCall::ConfigTaskData { handler, data })
    {
        Some(GuestReply::Unit) => Ok(()),
        Some(GuestReply::Error(reason)) => Err(reason),
        other => Err(format!("configuration task {id} answered {other:?}")),
    }
}
