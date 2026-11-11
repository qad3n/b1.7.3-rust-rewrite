use mod_protocol::Identifier;
use mod_protocol::spec::ParticleSpec;
use std::cell::RefCell;
use std::collections::BTreeMap;

// a mod particle: a beta particle by its spawnParticle name, with the parts the mod changes
#[derive(Clone)]
pub struct ParticleKind
{
    pub m_base: String,
    pub m_color: Option<u32>,
    pub m_scale: Option<f32>,
    pub m_lifetime: Option<i32>,
    pub m_gravity: Option<f32>,
}

thread_local!
{
    // mod particles by the name a spawn uses, <modid>:<name>
    static KINDS: RefCell<BTreeMap<String, ParticleKind>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn register(id: Identifier, spec: ParticleSpec)
{
    let kind: ParticleKind = ParticleKind
    {
        m_base: spec.base,
        m_color: spec.color,
        m_scale: spec.scale,
        m_lifetime: spec.lifetime,
        m_gravity: spec.gravity,
    };
    KINDS.with(|kinds| kinds.borrow_mut().insert(id.to_string(), kind));
}

pub fn kind(name: &str) -> Option<ParticleKind> { KINDS.with(|kinds| kinds.borrow().get(name).cloned()) }
