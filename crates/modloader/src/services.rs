use crate::state;
use crate::state::ModRuntime;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, GuestCall, GuestReply, Handle};
use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

#[derive(Clone)]
struct Provider
{
    m_mod: usize,
    m_version: u32,
    m_handler: Handle,
}

#[derive(Default)]
struct Registry
{
    m_services: BTreeMap<Identifier, Provider>,
    // api id and block id to the provider that answers lookups at that block
    m_lookups: HashMap<(Identifier, i32), Provider>,
    m_messages: Vec<(usize, String, String, Vec<u8>)>,
}

thread_local!
{
    static REGISTRY: RefCell<Registry> = RefCell::new(Registry::default());
}

pub fn provide(modIndex: usize, name: Identifier, version: u32, handler: Handle) -> Result<(), String>
{
    REGISTRY.with(|registry|
    {
        let mut registry: RefMut<crate::services::Registry> = registry.borrow_mut();
        if let Some(existing) = registry.m_services.get(&name)
        {
            return Err(format!("service {name} is already provided by {}", state::modId(existing.m_mod)));
        }

        registry.m_services.insert(name, Provider { m_mod: modIndex, m_version: version, m_handler: handler });
        Ok(())
    })
}

fn provider(name: &Identifier, version: u32) -> Result<Provider, String>
{
    let found: Option<Provider> = REGISTRY.with(|registry| registry.borrow().m_services.get(name).cloned());
    let provider: Provider = found.ok_or_else(|| format!("no mod provides {name}"))?;
    if provider.m_version < version
    {
        return Err(format!("{name} is version {} but {version} is needed", provider.m_version));
    }

    Ok(provider)
}

pub fn has(name: &Identifier, version: u32) -> bool { provider(name, version).is_ok() }

// arguments and results cross through the host as bytes the providing mod's API crate defines
pub fn call(name: &Identifier, version: u32, method: String, args: Vec<u8>) -> Result<Vec<u8>, String>
{
    let provider: Provider = provider(name, version)?;
    let runtime: Rc<ModRuntime> = state::modAt(provider.m_mod).ok_or("the providing mod is gone")?;
    match runtime.call(&GuestCall::Service { handler: provider.m_handler, method, args })
    {
        Some(GuestReply::Bytes(bytes)) => Ok(bytes),
        Some(GuestReply::Unit) => Ok(Vec::new()),
        Some(other) => Err(format!("service {name} answered {other:?}")),
        None => Err(format!("service {name} failed")),
    }
}

pub fn registerLookup(modIndex: usize, api: Identifier, block: i32, handler: Handle)
{
    REGISTRY.with(|registry|
    {
        registry
            .borrow_mut()
            .m_lookups
            .insert((api, block), Provider { m_mod: modIndex, m_version: 0, m_handler: handler })
    });
}

pub fn callLookup(
    api: &Identifier,
    world: i32,
    pos: BlockPos,
    side: i32,
    method: String,
    args: Vec<u8>,
) -> Result<Option<Vec<u8>>, String>
{
    let block: i32 = crate::host::context::world(world).map_or(0, |world| world.getBlockId(pos.x, pos.y, pos.z));
    let Some(provider) = REGISTRY.with(|registry| registry.borrow().m_lookups.get(&(api.clone(), block)).cloned())
    else
    {
        return Ok(None);
    };

    let runtime: Rc<ModRuntime> = state::modAt(provider.m_mod).ok_or("the providing mod is gone")?;
    match runtime.call(&GuestCall::Lookup { handler: provider.m_handler, world, pos, side, method, args })
    {
        Some(GuestReply::Bytes(bytes)) => Ok(Some(bytes)),
        Some(GuestReply::Unit) => Ok(Some(Vec::new())),
        _ => Err(format!("lookup {api} at {pos:?} failed")),
    }
}

pub fn sendInterMod(from: usize, to: String, kind: String, data: Vec<u8>)
{
    REGISTRY.with(|registry| registry.borrow_mut().m_messages.push((from, to, kind, data)));
}

// queued messages are delivered once, in the inter_mod step after every mod finished setup
pub fn deliverInterMod()
{
    let messages: Vec<(usize, String, String, Vec<u8>)> =
        REGISTRY.with(|registry| std::mem::take(&mut registry.borrow_mut().m_messages));

    for (from, to, kind, data) in messages
    {
        let Some(target) = state::byId(&to)
        else
        {
            crate::log::warn(&state::modId(from), &format!("message {kind} for {to} dropped, {to} is not loaded"));
            continue;
        };
        target.call(&GuestCall::InterMod { from: state::modId(from), kind, data });
    }
}
