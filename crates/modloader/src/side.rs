pub use game::port::Side;
use game::world::World;

pub fn physical() -> Side { game::port::side() }

pub fn isClient() -> bool { physical() == Side::Client }

pub fn isServer() -> bool { physical() == Side::Server }

// the dedicated server and single player own their worlds, a multiplayer client only mirrors one
pub fn isAuthority(world: &World) -> bool { !world.m_multiplayerWorld.get() }

// single player runs client and server logic in one process with no connection between them
pub fn isLoopback(world: &World) -> bool { isClient() && isAuthority(world) }
