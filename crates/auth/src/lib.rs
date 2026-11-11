mod cipher;
mod client;
mod limit;
mod message;
mod peer;
mod server;
mod suite;
mod vault;
mod wire;

pub use cipher::{FRAME_HEADER, Keys, Opener, PLAIN_LIMIT, Sealer};
pub use client::{ClientAuth, ClientStep};
pub use limit::Limiter;
pub use message::{Message, Mode};
pub use peer::tag;
pub use server::{Handshake, Outcome, Registration};
pub use vault::{Account, Vault};

pub type AuthResult<T> = Result<T, String>;

// outside the loader channel numbering so both can share packet 250
pub const CHANNEL: u32 = u32::MAX;
pub const PASSWORD_MIN: usize = 8;
// set in the client login dimension byte, vanilla and loader servers never read that byte from a client
pub const LOGIN_FLAG: i8 = 0x40;

const NAME_MIN: usize = 3;
const NAME_MAX: usize = 16;

// the lowercase account key, or none when the name has characters outside a-z 0-9 and underscore
pub fn canonical(name: &str) -> Option<String>
{
    let valid: bool = (NAME_MIN..=NAME_MAX).contains(&name.len())
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    valid.then(|| name.to_ascii_lowercase())
}
