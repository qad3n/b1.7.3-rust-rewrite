use game::block::map_color::MapColors;
use game::block::{MapColor, Material};
use game::fluid::{self, Fluid, FluidStyle};
use mod_protocol::Identifier;
use mod_protocol::abi::{ContainerSwap, FluidStack, StackData};
use mod_protocol::spec::{FluidContainerSpec, FluidSpec, FluidStyle as SpecStyle};
use std::cell::{Cell, RefCell, RefMut};
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
    // every item that holds a fluid, vanilla buckets first
    static CONTAINERS: RefCell<Vec<FluidContainerSpec>> = RefCell::new(vanillaContainers());
}

const BUCKET: i32 = 1000;
// a container row's full_meta that takes any damage
const ANY_DAMAGE: i32 = -1;

fn vanillaContainers() -> Vec<FluidContainerSpec>
{
    [("water_bucket", "water"), ("lava_bucket", "lava")]
        .into_iter()
        .map(|(full, fluid)| FluidContainerSpec
        {
            full: Identifier::minecraft(full),
            full_meta: 0,
            empty: Some(Identifier::minecraft("bucket")),
            fluid: Identifier::minecraft(fluid),
            amount: BUCKET,
        })
        .collect()
}

pub fn addContainer(spec: FluidContainerSpec) -> Result<(), String>
{
    if spec.amount <= 0
    {
        return Err(format!("container {} holds no fluid", spec.full));
    }

    CONTAINERS.with(|containers|
    {
        let mut containers: RefMut<Vec<FluidContainerSpec>> = containers.borrow_mut();
        if containers.iter().any(|known| known.full == spec.full && known.full_meta == spec.full_meta)
        {
            return Err(format!("container {} is already registered", spec.full));
        }

        containers.push(spec);
        Ok(())
    })
}

pub fn containers() -> Vec<FluidContainerSpec> { CONTAINERS.with(|containers| containers.borrow().clone()) }

thread_local!
{
    static OWN_CONTAINERS: crate::stash::Stash<Vec<FluidContainerSpec>> = const { crate::stash::Stash::new() };
}

// a modded server's containers stand in for the client's own until it disconnects
pub fn replaceContainers(containers: Vec<FluidContainerSpec>)
{
    let own: Vec<FluidContainerSpec> = CONTAINERS.with(|slot| slot.replace(containers));
    OWN_CONTAINERS.with(|stash| stash.keep(own));
}

pub fn restoreContainers()
{
    if let Some(own) = OWN_CONTAINERS.with(crate::stash::Stash::take)
    {
        CONTAINERS.with(|slot| slot.replace(own));
    }
}

fn itemName(numeric: i32) -> Option<Identifier>
{
    crate::registry::with(|registries| registries.m_items.byNumeric(numeric).map(|row| row.m_id.clone()))
}

fn itemNumeric(id: &Identifier) -> Option<i32> { crate::registry::with(|registries| registries.itemId(id)) }

fn holds(container: &FluidContainerSpec, name: &Identifier, damage: i32) -> bool
{
    container.full == *name && (container.full_meta == ANY_DAMAGE || container.full_meta == damage)
}

// the fluid a full container holds
pub fn contained(stack: &StackData) -> Option<FluidStack>
{
    let name: Identifier = itemName(stack.id)?;
    CONTAINERS.with(|containers|
    {
        containers
            .borrow()
            .iter()
            .find(|container| holds(container, &name, stack.damage))
            .map(|container| FluidStack { fluid: container.fluid.clone(), amount: container.amount })
    })
}

// whether the stack is an empty container some fluid fills
pub fn empty(stack: &StackData) -> bool
{
    let Some(name) = itemName(stack.id)
    else
    {
        return false;
    };

    CONTAINERS.with(|containers| containers.borrow().iter().any(|container| container.empty.as_ref() == Some(&name)))
}

pub fn fill(stack: &StackData, fluid: &Identifier) -> Option<ContainerSwap>
{
    let name: Identifier = itemName(stack.id)?;
    let container: FluidContainerSpec = CONTAINERS.with(|containers|
    {
        containers
            .borrow()
            .iter()
            .find(|container| container.empty.as_ref() == Some(&name) && container.fluid == *fluid)
            .cloned()
    })?;
    let full: StackData = StackData { id: itemNumeric(&container.full)?, count: 1, damage: container.full_meta.max(0) };
    Some(ContainerSwap { stack: Some(full), fluid: FluidStack { fluid: container.fluid, amount: container.amount } })
}

pub fn drain(stack: &StackData) -> Option<ContainerSwap>
{
    let name: Identifier = itemName(stack.id)?;
    let container: FluidContainerSpec = CONTAINERS.with(|containers|
    {
        containers.borrow().iter().find(|container| holds(container, &name, stack.damage)).cloned()
    })?;
    let left: Option<StackData> = match &container.empty
    {
        Some(empty) => Some(StackData { id: itemNumeric(empty)?, count: 1, damage: 0 }),
        None => None,
    };
    Some(ContainerSwap { stack: left, fluid: FluidStack { fluid: container.fluid, amount: container.amount } })
}

// the fluid row exists from registration on, so blocks that name it as their material find it when they are built
pub fn register(id: Identifier, spec: FluidSpec) -> Result<(), String>
{
    crate::registry::claim(mod_protocol::abi::RegistryKind::Fluid, &id)?;

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
        m_sprite: Cell::new(fluid::sprite(&Material::water())),
        m_flowSprite: Cell::new(fluid::sprite(&Material::water()) + 1),
        m_fog: Cell::new(None),
    });

    // a fluid's own bucket drains to a vanilla bucket like water's
    if let Some(bucket) = &spec.bucket
    {
        let container: FluidContainerSpec = FluidContainerSpec
        {
            full: bucket.clone(),
            full_meta: 0,
            empty: Some(Identifier::minecraft("bucket")),
            fluid: id.clone(),
            amount: BUCKET,
        };
        addContainer(container)?;
    }

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

        // the still and flowing blocks' own textures, animated when their sprites carry frames
        if let (Some(stillBlock), Some(flowingBlock)) =
            (game::block::Block::byId(still), game::block::Block::byId(flowing))
        {
            row.m_sprite.set(stillBlock.m_blockIndexInTexture.get());
            row.m_flowSprite.set(flowingBlock.m_blockIndexInTexture.get());
        }

        // the still block's colour tints the fog under the fluid
        let tint: Option<u32> = crate::registry::with(|registries|
        {
            registries.m_blocks.get(&spec.still).and_then(|row| row.m_value.m_spec.as_ref()).and_then(|spec| spec.color)
        });
        row.m_fog.set(tint);
    }
}
