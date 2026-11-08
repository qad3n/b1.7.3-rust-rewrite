use super::draw::{self, ButtonRect};
use crate::client::Minecraft;
use crate::gui::gui_container::{self, ContainerScreen, GUI_CONTAINER};
use crate::gui::{GUI_SCREEN, GuiScreen, GuiScreenClass};
use crate::render::RenderEngine;
use game::container::Container;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use modloader::mod_protocol::abi::{BlockPos, DrawList, ScreenOp};
use modloader::screens::{self, ScreenType};
use std::cell::RefCell;
use std::rc::Rc;

// the guest owns what a mod screen shows, the host only replays its draw list and forwards input
pub struct ModScreen
{
    m_screen: Rc<ScreenType>,
    m_pos: Option<BlockPos>,
    m_data: Vec<u8>,
    m_buttons: RefCell<Vec<ButtonRect>>,
}

fn state(this: &GuiScreen) -> Rc<ModScreen>
{
    match &gui_container::fields(this).m_screen
    {
        ContainerScreen::Mod { m_state } => match Rc::clone(m_state).downcast::<ModScreen>()
        {
            Ok(state) => state,
            Err(_) => unreachable!("a mod container screen holds mod state"),
        },
        _ => unreachable!("only mod screens use the mod screen classes"),
    }
}

fn plainState(this: &GuiScreen) -> &ModScreen { this.ext::<ModScreen>() }

fn send(screen: &ModScreen, op: ScreenOp) -> Option<DrawList> { screens::call(&screen.m_screen, op) }

fn init(screen: &ModScreen, width: i32, height: i32)
{
    send(screen, ScreenOp::Init { width, height, pos: screen.m_pos, data: screen.m_data.clone() });
}

fn paint(mc: &Rc<Minecraft>, screen: &ModScreen, origin: (i32, i32), mouse: [i32; 2], partial: f32)
{
    if let Some(list) = send(screen, ScreenOp::Draw { mouse: [mouse[0] - origin.0, mouse[1] - origin.1], partial })
    {
        *screen.m_buttons.borrow_mut() = draw::draw(mc, &list, origin.0, origin.1, mouse);
    }
}

// true when the click landed on a guest button
fn click(screen: &ModScreen, x: i32, y: i32, button: i32, origin: (i32, i32)) -> bool
{
    let hit: Option<i32> = screen.m_buttons.borrow().iter().find(|rect| rect.contains(x, y)).map(|rect| rect.m_id);
    match hit
    {
        Some(id) if button == 0 =>
        {
            send(screen, ScreenOp::Button { id });
            true
        }
        _ =>
        {
            send(screen, ScreenOp::Mouse { x: x - origin.0, y: y - origin.1, button });
            false
        }
    }
}

const PLAIN: GuiScreenClass = GuiScreenClass {
    m_name: "GuiModScreen",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| init(plainState(this), this.m_width.get(), this.m_height.get()),
    drawScreen: |this, mouseX, mouseY, partial| {
        this.drawDefaultBackground();
        let screen: &ModScreen = plainState(this);
        let spec = &screen.m_screen.m_spec;
        let origin: (i32, i32) = ((this.m_width.get() - spec.width) / 2, (this.m_height.get() - spec.height) / 2);
        paint(&this.mc(), screen, origin, [mouseX, mouseY], partial);
    },
    mouseClicked: |this, mouseX, mouseY, button| {
        let screen: &ModScreen = plainState(this);
        let spec = &screen.m_screen.m_spec;
        let origin: (i32, i32) = ((this.m_width.get() - spec.width) / 2, (this.m_height.get() - spec.height) / 2);
        click(screen, mouseX, mouseY, button, origin);
    },
    keyTyped: |this, chr, key| {
        send(plainState(this), ScreenOp::Key { character: chr as u32, key });
        (GUI_SCREEN.keyTyped)(this, chr, key);
    },
    updateScreen: |this| {
        send(plainState(this), ScreenOp::Tick);
    },
    onGuiClosed: |this| {
        send(plainState(this), ScreenOp::Close);
    },
    doesGuiPauseGame: |_| false,
    ..GUI_SCREEN
};

const CONTAINER: GuiScreenClass = GuiScreenClass {
    m_name: "GuiModContainer",
    m_parent: Some(&GUI_CONTAINER),
    m_interfaces: &[],
    initGui: |this| {
        (GUI_CONTAINER.initGui)(this);
        init(&state(this), this.m_width.get(), this.m_height.get());
    },
    drawGuiContainerBackgroundLayer: |this, partial| {
        let screen: Rc<ModScreen> = state(this);
        let mc: Rc<Minecraft> = this.mc();
        let origin: (i32, i32) = gui_container::origin(this);
        if let Some(background) =
            screen.m_screen.m_spec.background.as_deref().and_then(modloader::entities::modTexturePath)
        {
            let engine: Rc<RenderEngine> = mc.m_renderEngine.get().orNpe();
            GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
            engine.bindTexture(engine.getTexture(&background));
            let fields = gui_container::fields(this);
            this.m_gui.drawTexturedModalRect(origin.0, origin.1, 0, 0, fields.m_xSize.get(), fields.m_ySize.get());
        }
        paint(&mc, &screen, origin, [0, 0], partial);
    },
    drawGuiContainerForegroundLayer: |this| {
        let screen: Rc<ModScreen> = state(this);
        this.fontRenderer().drawString(&screen.m_screen.m_spec.title, 8, 6, 0x404040);
    },
    mouseClicked: |this, mouseX, mouseY, button| {
        if !click(&state(this), mouseX, mouseY, button, gui_container::origin(this))
        {
            (GUI_CONTAINER.mouseClicked)(this, mouseX, mouseY, button);
        }
    },
    updateScreen: |this| {
        (GUI_CONTAINER.updateScreen)(this);
        send(&state(this), ScreenOp::Tick);
    },
    onGuiClosed: |this| {
        send(&state(this), ScreenOp::Close);
        (GUI_CONTAINER.onGuiClosed)(this);
    },
    ..GUI_CONTAINER
};

pub fn plain(screen: Rc<ScreenType>, pos: Option<BlockPos>, data: Vec<u8>) -> Rc<GuiScreen>
{
    GuiScreen::construct(
        &PLAIN,
        Box::new(ModScreen { m_screen: screen, m_pos: pos, m_data: data, m_buttons: RefCell::new(Vec::new()) }),
    )
}

pub fn container(screen: Rc<ScreenType>, menu: Rc<Container>, pos: Option<BlockPos>, data: Vec<u8>) -> Rc<GuiScreen>
{
    let (width, height): (i32, i32) = (screen.m_spec.width, screen.m_spec.height);
    let state: Rc<dyn std::any::Any> =
        Rc::new(ModScreen { m_screen: screen, m_pos: pos, m_data: data, m_buttons: RefCell::new(Vec::new()) });
    let gui: Rc<GuiScreen> = gui_container::construct(&CONTAINER, menu, ContainerScreen::Mod { m_state: state });
    gui_container::fields(&gui).m_xSize.set(width);
    gui_container::fields(&gui).m_ySize.set(height);
    gui
}
