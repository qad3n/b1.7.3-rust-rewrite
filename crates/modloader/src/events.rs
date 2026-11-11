use crate::state;
use crate::state::ModRuntime;
use mod_protocol::Identifier;
use mod_protocol::abi::{EventData, EventEnvelope, EventKind, GuestCall, GuestReply, Handle, Priority};
use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone)]
struct Listener
{
    m_mod: usize,
    m_handler: Handle,
    m_priority: Priority,
    m_receiveCanceled: bool,
}

type Key = (EventKind, Option<Identifier>);

#[derive(Default)]
struct Bus
{
    m_listeners: HashMap<Key, Vec<Listener>>,
    m_custom: HashSet<Identifier>,
}

thread_local!
{
    static BUS: RefCell<Bus> = RefCell::new(Bus::default());
}

pub fn subscribe(
    modIndex: usize,
    kind: EventKind,
    custom: Option<Identifier>,
    priority: Priority,
    receiveCanceled: bool,
    handler: Handle,
) -> Result<(), String>
{
    if let Some(id) = &custom
        && !BUS.with(|bus| bus.borrow().m_custom.contains(id))
    {
        return Err(format!("custom event {id} is not defined, the defining mod must load first"));
    }

    let listener: Listener =
        Listener { m_mod: modIndex, m_handler: handler, m_priority: priority, m_receiveCanceled: receiveCanceled };

    BUS.with(|bus|
    {
        let mut bus: RefMut<Bus> = bus.borrow_mut();
        let list: &mut Vec<Listener> = bus.m_listeners.entry((kind, custom)).or_default();
        list.push(listener);
        // priority first, then mod load order, which both sides share
        list.sort_by_key(|listener| (listener.m_priority, listener.m_mod));
    });

    Ok(())
}

pub fn define(id: Identifier) { BUS.with(|bus| bus.borrow_mut().m_custom.insert(id)); }

// one hash lookup, so a hook site with no listeners skips building the event
pub fn hasListeners(kind: EventKind) -> bool
{
    BUS.with(|bus| bus.borrow().m_listeners.get(&(kind, None)).is_some_and(|list| !list.is_empty()))
}

fn listeners(key: &Key) -> Vec<Listener>
{
    BUS.with(|bus| bus.borrow().m_listeners.get(key).cloned().unwrap_or_default())
}

fn deliver(listener: &Listener, envelope: &EventEnvelope) -> Option<EventEnvelope>
{
    let runtime: Rc<ModRuntime> = state::modAt(listener.m_mod)?;
    match runtime.call(&GuestCall::Event { handler: listener.m_handler, event: envelope.clone() })
    {
        Some(GuestReply::Event(changed)) => Some(changed),
        _ => None,
    }
}

fn dispatch(key: Key, mut envelope: EventEnvelope) -> EventEnvelope
{
    for listener in listeners(&key)
    {
        if envelope.canceled && !listener.m_receiveCanceled && listener.m_priority != Priority::Monitor
        {
            continue;
        }
        let Some(changed) = deliver(&listener, &envelope)
        else
        {
            continue;
        };

        // monitor listeners see the final state and cannot change it
        if listener.m_priority == Priority::Monitor
        {
            continue;
        }

        let result: Option<i32> = envelope.result.or(changed.result);
        envelope = EventEnvelope { result, ..changed };
    }

    envelope
}

pub fn post(kind: EventKind, data: EventData) -> EventEnvelope
{
    let envelope: EventEnvelope = EventEnvelope { kind, canceled: false, result: None, data };

    if !hasListeners(kind)
    {
        return envelope;
    }

    dispatch((kind, None), envelope)
}

pub fn postCustom(id: Identifier, data: Vec<u8>) -> Result<EventEnvelope, String>
{
    if !BUS.with(|bus| bus.borrow().m_custom.contains(&id))
    {
        return Err(format!("custom event {id} is not defined"));
    }

    let envelope: EventEnvelope = EventEnvelope
    {
        kind: EventKind::Custom,
        canceled: false,
        result: None,
        data: EventData::Custom { event: id.clone(), data },
    };
    Ok(dispatch((EventKind::Custom, Some(id)), envelope))
}

pub fn removeMod(modIndex: usize)
{
    BUS.with(|bus|
    {
        for list in bus.borrow_mut().m_listeners.values_mut()
        {
            list.retain(|listener| listener.m_mod != modIndex);
        }
    });
}
