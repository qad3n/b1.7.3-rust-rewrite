use crate::client::Minecraft;
use crate::entity::entity_player_sp;
// a discarded world is emptied in port to break rust reference cycles
use crate::port::world_release::releaseDiscardedWorld;
use crate::settings::MovementInput;
use common::world::IProgressUpdate;
use game::entity::{Entity, entity_player};
use game::math::math_helper;
use game::stats::StatList;
use game::storage::ISaveFormat;
use game::world::{ChunkCoordinates, IChunkProvider, ISaveHandler, Teleporter, World, WorldProvider};
use jvm::throwable::OrNpe;
use std::rc::Rc;

fn setMovementInputFromOptions(minecraft: &Minecraft, player: &Entity)
{
    let input: MovementInput = MovementInput::fromOptions(minecraft.gameSettings());
    entity_player_sp::fields(player).m_movementInput.set(Some(Rc::new(input)));
}

fn setCurrentChunkOver(world: &World, x: i32, z: i32)
{
    let provider: Rc<dyn IChunkProvider> = world.getIChunkProvider();
    if let Some(provider) = provider.asLoadOrGenerate()
    {
        provider.setCurrentChunkOver(x, z);
    }
}

impl Minecraft
{
    pub fn isMultiplayerWorld(&self) -> bool
    {
        self.m_theWorld.get().is_some_and(|world| world.m_multiplayerWorld.get())
    }

    pub fn startWorld(&self, saveName: &str, name: &str, seed: i64)
    {
        self.changeWorld1(None);
        let saveLoader: Rc<dyn ISaveFormat> = self.m_saveLoader.get().orNpe();
        if saveLoader.isOldMapFormat(saveName)
        {
            self.convertMapFormat(saveName, name);
        }
        else
        {
            let iSaveHandler: Rc<dyn ISaveHandler> = saveLoader.getSaveLoader(saveName, false);
            let world: Rc<World> = World::newFromSave(iSaveHandler, name, seed, None);
            if world.m_isNewWorld.get()
            {
                self.statFileWriter().readStat(&StatList::createWorldStat().orNpe(), 1);
                self.statFileWriter().readStat(&StatList::startGameStat().orNpe(), 1);
                self.changeWorld2(Some(world), "Generating level");
            }
            else
            {
                self.statFileWriter().readStat(&StatList::loadWorldStat().orNpe(), 1);
                self.statFileWriter().readStat(&StatList::startGameStat().orNpe(), 1);
                self.changeWorld2(Some(world), "Loading level");
            }
        }
    }

    pub fn usePortal(&self)
    {
        jvm::system::println("Toggling dimension!!");
        let player: Rc<Entity> = self.thePlayer();
        let playerFields: &entity_player::PlayerFields = entity_player::fields(&player);
        if playerFields.m_dimension.get() == -1
        {
            playerFields.m_dimension.set(0);
        }
        else
        {
            playerFields.m_dimension.set(-1);
        }
        self.theWorld().setEntityDead(&player);
        player.m_isDead.set(false);
        let mut d0: f64 = player.posX();
        let mut d1: f64 = player.posZ();
        let d2: f64 = 8.0;
        if playerFields.m_dimension.get() == -1
        {
            d0 /= d2;
            d1 /= d2;
            player.setLocationAndAngles(
                d0,
                player.posY(),
                d1,
                player.m_rotationYaw.get(),
                player.m_rotationPitch.get(),
            );
            if player.isEntityAlive()
            {
                self.theWorld().updateEntityWithOptionalForce(&player, false);
            }
            let world: Rc<World> = World::newFromWorld(&self.theWorld(), WorldProvider::getProviderForDimension(-1));
            self.changeWorld(Some(world), "Entering the Nether", Some(&player));
        }
        else
        {
            d0 *= d2;
            d1 *= d2;
            player.setLocationAndAngles(
                d0,
                player.posY(),
                d1,
                player.m_rotationYaw.get(),
                player.m_rotationPitch.get(),
            );
            if player.isEntityAlive()
            {
                self.theWorld().updateEntityWithOptionalForce(&player, false);
            }
            let world1: Rc<World> = World::newFromWorld(&self.theWorld(), WorldProvider::getProviderForDimension(0));
            self.changeWorld(Some(world1), "Leaving the Nether", Some(&player));
        }
        player.m_worldObj.set(self.m_theWorld.get());
        if player.isEntityAlive()
        {
            player.setLocationAndAngles(
                d0,
                player.posY(),
                d1,
                player.m_rotationYaw.get(),
                player.m_rotationPitch.get(),
            );
            self.theWorld().updateEntityWithOptionalForce(&player, false);
            Teleporter::new().onDimensionChanged(&self.theWorld(), &player);
        }
    }

