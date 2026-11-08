use crate::client::Minecraft;
use crate::entity::entity_client_player_mp;
use crate::network::net_client_handler::NetClientHandler;
use crate::render::OpenGlCapsChecker;
use game::storage::ISaveFormat;
use jvm::throwable::OrNpe;
use std::path::{Path, PathBuf};
use std::rc::Rc;

impl Minecraft
{
    pub fn getMinecraftDir() -> PathBuf
    {
        if let Some(dir) = Minecraft::minecraftDir()
        {
            return dir;
        }
        // rewrite: the per-user beta173 data folder replaces .minecraft
        let dir: PathBuf = Minecraft::makeDirs(jvm::data_dir::root());
        Minecraft::setMinecraftDir(dir.clone());
        dir
    }

    fn makeDirs(file: PathBuf) -> PathBuf
    {
        if !file.exists() && std::fs::create_dir_all(&file).is_err()
        {
            jvm::throwable::throw(
                &jvm::throwable::RUNTIME,
                &format!("The working directory could not be created: {}", file.display()),
            );
        }
        file
    }

    pub fn getSaveLoader(&self) -> Rc<dyn ISaveFormat> { self.m_saveLoader.get().orNpe() }

    pub fn installResource(&self, path: &str, file: &Path)
    {
        let Some(n) = path.find('/')
        else
        {
            jvm::throwable::throw(&jvm::throwable::STRING_INDEX, "String index out of range: -1");
        };
        let string: &str = &path[..n];
        let path: &str = &path[n + 1..];
        if string.eq_ignore_ascii_case("sound") || string.eq_ignore_ascii_case("newsound")
        {
            self.sndManager().addSound(path, file);
        }
        else if string.eq_ignore_ascii_case("streaming")
        {
            self.sndManager().addStreaming(path, file);
        }
        else if string.eq_ignore_ascii_case("music") || string.eq_ignore_ascii_case("newmusic")
        {
            self.sndManager().addMusic(path, file);
        }
    }

    pub fn getOpenGlCapsChecker(&self) -> Option<Rc<OpenGlCapsChecker>> { self.m_glCapabilities.get() }

    pub fn getRenderChunkDebugInfo(&self) -> String { self.m_renderGlobal.get().orNpe().getDebugInfoRenders() }

    pub fn getRenderEntityDebugInfo(&self) -> String { self.m_renderGlobal.get().orNpe().getDebugInfoEntities() }

    pub fn getChunkSourceDebugInfo(&self) -> String { self.theWorld().getChunkSourceDebugInfo() }

    pub fn getWorldDebugInfo(&self) -> String
    {
        format!("P: {}. T: {}", self.m_effectRenderer.get().orNpe().getStatistics(), self.theWorld().getDebugInfo())
    }

    pub fn getSendQueue(&self) -> Rc<NetClientHandler>
    {
        let player: Option<Rc<game::entity::Entity>> =
            self.m_thePlayer.get().filter(|player| player.instanceOf("EntityClientPlayerMP"));
        Rc::clone(&entity_client_player_mp::fields(&player.orNpe()).m_sendQueue)
    }

    pub fn isGuiEnabled() -> bool
    {
        Minecraft::theMinecraft().is_none_or(|minecraft| !minecraft.gameSettings().m_hideGUI.get())
    }

    pub fn isFancyGraphicsEnabled() -> bool
    {
        Minecraft::theMinecraft().is_some_and(|minecraft| minecraft.gameSettings().m_fancyGraphics.get())
    }

    pub fn isAmbientOcclusionEnabled() -> bool
    {
        Minecraft::theMinecraft().is_some_and(|minecraft| minecraft.gameSettings().m_ambientOcclusion.get())
    }

    pub fn isDebugInfoEnabled() -> bool
    {
        Minecraft::theMinecraft().is_some_and(|minecraft| minecraft.gameSettings().m_showDebugInfo.get())
    }

    // single player has no server, so mod commands typed in chat run here
    pub fn lineIsCommand(&self, message: &str) -> bool
    {
        if self.isMultiplayerWorld()
        {
            return false;
        }
        let Some(line) = message.strip_prefix('/')
        else
        {
            return false;
        };
        let player: Option<Rc<game::entity::Entity>> = self.m_thePlayer.get();
        match modloader::play::command(line, player.as_ref(), true)
        {
            Some(reply) =>
            {
                if let Some(player) = player.filter(|_| !reply.is_empty())
                {
                    player.addChatMessage(&reply);
                }
                true
            }
            None => false,
        }
    }
}
