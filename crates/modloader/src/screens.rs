use crate::state;
use crate::state::ModRuntime;
use mod_protocol::Identifier;
use mod_protocol::abi::
{
    BlockPos, DrawList, GuestCall, GuestReply, Handle, OverlayOp, ScreenOp, ScreenSpec, StackData,
};
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
    m_overlays: Vec<(usize, String, Handle)>,
    m_tooltips: Vec<(usize, Handle)>,
    m_layers: Vec<(usize, Handle)>,
    m_worldRenders: Vec<(usize, Handle)>,
    // set by the client, runs the world draw lists in its GL state
    m_worldDrawer: Option<Box<dyn Fn(f32)>>,
    // set by the client, absent on the dedicated server
    m_opener: Option<Opener>,
}

thread_local!
{
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

pub fn registerOverlay(modIndex: usize, target: String, handler: Handle)
{
    SCREENS.with(|screens| screens.borrow_mut().m_overlays.push((modIndex, target, handler)));
}

pub fn registerTooltip(modIndex: usize, handler: Handle)
{
    SCREENS.with(|screens| screens.borrow_mut().m_tooltips.push((modIndex, handler)));
}

pub fn registerWorldRender(modIndex: usize, handler: Handle)
{
    SCREENS.with(|screens| screens.borrow_mut().m_worldRenders.push((modIndex, handler)));
}

pub fn setWorldDrawer(drawer: Box<dyn Fn(f32)>)
{
    SCREENS.with(|screens| screens.borrow_mut().m_worldDrawer = Some(drawer));
}

// every world render handler's ops for this frame, in mod load order
pub fn worldOps(partial: f32, camera: [f64; 3]) -> Vec<Vec<mod_protocol::abi::ModelOp>>
{
    let handlers: Vec<(usize, Handle)> = SCREENS.with(|screens| screens.borrow().m_worldRenders.clone());
    handlers
        .into_iter()
        .filter_map(|(owner, handler)|
        {
            match state::modAt(owner)?.call(&GuestCall::WorldRender { handler, partial, camera })
            {
                Some(GuestReply::Pose(pose)) => Some(pose.model),
                _ => None,
            }
        })
        .collect()
}

pub fn drawWorld(partial: f32)
{
    SCREENS.with(|screens|
    {
        let screens: std::cell::Ref<Screens> = screens.borrow();
        if let Some(drawer) = screens.m_worldDrawer.as_ref()
        {
            drawer(partial);
        }
    });
}

// lines every tooltip handler adds under the item name, in mod load order
pub fn tooltipLines(stack: &StackData) -> Vec<String>
{
    let handlers: Vec<(usize, Handle)> = SCREENS.with(|screens| screens.borrow().m_tooltips.clone());
    let mut lines: Vec<String> = Vec::new();

    for (owner, handler) in handlers
    {
        let reply: Option<GuestReply> =
            state::modAt(owner).and_then(|runtime| runtime.call(&GuestCall::Tooltip { handler, stack: stack.clone() }));

        if let Some(GuestReply::Lines(added)) = reply
        {
            lines.extend(added);
        }
    }

    lines
}

fn overlaysOn(target: &str) -> Vec<(usize, Handle)>
{
    SCREENS.with(|screens|
    {
        screens
            .borrow()
            .m_overlays
            .iter()
            .filter(|(_, on, _)| on == target)
            .map(|(owner, _, handler)| (*owner, *handler))
            .collect()
    })
}

// an overlay filling the container hides every other overlay on the screen
fn shownOn(target: &str) -> Vec<(usize, Handle)>
{
    let overlays: Vec<(usize, Handle)> = overlaysOn(target);
    match overlays.iter().find(|(owner, handler)| askOverlay(*owner, *handler, OverlayOp::Exclusive))
    {
        Some(exclusive) => vec![*exclusive],
        None => overlays,
    }
}

fn callOverlay(owner: usize, handler: Handle, op: OverlayOp) -> Option<GuestReply>
{
    state::modAt(owner)?.call(&GuestCall::Overlay { handler, op })
}

pub fn drawOverlays(target: &str, size: [i32; 2], mouse: [i32; 2]) -> Vec<DrawList>
{
    shownOn(target)
        .into_iter()
        .filter_map(|(owner, handler)| match callOverlay(owner, handler, OverlayOp::Draw { size, mouse })?
        {
            GuestReply::Draw(list) => Some(list),
            _ => None,
        })
        .collect()
}

// the first overlay to take a click keeps it from the screen under it
pub fn clickOverlays(target: &str, x: i32, y: i32, button: i32) -> bool
{
    shownOn(target).into_iter().any(|(owner, handler)|
    {
        matches!(callOverlay(owner, handler, OverlayOp::Mouse { x, y, button }), Some(GuestReply::Bool(true)))
    })
}

fn askOverlay(owner: usize, handler: Handle, op: OverlayOp) -> bool
{
    let Some(runtime) = state::modAt(owner)
    else
    {
        return false;
    };

    matches!(runtime.call(&GuestCall::Overlay { handler, op }), Some(GuestReply::Bool(true)))
}

// the first overlay to take a key keeps it from the screen under it
pub fn keyOverlays(target: &str, character: u32, key: i32, hovered: Option<StackData>) -> bool
{
    shownOn(target)
        .into_iter()
        .any(|(owner, handler)| askOverlay(owner, handler, OverlayOp::Key { character, key, hovered: hovered.clone() }))
}

pub fn scrollOverlays(target: &str, x: i32, y: i32, delta: i32) -> bool
{
    shownOn(target).into_iter().any(|(owner, handler)| askOverlay(owner, handler, OverlayOp::Scroll { x, y, delta }))
}

// whether any overlay covers the point, so the vanilla slot under it is not hovered
pub fn coverOverlays(target: &str, x: i32, y: i32) -> bool
{
    shownOn(target).into_iter().any(|(owner, handler)| askOverlay(owner, handler, OverlayOp::Cover { x, y }))
}

pub fn registerPlayerLayer(modIndex: usize, handler: Handle)
{
    SCREENS.with(|screens| screens.borrow_mut().m_layers.push((modIndex, handler)));
}

pub fn playerLayers(player: &Rc<game::entity::Entity>) -> Vec<mod_protocol::abi::PlayerLayer>
{
    let layers: Vec<(usize, Handle)> = SCREENS.with(|screens| screens.borrow().m_layers.clone());
    if layers.is_empty()
    {
        return Vec::new();
    }

    let info = crate::host::context::entityInfo(player);
    layers
        .into_iter()
        .filter_map(|(owner, handler)|
        {
            let runtime: Rc<ModRuntime> = state::modAt(owner)?;
            let synced: Vec<u8> = crate::entities::synced(player, runtime.id());
            match runtime.call(&GuestCall::PlayerLayers { handler, player: info.clone(), synced })?
            {
                GuestReply::Layers(list) => Some(list),
                _ => None,
            }
        })
        .flatten()
        .collect()
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
    let runtime: Rc<ModRuntime> = state::modAt(screen.m_mod)?;
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
        .filter_map(|(modIndex, handler)|
        {
            let runtime: Rc<ModRuntime> = state::modAt(modIndex)?;
            match runtime.call(&GuestCall::Hud { handler, partial, width, height })?
            {
                GuestReply::Draw(list) => Some(list),
                _ => None,
            }
        })
        .collect()
}
