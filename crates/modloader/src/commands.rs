use crate::state;
use crate::state::ModRuntime;
use mod_protocol::abi::{GuestCall, GuestReply, Handle, Permission, Sender};
use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::str::SplitWhitespace;

#[derive(Clone)]
pub struct Command
{
    pub m_mod: usize,
    pub m_handler: Handle,
    pub m_permission: Permission,
    pub m_usage: String,
}

thread_local!
{
    static COMMANDS: RefCell<BTreeMap<String, Command>> = const { RefCell::new(BTreeMap::new()) };
}

// vanilla server commands keep their names, a mod command may not shadow them
const VANILLA: [&str; 17] = [
    "list",
    "stop",
    "save-all",
    "save-off",
    "save-on",
    "op",
    "deop",
    "ban-ip",
    "pardon-ip",
    "ban",
    "pardon",
    "kick",
    "tp",
    "give",
    "tell",
    "time",
    "say",
];

// every command answers to /<modid>:<name>, the plain /<name> goes to the first mod in load order that asks for it
pub fn register(
    modIndex: usize,
    name: &str,
    permission: Permission,
    usage: String,
    handler: Handle,
) -> Result<(), String>
{
    let owner: String = state::modId(modIndex);
    let command: Command = Command { m_mod: modIndex, m_handler: handler, m_permission: permission, m_usage: usage };

    COMMANDS.with(|commands|
    {
        let mut commands: RefMut<BTreeMap<String, crate::commands::Command>> = commands.borrow_mut();
        let qualified: String = format!("{owner}:{name}");
        if commands.contains_key(&qualified)
        {
            return Err(format!("/{qualified} is registered twice"));
        }

        commands.insert(qualified.clone(), command.clone());

        // only a server has vanilla commands, single player runs mod commands on the client
        if VANILLA.contains(&name) && !crate::side::isClient()
        {
            crate::log::warn(&owner, &format!("/{name} is a vanilla command, use /{qualified}"));
        }
        else if let Some(existing) = commands.get(name)
        {
            crate::log::warn(&owner, &format!("/{name} belongs to {}, use /{qualified}", state::modId(existing.m_mod)));
        }
        else
        {
            commands.insert(name.to_owned(), command);
        }
        Ok(())
    })
}

pub fn names() -> Vec<(String, String)>
{
    COMMANDS.with(|commands|
    {
        commands.borrow().iter().map(|(name, command)| (name.clone(), command.m_usage.clone())).collect()
    })
}

fn allowed(command: &Command, sender: &Sender) -> bool
{
    match command.m_permission
    {
        Permission::Anyone => true,
        Permission::Op => sender.op || sender.player.is_none(),
        Permission::Console => sender.player.is_none(),
    }
}

// the line without its slash, Some(reply) when a mod command handled it
pub fn execute(line: &str, sender: Sender) -> Option<String>
{
    let mut parts: SplitWhitespace = line.split_whitespace();
    let name: &str = parts.next()?;
    if name == crate::devtools::COMMAND
    {
        let args: Vec<String> = parts.map(str::to_owned).collect();
        return Some(crate::devtools::execute(&args, &sender));
    }

    let command: Command = COMMANDS.with(|commands| commands.borrow().get(name).cloned())?;
    if !allowed(&command, &sender)
    {
        return Some(format!("\u{a7}cYou do not have permission to use /{name}"));
    }

    let args: Vec<String> = parts.map(str::to_owned).collect();
    let runtime: Rc<ModRuntime> = state::modAt(command.m_mod)?;
    match runtime.call(&GuestCall::Command { handler: command.m_handler, sender, args })
    {
        Some(GuestReply::Bytes(text)) =>
        {
            Some(String::from_utf8_lossy(&text).into_owned()).filter(|text| !text.is_empty())
        }

        Some(_) => Some(String::new()),
        None => Some(format!("\u{a7}cUsage: /{name} {}", command.m_usage)),
    }
}
