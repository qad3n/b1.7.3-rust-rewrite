use crate::player::entity_player_mp;
use crate::server::{ICommandListener, MinecraftServer, ServerCommand, ServerConfigurationManager};
use crate::world::world_server;
use game::entity::{Entity, entity_player};
use game::item::{Item, ItemStack};
use game::network::packets::Packet3Chat;
use game::world::World;
use jvm::logging::Logger;
use jvm::string::JString;
use jvm::throwable::{NUMBER_FORMAT, OrNpe};
use std::rc::Rc;

fn minecraftLogger() -> &'static Logger { Logger::getLogger("Minecraft") }

fn split(text: &JString) -> Vec<JString>
{
    game::port::text::splitUnits(text.chars(), b' ' as u16).into_iter().map(JString::fromUtf16).collect()
}

pub struct ConsoleCommandHandler
{
    m_minecraftServer: Rc<MinecraftServer>,
}

impl ConsoleCommandHandler
{
    pub fn new(server: &Rc<MinecraftServer>) -> ConsoleCommandHandler
    {
        ConsoleCommandHandler { m_minecraftServer: Rc::clone(server) }
    }

    fn worlds(&self) -> Vec<Rc<World>> { self.m_minecraftServer.worldMngr() }

    pub fn handleCommand(&self, command: &ServerCommand)
    {
        let mut string: JString = JString::from(command.m_command.as_str());
        let iCommandListener: &Rc<dyn ICommandListener> = &command.m_commandListener;
        let string2: String = iCommandListener.getUsername();
        let serverConfigurationManager: Rc<ServerConfigurationManager> = self.m_minecraftServer.configManager();
        let lower: JString = string.toLowerCase();
        if lower.startsWith("help") || lower.startsWith("?")
        {
            self.printHelp(iCommandListener);
        }
        else if lower.startsWith("list")
        {
            iCommandListener.log(&format!("Connected players: {}", serverConfigurationManager.getPlayerList()));
        }
        else if lower.startsWith("stop")
        {
            self.sendNoticeToOps(&string2, "Stopping the server..");
            self.m_minecraftServer.initiateShutdown();
        }
        else if lower.startsWith("save-all")
        {
            self.sendNoticeToOps(&string2, "Forcing save..");
            serverConfigurationManager.savePlayerStates();
            for worldServer in self.worlds()
            {
                worldServer.saveWorld(true, None);
            }
            self.sendNoticeToOps(&string2, "Save complete.");
        }
        else if lower.startsWith("save-off")
        {
            self.sendNoticeToOps(&string2, "Disabling level saving..");
            for worldServer in self.worlds()
            {
                world_server::fields(&worldServer).m_levelSaving.set(true);
            }
        }
        else if lower.startsWith("save-on")
        {
            self.sendNoticeToOps(&string2, "Enabling level saving..");
            for worldServer in self.worlds()
            {
                world_server::fields(&worldServer).m_levelSaving.set(false);
            }
        }
        else if lower.startsWith("op ")
        {
            let string3: JString = string.substringFrom(string.indexOf(b' ' as u16)).trim();
            serverConfigurationManager.opPlayer(&string3);
            self.sendNoticeToOps(&string2, &format!("Opping {string3}"));
            serverConfigurationManager.sendChatMessageToPlayer(&string3, &JString::from("\u{a7}eYou are now op!"));
        }
        else if lower.startsWith("deop ")
        {
            let string4: JString = string.substringFrom(string.indexOf(b' ' as u16)).trim();
            serverConfigurationManager.deopPlayer(&string4);
            serverConfigurationManager
                .sendChatMessageToPlayer(&string4, &JString::from("\u{a7}eYou are no longer op!"));
            self.sendNoticeToOps(&string2, &format!("De-opping {string4}"));
        }
        else if lower.startsWith("ban-ip ")
        {
            let string5: JString = string.substringFrom(string.indexOf(b' ' as u16)).trim();
            serverConfigurationManager.banIP(&string5);
            self.sendNoticeToOps(&string2, &format!("Banning ip {string5}"));
        }
        else if lower.startsWith("pardon-ip ")
        {
            let string6: JString = string.substringFrom(string.indexOf(b' ' as u16)).trim();
            serverConfigurationManager.pardonIP(&string6);
            self.sendNoticeToOps(&string2, &format!("Pardoning ip {string6}"));
        }
        else if lower.startsWith("ban ")
        {
            let string7: JString = string.substringFrom(string.indexOf(b' ' as u16)).trim();
            serverConfigurationManager.banPlayer(&string7);
            self.sendNoticeToOps(&string2, &format!("Banning {string7}"));
            if let Some(entityPlayerMP) = serverConfigurationManager.getPlayerEntity(&string7)
            {
                entity_player_mp::netServerHandler(&entityPlayerMP).kickPlayer("Banned by admin");
            }
        }
        else if lower.startsWith("pardon ")
        {
            let string8: JString = string.substringFrom(string.indexOf(b' ' as u16)).trim();
            serverConfigurationManager.pardonPlayer(&string8);
            self.sendNoticeToOps(&string2, &format!("Pardoning {string8}"));
        }
        else if lower.startsWith("kick ")
        {
            let string9: JString = string.substringFrom(string.indexOf(b' ' as u16)).trim();
            let mut entityPlayerMP: Option<Rc<Entity>> = None;
            let mut i: i32 = 0;
            while i < serverConfigurationManager.m_playerEntities.size()
            {
                let entityPlayerMP2: Rc<Entity> = serverConfigurationManager.m_playerEntities.get(i);
                i += 1;
                if !entity_player::fields(&entityPlayerMP2).m_username.get().orNpe().equalsIgnoreCase(Some(&string9))
                {
                    continue;
                }
                entityPlayerMP = Some(entityPlayerMP2);
            }
            match entityPlayerMP
            {
                Some(entityPlayerMP) =>
                {
                    entity_player_mp::netServerHandler(&entityPlayerMP).kickPlayer("Kicked by admin");
                    self.sendNoticeToOps(
                        &string2,
                        &format!(
                            "Kicking {}",
                            JString::display(entity_player::fields(&entityPlayerMP).m_username.get().as_ref())
                        ),
                    );
                }
                None => iCommandListener.log(&format!("Can't find user {string9}. No kick.")),
            }
        }
        else if lower.startsWith("tp ")
        {
            let stringArray: Vec<JString> = split(&string);
            if stringArray.len() == 3
            {
                let entityPlayerMP: Option<Rc<Entity>> = serverConfigurationManager.getPlayerEntity(&stringArray[1]);
                let entityPlayerMP3: Option<Rc<Entity>> = serverConfigurationManager.getPlayerEntity(&stringArray[2]);
                match (entityPlayerMP, entityPlayerMP3)
                {
                    (None, _) => iCommandListener.log(&format!("Can't find user {}. No tp.", stringArray[1])),
                    (_, None) => iCommandListener.log(&format!("Can't find user {}. No tp.", stringArray[2])),
                    (Some(entityPlayerMP), Some(entityPlayerMP3))
                        if entity_player::fields(&entityPlayerMP).m_dimension.get()
                            != entity_player::fields(&entityPlayerMP3).m_dimension.get() =>
                    {
                        iCommandListener.log(&format!(
                            "User {} and {} are in different dimensions. No tp.",
                            stringArray[1], stringArray[2]
                        ));
                    }
                    (Some(entityPlayerMP), Some(entityPlayerMP3)) =>
                    {
                        entity_player_mp::netServerHandler(&entityPlayerMP).teleportTo(
                            entityPlayerMP3.m_posX.get(),
                            entityPlayerMP3.m_posY.get(),
                            entityPlayerMP3.m_posZ.get(),
                            entityPlayerMP3.m_rotationYaw.get(),
                            entityPlayerMP3.m_rotationPitch.get(),
                        );
                        self.sendNoticeToOps(
                            &string2,
                            &format!("Teleporting {} to {}.", stringArray[1], stringArray[2]),
                        );
                    }
                }
            }
            else
            {
                iCommandListener.log("Syntax error, please provice a source and a target.");
            }
        }
        else if lower.startsWith("give ")
        {
            let stringArray: Vec<JString> = split(&string);
            if stringArray.len() != 3 && stringArray.len() != 4
            {
                return;
            }
            let string10: &JString = &stringArray[1];
            match serverConfigurationManager.getPlayerEntity(string10)
            {
                Some(entityPlayerMP) => match jvm::integer::parseInt(&stringArray[2])
                {
                    Ok(n) =>
                    {
                        if Item::byId(n).is_some()
                        {
                            self.sendNoticeToOps(
                                &string2,
                                &format!(
                                    "Giving {} some {n}",
                                    JString::display(entity_player::fields(&entityPlayerMP).m_username.get().as_ref())
                                ),
                            );
                            let mut n2: i32 = 1;
                            if stringArray.len() > 3
                            {
                                n2 = self.tryParse(&stringArray[3], 1);
                            }
                            if n2 < 1
                            {
                                n2 = 1;
                            }
                            if n2 > 64
                            {
                                n2 = 64;
                            }
                            entity_player::dropPlayerItem(&entityPlayerMP, Some(ItemStack::withDamage(n, n2, 0)));
                            return;
                        }
                        iCommandListener.log(&format!("There's no item with id {n}"));
                    }
                    Err(exception) if exception.instanceOf(&NUMBER_FORMAT) =>
                    {
                        iCommandListener.log(&format!("There's no item with id {}", stringArray[2]));
                    }
                    Err(exception) => jvm::throwable::rethrow(exception),
                },
                None => iCommandListener.log(&format!("Can't find user {string10}")),
            }
        }
        else if lower.startsWith("time ")
        {
            let stringArray: Vec<JString> = split(&string);
            if stringArray.len() != 3
            {
                return;
            }
            let string11: &JString = &stringArray[1];
            match jvm::integer::parseInt(&stringArray[2])
            {
                Ok(n) =>
                {
                    if JString::from("add").equalsIgnoreCase(Some(string11))
                    {
                        for worldServer in self.worlds()
                        {
                            worldServer.changeTime(worldServer.getWorldTime() + n as i64);
                        }
                        self.sendNoticeToOps(&string2, &format!("Added {n} to time"));
                        return;
                    }
                    if JString::from("set").equalsIgnoreCase(Some(string11))
                    {
                        for worldServer in self.worlds()
                        {
                            worldServer.changeTime(n as i64);
                        }
                        self.sendNoticeToOps(&string2, &format!("Set time to {n}"));
                        return;
                    }
                    iCommandListener.log("Unknown method, use either \"add\" or \"set\"");
                }
                Err(exception) if exception.instanceOf(&NUMBER_FORMAT) =>
                {
                    iCommandListener.log(&format!("Unable to convert time value, {}", stringArray[2]));
                }
                Err(exception) => jvm::throwable::rethrow(exception),
            }
        }
        else if lower.startsWith("say ")
        {
            string = string.substringFrom(string.indexOf(b' ' as u16)).trim();
            minecraftLogger().info(&format!("[{string2}] {string}"));
            serverConfigurationManager.sendPacketToAllPlayers(Box::new(Packet3Chat::new(JString::from(
                format!("\u{a7}d[Server] {string}").as_str(),
            ))));
        }
        else if lower.startsWith("tell ")
        {
            let stringArray: Vec<JString> = split(&string);
            if stringArray.len() >= 3
            {
                string = string.substringFrom(string.indexOf(b' ' as u16)).trim();
                string = string.substringFrom(string.indexOf(b' ' as u16)).trim();
                minecraftLogger().info(&format!("[{string2}->{}] {string}", stringArray[1]));
                string = JString::from(format!("\u{a7}7{string2} whispers {string}").as_str());
                minecraftLogger().info(&string.to_string());
                if !serverConfigurationManager.sendPacketToPlayer(&stringArray[1], Box::new(Packet3Chat::new(string)))
                {
                    iCommandListener.log("There's no player by that name online.");
                }
            }
        }
        else if lower.startsWith("whitelist ")
        {
            self.handleWhitelist(&string2, &string, iCommandListener);
        }
        else
        {
            minecraftLogger().info("Unknown console command. Type \"help\" for help.");
        }
    }

