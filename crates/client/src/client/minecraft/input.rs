use super::atLeastOne;
use crate::client::Minecraft;
use crate::entity::entity_player_sp;
use crate::gui::{
    GuiChat, GuiGameOver, GuiIngameMenu, GuiInventory, GuiMainMenu, GuiScreen, ScaledResolution, newSleepMP,
};
use crate::settings::{Component, EnumOptions, GameSettings};
use game::block::Block;
use game::entity::{Entity, entity_player};
use game::inventory::InventoryPlayer;
use game::item::ItemStack;
use game::math::{EnumMovingObjectType, MovingObjectPosition, math_helper};
use game::world::World;
use jvm::throwable::OrNpe;
use lwjgl::input::{Keyboard, Mouse};
use lwjgl::opengl::Display;
use std::rc::Rc;

const LICENSE_CHECK_TICK: i32 = 6000;
const PAID_RESPONSE_CODE: i32 = 400;

fn inventory(player: &Entity) -> Rc<InventoryPlayer> { entity_player::fields(player).m_inventory.get().orNpe() }

impl Minecraft
{
    pub fn displayGuiScreen(&self, mut screen: Option<Rc<GuiScreen>>)
    {
        if self.m_currentScreen.get().is_some_and(|current| current.instanceOf("GuiUnused"))
        {
            return;
        }
        if let Some(current) = self.m_currentScreen.get()
        {
            current.onGuiClosed();
        }
        if screen.as_ref().is_some_and(|screen| screen.instanceOf("GuiMainMenu"))
        {
            self.statFileWriter().func_27175_b();
        }
        self.statFileWriter().syncStats();
        if screen.is_none() && self.m_theWorld.with(Option::is_none)
        {
            screen = Some(GuiMainMenu::new());
        }
        else if screen.is_none() && game::entity::entity_living::fields(&self.thePlayer()).m_health.get() <= 0
        {
            screen = Some(GuiGameOver::new());
        }
        if screen.as_ref().is_some_and(|screen| screen.instanceOf("GuiMainMenu"))
        {
            self.ingameGUI().clearChatMessages();
        }
        self.m_currentScreen.set(screen.clone());
        match screen
        {
            Some(screen) =>
            {
                self.setIngameNotInFocus();
                let scaledResolution: ScaledResolution =
                    ScaledResolution::new(&self.gameSettings(), self.m_displayWidth.get(), self.m_displayHeight.get());
                let n: i32 = scaledResolution.getScaledWidth();
                let n2: i32 = scaledResolution.getScaledHeight();
                screen.setWorldAndResolution(&self.rc(), n, n2);
                self.m_skipRenderWorld.set(false);
            }
            None => self.setIngameFocus(),
        }
    }

    pub fn setIngameFocus(&self)
    {
        if !Display::isActive()
        {
            return;
        }
        if self.m_inGameHasFocus.get()
        {
            return;
        }
        self.m_inGameHasFocus.set(true);
        self.m_mouseHelper.get().orNpe().grabMouseCursor();
        self.displayGuiScreen(None);
        self.m_leftClickCounter.set(10000);
        self.m_mouseTicksRan.set(self.m_ticksRan.get() + 10000);
    }

    pub fn setIngameNotInFocus(&self)
    {
        if !self.m_inGameHasFocus.get()
        {
            return;
        }
        if let Some(player) = self.m_thePlayer.get()
        {
            entity_player_sp::resetPlayerKeyState(&player);
        }
        self.m_inGameHasFocus.set(false);
        self.m_mouseHelper.get().orNpe().ungrabMouseCursor();
    }

    pub fn displayInGameMenu(&self)
    {
        if self.m_currentScreen.with(Option::is_some)
        {
            return;
        }
        self.displayGuiScreen(Some(GuiIngameMenu::new()));
    }