    pub fn changeWorld1(&self, world: Option<Rc<World>>) { self.changeWorld2(world, ""); }

    pub fn changeWorld2(&self, world: Option<Rc<World>>, message: &str) { self.changeWorld(world, message, None); }

    pub fn changeWorld(&self, world: Option<Rc<World>>, message: &str, player: Option<&Rc<Entity>>)
    {
        let previous: Option<Rc<World>> = self.m_theWorld.get();
        self.statFileWriter().func_27175_b();
        self.statFileWriter().syncStats();
        self.m_renderViewEntity.set(None);
        let loadingScreen: Rc<crate::gui::LoadingScreenRenderer> = self.loadingScreen();
        loadingScreen.printText(message);
        loadingScreen.displayLoadingString("");
        self.sndManager().playStreaming(None, 0.0, 0.0, 0.0, 0.0, 0.0);
        if let Some(previous) = self.m_theWorld.get()
        {
            previous.saveWorldIndirectly(Some(&*loadingScreen as &dyn IProgressUpdate));
        }
        self.m_theWorld.set(world.clone());
        let Some(world) = world
        else
        {
            self.m_thePlayer.set(None);
            self.m_systemTime.set(0);
            releaseDiscardedWorld(previous, None);
            return;
        };
        self.playerController().initWorld(&world);
        if !self.isMultiplayerWorld()
        {
            if player.is_none()
            {
                self.m_thePlayer.set(world.getEntityOfSubtype("EntityPlayerSP"));
            }
        }
        else if let Some(thePlayer) = self.m_thePlayer.get()
        {
            thePlayer.preparePlayerToSpawn();
            world.entityJoinedWorld(&thePlayer);
        }
        if !world.m_multiplayerWorld.get()
        {
            self.prepareWorld(message);
        }
        if self.m_thePlayer.with(Option::is_none)
        {
            let created: Rc<Entity> = self.playerController().createPlayer(&world);
            self.m_thePlayer.set(Some(Rc::clone(&created)));
            created.preparePlayerToSpawn();
            self.playerController().flipPlayer(&created);
        }
        let thePlayer: Rc<Entity> = self.thePlayer();
        setMovementInputFromOptions(self, &thePlayer);
        if let Some(renderGlobal) = self.m_renderGlobal.get()
        {
            renderGlobal.changeWorld(Some(Rc::clone(&world)));
        }
        if let Some(effectRenderer) = self.m_effectRenderer.get()
        {
            effectRenderer.clearEffects(Some(Rc::clone(&world)));
        }
        self.playerController().adjustPlayer(&thePlayer);
        if player.is_some()
        {
            world.emptyMethod1();
        }
        let i: i32 = math_helper::floor_float(thePlayer.posX() as i32 as f32) >> 4;
        let j: i32 = math_helper::floor_float(thePlayer.posZ() as i32 as f32) >> 4;
        setCurrentChunkOver(&world, i, j);
        world.spawnPlayerWithLoadedChunks(&thePlayer);
        if world.m_isNewWorld.get()
        {
            world.saveWorldIndirectly(Some(&*loadingScreen as &dyn IProgressUpdate));
        }
        self.m_renderViewEntity.set(Some(thePlayer));
        self.m_systemTime.set(0);
        releaseDiscardedWorld(previous, self.m_theWorld.get());
    }

    fn convertMapFormat(&self, saveName: &str, name: &str)
    {
        let saveLoader: Rc<dyn ISaveFormat> = self.m_saveLoader.get().orNpe();
        let loadingScreen: Rc<crate::gui::LoadingScreenRenderer> = self.loadingScreen();
        loadingScreen.printText(&format!("Converting World to {}", saveLoader.getName()));
        loadingScreen.displayLoadingString("This may take a while :)");
        saveLoader.convertMapFormat(saveName, &*loadingScreen);
        self.startWorld(saveName, name, 0);
    }

