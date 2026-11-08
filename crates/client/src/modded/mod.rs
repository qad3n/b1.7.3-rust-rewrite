mod draw;
mod menu;
mod model;
mod screen;

use crate::client::Minecraft;
use crate::gui::{GuiScreen, ScaledResolution};
use game::entity::{Entity, entity_player};
use game::inventory::{IInventory, InventoryBasic};
use jvm::string::JString;
use jvm::throwable::OrNpe;
pub use menu::{modsButtonVisible, modsScreen, startScreen};
pub use model::renderModel;
use modloader::screens::OpenRequest;
use std::rc::{Rc, Weak};

// what only the client can do for mods: open screens, place server entities, measure the screen
struct ClientPlatform
{
    m_mc: Weak<Minecraft>,
}

impl ClientPlatform
{
    fn mc(&self) -> Result<Rc<Minecraft>, String> { self.m_mc.upgrade().ok_or_else(|| "the client is gone".to_owned()) }
}

impl modloader::netplay::Platform for ClientPlatform
{
    fn openScreen(&self, _player: &Rc<Entity>, request: OpenRequest) -> Result<(), String>
    {
        open(&self.mc()?, request);
        Ok(())
    }

    fn teleport(&self, player: &Rc<Entity>, dimension: i32, pos: [f64; 3]) -> Result<(), String>
    {
        let current: i32 = entity_player::fields(player).m_dimension.get();
        if current != dimension
        {
            travel(&self.mc()?, player, dimension, pos);
            return Ok(());
        }
        player.setLocationAndAngles(pos[0], pos[1], pos[2], player.m_rotationYaw.get(), player.m_rotationPitch.get());
        Ok(())
    }

    fn screenSize(&self) -> Option<(i32, i32)>
    {
        let mc: Rc<Minecraft> = self.mc().ok()?;
        let resolution: ScaledResolution =
            ScaledResolution::new(&mc.m_gameSettings.get()?, mc.m_displayWidth.get(), mc.m_displayHeight.get());
        Some((resolution.getScaledWidth(), resolution.getScaledHeight()))
    }

    fn addClientEntity(&self, id: i32, entity: Rc<Entity>) -> Result<(), String>
    {
        let world = self.mc()?.m_theWorld.get().ok_or("no client world")?;
        crate::world::world_client::forceEntity(&world, id, &entity);
        Ok(())
    }
}

// single player dimension travel, as Minecraft::usePortal does for the nether but to any dimension and point
fn travel(mc: &Rc<Minecraft>, player: &Rc<Entity>, dimension: i32, pos: [f64; 3])
{
    let Some(world) = mc.m_theWorld.get()
    else
    {
        return;
    };
    entity_player::fields(player).m_dimension.set(dimension);
    world.setEntityDead(player);
    player.m_isDead.set(false);
    let target =
        game::world::World::newFromWorld(&world, game::world::WorldProvider::getProviderForDimension(dimension));
    mc.changeWorld(Some(target), "Changing dimension", Some(player));
    player.m_worldObj.set(mc.m_theWorld.get());
    player.setLocationAndAngles(pos[0], pos[1], pos[2], player.m_rotationYaw.get(), player.m_rotationPitch.get());
    if let Some(world) = mc.m_theWorld.get()
    {
        world.updateEntityWithOptionalForce(player, false);
    }
    modloader::play::dimensionChanged(player, world.worldProvider().m_worldType.get(), dimension);
}

// a single player container reads the real tile inventory, a server window mirrors it with a plain inventory
fn open(mc: &Rc<Minecraft>, request: OpenRequest)
{
    let OpenRequest { m_screen: screenType, m_pos: pos, m_data: data, m_window: window } = request;
    if screenType.m_spec.slots.is_empty()
    {
        mc.displayGuiScreen(Some(screen::plain(screenType, pos, data)));
        return;
    }
    let Some(player) = mc.m_thePlayer.get()
    else
    {
        return;
    };
    let local: Option<Rc<dyn IInventory>> = (window == 0)
        .then(|| {
            mc.m_theWorld.get().zip(pos).and_then(|(world, pos)| modloader::containers::tileInventory(&world, pos))
        })
        .flatten();
    let inventory: Rc<dyn IInventory> = local.unwrap_or_else(|| {
        let size: i32 = modloader::containers::slotCount(&screenType);
        Rc::new(InventoryBasic::new(JString::from(screenType.m_spec.title.as_str()), size))
    });
    let playerInventory: Rc<dyn IInventory> = entity_player::inventory(&player);
    let menu = modloader::containers::create(&screenType, playerInventory, inventory);
    mc.displayGuiScreen(Some(screen::container(screenType, Rc::clone(&menu), pos, data)));
    if window != 0
    {
        entity_player::fields(&player).m_craftingInventory.get().orNpe().m_windowId.set(window);
    }
}

pub fn install(mc: &Rc<Minecraft>)
{
    modloader::netplay::install(Rc::new(ClientPlatform { m_mc: Rc::downgrade(mc) }));
    let weak: Weak<Minecraft> = Rc::downgrade(mc);
    modloader::screens::setOpener(Box::new(move |request: OpenRequest| {
        if let Some(mc) = weak.upgrade()
        {
            open(&mc, request);
        }
    }));
}

// mod huds draw over the vanilla overlay, render_hud listeners see the same frame
pub fn drawHud(mc: &Rc<Minecraft>, partial: f32)
{
    if !modloader::play::active()
    {
        return;
    }
    let Some(settings) = mc.m_gameSettings.get()
    else
    {
        return;
    };
    let resolution: ScaledResolution =
        ScaledResolution::new(&settings, mc.m_displayWidth.get(), mc.m_displayHeight.get());
    for list in modloader::play::hud(partial, resolution.getScaledWidth(), resolution.getScaledHeight())
    {
        draw::draw(mc, &list, 0, 0, [-1, -1]);
    }
}

pub fn screenOpening(screen: Option<&Rc<GuiScreen>>) -> bool
{
    let name: String = screen.map_or_else(String::new, |screen| screen.className().to_owned());
    modloader::play::screenOpen(&name)
}
