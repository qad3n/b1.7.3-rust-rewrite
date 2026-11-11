mod config_menu;
mod draw;
pub mod entity_model;
mod menu;
mod model;
pub mod music;
pub mod particles;
pub mod player_layers;
mod screen;
mod stack_draw;
pub mod textures;

use crate::client::Minecraft;
use crate::gui::{GuiIngame, GuiScreen, ScaledResolution};
use draw::ButtonRect;
use game::container::Container;
use game::entity::{Entity, entity_player};
use game::inventory::{IInventory, InventoryPlayer};
use game::item::ItemStack;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
pub use menu::{MODS_BUTTON, addModsButton, drawModCount, modsScreen, startScreen};
pub use model::{movingShapes, renderItemModel, renderModel, renderOverlays};
use modloader::mod_protocol::abi::{DrawList, StackData};
use modloader::screens::OpenRequest;
use std::cell::{RefCell, RefMut};
use std::rc::{Rc, Weak};

const CLICK_SOUND: &str = "random.click";

thread_local!
{
    // buttons the container overlays drew last frame, kept so a click can find them
    static OVERLAY_BUTTONS: RefCell<Vec<ButtonRect>> = const { RefCell::new(Vec::new()) };
}

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
            travel(&*self.mc()?, player, dimension, pos);
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

    fn textWidth(&self, text: &str) -> i32
    {
        self.mc().ok().and_then(|mc| mc.m_fontRenderer.get()).map_or(0, |font| font.getStringWidth(text))
    }

    fn addClientEntity(&self, id: i32, entity: Rc<Entity>) -> Result<(), String>
    {
        let world: Rc<game::world::World> = self.mc()?.m_theWorld.get().ok_or("no client world")?;
        crate::world::world_client::forceEntity(&world, id, &entity);
        Ok(())
    }
}

// single player dimension travel, as Minecraft::usePortal does for the nether but to any dimension and point
pub(crate) fn travel(mc: &Minecraft, player: &Rc<Entity>, dimension: i32, pos: [f64; 3])
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
        .then(||
        {
            mc.m_theWorld.get().zip(pos).and_then(|(world, pos)| modloader::containers::tileInventory(&world, pos))
        })
        .flatten();
    let inventory: Rc<dyn IInventory> = local.unwrap_or_else(|| modloader::containers::scratchInventory(&screenType));
    let playerInventory: Rc<dyn IInventory> = entity_player::inventory(&player);
    let menu: Rc<Container> =
        modloader::containers::create(&screenType, &player, playerInventory, inventory, pos.is_none());
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

    modloader::screens::setOpener(Box::new(move |request: OpenRequest|
    {
        if let Some(mc) = weak.upgrade()
        {
            open(&mc, request);
        }
    }));

    let weak: Weak<Minecraft> = Rc::downgrade(mc);

    modloader::screens::setWorldDrawer(Box::new(move |partial: f32|
    {
        let Some(mc) = weak.upgrade()
        else
        {
            return;
        };

        // the render engine exists once a world draws, the mod sprite effect joins vanilla's list then
        if let Some(engine) = mc.m_renderEngine.get()
        {
            textures::register(&engine);
        }

        drawWorld(&mc, partial);
    }));
}

// world render handlers draw from the camera, the interpolated view entity, in the GL state the world left
fn drawWorld(mc: &Rc<Minecraft>, partial: f32)
{
    let Some(viewer) = mc.m_renderViewEntity.get()
    else
    {
        return;
    };

    let between = |last: f64, now: f64| last + (now - last) * f64::from(partial);
    let camera: [f64; 3] = [
        between(viewer.m_lastTickPosX.get(), viewer.posX()),
        between(viewer.m_lastTickPosY.get(), viewer.posY()),
        between(viewer.m_lastTickPosZ.get(), viewer.posZ()),
    ];
    // world draw lists are sheets seen from both sides, as vanilla weather is
    GL11::glDisable(GL11::GL_CULL_FACE);

    for ops in modloader::screens::worldOps(partial, camera)
    {
        GL11::glPushMatrix();

        for op in &ops
        {
            entity_model::glOp(op, false);
        }

        GL11::glPopMatrix();
        // a handler leaves lighting as it found it, colour and blend come back to the world's
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glDisable(GL11::GL_BLEND);
    }

    GL11::glEnable(GL11::GL_CULL_FACE);
}

const UNKNOWN_COMMAND: &str = "Unknown command, try /help";

