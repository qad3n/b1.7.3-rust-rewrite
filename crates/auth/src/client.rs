use crate::suite::{self, CONTEXT, Suite};
use crate::{AuthResult, Keys, Message, PASSWORD_MIN, canonical};
use opaque_ke::{
    ClientLogin, ClientLoginFinishParameters, ClientRegistration, ClientRegistrationFinishParameters,
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
    fn new(name: &str, key: &[u8], password: &str) -> AuthResult<ClientAuth>
    {
        let name: String = canonical(name).ok_or("This name cannot log in to an auth server")?;
        checkPassword(password)?;
        Ok(ClientAuth {
            m_name: name,
            m_key: key.to_vec(),
            m_password: Zeroizing::new(password.as_bytes().to_vec()),
            m_next: None,
            m_change: false,
            m_state: State::Done,
        })
    }

    pub fn register(name: &str, key: &[u8], password: &str, invite: &str) -> AuthResult<(ClientAuth, Message)>
    {
        let mut auth: ClientAuth = ClientAuth::new(name, key, password)?;
        let message: Message = auth.startRegister(invite)?;
        Ok((auth, message))
    }

    pub fn login(name: &str, key: &[u8], password: &str) -> AuthResult<(ClientAuth, Message)>
    {
        let mut auth: ClientAuth = ClientAuth::new(name, key, password)?;
        let message: Message = auth.startLogin()?;
        Ok((auth, message))
    }

    pub fn change(name: &str, key: &[u8], old: &str, new: &str) -> AuthResult<(ClientAuth, Message)>
    {
        checkPassword(new)?;
        let mut auth: ClientAuth = ClientAuth::new(name, key, old)?;
        auth.m_next = Some(Zeroizing::new(new.as_bytes().to_vec()));
        auth.m_change = true;
        let message: Message = auth.startLogin()?;
        Ok((auth, message))
    }

    fn startRegister(&mut self, invite: &str) -> AuthResult<Message>
    {
        let started = ClientRegistration::<Suite>::start(&mut OsRng, &self.m_password).map_err(protocol)?;
        self.m_state = State::Register(Box::new(started.state));
        Ok(Message::RegisterRequest { invite: invite.to_string(), request: started.message.serialize().to_vec() })
    }

    fn startLogin(&mut self) -> AuthResult<Message>
    {
        let started = ClientLogin::<Suite>::start(&mut OsRng, &self.m_password).map_err(protocol)?;
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
        let finished = registration.finish(&mut OsRng, &self.m_password, response, parameters).map_err(protocol)?;
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
        let finished = login
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

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::server::{Handshake, Outcome, Registration};
    use crate::{Mode, Vault};
    use std::path::PathBuf;

    const PASSPHRASE: &[u8] = b"correct horse battery";

    fn vault(tag: &str) -> Vault
    {
        let path: PathBuf =
            std::env::temp_dir().join(format!("b173-auth-flow-{tag}-{}", std::process::id())).join("vault.bin");
        let _ = std::fs::remove_file(&path);
        Vault::open(&path, PASSPHRASE).expect("vault")
    }

    fn reply(outcome: Outcome) -> Message
    {
        match outcome
        {
            Outcome::Send(message) => message,
            _ => panic!("expected a reply"),
        }
    }

    fn send(step: ClientStep) -> Vec<Message>
    {
        match step
        {
            ClientStep::Send(messages) => messages,
            ClientStep::Secure(..) => panic!("expected messages"),
        }
    }

    fn register(vault: &mut Vault, name: &str, password: &str, invite: &str) -> Outcome
    {
        let (mut server, start): (Handshake, Message) =
            Handshake::join(vault, name, Registration::Open, false).expect("join");
        let Message::Start { mode: Mode::Register, key, .. } = start
        else
        {
            panic!("expected registration");
        };

        let (mut client, request): (ClientAuth, Message) =
            ClientAuth::register(name, &key, password, invite).expect("start");
        let response: Message = reply(server.receive(vault, request));
        let mut messages: Vec<Message> = send(client.receive(response).expect("upload"));
        let ke1: Message = messages.pop().expect("ke1");
        let upload: Message = messages.pop().expect("upload");
        assert!(matches!(server.receive(vault, upload), Outcome::Wait));
        let ke2: Message = reply(server.receive(vault, ke1));
        let ClientStep::Secure(_, ke3) = client.receive(ke2).expect("ke3")
        else
        {
            panic!("expected keys");
        };
        server.receive(vault, ke3)
    }

    fn login(vault: &mut Vault, name: &str, password: &str) -> AuthResult<Outcome>
    {
        let (mut server, start): (Handshake, Message) =
            Handshake::join(vault, name, Registration::Open, false).expect("join");
        let Message::Start { mode: Mode::Login, key, .. } = start
        else
        {
            panic!("expected login");
        };

        let (mut client, ke1): (ClientAuth, Message) = ClientAuth::login(name, &key, password)?;
        let ke2: Message = reply(server.receive(vault, ke1));
        let ClientStep::Secure(_, ke3) = client.receive(ke2)?
        else
        {
            panic!("expected keys");
        };
        Ok(server.receive(vault, ke3))
    }

    #[test]
    fn register_then_login_succeeds_and_wrong_password_fails()
    {
        let mut vault: Vault = vault("login");
        assert!(matches!(register(&mut vault, "Notch", "hunter2hunter2", ""), Outcome::Authed(_)));
        assert!(matches!(login(&mut vault, "notch", "hunter2hunter2"), Ok(Outcome::Authed(_))));
        assert_eq!(login(&mut vault, "NOTCH", "hunter3hunter3").err().as_deref(), Some("Wrong password"));
        assert_eq!(vault.account("notch").map(|account| account.m_display.as_str()), Some("Notch"));
    }

    #[test]
    fn session_keys_match_on_both_sides()
    {
        let mut vault: Vault = vault("keys");
        register(&mut vault, "jeb_", "hunter2hunter2", "");
        let (mut server, start): (Handshake, Message) =
            Handshake::join(&vault, "jeb_", Registration::Open, false).expect("join");
        let Message::Start { key, .. } = start
        else
        {
            panic!("expected start");
        };

        let (mut client, ke1): (ClientAuth, Message) =
            ClientAuth::login("jeb_", &key, "hunter2hunter2").expect("login");
        let ke2: Message = reply(server.receive(&mut vault, ke1));
        let ClientStep::Secure(mut clientKeys, ke3) = client.receive(ke2).expect("ke3")
        else
        {
            panic!("expected keys");
        };
        let Outcome::Authed(mut serverKeys) = server.receive(&mut vault, ke3)
        else
        {
            panic!("expected auth");
        };

        let frame: Vec<u8> = clientKeys.m_seal.seal(b"ping").expect("seal");
        assert_eq!(serverKeys.m_open.open(&frame[crate::FRAME_HEADER..]).as_deref(), Ok(&b"ping"[..]));
    }

    #[test]
    fn a_forged_server_key_is_refused()
    {
        let mut vault: Vault = vault("forged");
        register(&mut vault, "alex", "hunter2hunter2", "");
        let (mut server, _): (Handshake, Message) =
            Handshake::join(&vault, "alex", Registration::Open, false).expect("join");
        let (mut client, ke1): (ClientAuth, Message) =
            ClientAuth::login("alex", &[9; 32], "hunter2hunter2").expect("login");
        let ke2: Message = reply(server.receive(&mut vault, ke1));
        assert_eq!(client.receive(ke2).err().as_deref(), Some(KEY_MISMATCH));
    }

    #[test]
    fn op_names_need_an_invite_even_with_open_registration()
    {
        let mut vault: Vault = vault("op");
        let (mut server, start): (Handshake, Message) =
            Handshake::join(&vault, "Admin", Registration::Open, true).expect("join");
        let Message::Start { mode: Mode::Register, invite: true, key } = start
        else
        {
            panic!("expected an invite gate");
        };

        let (_, request): (ClientAuth, Message) =
            ClientAuth::register("Admin", &key, "hunter2hunter2", "WRONGCODE").expect("start");
        assert!(matches!(server.receive(&mut vault, request), Outcome::Fail(_)));

        let code: String = vault.invite("admin").expect("invite");
        let (mut server, _): (Handshake, Message) =
            Handshake::join(&vault, "Admin", Registration::Open, true).expect("join");
        let (mut client, request): (ClientAuth, Message) =
            ClientAuth::register("Admin", &key, "hunter2hunter2", &code).expect("start");
        let response: Message = reply(server.receive(&mut vault, request));
        let mut messages: Vec<Message> = send(client.receive(response).expect("upload"));
        messages.pop();
        let upload: Message = messages.pop().expect("upload");
        assert!(matches!(server.receive(&mut vault, upload), Outcome::Wait));
        assert!(!vault.inviteValid("admin", &code));
    }

    #[test]
    fn closed_registration_and_bad_names_are_refused()
    {
        let vault: Vault = vault("closed");
        assert!(Handshake::join(&vault, "newbie", Registration::Closed, false).is_err());
        assert!(Handshake::join(&vault, "N\u{43e}tch", Registration::Open, false).is_err());
        assert!(ClientAuth::login("notch", &vault.key(), "short").is_err());
    }

    #[test]
    fn out_of_order_messages_are_refused()
    {
        let mut vault: Vault = vault("order");
        let (mut server, _): (Handshake, Message) =
            Handshake::join(&vault, "steve", Registration::Open, false).expect("join");
        assert!(matches!(server.receive(&mut vault, Message::Ke3(vec![0; 64])), Outcome::Refuse(_)));
        assert!(matches!(server.receive(&mut vault, Message::RegisterUpload(vec![])), Outcome::Refuse(_)));
    }

    #[test]
    fn password_change_needs_the_old_password()
    {
        let mut vault: Vault = vault("change");
        register(&mut vault, "herobrine", "hunter2hunter2", "");
        let key: Vec<u8> = vault.key();

        let mut server: Handshake = Handshake::change("herobrine", "herobrine");
        let (mut client, ke1): (ClientAuth, Message) =
            ClientAuth::change("herobrine", &key, "badbadbadbad", "newpassword1").expect("change");
        let ke2: Message = reply(server.receive(&mut vault, ke1));
        assert!(client.receive(ke2).is_err());

        let mut server: Handshake = Handshake::change("herobrine", "herobrine");
        let (mut client, ke1): (ClientAuth, Message) =
            ClientAuth::change("herobrine", &key, "hunter2hunter2", "newpassword1").expect("change");
        let ke2: Message = reply(server.receive(&mut vault, ke1));
        let mut messages: Vec<Message> = send(client.receive(ke2).expect("ke3"));
        let request: Message = messages.pop().expect("request");
        let ke3: Message = messages.pop().expect("ke3");
        assert!(matches!(server.receive(&mut vault, ke3), Outcome::Wait));
        let response: Message = reply(server.receive(&mut vault, request));
        let upload: Message = send(client.receive(response).expect("upload")).pop().expect("upload");
        assert!(matches!(server.receive(&mut vault, upload), Outcome::Changed));

        assert!(matches!(login(&mut vault, "herobrine", "newpassword1"), Ok(Outcome::Authed(_))));
        assert!(login(&mut vault, "herobrine", "hunter2hunter2").is_err());
    }
}