    fn handleMouseDown(&self, button: i32, holdingAttack: bool)
    {
        if self.playerController().m_creative.get()
        {
            return;
        }
        if !holdingAttack
        {
            self.m_leftClickCounter.set(0);
        }
        if button == 0 && self.m_leftClickCounter.get() > 0
        {
            return;
        }
        let target: Option<MovingObjectPosition> = self.m_objectMouseOver.get();
        match target
        {
            Some(hit) if holdingAttack && hit.m_typeOfHit == EnumMovingObjectType::TILE && button == 0 =>
            {
                self.playerController().sendBlockRemoving(hit.m_blockX, hit.m_blockY, hit.m_blockZ, hit.m_sideHit);
                self.m_effectRenderer.get().orNpe().addBlockHitEffects(
                    hit.m_blockX,
                    hit.m_blockY,
                    hit.m_blockZ,
                    hit.m_sideHit,
                );
            }
            _ => self.playerController().resetBlockRemoving(),
        }
    }

    fn clickMouse(&self, button: i32)
    {
        if button == 0 && self.m_leftClickCounter.get() > 0
        {
            return;
        }
        let player: Rc<Entity> = self.thePlayer();
        if button == 0
        {
            player.swingItem();
        }
        let mut bl: bool = true;
        match self.m_objectMouseOver.get()
        {
            None =>
            {
                if button == 0 && !self.playerController().instanceOf("PlayerControllerTest")
                {
                    self.m_leftClickCounter.set(10);
                }
            }
            Some(hit) if hit.m_typeOfHit == EnumMovingObjectType::ENTITY =>
            {
                let target: Rc<Entity> = hit.m_entityHit.clone().orNpe();
                if button == 0
                {
                    self.playerController().attackEntity(&player, &target);
                }
                if button == 1
                {
                    self.playerController().interactWithEntity(&player, &target);
                }
            }
            Some(hit) if hit.m_typeOfHit == EnumMovingObjectType::TILE =>
            {
                let n: i32 = hit.m_blockX;
                let n2: i32 = hit.m_blockY;
                let n3: i32 = hit.m_blockZ;
                let n4: i32 = hit.m_sideHit;
                if button == 0
                {
                    self.playerController().clickBlock(n, n2, n3, hit.m_sideHit);
                }
                else
                {
                    let inventory: Rc<InventoryPlayer> = inventory(&player);
                    let itemStack2: Option<Rc<ItemStack>> = inventory.getCurrentItem();
                    let n5: i32 = itemStack2.as_ref().map_or(0, |stack| stack.m_stackSize.get());
                    let world: Rc<World> = self.theWorld();
                    if self.playerController().sendPlaceBlock(&player, &world, itemStack2.as_ref(), n, n2, n3, n4)
                    {
                        bl = false;
                        player.swingItem();
                    }
                    let Some(itemStack2) = itemStack2
                    else
                    {
                        return;
                    };
                    if itemStack2.m_stackSize.get() == 0
                    {
                        inventory.m_mainInventory.borrow_mut()[inventory.m_currentItem.get() as usize] = None;
                    }
                    else if itemStack2.m_stackSize.get() != n5
                    {
                        self.entityRenderer().m_itemRenderer.onBlockUsed();
                    }
                }
            }
            Some(_) =>
            {}
        }
        if bl
            && button == 1
            && let Some(itemStack) = inventory(&player).getCurrentItem()
            && self.playerController().sendUseItem(&player, &self.theWorld(), &itemStack)
        {
            self.entityRenderer().m_itemRenderer.onItemUsed();
        }
    }

    pub fn toggleFullscreen(&self)
    {
        if let Err(exception) = self.switchFullscreen()
        {
            exception.printStackTrace();
        }
    }

    fn switchFullscreen(&self) -> Result<(), lwjgl::LWJGLException>
    {
        self.m_fullscreen.set(!self.m_fullscreen.get());
        if self.m_fullscreen.get()
        {
            Display::setDisplayMode(Display::getDesktopDisplayMode())?;
            self.m_displayWidth.set(atLeastOne(Display::getDisplayMode().getWidth()));
            self.m_displayHeight.set(atLeastOne(Display::getDisplayMode().getHeight()));
        }
        else
        {
            match self.m_mcCanvas.get()
            {
                Some(canvas) =>
                {
                    self.m_displayWidth.set(canvas.getWidth());
                    self.m_displayHeight.set(canvas.getHeight());
                }
                None =>
                {
                    self.m_displayWidth.set(self.m_tempDisplayWidth.get());
                    self.m_displayHeight.set(self.m_tempDisplayHeight.get());
                }
            }
            self.m_displayWidth.set(atLeastOne(self.m_displayWidth.get()));
            self.m_displayHeight.set(atLeastOne(self.m_displayHeight.get()));
        }
        if self.m_currentScreen.with(Option::is_some)
        {
            self.resize(self.m_displayWidth.get(), self.m_displayHeight.get());
        }
        Display::setFullscreen(self.m_fullscreen.get())?;
        Display::update();
        Ok(())
    }