// single player has no server, so a slash line runs mod commands here and any other line shows as chat
pub fn localChat(mc: &Rc<Minecraft>, player: &Rc<Entity>, line: &str)
{
    let gui: Rc<GuiIngame> = mc.m_ingameGUI.get().orNpe();

    let reply: Option<String> = match line.strip_prefix('/')
    {
        Some(command) if command.split_whitespace().next() == Some("gamerule") =>
        {
            let args: Vec<&str> = command.split_whitespace().skip(1).collect();
            Some(game::gamerule::command(&player.world().rules(), &args).unwrap_or_else(|error| error))
        }

        Some(command) =>
        {
            Some(modloader::play::command(command, Some(player), true).unwrap_or_else(|| UNKNOWN_COMMAND.to_owned()))
        }

        None => modloader::play::chat(player, line).map(|message|
        {
            let name: String =
                entity_player::fields(player).m_username.get().map(|name| name.to_string()).unwrap_or_default();
            format!("<{name}> {message}")
        }),
    };
    for row in reply.iter().flat_map(|reply| reply.lines())
    {
        gui.addChatMessage(row);
    }
}

// mod overlays on a vanilla container screen, drawn in screen space from the container's corner
pub fn drawOverlays(screen: &Rc<crate::gui::GuiScreen>, origin: (i32, i32), size: [i32; 2], mouse: [i32; 2])
{
    if !modloader::play::active()
    {
        return;
    }

    // slot items the container drew would hide overlay items behind their depth
    GL11::glClear(GL11::GL_DEPTH_BUFFER_BIT);

    let relative: [i32; 2] = [mouse[0] - origin.0, mouse[1] - origin.1];
    let lists: Vec<DrawList> = modloader::screens::drawOverlays(screen.className(), size, relative);

    let mc: Rc<Minecraft> = screen.mc();

    OVERLAY_BUTTONS.with(|buttons|
    {
        let mut rects: RefMut<'_, Vec<ButtonRect>> = buttons.borrow_mut();
        rects.clear();

        for list in &lists
        {
            rects.extend(draw::draw(&mc, list, origin.0, origin.1, mouse));
        }
    });
}

// an overlay that took a left click on one of its buttons gets the vanilla button sound, positions in screen space
pub fn overlayClicked(mc: &Minecraft, x: i32, y: i32, button: i32)
{
    let hit: bool = OVERLAY_BUTTONS.with(|buttons| buttons.borrow().iter().any(|rect| rect.contains(x, y)));
    if button == 0 && hit
    {
        clickSound(mc);
    }
}

// the sound GuiScreen.mouseClicked plays for a pressed button
fn clickSound(mc: &Minecraft) { mc.m_sndManager.get().orNpe().playSoundFX(CLICK_SOUND, 1.0, 1.0); }

// lines mods add under the item name of a slot tooltip
pub fn tooltipLines(stack: &ItemStack) -> Vec<String>
{
    if !modloader::play::active()
    {
        return Vec::new();
    }

    modloader::screens::tooltipLines(&modloader::host::context::stackData(stack))
}

// a key typed on a vanilla container screen reaches mod overlays first, with the stack in the slot under the mouse
pub fn overlayKey(screen: &GuiScreen, character: u16, key: i32, hovered: Option<Rc<ItemStack>>) -> bool
{
    let hovered: Option<StackData> = hovered.map(|stack| modloader::host::context::stackData(&stack));
    modloader::play::active() && modloader::screens::keyOverlays(screen.className(), u32::from(character), key, hovered)
}

// whether a mod overlay covers the point on a vanilla container screen, positions from the container's corner
pub fn overlayCovered(screen: &GuiScreen, x: i32, y: i32) -> bool
{
    modloader::play::active() && modloader::screens::coverOverlays(screen.className(), x, y)
}

// the mouse wheel on a vanilla container screen, positions from the container's corner
pub fn overlayScroll(screen: &GuiScreen, x: i32, y: i32, delta: i32) -> bool
{
    modloader::play::active() && modloader::screens::scrollOverlays(screen.className(), x, y, delta)
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

fn devToolsEnabled() -> bool
{
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("B173_DEVSHOT").is_some())
}

// with B173_DEVSHOT set, a devcmd.request file in the game folder runs test lines on the main thread and answers in
// devcmd.done: world <save> [seed], quit, set <x> <y> <z> <id>, hold <item> [count], held, use, and /<mod command> such as /loader column 0 8 8
pub fn devCommands(mc: &Minecraft)
{
    if !devToolsEnabled()
    {
        return;
    }
    let Some(dir) = Minecraft::minecraftDir()
    else
    {
        return;
    };
    let Ok(text) = std::fs::read_to_string(dir.join("devcmd.request"))
    else
    {
        return;
    };

    if let Err(error) = std::fs::remove_file(dir.join("devcmd.request"))
    {
        modloader::log::warn(modloader::log::LOADER, &format!("devcmd request not removed: {error}"));
        return;
    }

    let replies: Vec<String> =
        text.lines().filter(|line| !line.trim().is_empty()).map(|line| devCommand(mc, line.trim())).collect();

    if let Err(error) = std::fs::write(dir.join("devcmd.done"), replies.join("\n"))
    {
        modloader::log::warn(modloader::log::LOADER, &format!("devcmd result not written: {error}"));
    }
}

