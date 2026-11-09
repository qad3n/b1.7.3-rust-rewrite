use crate::client::Minecraft;
use game::entity::entity_player;
use game::world::World;
use jvm::string::JString;
use jvm::throwable::{THROWABLE, catchThrowable};
use std::rc::Rc;

impl Minecraft
{
    // what the game and the mod loader were doing, read before the crash handler leaves the world
    pub fn crashDetails(&self) -> String
    {
        let mut text: String = String::from("-- Game --\n");
        text += &format!("Ticks: {}\n", self.m_ticksRan.get());
        let server: String = self.m_serverName.get().map_or_else(|| "single player".to_owned(), |name| name.clone());
        text += &format!("Server: {server}\n");
        text += &catchThrowable(&THROWABLE, || self.worldDetails())
            .unwrap_or_else(|throwable| format!("World: [failed to read ({throwable})]\n"));
        text += "\n-- Mods --\n";
        text += &modDetails();
        text
    }

    fn worldDetails(&self) -> String
    {
        let Some(world) = self.m_theWorld.get()
        else
        {
            return "World: none\n".to_owned();
        };

        let mut text: String = worldLine(&world);
        if let Some(player) = self.m_thePlayer.get()
        {
            let name: String = JString::display(entity_player::fields(&player).m_username.get().as_ref());
            text += &format!("Player: {name} at {:.2} {:.2} {:.2}\n", player.posX(), player.posY(), player.posZ());
        }
        text
    }
}

fn worldLine(world: &Rc<World>) -> String
{
    let name: String = JString::display(world.getWorldInfo().getWorldName().as_ref());
    format!(
        "World: {name}, time {}, {} entities, {} weather effects\n",
        world.getWorldTime(),
        world.m_loadedEntityList.size(),
        world.m_weatherEffects.size()
    )
}

fn modDetails() -> String
{
    let mods: Vec<Rc<modloader::state::ModRuntime>> = modloader::state::mods();
    if mods.is_empty()
    {
        return "none\n".to_owned();
    }

    let mut text: String = String::new();
    for runtime in mods
    {
        text += &format!("{} {} ({})", runtime.id(), runtime.m_manifest.version, runtime.m_file);
        if let Some(reason) = runtime.failure()
        {
            text += &format!(", disabled: {reason}");
        }
        text += "\n";
    }
    text
}