    pub(super) fn resize(&self, width: i32, height: i32)
    {
        let width: i32 = atLeastOne(width);
        let height: i32 = atLeastOne(height);
        self.m_displayWidth.set(width);
        self.m_displayHeight.set(height);
        if let Some(screen) = self.m_currentScreen.get()
        {
            let scaledResolution: ScaledResolution = ScaledResolution::new(&self.gameSettings(), width, height);
            let n: i32 = scaledResolution.getScaledWidth();
            let n2: i32 = scaledResolution.getScaledHeight();
            screen.setWorldAndResolution(&self.rc(), n, n2);
        }
    }

    fn clickMiddleMouseButton(&self)
    {
        let Some(hit) = self.m_objectMouseOver.get()
        else
        {
            return;
        };
        let mut n: i32 = self.theWorld().getBlockId(hit.m_blockX, hit.m_blockY, hit.m_blockZ);
        if n == Block::grass().m_blockID
        {
            n = Block::dirt().m_blockID;
        }
        if n == Block::stairDouble().m_blockID
        {
            n = Block::stairSingle().m_blockID;
        }
        if n == Block::bedrock().m_blockID
        {
            n = Block::stone().m_blockID;
        }
        inventory(&self.thePlayer()).setCurrentItem(n, self.playerController().instanceOf("PlayerControllerTest"));
    }

    // the https login url fails on the plain http stand-in like the dead endpoint
    fn initLicenseCheckThread(&self)
    {
        let session: Rc<crate::client::Session> = self.m_session.get().orNpe();
        let url: String = format!(
            "https://login.minecraft.net/session?name={}&session={}",
            session.m_username.get().unwrap_or_else(|| "null".to_owned()),
            session.m_sessionId.get().unwrap_or_else(|| "null".to_owned()),
        );
        let started: std::io::Result<std::thread::JoinHandle<()>> =
            jvm::thread::start(jvm::thread::nextName(), move || match jvm::net::getResponseCode(&url)
            {
                Ok(PAID_RESPONSE_CODE) => Minecraft::setHasPaidCheckTime(jvm::system::currentTimeMillis()),
                Ok(_) =>
                {}
                Err(exception) => exception.printStackTrace(),
            });
        if let Err(error) = started
        {
            jvm::io::fromStd(&error).printStackTrace();
        }
    }

    pub fn runTick(&self)
    {
        if self.m_ticksRan.get() == LICENSE_CHECK_TICK
        {
            self.initLicenseCheckThread();
        }
        self.statFileWriter().tick();
        self.ingameGUI().updateTick();
        self.entityRenderer().getMouseOver(1.0);
        if let Some(player) = self.m_thePlayer.get()
            && let Some(provider) = self.theWorld().getIChunkProvider().asLoadOrGenerate()
        {
            let n: i32 = math_helper::floor_float(player.posX() as i32 as f32) >> 4;
            let n2: i32 = math_helper::floor_float(player.posZ() as i32 as f32) >> 4;
            provider.setCurrentChunkOver(n, n2);
        }
        if !self.isGamePaused() && self.m_theWorld.with(Option::is_some)
        {
            self.playerController().updateController();
        }
        lwjgl::opengl::GL11::glBindTexture(3553, self.renderEngine().getTexture("/terrain.png"));
        if !self.isGamePaused()
        {
            self.renderEngine().updateDynamicTextures();
        }
        self.updateScreenForPlayer();
        if self.m_currentScreen.with(Option::is_some)
        {
            self.m_leftClickCounter.set(10000);
            self.m_mouseTicksRan.set(self.m_ticksRan.get() + 10000);
        }
        if let Some(screen) = self.m_currentScreen.get()
        {
            screen.handleInput();
            if let Some(screen) = self.m_currentScreen.get()
            {
                screen.m_particlesGui.get().orNpe().tick();
                screen.updateScreen();
            }
        }
        if self.m_currentScreen.get().is_none_or(|screen| screen.m_passEvents.get())
        {
            self.handleMouseEvents();
            if self.m_leftClickCounter.get() > 0
            {
                self.m_leftClickCounter.set(self.m_leftClickCounter.get() - 1);
            }
            self.handleKeyboardEvents();
            self.handleHeldButtons();
        }
        self.tickWorld();
        self.m_systemTime.set(jvm::system::currentTimeMillis());
    }

