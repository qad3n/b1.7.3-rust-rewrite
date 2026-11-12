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
    // a sprite on the particle sheet when the mod ships textures/particle/<name>.png
    pub m_texture: Option<Identifier>,
}

thread_local!
{
    // mod particles by the name a spawn uses, <modid>:<name>
    static KINDS: RefCell<BTreeMap<String, ParticleKind>> = const { RefCell::new(BTreeMap::new()) };
}

pub fn register(id: Identifier, spec: ParticleSpec) -> Result<(), String>
{
    crate::registry::claim(mod_protocol::abi::RegistryKind::Particle, &id)?;

    let kind: ParticleKind = ParticleKind
    {
        m_base: spec.base,
        m_color: spec.color,
        m_scale: spec.scale,
        m_lifetime: spec.lifetime,
        m_gravity: spec.gravity,
        m_texture: None,
    };
    KINDS.with(|kinds| kinds.borrow_mut().insert(id.to_string(), kind));
    Ok(())
}

// a mod particle with textures/particle/<name>.png in its mod or a pack draws that sprite, client only
pub fn requestTextures()
{
    KINDS.with(|kinds|
    {
        for (name, kind) in kinds.borrow_mut().iter_mut()
        {
            let Ok(id) = Identifier::parse(name)
            else
            {
                continue;
            };

            let Ok(texture) = Identifier::new(id.namespace(), &format!("particle/{}", id.path()))
            else
            {
                continue;
            };
            let location: game::resources::Location =
                game::resources::Location::new(texture.namespace(), &format!("textures/{}.png", texture.path()));
            if game::resources::read(&location).is_some()
            {
                crate::atlas::request(crate::atlas::Atlas::Particles, &texture.to_string());
                kind.m_texture = Some(texture);
            }
        }
    });
}

pub fn kind(name: &str) -> Option<ParticleKind> { KINDS.with(|kinds| kinds.borrow().get(name).cloned()) }
