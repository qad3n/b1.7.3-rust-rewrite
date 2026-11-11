use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::net::IpAddr;
use std::sync::OnceLock;
use zeroize::Zeroizing;

const KEY: usize = 32;
const TAG_BYTES: usize = 4;

// a fresh key each run so a leaked log cannot be reversed by hashing every ipv4 address
fn key() -> &'static Zeroizing<[u8; KEY]>
{
    static RUN_KEY: OnceLock<Zeroizing<[u8; KEY]>> = OnceLock::new();
    RUN_KEY.get_or_init(||
    {
        let mut key: Zeroizing<[u8; KEY]> = Zeroizing::new([0; KEY]);
        OsRng.fill_bytes(key.as_mut());
        key
    })
}

pub(crate) fn digest(key: &[u8], ip: IpAddr) -> [u8; 32]
{
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(key);

    match ip.to_canonical()
    {
        IpAddr::V4(ip) => hasher.update(ip.octets()),
        IpAddr::V6(ip) => hasher.update(ip.octets()),
    }

    hasher.finalize().into()
}

// stands in for an address in logs, equal addresses share a tag until the server restarts
pub fn tag(ip: IpAddr) -> String
{
    let hash: [u8; 32] = digest(key().as_ref(), ip);
    let hex: String = hash[..TAG_BYTES].iter().map(|byte| format!("{byte:02x}")).collect();
    format!("peer {hex}")
}
