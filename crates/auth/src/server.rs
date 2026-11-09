use crate::suite::{self, CONTEXT, SUITE_VERSION, Suite};
use crate::vault::{self, Account, Vault};
use crate::{Keys, Message, Mode, canonical};
use opaque_ke::{
    CredentialFinalization, CredentialRequest, RegistrationRequest, RegistrationUpload, ServerLogin,
    ServerLoginParameters, ServerRegistration,
};
use rand_core::OsRng;

const PROTOCOL_ERROR: &str = "Protocol error";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Registration
{
    Open,
    Invite,
    Closed,
}

pub enum Outcome
{
    Send(Message),
    Wait,
    Authed(Keys),
    Changed,
    // a wrong password or invite, counted against the rate limits
    Fail(String),
    Refuse(String),
}

enum State
{
    Register,
    Upload
    {
        invite: String,
    },
    Login,
    Verify(Box<ServerLogin<Suite>>),
    Verified,
    Done,
}

// one connection's join, or one player's password change, on the server side
pub struct Handshake
{
    m_name: String,
    m_display: String,
    m_change: bool,
    m_invite: bool,
    m_state: State,
}

impl Handshake
{
    // op names always need an invite so nobody can register an op account before its owner does
    pub fn join(vault: &Vault, name: &str, policy: Registration, op: bool) -> Result<(Handshake, Message), String>
    {
        let key: String = canonical(name).ok_or("Names may only use letters, digits and underscores (3 to 16)")?;
        let (mode, display, invite): (Mode, String, bool) = match vault.account(&key)
        {
            Some(account) => (Mode::Login, account.m_display.clone(), false),
            None if policy == Registration::Closed =>
            {
                return Err(String::from("Registration is closed on this server"));
            }
            None => (Mode::Register, name.to_string(), policy == Registration::Invite || op),
        };
        let state: State = if mode == Mode::Login
        {
            State::Login
        }
        else
        {
            State::Register
        };
        let handshake: Handshake =
            Handshake { m_name: key, m_display: display, m_change: false, m_invite: invite, m_state: state };
        Ok((handshake, Message::Start { mode, invite, key: vault.key() }))
    }

    pub fn change(name: &str, display: &str) -> Handshake
    {
        Handshake {
            m_name: name.to_string(),
            m_display: display.to_string(),
            m_change: true,
            m_invite: false,
            m_state: State::Login,
        }
    }

    pub fn name(&self) -> &str { &self.m_name }

    pub fn display(&self) -> &str { &self.m_display }

    pub fn receive(&mut self, vault: &mut Vault, message: Message) -> Outcome
    {
        let state: State = std::mem::replace(&mut self.m_state, State::Done);
        match (state, message)
        {
            (State::Register, Message::RegisterRequest { invite, request }) => self.request(vault, invite, &request),
            (State::Verified, Message::RegisterRequest { request, .. }) if self.m_change =>
            {
                self.request(vault, String::new(), &request)
            }
            (State::Upload { invite }, Message::RegisterUpload(body)) => self.upload(vault, &invite, &body),
            (State::Login, Message::Ke1(body)) => self.ke1(vault, &body),
            (State::Verify(login), Message::Ke3(body)) => self.ke3(*login, &body),
            _ => Outcome::Refuse(String::from(PROTOCOL_ERROR)),
        }
    }

    fn request(&mut self, vault: &Vault, invite: String, body: &[u8]) -> Outcome
    {
        if self.m_invite && !vault.inviteValid(&self.m_name, &invite)
        {
            return Outcome::Fail(String::from("Invalid invite code"));
        }
        if !self.m_change && vault.account(&self.m_name).is_some()
        {
            return Outcome::Refuse(String::from("That name was just registered"));
        }
        let Ok(request) = RegistrationRequest::<Suite>::deserialize(body)
        else
        {
            return Outcome::Refuse(String::from(PROTOCOL_ERROR));
        };

        match ServerRegistration::<Suite>::start(vault.setup(), request, self.m_name.as_bytes())
        {
            Ok(result) =>
            {
                self.m_state = State::Upload { invite };
                Outcome::Send(Message::RegisterResponse(result.message.serialize().to_vec()))
            }
            Err(_) => Outcome::Refuse(String::from(PROTOCOL_ERROR)),
        }
    }

    fn upload(&mut self, vault: &mut Vault, invite: &str, body: &[u8]) -> Outcome
    {
        let Ok(upload) = RegistrationUpload::<Suite>::deserialize(body)
        else
        {
            return Outcome::Refuse(String::from(PROTOCOL_ERROR));
        };

        if !self.m_change
        {
            if vault.account(&self.m_name).is_some()
            {
                return Outcome::Refuse(String::from("That name was just registered"));
            }
            if self.m_invite && vault.redeem(&self.m_name, invite) != Ok(true)
            {
                return Outcome::Fail(String::from("Invalid invite code"));
            }
        }

        let created: u64 = vault.account(&self.m_name).map_or_else(vault::unixNow, |account| account.m_created);
        let account: Account = Account {
            m_display: self.m_display.clone(),
            m_suite: SUITE_VERSION,
            m_record: ServerRegistration::<Suite>::finish(upload).serialize().to_vec(),
            m_created: created,
        };
        if let Err(error) = vault.store(&self.m_name, account)
        {
            return Outcome::Refuse(error);
        }

        if self.m_change
        {
            return Outcome::Changed;
        }
        self.m_state = State::Login;
        Outcome::Wait
    }

    fn ke1(&mut self, vault: &Vault, body: &[u8]) -> Outcome
    {
        let record: Option<ServerRegistration<Suite>> = vault
            .account(&self.m_name)
            .and_then(|account| ServerRegistration::<Suite>::deserialize(&account.m_record).ok());
        let (Some(record), Ok(request)) = (record, CredentialRequest::<Suite>::deserialize(body))
        else
        {
            return Outcome::Refuse(String::from(PROTOCOL_ERROR));
        };

        let started = ServerLogin::start(
            &mut OsRng,
            vault.setup(),
            Some(record),
            request,
            self.m_name.as_bytes(),
            self.parameters(),
        );
        match started
        {
            Ok(result) =>
            {
                self.m_state = State::Verify(Box::new(result.state));
                Outcome::Send(Message::Ke2(result.message.serialize().to_vec()))
            }
            Err(_) => Outcome::Refuse(String::from(PROTOCOL_ERROR)),
        }
    }

    fn ke3(&mut self, login: ServerLogin<Suite>, body: &[u8]) -> Outcome
    {
        let Ok(finalization) = CredentialFinalization::<Suite>::deserialize(body)
        else
        {
            return Outcome::Refuse(String::from(PROTOCOL_ERROR));
        };

        match login.finish(finalization, self.parameters())
        {
            Ok(_) if self.m_change =>
            {
                self.m_state = State::Verified;
                Outcome::Wait
            }
            Ok(result) => Outcome::Authed(Keys::derive(&result.session_key, true)),
            Err(_) => Outcome::Fail(String::from("Wrong password")),
        }
    }

    fn parameters(&self) -> ServerLoginParameters<'_, '_>
    {
        ServerLoginParameters { context: Some(CONTEXT), identifiers: suite::identifiers(&self.m_name) }
    }
}
