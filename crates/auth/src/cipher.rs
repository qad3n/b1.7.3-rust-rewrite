use crate::AuthResult;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroizing;

pub const FRAME_HEADER: usize = 4;
pub const PLAIN_LIMIT: usize = 1 << 20;

const TAG_LEN: usize = 16;
const KEY_LEN: usize = 32;
const CLIENT_TO_SERVER: &[u8] = b"b173 c2s v1";
const SERVER_TO_CLIENT: &[u8] = b"b173 s2c v1";

pub struct Keys
{
    pub m_seal: Sealer,
    pub m_open: Opener,
}

impl Keys
{
    pub fn derive(session: &[u8], server: bool) -> Keys
    {
        let hkdf: Hkdf<Sha256> = Hkdf::new(None, session);
        let toServer: ChaCha20Poly1305 = expand(&hkdf, CLIENT_TO_SERVER);
        let toClient: ChaCha20Poly1305 = expand(&hkdf, SERVER_TO_CLIENT);
        let (seal, open): (ChaCha20Poly1305, ChaCha20Poly1305) = if server
        {
            (toClient, toServer)
        }
        else
        {
            (toServer, toClient)
        };
        Keys { m_seal: Sealer { m_cipher: seal, m_counter: 0 }, m_open: Opener { m_cipher: open, m_counter: 0 } }
    }
}

fn expand(hkdf: &Hkdf<Sha256>, label: &[u8]) -> ChaCha20Poly1305
{
    let mut key: Zeroizing<[u8; KEY_LEN]> = Zeroizing::new([0; KEY_LEN]);
    hkdf.expand(label, key.as_mut()).expect("32 bytes is a valid hkdf length");
    ChaCha20Poly1305::new(Key::from_slice(key.as_ref()))
}

// the counter is the nonce so a replayed dropped or reordered frame fails its tag check
fn nonce(counter: &mut u64) -> AuthResult<Nonce>
{
    let current: u64 = *counter;
    *counter = current.checked_add(1).ok_or("session cipher counter exhausted")?;
    let mut bytes: [u8; 12] = [0; 12];
    bytes[4..].copy_from_slice(&current.to_le_bytes());
    Ok(*Nonce::from_slice(&bytes))
}

pub struct Sealer
{
    m_cipher: ChaCha20Poly1305,
    m_counter: u64,
}

impl Sealer
{
    // one length prefixed frame per call, the caller splits plaintext at PLAIN_LIMIT
    pub fn seal(&mut self, plain: &[u8]) -> AuthResult<Vec<u8>>
    {
        if plain.len() > PLAIN_LIMIT
        {
            return Err(format!("frame of {} bytes is over the limit", plain.len()));
        }

        let nonce: Nonce = nonce(&mut self.m_counter)?;
        let sealed: Vec<u8> =
            self.m_cipher.encrypt(&nonce, plain).map_err(|_| String::from("session cipher failed to seal"))?;
        let mut frame: Vec<u8> = Vec::with_capacity(FRAME_HEADER + sealed.len());
        frame.extend_from_slice(&(sealed.len() as u32).to_be_bytes());
        frame.extend_from_slice(&sealed);
        Ok(frame)
    }
}

pub struct Opener
{
    m_cipher: ChaCha20Poly1305,
    m_counter: u64,
}

impl Opener
{
    pub fn length(header: [u8; FRAME_HEADER]) -> AuthResult<usize>
    {
        let len: usize = u32::from_be_bytes(header) as usize;
        if !(TAG_LEN..=PLAIN_LIMIT + TAG_LEN).contains(&len)
        {
            return Err(format!("encrypted frame of {len} bytes is out of range"));
        }

        Ok(len)
    }

    pub fn open(&mut self, sealed: &[u8]) -> AuthResult<Vec<u8>>
    {
        let nonce: Nonce = nonce(&mut self.m_counter)?;
        self.m_cipher.decrypt(&nonce, sealed).map_err(|_| String::from("encrypted frame failed authentication"))
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn pair() -> (Keys, Keys) { (Keys::derive(&[7; 64], false), Keys::derive(&[7; 64], true)) }

    fn body(frame: &[u8]) -> &[u8]
    {
        let header: [u8; FRAME_HEADER] = frame[..FRAME_HEADER].try_into().expect("frame has a header");
        assert_eq!(Opener::length(header), Ok(frame.len() - FRAME_HEADER));
        &frame[FRAME_HEADER..]
    }

    #[test]
    fn frames_round_trip_both_ways()
    {
        let (mut client, mut server): (Keys, Keys) = pair();
        let up: Vec<u8> = client.m_seal.seal(b"hello server").expect("seal");
        assert_eq!(server.m_open.open(body(&up)).as_deref(), Ok(&b"hello server"[..]));
        let down: Vec<u8> = server.m_seal.seal(b"hello client").expect("seal");
        assert_eq!(client.m_open.open(body(&down)).as_deref(), Ok(&b"hello client"[..]));
    }

    #[test]
    fn tampered_replayed_and_reordered_frames_fail()
    {
        let (mut client, mut server): (Keys, Keys) = pair();
        let first: Vec<u8> = client.m_seal.seal(b"one").expect("seal");
        let second: Vec<u8> = client.m_seal.seal(b"two").expect("seal");
        let mut tampered: Vec<u8> = first.clone();
        tampered[FRAME_HEADER] ^= 1;
        assert!(server.m_open.open(body(&tampered)).is_err());

        let (_, mut fresh): (Keys, Keys) = pair();
        assert!(fresh.m_open.open(body(&second)).is_err());

        let (_, mut replay): (Keys, Keys) = pair();
        assert!(replay.m_open.open(body(&first)).is_ok());
        assert!(replay.m_open.open(body(&first)).is_err());
    }

    #[test]
    fn a_direction_cannot_open_its_own_frames()
    {
        let (mut client, _): (Keys, Keys) = pair();
        let frame: Vec<u8> = client.m_seal.seal(b"loop").expect("seal");
        assert!(client.m_open.open(body(&frame)).is_err());
    }

    #[test]
    fn oversized_frames_are_refused()
    {
        let (mut client, _): (Keys, Keys) = pair();
        assert!(client.m_seal.seal(&vec![0; PLAIN_LIMIT + 1]).is_err());
        assert!(Opener::length(((PLAIN_LIMIT + TAG_LEN + 1) as u32).to_be_bytes()).is_err());
        assert!(Opener::length([0; FRAME_HEADER]).is_err());
    }
}
