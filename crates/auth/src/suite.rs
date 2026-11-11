use opaque_ke::argon2::{Algorithm, Argon2, Params, Version};
use opaque_ke::errors::InternalError;
use opaque_ke::generic_array::{ArrayLength, GenericArray};
use opaque_ke::ksf::Ksf;
use opaque_ke::{CipherSuite, Identifiers, Ristretto255, TripleDh};
use sha2::Sha512;

pub(crate) const CONTEXT: &[u8] = b"b1.7.3-rewrite auth v1";
// stored with every record so the stretch parameters can change without breaking old accounts
pub(crate) const SUITE_VERSION: u8 = 1;

const MEMORY_KIB: u32 = 64 * 1024;
const PASSES: u32 = 3;
const LANES: u32 = 1;

pub struct Suite;

impl CipherSuite for Suite
{
    type OprfCs = Ristretto255;
    type KeyExchange = TripleDh<Ristretto255, Sha512>;
    type Ksf = Stretch;
}

pub struct Stretch(Argon2<'static>);

impl Default for Stretch
{
    fn default() -> Self { Self(argon2()) }
}

impl Ksf for Stretch
{
    fn hash<L: ArrayLength<u8>>(&self, input: GenericArray<u8, L>) -> Result<GenericArray<u8, L>, InternalError>
    {
        self.0.hash(input)
    }
}

pub(crate) fn argon2() -> Argon2<'static>
{
    let params: Params = Params::new(MEMORY_KIB, PASSES, LANES, None).expect("argon2 parameters are valid constants");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

pub(crate) fn identifiers(name: &str) -> Identifiers<'_> { Identifiers { client: Some(name.as_bytes()), server: None } }