    fn handleWhitelist(&self, player: &str, command: &JString, source: &Rc<dyn ICommandListener>)
    {
        let stringArray: Vec<JString> = split(command);
        if stringArray.len() < 2
        {
            return;
        }
        let string: String = stringArray[1].toLowerCase().to_string();
        if "on" == string
        {
            self.sendNoticeToOps(player, "Turned on white-listing");
            self.m_minecraftServer.propertyManager().setProperty("white-list", true);
        }
        else if "off" == string
        {
            self.sendNoticeToOps(player, "Turned off white-listing");
            self.m_minecraftServer.propertyManager().setProperty("white-list", false);
        }
        else if "list" == string
        {
            let set: Vec<JString> = self.m_minecraftServer.configManager().getWhiteListedIPs();
            let mut string2: String = String::new();
            for string3 in &set
            {
                string2 = format!("{string2}{string3} ");
            }
            source.log(&format!("White-listed players: {string2}"));
        }
        else if "add" == string && stringArray.len() == 3
        {
            let string4: JString = stringArray[2].toLowerCase();
            self.m_minecraftServer.configManager().addToWhiteList(&string4);
            self.sendNoticeToOps(player, &format!("Added {string4} to white-list"));
        }
        else if "remove" == string && stringArray.len() == 3
        {
            let string5: JString = stringArray[2].toLowerCase();
            self.m_minecraftServer.configManager().removeFromWhiteList(&string5);
            self.sendNoticeToOps(player, &format!("Removed {string5} from white-list"));
        }
        else if "reload" == string
        {
            self.m_minecraftServer.configManager().reloadWhiteList();
            self.sendNoticeToOps(player, "Reloaded white-list from file");
        }
    }

