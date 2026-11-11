use crate::content::tags;
use crate::registry::Registries;
use mod_protocol::Identifier;
use mod_protocol::abi::{FluidStack, JobMatch, JobOutput, RegistryKind, StackData};
use mod_protocol::spec::
{
    JobFluidSpec, JobInputSpec, JobOutputSpec, MachineFuelSpec, MachineJobSpec, MachineRecipeSpec, MachineSpec,
};
use std::cell::{Ref, RefCell};

// vanilla categories mods may only add workstations to
const VANILLA: [&str; 2] = ["crafting", "smelting"];

pub struct Machine
{
    pub m_id: Identifier,
    pub m_spec: MachineSpec,
    pub m_recipes: Vec<MachineRecipeSpec>,
    pub m_jobs: Vec<MachineJobSpec>,
    pub m_fuels: Vec<MachineFuelSpec>,
}

impl Machine
{
    fn new(id: Identifier, spec: MachineSpec) -> Self
    {
        Self { m_id: id, m_spec: spec, m_recipes: Vec::new(), m_jobs: Vec::new(), m_fuels: Vec::new() }
    }
}

thread_local!
{
    // in registration order, which recipe viewers keep for their tabs
    static MACHINES: RefCell<Vec<Machine>> = const { RefCell::new(Vec::new()) };
}

fn vanilla(id: &Identifier) -> bool { id.isVanilla() && VANILLA.contains(&id.path()) }

pub fn register(id: Identifier, spec: MachineSpec) -> Result<(), String>
{
    MACHINES.with(|machines|
    {
        let mut machines = machines.borrow_mut();
        if vanilla(&id) || machines.iter().any(|machine| machine.m_id == id)
        {
            return Err(format!("machine {id} is already registered"));
        }

        machines.push(Machine::new(id, spec));
        Ok(())
    })
}

// a vanilla category gets its row the first time a mod adds a workstation to it
fn with(id: &Identifier, change: impl FnOnce(&mut Machine)) -> Result<(), String>
{
    MACHINES.with(|machines|
    {
        let mut machines = machines.borrow_mut();
        if !machines.iter().any(|machine| machine.m_id == *id) && vanilla(id)
        {
            machines.push(Machine::new(id.clone(), MachineSpec::default()));
        }

        let machine: &mut Machine = machines
            .iter_mut()
            .find(|machine| machine.m_id == *id)
            .ok_or_else(|| format!("machine {id} is not registered"))?;
        change(machine);
        Ok(())
    })
}

pub fn addRecipe(id: &Identifier, recipe: MachineRecipeSpec) -> Result<(), String>
{
    if vanilla(id)
    {
        return Err(format!("{id} recipes come from the recipe registry"));
    }

    with(id, |machine| machine.m_recipes.push(recipe))
}

pub fn addJob(id: &Identifier, job: MachineJobSpec) -> Result<(), String>
{
    if vanilla(id)
    {
        return Err(format!("{id} runs the recipe registry, not jobs"));
    }

    with(id, |machine| machine.m_jobs.push(job))
}

pub fn addFuel(id: &Identifier, fuel: MachineFuelSpec) -> Result<(), String>
{
    if vanilla(id)
    {
        return Err(format!("{id} burns beta's fuels"));
    }

    with(id, |machine| machine.m_fuels.push(fuel))
}

// an item or tag with meta -1 for any damage, tested against a stack this session's ids name
fn takes(
    registries: &Registries,
    item: Option<&Identifier>,
    tag: Option<&Identifier>,
    meta: i32,
    stack: &StackData,
) -> bool
{
    let damage: bool = meta < 0 || meta == stack.damage;
    let named: bool = match (item, tag)
    {
        (Some(item), _) => registries.itemId(item) == Some(stack.id),
        (None, Some(tag)) =>
        {
            registries.m_items.byNumeric(stack.id).is_some_and(|row| tags::contains(RegistryKind::Item, tag, &row.m_id))
        }
        (None, None) => false,
    };
    damage && named
}

fn fits(registries: &Registries, input: &JobInputSpec, stack: &StackData) -> bool
{
    stack.count >= input.count && takes(registries, input.item.as_ref(), input.tag.as_ref(), input.meta, stack)
}

