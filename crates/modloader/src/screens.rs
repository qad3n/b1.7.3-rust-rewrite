use crate::state;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, DrawList, GuestCall, GuestReply, Handle, ScreenOp, ScreenSpec};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone)]
pub struct ScreenType
{
    pub m_id: Identifier,
    pub m_mod: usize,
    pub m_spec: ScreenSpec,
    pub m_handler: Handle,
}

pub struct OpenRequest
{
    pub m_screen: Rc<ScreenType>,
    pub m_pos: Option<BlockPos>,
    pub m_data: Vec<u8>,
    pub m_window: i32,
}

pub type Opener = Box<dyn Fn(OpenRequest)>;

#[derive(Default)]
struct Screens
{
    m_types: BTreeMap<Identifier, Rc<ScreenType>>,
    m_huds: Vec<(usize, Handle)>,
    // set by the client, absent on the dedicated server
    m_opener: Option<Opener>,
}

thread_local! {
    static SCREENS: RefCell<Screens> = RefCell::new(Screens::default());
}

pub fn register(modIndex: usize, id: Identifier, spec: ScreenSpec, handler: Handle)
{
    let screen: ScreenType = ScreenType { m_id: id.clone(), m_mod: modIndex, m_spec: spec, m_handler: handler };
    SCREENS.with(|screens| screens.borrow_mut().m_types.insert(id, Rc::new(screen)));
}

pub fn byId(id: &Identifier) -> Option<Rc<ScreenType>>
{
    SCREENS.with(|screens| screens.borrow().m_types.get(id).cloned())
}

// screen numeric ids are their index in sorted identifier order, the same on every side with the same mods
pub fn numeric(id: &Identifier) -> Option<i32>
{
    SCREENS.with(|screens| screens.borrow().m_types.keys().position(|key| key == id).map(|index| index as i32))
}

pub fn byNumeric(numeric: i32) -> Option<Rc<ScreenType>>
{
    SCREENS.with(|screens| screens.borrow().m_types.values().nth(numeric.max(0) as usize).cloned())
}

pub fn registerHud(modIndex: usize, handler: Handle)
{
    SCREENS.with(|screens| screens.borrow_mut().m_huds.push((modIndex, handler)));
}

pub fn hasHuds() -> bool { SCREENS.with(|screens| !screens.borrow().m_huds.is_empty()) }

pub fn setOpener(opener: Opener) { SCREENS.with(|screens| screens.borrow_mut().m_opener = Some(opener)); }

pub fn openLocal(request: OpenRequest) -> bool
{
    SCREENS.with(|screens| match &screens.borrow().m_opener
    {
        Some(opener) =>
        {
            opener(request);
            true
        }
        None => false,
    })
}

pub fn call(screen: &ScreenType, op: ScreenOp) -> Option<DrawList>
{
    let runtime = state::modAt(screen.m_mod)?;
    match runtime.call(&GuestCall::Screen { handler: screen.m_handler, op })?
    {
        GuestReply::Draw(list) => Some(list),
        _ => None,
    }
}

pub fn drawHuds(partial: f32, width: i32, height: i32) -> Vec<DrawList>
{
    let huds: Vec<(usize, Handle)> = SCREENS.with(|screens| screens.borrow().m_huds.clone());
    huds.into_iter()
        .filter_map(|(modIndex, handler)| {
            let runtime = state::modAt(modIndex)?;
            match runtime.call(&GuestCall::Hud { handler, partial, width, height })?
            {
                GuestReply::Draw(list) => Some(list),
                _ => None,
            }
        })
        .collect()
}