    fn prepareWorld(&self, saveName: &str)
    {
        let loadingScreen: Rc<crate::gui::LoadingScreenRenderer> = self.loadingScreen();
        loadingScreen.printText(saveName);
        loadingScreen.displayLoadingString("Building terrain");
        let n: i32 = 128;
        let mut n2: i32 = 0;
        let mut n3: i32 = n * 2 / 16 + 1;
        n3 *= n3;
        let world: Rc<World> = self.theWorld();
        let mut chunkCoordinates: ChunkCoordinates = world.getSpawnPoint();
        if let Some(player) = self.m_thePlayer.get()
        {
            chunkCoordinates.m_x = player.posX() as i32;
            chunkCoordinates.m_z = player.posZ() as i32;
        }
        setCurrentChunkOver(&world, chunkCoordinates.m_x >> 4, chunkCoordinates.m_z >> 4);
        let mut i: i32 = -n;
        while i <= n
        {
            let mut j: i32 = -n;
            while j <= n
            {
                loadingScreen.setLoadingProgress(n2 * 100 / n3);
                n2 += 1;
                world.getBlockId(chunkCoordinates.m_x + i, 64, chunkCoordinates.m_z + j);
                while world.updatingLighting()
                {}
                j += 16;
            }
            i += 16;
        }
        loadingScreen.displayLoadingString("Simulating world for a bit");
        world.prepare();
    }

    pub fn respawn(&self, worldSpawn: bool, dimension: i32)
    {
        if !self.theWorld().m_multiplayerWorld.get() && !self.theWorld().worldProvider().canRespawnHere()
        {
            self.usePortal();
        }
        let mut chunkcoordinates: Option<ChunkCoordinates> = None;
        let mut chunkcoordinates1: Option<ChunkCoordinates> = None;
        let mut flag: bool = true;
        if let Some(player) = self.m_thePlayer.get()
            && !worldSpawn
        {
            chunkcoordinates = entity_player::getPlayerSpawnCoordinate(&player);
            if let Some(coordinates) = chunkcoordinates.as_ref()
            {
                chunkcoordinates1 = entity_player::loadSpawnPoint(&self.theWorld(), coordinates);
                if chunkcoordinates1.is_none()
                {
                    player.addChatMessage("tile.bed.notValid");
                }
            }
        }
        let spawn: ChunkCoordinates = match chunkcoordinates1
        {
            Some(spawn) => spawn,
            None =>
            {
                flag = false;
                self.theWorld().getSpawnPoint()
            }
        };
        let world: Rc<World> = self.theWorld();
        setCurrentChunkOver(&world, spawn.m_x >> 4, spawn.m_z >> 4);
        world.setSpawnLocation();
        world.updateEntityList();
        let mut i: i32 = 0;
        if let Some(player) = self.m_thePlayer.get()
        {
            i = player.m_entityId.get();
            world.setEntityDead(&player);
        }
        self.m_renderViewEntity.set(None);
        let player: Rc<Entity> = self.playerController().createPlayer(&world);
        self.m_thePlayer.set(Some(Rc::clone(&player)));
        entity_player::fields(&player).m_dimension.set(dimension);
        self.m_renderViewEntity.set(Some(Rc::clone(&player)));
        player.preparePlayerToSpawn();
        if flag
        {
            entity_player::setPlayerSpawnCoordinate(&player, chunkcoordinates);
            player.setLocationAndAngles(
                (spawn.m_x as f32 + 0.5) as f64,
                (spawn.m_y as f32 + 0.1) as f64,
                (spawn.m_z as f32 + 0.5) as f64,
                0.0,
                0.0,
            );
        }
        self.playerController().flipPlayer(&player);
        world.spawnPlayerWithLoadedChunks(&player);
        setMovementInputFromOptions(self, &player);
        player.m_entityId.set(i);
        player.addRespawnParticles();
        self.playerController().adjustPlayer(&player);
        self.prepareWorld("Respawning");
        if self.m_currentScreen.get().is_some_and(|screen| screen.instanceOf("GuiGameOver"))
        {
            self.displayGuiScreen(None);
        }
    }
}