    fn updateScreenForPlayer(&self)
    {
        match (self.m_currentScreen.get(), self.m_thePlayer.get())
        {
            (None, Some(player)) =>
            {
                if game::entity::entity_living::fields(&player).m_health.get() <= 0
                {
                    self.displayGuiScreen(None);
                }
                else if player.isPlayerSleeping()
                    && self.m_theWorld.get().is_some_and(|world| world.m_multiplayerWorld.get())
                {
                    self.displayGuiScreen(Some(newSleepMP()));
                }
            }
            (Some(screen), _) if screen.instanceOf("GuiSleepMP") && !self.thePlayer().isPlayerSleeping() =>
            {
                self.displayGuiScreen(None);
            }
            _ =>
            {}
        }
    }

    fn handleMouseEvents(&self)
    {
        while Mouse::next()
        {
            let l: i64 = jvm::system::currentTimeMillis() - self.m_systemTime.get();
            if l > 200
            {
                continue;
            }
            let mut n: i32 = Mouse::getEventDWheel();
            if n != 0
            {
                inventory(&self.thePlayer()).changeCurrentItem(n);
                let settings: Rc<GameSettings> = self.gameSettings();
                if settings.m_discreteScroll.get()
                {
                    if n > 0
                    {
                        n = 1;
                    }
                    if n < 0
                    {
                        n = -1;
                    }
                    settings.m_totalDiscreteScroll.set(settings.m_totalDiscreteScroll.get() + n as f32 * 0.25);
                }
            }
            match self.m_currentScreen.get()
            {
                None =>
                {
                    if !self.m_inGameHasFocus.get() && Mouse::getEventButtonState()
                    {
                        self.setIngameFocus();
                        continue;
                    }
                    if Mouse::getEventButton() == 0 && Mouse::getEventButtonState()
                    {
                        self.clickMouse(0);
                        self.m_mouseTicksRan.set(self.m_ticksRan.get());
                    }
                    if Mouse::getEventButton() == 1 && Mouse::getEventButtonState()
                    {
                        self.clickMouse(1);
                        self.m_mouseTicksRan.set(self.m_ticksRan.get());
                    }
                    if Mouse::getEventButton() == 2 && Mouse::getEventButtonState()
                    {
                        self.clickMiddleMouseButton();
                    }
                }
                Some(screen) => screen.handleMouseInput(),
            }
        }
    }

    fn handleKeyboardEvents(&self)
    {
        while Keyboard::next()
        {
            entity_player_sp::handleKeyPress(&self.thePlayer(), Keyboard::getEventKey(), Keyboard::getEventKeyState());
            if !Keyboard::getEventKeyState()
            {
                continue;
            }
            if Keyboard::getEventKey() == 87
            {
                self.toggleFullscreen();
                continue;
            }
            match self.m_currentScreen.get()
            {
                Some(screen) => screen.handleKeyboardInput(),
                None => self.handleGameKey(Keyboard::getEventKey()),
            }
            for i in 0..9
            {
                if Keyboard::getEventKey() == 2 + i
                {
                    inventory(&self.thePlayer()).m_currentItem.set(i);
                }
            }
            if Keyboard::getEventKey() == self.gameSettings().m_keyBindToggleFog.m_keyCode.get()
            {
                let value: i32 = if Keyboard::isKeyDown(42) || Keyboard::isKeyDown(54)
                {
                    -1
                }
                else
                {
                    1
                };
                self.gameSettings().setOptionValue(EnumOptions::RenderDistance, value);
            }
        }
    }