fn devCommand(mc: &Minecraft, line: &str) -> String
{
    let words: Vec<&str> = line.split_whitespace().collect();
    match words.as_slice()
    {
        ["world", save, rest @ ..] =>
        {
            let seed: i64 = rest.first().and_then(|seed| seed.parse().ok()).unwrap_or(0);
            // as GuiSelectWorld and GuiCreateWorld do before a single player world starts
            mc.m_playerController.set(Some(crate::controller::player_controller_sp::new(&mc.rc())));
            mc.startWorld(save, save, seed);
            mc.displayGuiScreen(None);
            format!("opened {save}")
        }

        ["quit"] =>
        {
            // as the pause menu's quit button, a server hears that the player left
            if mc.isMultiplayerWorld()
                && let Some(world) = mc.m_theWorld.get()
            {
                world.sendQuittingDisconnectingPacket();
            }

            mc.changeWorld1(None);
            mc.displayGuiScreen(Some(crate::gui::GuiMainMenu::new()));
            "quit to title".to_owned()
        }

        ["hold", item, rest @ ..] =>
        {
            let count: i32 = rest.first().and_then(|count| count.parse().ok()).unwrap_or(1);

            let Some(player) = mc.m_thePlayer.get()
            else
            {
                return "no player".to_owned();
            };
            let Some(id) = itemByName(item)
            else
            {
                return format!("no item {item}");
            };

            let inventory: Rc<InventoryPlayer> = entity_player::inventory(&player);
            inventory.setInventorySlotContents(
                inventory.m_currentItem.get(),
                Some(game::item::ItemStack::withDamage(id, count, 0)),
            );
            format!("holding {count} {item}")
        }

        ["set", x, y, z, block] =>
        {
            let Some(world) = mc.m_theWorld.get()
            else
            {
                return "no world".to_owned();
            };
            let numbers: Option<Vec<i32>> = [x, y, z, block].iter().map(|text| text.parse().ok()).collect();
            let Some([x, y, z, block]) =
                numbers.as_deref().map(|numbers| [numbers[0], numbers[1], numbers[2], numbers[3]])
            else
            {
                return format!("set takes numbers: {line}");
            };

            world.setBlockWithNotify(x, y, z, block);
            format!("set {x} {y} {z} to {block}")
        }

        ["held"] =>
        {
            let held: Option<Rc<game::item::ItemStack>> =
                mc.m_thePlayer.get().and_then(|player| entity_player::inventory(&player).getCurrentItem());

            held.map_or("empty hand".to_owned(), |stack|
            {
                format!(
                    "{} x{} components {:?}",
                    stack.m_itemID.get(),
                    stack.m_stackSize.get(),
                    stack.m_components.borrow()
                )
            })
        }

        ["use"] =>
        {
            mc.clickMouse(1);
            "used held item".to_owned()
        }

        _ => match line.strip_prefix('/')
        {
            Some(command) => modloader::play::command(command, mc.m_thePlayer.get().as_ref(), true)
                .unwrap_or_else(|| format!("unknown command {command}")),
            None => format!("unknown dev line {line}"),
        },
    }
}

fn itemByName(name: &str) -> Option<i32>
{
    let id: modloader::Identifier = modloader::Identifier::parse(name).ok()?;
    modloader::registry::with(|registries| registries.itemId(&id))
}

// with B173_DEVSHOT set, a devshot.request file in the game folder takes one screenshot of the game window
pub fn devShot(width: i32, height: i32)
{
    if !devToolsEnabled()
    {
        return;
    }
    let Some(dir) = Minecraft::minecraftDir()
    else
    {
        return;
    };

    let request: std::path::PathBuf = dir.join("devshot.request");
    if std::fs::remove_file(&request).is_ok()
    {
        let message: String = crate::client::ScreenShotHelper::saveScreenshot(Some(&dir), width, height);
        if let Err(error) = std::fs::write(dir.join("devshot.done"), message)
        {
            modloader::log::warn(modloader::log::LOADER, &format!("devshot result not written: {error}"));
        }
    }
}
