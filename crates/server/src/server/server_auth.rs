use crate::server::PropertyManager;
use auth::{Limiter, Registration, Vault};
use std::cell::{RefCell, RefMut};
use std::net::IpAddr;
use std::path::Path;
use std::rc::Rc;
use zeroize::Zeroizing;

pub const PASSPHRASE_VAR: &str = "B173_AUTH_PASSPHRASE";
const VAULT_PATH: &str = "auth/vault.bin";

struct PolicyRow
{
    m_key: &'static str,
    m_policy: Registration,
}

const POLICIES: [PolicyRow; 3] = [
    PolicyRow { m_key: "open", m_policy: Registration::Open },
    PolicyRow { m_key: "invite", m_policy: Registration::Invite },
    PolicyRow { m_key: "closed", m_policy: Registration::Closed },
];

// password accounts for offline mode, the vault holds every secret and is sealed on disk
pub struct ServerAuth
{
    m_vault: RefCell<Vault>,
    m_limiter: RefCell<Limiter>,
    m_policy: Registration,
}

impl ServerAuth
{
    pub fn load(properties: &PropertyManager) -> Result<Rc<ServerAuth>, String>
    {
        let key: String = properties.getStringProperty("auth-registration", "open");
        let policy: Registration =
            POLICIES.iter().find(|row| row.m_key == key).map(|row| row.m_policy).ok_or_else(|| {
                format!("unknown auth-registration \"{key}\" in server.properties, use open, invite or closed")
            })?;

        let passphrase: Zeroizing<String> = Zeroizing::new(std::env::var(PASSPHRASE_VAR).map_err(|_| {
            format!("auth is required but {PASSPHRASE_VAR} is not set, it encrypts {VAULT_PATH} and needs 12 or more characters")
        })?);
        let vault: Vault = Vault::open(Path::new(VAULT_PATH), passphrase.as_bytes())?;
        Ok(Rc::new(ServerAuth {
            m_vault: RefCell::new(vault),
            m_limiter: RefCell::new(Limiter::new()),
            m_policy: policy,
        }))
    }

    pub fn vault(&self) -> RefMut<'_, Vault> { self.m_vault.borrow_mut() }

    pub fn limiter(&self) -> RefMut<'_, Limiter> { self.m_limiter.borrow_mut() }

    pub fn policy(&self) -> Registration { self.m_policy }

    pub fn ticket(self: &Rc<Self>, ip: IpAddr) -> Result<Ticket, String>
    {
        self.limiter().open(ip)?;
        Ok(Ticket { m_auth: Rc::clone(self), m_ip: ip })
    }
}

// holds one pending login slot for an address until the login handler is dropped
pub struct Ticket
{
    m_auth: Rc<ServerAuth>,
    m_ip: IpAddr,
}

impl Ticket
{
    pub fn ip(&self) -> IpAddr { self.m_ip }
}

impl Drop for Ticket
{
    fn drop(&mut self) { self.m_auth.limiter().close(self.m_ip); }
}
