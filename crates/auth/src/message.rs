use crate::AuthResult;
use crate::wire::{Reader, Writer};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode
{
    Login,
    Register,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Message
{
    Start
    {
        mode: Mode,
        invite: bool,
        key: Vec<u8>,
    },

    RegisterRequest
    {
        invite: String,
        request: Vec<u8>,
    },
    RegisterResponse(Vec<u8>),
    RegisterUpload(Vec<u8>),
    Ke1(Vec<u8>),
    Ke2(Vec<u8>),
    Ke3(Vec<u8>),
    // the sender switches its writer to the session cipher right after this message
    Secure,
    Result(String),
}

mod tag
{
    pub const START: u8 = 0;
    pub const REGISTER_REQUEST: u8 = 1;
    pub const REGISTER_RESPONSE: u8 = 2;
    pub const REGISTER_UPLOAD: u8 = 3;
    pub const KE1: u8 = 4;
    pub const KE2: u8 = 5;
    pub const KE3: u8 = 6;
    pub const SECURE: u8 = 7;
    pub const RESULT: u8 = 8;
}

impl Message
{
    pub fn encode(&self) -> Vec<u8>
    {
        let mut writer: Writer = Writer::new();

        match self
        {
            Self::Start { mode, invite, key } =>
            {
                writer.byte(tag::START);
                writer.byte(matches!(mode, Mode::Register) as u8);
                writer.byte(*invite as u8);
                writer.bytes(key);
            }

            Self::RegisterRequest { invite, request } =>
            {
                writer.byte(tag::REGISTER_REQUEST);
                writer.text(invite);
                writer.bytes(request);
            }

            Self::RegisterResponse(body) => Self::tagged(&mut writer, tag::REGISTER_RESPONSE, body),
            Self::RegisterUpload(body) => Self::tagged(&mut writer, tag::REGISTER_UPLOAD, body),
            Self::Ke1(body) => Self::tagged(&mut writer, tag::KE1, body),
            Self::Ke2(body) => Self::tagged(&mut writer, tag::KE2, body),
            Self::Ke3(body) => Self::tagged(&mut writer, tag::KE3, body),
            Self::Secure => writer.byte(tag::SECURE),

            Self::Result(text) =>
            {
                writer.byte(tag::RESULT);
                writer.text(text);
            }
        }

        writer.finish()
    }

    fn tagged(writer: &mut Writer, tag: u8, body: &[u8])
    {
        writer.byte(tag);
        writer.bytes(body);
    }

    pub fn decode(data: &[u8]) -> AuthResult<Self>
    {
        let mut reader: Reader<'_> = Reader::new(data);
        let message: Self = match reader.byte()?
        {
            tag::START =>
            {
                let mode: Mode = if reader.byte()? != 0
                {
                    Mode::Register
                }
                else
                {
                    Mode::Login
                };
                let invite: bool = reader.byte()? != 0;
                Self::Start { mode, invite, key: reader.bytes()? }
            }

            tag::REGISTER_REQUEST =>
            {
                let invite: String = reader.text()?;
                Self::RegisterRequest { invite, request: reader.bytes()? }
            }

            tag::REGISTER_RESPONSE => Self::RegisterResponse(reader.bytes()?),
            tag::REGISTER_UPLOAD => Self::RegisterUpload(reader.bytes()?),
            tag::KE1 => Self::Ke1(reader.bytes()?),
            tag::KE2 => Self::Ke2(reader.bytes()?),
            tag::KE3 => Self::Ke3(reader.bytes()?),
            tag::SECURE => Self::Secure,
            tag::RESULT => Self::Result(reader.text()?),
            tag => return Err(format!("unknown auth message {tag}")),
        };

        reader.end()?;
        Ok(message)
    }

    pub fn secure(data: &[u8]) -> bool { data == [tag::SECURE] }
}
