pub mod context;
mod register;
mod world;

use crate::state;
use mod_protocol::abi::{HostCall, HostReply};
use std::rc::Rc;

pub fn ok<T>(result: Result<T, String>, wrap: impl FnOnce(T) -> HostReply) -> HostReply
{
    match result
    {
        Ok(value) => wrap(value),
        Err(reason) => HostReply::Error(reason),
    }
}

pub fn unit(result: Result<(), String>) -> HostReply { ok(result, |()| HostReply::Unit) }

// every guest request lands here on the main thread, named by the mod that made it
pub fn handle(modIndex: usize, call: HostCall) -> HostReply
{
    let Some(runtime) = state::modAt(modIndex)
    else
    {
        return HostReply::Error(format!("unknown mod index {modIndex}"));
    };

    match register::handle(&runtime, call)
    {
        Ok(reply) => reply,
        Err(call) => world::handle(&runtime, *call),
    }
}

pub fn worldPlayer(id: i32) -> Result<Rc<game::entity::Entity>, String> { world::player(id) }
