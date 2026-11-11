use super::displayName;
use crate::content::machines::{self, Machine};
use crate::content::tags;
use mod_protocol::Identifier;
use mod_protocol::abi::{MachineRecipeView, MachineSlot, MachineView, RegistryKind, StackData};
use mod_protocol::spec::
{
    JobFluidSpec, JobInputSpec, JobOutputSpec, MachineJobSpec, MachineRecipeSpec, MachineSlotSpec,
};

// every item a slot names, a tag through its members, warned about once per list when nothing resolves
fn slot(spec: &MachineSlotSpec, problems: &mut Vec<String>) -> MachineSlot
{
    let names: Vec<Identifier> = match (&spec.item, &spec.tag)
    {
        (Some(item), _) => vec![item.clone()],
        (None, Some(tag)) => tags::values(RegistryKind::Item, tag),
        (None, None) => Vec::new(),
    };

    let stacks: Vec<StackData> = crate::registry::with(|registries|
    {
        names
            .iter()
            .filter_map(|name| registries.itemId(name))
            .map(|id| StackData { id, count: spec.count, damage: spec.meta })
            .collect()
    });

    if stacks.is_empty()
    {
        problems.push(format!(
            "a machine recipe slot {:?} names no registered item",
            spec.item.as_ref().or(spec.tag.as_ref())
        ));
    }

    MachineSlot { stacks, chance: spec.chance }
}

fn slots(specs: &[MachineSlotSpec], problems: &mut Vec<String>) -> Vec<MachineSlot>
{
    specs.iter().map(|spec| slot(spec, problems)).collect()
}

fn recipe(spec: &MachineRecipeSpec, problems: &mut Vec<String>) -> MachineRecipeView
{
    MachineRecipeView
    {
        inputs: slots(&spec.inputs, problems),
        outputs: slots(&spec.outputs, problems),
        notes: spec.notes.clone(),
    }
}

fn inputSlot(input: &JobInputSpec) -> MachineSlotSpec
{
    MachineSlotSpec
    {
        item: input.item.clone(),
        tag: input.tag.clone(),
        meta: input.meta,
        count: input.count,
        chance: 1.0,
    }
}

// an output handing back an input shows that input's item at the damage it leaves with
fn outputSlot(job: &MachineJobSpec, output: &JobOutputSpec) -> MachineSlotSpec
{
    let source: Option<&JobInputSpec> = output.from_input.and_then(|index| job.inputs.get(index as usize));
    match source
    {
        Some(input) => MachineSlotSpec
        {
            meta: output.damage.unwrap_or(input.meta),
            count: output.count,
            chance: output.chance,
            ..inputSlot(input)
        },
        None => MachineSlotSpec
        {
            item: output.item.clone(),
            tag: None,
            meta: output.meta,
            count: output.count,
            chance: output.chance,
        },
    }
}

fn fluidName(fluid: &JobFluidSpec) -> String
{
    match (&fluid.fluid, &fluid.tag)
    {
        (Some(name), _) => name.to_string(),
        (None, Some(tag)) => format!("#{tag}"),
        (None, None) => String::new(),
    }
}

fn jobNotes(job: &MachineJobSpec) -> Vec<String>
{
    let mut notes: Vec<String> = vec![if job.duration_from_fuel
    {
        "Runs as long as its fuel burns".to_owned()
    }
    else
    {
        format!("{} ticks", job.duration)
    }];

    for (verb, energy) in [("Uses", &job.energy_in), ("Makes", &job.energy_out)]
    {
        if let Some(energy) = energy
        {
            let each: &str = if energy.per_tick { " a tick" } else { "" };
            notes.push(format!("{verb} {} {}{each}", energy.amount, energy.kind));
        }
    }

    notes.extend(job.fluid_inputs.iter().map(|fluid| format!("Takes {} mB of {}", fluid.amount, fluidName(fluid))));
    notes.extend(job.fluid_outputs.iter().map(|fluid| format!("Gives {} mB of {}", fluid.amount, fluidName(fluid))));
    notes.extend(job.notes.iter().cloned());
    notes
}

// a job as the recipe page viewers already read
fn job(ruled: &crate::content::machines::Job, problems: &mut Vec<String>) -> MachineRecipeView
{
    let job: &MachineJobSpec = &ruled.m_spec;
    let inputs: Vec<MachineSlotSpec> = job.inputs.iter().map(inputSlot).collect();
    let outputs: Vec<MachineSlotSpec> = job.outputs.iter().map(|output| outputSlot(job, output)).collect();
    let mut notes: Vec<String> = jobNotes(job);

    // an input taking any item has no slot to show, so its rule becomes a line
    for rule in &ruled.m_rules
    {
        notes.push(match &rule.except
        {
            Some(except) => format!("Takes any item except #{except}"),
            None => "Takes any item".to_owned(),
        });
    }

    MachineRecipeView { inputs: slots(&inputs, problems), outputs: slots(&outputs, problems), notes }
}

fn view(machine: &Machine, problems: &mut Vec<String>) -> MachineView
{
    let workstations: Vec<StackData> = crate::registry::with(|registries|
    {
        machine
            .m_spec
            .workstations
            .iter()
            .filter_map(|item| registries.itemId(item))
            .map(|id| StackData { id, count: 1, damage: 0 })
            .collect()
    });

    let title: String = machine.m_spec.title.clone().unwrap_or_else(||
    {
        workstations
            .first()
            .and_then(crate::host::context::stackFrom)
            .map_or_else(|| machine.m_id.path().to_owned(), |stack| displayName(&stack))
    });

    let mut recipes: Vec<MachineRecipeView> = machine.m_recipes.iter().map(|spec| recipe(spec, problems)).collect();
    recipes.extend(machine.m_jobs.iter().map(|ruled| job(ruled, problems)));

    MachineView { id: machine.m_id.clone(), title, workstations, recipes }
}

pub fn list() -> Vec<MachineView>
{
    let mut problems: Vec<String> = Vec::new();
    let views: Vec<MachineView> =
        machines::read(|machines| machines.iter().map(|machine| view(machine, &mut problems)).collect());

    for problem in problems
    {
        crate::log::warn(crate::log::LOADER, &problem);
    }

    views
}
