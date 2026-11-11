use crate::suite::{self, CONTEXT, Suite};
use crate::{AuthResult, Keys, Message, PASSWORD_MIN, canonical};
use opaque_ke::
{
    ClientLogin, ClientLoginFinishParameters, ClientLoginFinishResult, ClientLoginStartResult, ClientRegistration,
    ClientRegistrationFinishParameters, ClientRegistrationFinishResult, ClientRegistrationStartResult,
    CredentialResponse, RegistrationResponse,
};
use rand_core::OsRng;
use zeroize::Zeroizing;

const PROTOCOL_ERROR: &str = "Protocol error";
const KEY_MISMATCH: &str = "The server key does not match the one this client trusts";

pub enum ClientStep
{
    Send(Vec<Message>),
    // send the message, arm the keys, then switch on the server's secure marker
    Secure(Keys, Message),
}

enum State
{
    Register(Box<ClientRegistration<Suite>>),
    Login(Box<ClientLogin<Suite>>),
    Done,
}

pub struct ClientAuth
{
    m_name: String,
    m_key: Vec<u8>,
    m_password: Zeroizing<Vec<u8>>,
    m_next: Option<Zeroizing<Vec<u8>>>,
    m_change: bool,
    m_state: State,
}

impl ClientAuth
{
    fn new(name: &str, key: &[u8], password: &str) -> AuthResult<Self>
    {
        let name: String = canonical(name).ok_or("This name cannot log in to an auth server")?;

        checkPassword(password)?;

        Ok(Self
        {
            m_name: name,
            m_key: key.to_vec(),
            m_password: Zeroizing::new(password.as_bytes().to_vec()),
            m_next: None,
            m_change: false,
            m_state: State::Done,
        })
    }

    pub fn register(name: &str, key: &[u8], password: &str, invite: &str) -> AuthResult<(Self, Message)>
    {
        let mut auth: Self = Self::new(name, key, password)?;
        let message: Message = auth.startRegister(invite)?;
        Ok((auth, message))
    }

    pub fn login(name: &str, key: &[u8], password: &str) -> AuthResult<(Self, Message)>
    {
        let mut auth: Self = Self::new(name, key, password)?;
        let message: Message = auth.startLogin()?;
        Ok((auth, message))
    }

    pub fn change(name: &str, key: &[u8], old: &str, new: &str) -> AuthResult<(Self, Message)>
    {
        checkPassword(new)?;

        let mut auth: Self = Self::new(name, key, old)?;
        auth.m_next = Some(Zeroizing::new(new.as_bytes().to_vec()));
        auth.m_change = true;

        let message: Message = auth.startLogin()?;
        Ok((auth, message))
    }

    fn startRegister(&mut self, invite: &str) -> AuthResult<Message>
    {
        let started: ClientRegistrationStartResult<Suite> =
            ClientRegistration::<Suite>::start(&mut OsRng, &self.m_password).map_err(protocol)?;
        self.m_state = State::Register(Box::new(started.state));
        Ok(Message::RegisterRequest { invite: invite.to_string(), request: started.message.serialize().to_vec() })
    }

    fn startLogin(&mut self) -> AuthResult<Message>
    {
        let started: ClientLoginStartResult<Suite> =
            ClientLogin::<Suite>::start(&mut OsRng, &self.m_password).map_err(protocol)?;
        self.m_state = State::Login(Box::new(started.state));
        Ok(Message::Ke1(started.message.serialize().to_vec()))
    }

    pub fn receive(&mut self, message: Message) -> AuthResult<ClientStep>
    {
        let state: State = std::mem::replace(&mut self.m_state, State::Done);
        match (state, message)
        {
            (State::Register(registration), Message::RegisterResponse(body)) => self.upload(*registration, &body),
            (State::Login(login), Message::Ke2(body)) => self.ke3(*login, &body),
            _ => Err(String::from(PROTOCOL_ERROR)),
        }
    }

    fn upload(&mut self, registration: ClientRegistration<Suite>, body: &[u8]) -> AuthResult<ClientStep>
    {
        let response: RegistrationResponse<Suite> = RegistrationResponse::deserialize(body).map_err(protocol)?;
        let parameters: ClientRegistrationFinishParameters<'_, '_, Suite> =
            ClientRegistrationFinishParameters::new(suite::identifiers(&self.m_name), None);
        let finished: ClientRegistrationFinishResult<Suite> =
            registration.finish(&mut OsRng, &self.m_password, response, parameters).map_err(protocol)?;

        if finished.server_s_pk.serialize().as_slice() != self.m_key.as_slice()
        {
            return Err(String::from(KEY_MISMATCH));
        }

        let upload: Message = Message::RegisterUpload(finished.message.serialize().to_vec());

        if self.m_change
        {
            self.m_password = Zeroizing::new(Vec::new());
            return Ok(ClientStep::Send(vec![upload]));
        }

        let login: Message = self.startLogin()?;
        Ok(ClientStep::Send(vec![upload, login]))
    }

    fn ke3(&mut self, login: ClientLogin<Suite>, body: &[u8]) -> AuthResult<ClientStep>
    {
        let response: CredentialResponse<Suite> = CredentialResponse::deserialize(body).map_err(protocol)?;
        let parameters: ClientLoginFinishParameters<'_, '_, '_, Suite> =
            ClientLoginFinishParameters::new(Some(CONTEXT), suite::identifiers(&self.m_name), None);
        let finished: ClientLoginFinishResult<Suite> = login
            .finish(&mut OsRng, &self.m_password, response, parameters)
            .map_err(|_| String::from("Wrong password"))?;

        if finished.server_s_pk.serialize().as_slice() != self.m_key.as_slice()
        {
            return Err(String::from(KEY_MISMATCH));
        }

        let ke3: Message = Message::Ke3(finished.message.serialize().to_vec());

        match self.m_next.take()
        {
            Some(next) =>
            {
                self.m_password = next;

                let request: Message = self.startRegister("")?;
                Ok(ClientStep::Send(vec![ke3, request]))
            }

            None =>
            {
                self.m_password = Zeroizing::new(Vec::new());
                Ok(ClientStep::Secure(Keys::derive(&finished.session_key, false), ke3))
            }
        }
    }
}

fn protocol<T>(_: T) -> String { String::from(PROTOCOL_ERROR) }

fn checkPassword(password: &str) -> AuthResult<()>
{
    if password.chars().count() < PASSWORD_MIN
    {
        return Err(format!("Passwords need at least {PASSWORD_MIN} characters"));
    }

    Ok(())
}
