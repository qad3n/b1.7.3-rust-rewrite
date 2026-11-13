pub mod context;
mod register;
mod world;

use crate::state;
use mod_protocol::abi::{HostCall, HostReply};
use mod_protocol::rules::CallSide;
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

    if call.side() == CallSide::Authority && context::mirroring()
    {
        return HostReply::Error(refusal(&call));
    }

    match register::handle(&runtime, call)
    {
        Ok(reply) => reply,
        Err(call) => world::handle(&runtime, *call),
    }
}

// the call's name is the first word of its debug form
fn refusal(call: &HostCall) -> String
{
    let debug: String = format!("{call:?}");
    let name: &str = debug.split(|character: char| !character.is_ascii_alphanumeric()).next().unwrap_or("this call");

    format!(
        "{name} changes the world and only runs where the world is owned: this client mirrors a server's world, \
         so send a payload to the server and make the change there"
    )
}

pub fn worldPlayer(id: i32) -> Result<Rc<game::entity::Entity>, String> { world::player(id) }