    fn handleGameKey(&self, key: i32)
    {
        let settings: Rc<GameSettings> = self.gameSettings();
        if key == 1
        {
            self.displayInGameMenu();
        }
        if key == 31 && Keyboard::isKeyDown(61)
        {
            self.forceReload();
        }
        if key == 59
        {
            settings.m_hideGUI.set(!settings.m_hideGUI.get());
        }
        if key == 61
        {
            settings.m_showDebugInfo.set(!settings.m_showDebugInfo.get());
        }
        if key == 63
        {
            settings.m_thirdPersonView.set(!settings.m_thirdPersonView.get());
        }
        if key == 66
        {
            settings.m_smoothCamera.set(!settings.m_smoothCamera.get());
        }
        if key == settings.m_keyBindInventory.m_keyCode.get()
        {
            self.displayGuiScreen(Some(GuiInventory::new(&self.thePlayer())));
        }
        if key == settings.m_keyBindDrop.m_keyCode.get()
        {
            self.thePlayer().dropCurrentItem();
        }
        if self.isMultiplayerWorld() && key == settings.m_keyBindChat.m_keyCode.get()
        {
            self.displayGuiScreen(Some(GuiChat::new()));
        }
    }

    fn handleHeldButtons(&self)
    {
        if self.m_currentScreen.with(Option::is_none)
        {
            let interval: f32 = self.timer().m_ticksPerSecond.get() / 4.0;
            for button in 0..2
            {
                if Mouse::isButtonDown(button)
                    && (self.m_ticksRan.get() - self.m_mouseTicksRan.get()) as f32 >= interval
                    && self.m_inGameHasFocus.get()
                {
                    self.clickMouse(button);
                    self.m_mouseTicksRan.set(self.m_ticksRan.get());
                }
            }
        }
        self.handleMouseDown(
            0,
            self.m_currentScreen.with(Option::is_none) && Mouse::isButtonDown(0) && self.m_inGameHasFocus.get(),
        );
    }

    fn tickWorld(&self)
    {
        let Some(world) = self.m_theWorld.get()
        else
        {
            return;
        };
        if let Some(player) = self.m_thePlayer.get()
        {
            self.m_joinPlayerCounter.set(self.m_joinPlayerCounter.get() + 1);
            if self.m_joinPlayerCounter.get() == 30
            {
                self.m_joinPlayerCounter.set(0);
                world.joinEntityInSurroundings(&player);
            }
        }
        world.m_difficultySetting.set(self.gameSettings().m_difficulty.get());
        if world.m_multiplayerWorld.get()
        {
            world.m_difficultySetting.set(3);
        }
        if !self.isGamePaused()
        {
            self.entityRenderer().updateRenderer();
        }
        if !self.isGamePaused()
        {
            self.m_renderGlobal.get().orNpe().updateClouds();
        }
        if !self.isGamePaused()
        {
            if world.m_lightningCooldown.get() > 0
            {
                world.m_lightningCooldown.set(world.m_lightningCooldown.get() - 1);
            }
            world.updateEntities();
        }
        if !self.isGamePaused() || self.isMultiplayerWorld()
        {
            world.setAllowedMobSpawns(self.gameSettings().m_difficulty.get() > 0, true);
            world.tick();
        }
        if !self.isGamePaused()
            && let Some(world) = self.m_theWorld.get()
        {
            let player: Rc<Entity> = self.thePlayer();
            world.randomDisplayUpdates(
                math_helper::floor_double(player.posX()),
                math_helper::floor_double(player.posY()),
                math_helper::floor_double(player.posZ()),
            );
        }
        if !self.isGamePaused()
        {
            self.m_effectRenderer.get().orNpe().updateEffects();
        }
    }

    fn forceReload(&self)
    {
        jvm::system::println("FORCING RELOAD!");
        self.m_sndManager.set(Some(Rc::new(crate::sound::SoundManager::new())));
        self.sndManager().loadSoundSettings(self.m_gameSettings.get());
        self.m_downloadResourcesThread.get().orNpe().reloadResources();
    }
}
