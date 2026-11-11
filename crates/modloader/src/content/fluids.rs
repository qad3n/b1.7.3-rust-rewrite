use game::block::map_color::MapColors;
use game::block::{MapColor, Material};
use game::fluid::{self, Fluid, FluidStyle};
use mod_protocol::Identifier;
use mod_protocol::spec::{FluidSpec, FluidStyle as SpecStyle};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

// beta numbers a mod fluid takes unless its spec sets its own
mod defaults
{
    pub const WATER_TICKS: i32 = 5;
    pub const LAVA_TICKS: i32 = 30;
    pub const WATER_DECAY: i32 = 1;
    pub const LAVA_DECAY: i32 = 2;
    pub const NETHER_DECAY: i32 = 1;
}

thread_local!
{
    // mod fluids by identifier, their blocks and bucket resolved to numbers on every content build
    static SPECS: RefCell<BTreeMap<Identifier, FluidSpec>> = const { RefCell::new(BTreeMap::new()) };
}

// the fluid row exists from registration on, so blocks that name it as their material find it when they are built
pub fn register(id: Identifier, spec: FluidSpec) -> Result<(), String>
{
    if fluid::byName(&id.to_string()).is_some()
    {
        return Err(format!("fluid {id} is already registered"));
    }

    let style: FluidStyle = match spec.style
    {
        SpecStyle::Water => FluidStyle::Water,
        SpecStyle::Lava => FluidStyle::Lava,
    };
    let lava: bool = style == FluidStyle::Lava;
    let colors: Rc<MapColors> = MapColor::colors();

    let color: &Rc<MapColor> = if lava
    {
        &colors.m_tntColor
    }
    else
    {
        &colors.m_waterColor
    };

    fluid::add(Fluid
    {
        m_name: id.to_string(),
        m_material: Material::liquid(color),
        m_style: style,
        m_flowing: Cell::new(0),
        m_still: Cell::new(0),
        m_bucket: Cell::new(0),
        m_tickRate: spec.tick_rate.unwrap_or(
            if lava
            {
                defaults::LAVA_TICKS
            }
            else
            {
                defaults::WATER_TICKS
            },
        ),
        m_decay: spec.decay.unwrap_or(
            if lava
            {
                defaults::LAVA_DECAY
            }
            else
            {
                defaults::WATER_DECAY
            },
        ),
        m_netherDecay: spec.nether_decay.unwrap_or(defaults::NETHER_DECAY),
        m_infinite: spec.infinite.unwrap_or(!lava),
        m_sourceRule: None,
        m_sprite: fluid::sprite(&Material::water()),
    });

    SPECS.with(|specs| specs.borrow_mut().insert(id, spec));
    Ok(())
}

pub fn material(name: &str) -> Option<Rc<Material>>
{
    let id: Identifier = Identifier::parse(name).ok()?;
    if !SPECS.with(|specs| specs.borrow().contains_key(&id))
    {
        return None;
    }

    fluid::byName(name).map(|fluid| Rc::clone(&fluid.m_material))
}

// the flowing block a fluid bucket item places
pub fn bucketFlowing(bucket: &Identifier) -> Option<i32>
{
    let flowing: Identifier = SPECS.with(|specs|
    {
        specs.borrow().values().find(|spec| spec.bucket.as_ref() == Some(bucket)).map(|spec| spec.flowing.clone())
    })?;
    crate::registry::with(|registries| registries.m_blocks.numeric(&flowing))
}

// block and bucket numbers change with every id map, so each content build sets them again
pub fn push()
{
    let specs: Vec<(Identifier, FluidSpec)> =
        SPECS.with(|specs| specs.borrow().iter().map(|(id, spec)| (id.clone(), spec.clone())).collect());

    for (id, spec) in specs
    {
        let Some(row) = fluid::byName(&id.to_string())
        else
        {
            continue;
        };

        let (flowing, still, bucket): (Option<i32>, Option<i32>, Option<i32>) = crate::registry::with(|registries|
        {
            (
                registries.m_blocks.numeric(&spec.flowing),
                registries.m_blocks.numeric(&spec.still),
                spec.bucket.as_ref().and_then(|bucket| registries.itemId(bucket)),
            )
        });
        let (Some(flowing), Some(still)) = (flowing, still)
        else
        {
            crate::log::warn(crate::log::LOADER, &format!("fluid {id} names blocks that are not registered"));
            continue;
        };
        row.m_flowing.set(flowing);
        row.m_still.set(still);
        row.m_bucket.set(bucket.unwrap_or(0));
    }
}