    fn printHelp(&self, source: &Rc<dyn ICommandListener>)
    {
        source.log("To run the server without a gui, start it like this:");
        source.log("   java -Xmx1024M -Xms1024M -jar minecraft_server.jar nogui");
        source.log("Console commands:");
        source.log("   help  or  ?               shows this message");
        source.log("   kick <player>             removes a player from the server");
        source.log("   ban <player>              bans a player from the server");
        source.log("   pardon <player>           pardons a banned player so that they can connect again");
        source.log("   ban-ip <ip>               bans an IP address from the server");
        source.log("   pardon-ip <ip>            pardons a banned IP address so that they can connect again");
        source.log("   op <player>               turns a player into an op");
        source.log("   deop <player>             removes op status from a player");
        source.log("   tp <player1> <player2>    moves one player to the same location as another player");
        source.log("   give <player> <id> [num]  gives a player a resource");
        source.log("   tell <player> <message>   sends a private message to a player");
        source.log("   stop                      gracefully stops the server");
        source.log("   save-all                  forces a server-wide level save");
        source.log("   save-off                  disables terrain saving (useful for backup scripts)");
        source.log("   save-on                   re-enables terrain saving");
        source.log("   list                      lists all currently connected players");
        source.log("   say <message>             broadcasts a message to all players");
        source.log("   time <add|set> <amount>   adds to or sets the world time (0-24000)");
    }

    fn sendNoticeToOps(&self, player: &str, message: &str)
    {
        let string: String = format!("{player}: {message}");
        self.m_minecraftServer
            .configManager()
            .sendChatMessageToAllOps(&JString::from(format!("\u{a7}7({string})").as_str()));
        minecraftLogger().info(&string);
    }

    fn tryParse(&self, s: &JString, defaultValue: i32) -> i32
    {
        match jvm::integer::parseInt(s)
        {
            Ok(value) => value,
            Err(exception) if exception.instanceOf(&NUMBER_FORMAT) => defaultValue,
            Err(exception) => jvm::throwable::rethrow(exception),
        }
    }
}