// each job input takes the first stack in order no earlier input took
fn assign(registries: &Registries, job: &MachineJobSpec, stacks: &[Option<StackData>]) -> Option<Vec<u32>>
{
    let mut used: Vec<bool> = vec![false; stacks.len()];
    let mut slots: Vec<u32> = Vec::with_capacity(job.inputs.len());

    for input in &job.inputs
    {
        let index: usize = stacks.iter().enumerate().position(|(index, stack)|
        {
            !used[index] && stack.as_ref().is_some_and(|stack| fits(registries, input, stack))
        })?;
        used[index] = true;
        slots.push(index as u32);
    }

    Some(slots)
}

fn pours(need: &JobFluidSpec, have: &FluidStack) -> bool
{
    let named: bool = match (&need.fluid, &need.tag)
    {
        (Some(fluid), _) => *fluid == have.fluid,
        (None, Some(tag)) => tags::contains(RegistryKind::Fluid, tag, &have.fluid),
        (None, None) => false,
    };
    named && have.amount >= need.amount
}

// each fluid input names the first given fluid that covers it
fn covered(job: &MachineJobSpec, fluids: &[FluidStack]) -> Option<Vec<FluidStack>>
{
    job.fluid_inputs
        .iter()
        .map(|need|
        {
            let have: &FluidStack = fluids.iter().find(|have| pours(need, have))?;
            Some(FluidStack { fluid: have.fluid.clone(), amount: need.amount })
        })
        .collect()
}

fn output(registries: &Registries, spec: &JobOutputSpec) -> JobOutput
{
    let stack: Option<StackData> = spec.item.as_ref().and_then(|item| registries.itemId(item)).map(|id| StackData
    {
        id,
        count: spec.count,
        damage: spec.meta,
    });
    JobOutput { stack, count: spec.count, chance: spec.chance, from_input: spec.from_input, damage: spec.damage }
}

// an output names its fluid, a tag there gives nothing
fn fluidStacks(specs: &[JobFluidSpec]) -> Vec<FluidStack>
{
    specs.iter().filter_map(|spec| Some(FluidStack { fluid: spec.fluid.clone()?, amount: spec.amount })).collect()
}

// the first job in load order the stacks and fluids can run
pub fn matchJob(id: &Identifier, stacks: &[Option<StackData>], fluids: &[FluidStack]) -> Option<JobMatch>
{
    read(|machines|
    {
        let machine: &Machine = machines.iter().find(|machine| machine.m_id == *id)?;
        crate::registry::with(|registries|
        {
            machine.m_jobs.iter().enumerate().find_map(|(index, job)|
            {
                let slots: Vec<u32> = assign(registries, job, stacks)?;
                let poured: Vec<FluidStack> = covered(job, fluids)?;
                let duration: i32 = if job.duration_from_fuel
                {
                    let first: &StackData = slots.first().and_then(|slot| stacks[*slot as usize].as_ref())?;
                    fuelValue(registries, machine, first, true)
                }
                else
                {
                    job.duration
                };

                Some(JobMatch
                {
                    index: index as u32,
                    slots,
                    inputs: job.inputs.clone(),
                    outputs: job.outputs.iter().map(|spec| output(registries, spec)).collect(),
                    fluid_inputs: poured,
                    fluid_outputs: fluidStacks(&job.fluid_outputs),
                    duration,
                    energy_in: job.energy_in.clone(),
                    energy_out: job.energy_out.clone(),
                })
            })
        })
    })
}

// the machine's own entry for the stack, else with furnace its furnace burn time
fn fuelValue(registries: &Registries, machine: &Machine, stack: &StackData, furnace: bool) -> i32
{
    let own: Option<i32> = machine
        .m_fuels
        .iter()
        .find(|fuel| takes(registries, fuel.item.as_ref(), fuel.tag.as_ref(), fuel.meta, stack))
        .map(|fuel| fuel.value);

    match own
    {
        Some(value) => value,
        None if furnace =>
        {
            game::tileentity::tile_entity_furnace::getItemBurnTime(crate::host::context::stackFrom(stack))
        }
        None => 0,
    }
}

pub fn fuel(id: &Identifier, stack: &StackData, furnace: bool) -> i32
{
    read(|machines|
    {
        let Some(machine) = machines.iter().find(|machine| machine.m_id == *id)
        else
        {
            return 0;
        };

        crate::registry::with(|registries| fuelValue(registries, machine, stack, furnace))
    })
}

pub fn addWorkstation(id: &Identifier, item: Identifier) -> Result<(), String>
{
    with(id, |machine| machine.m_spec.workstations.push(item))
}

pub fn read<T>(reader: impl FnOnce(&[Machine]) -> T) -> T
{
    MACHINES.with(|machines|
    {
        let machines: Ref<Vec<Machine>> = machines.borrow();
        reader(&machines)
    })
}
