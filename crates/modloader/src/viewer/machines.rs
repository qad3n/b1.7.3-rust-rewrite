use super::displayName;
use crate::content::machines::{self, Machine};
use crate::content::tags;
use mod_protocol::Identifier;
use mod_protocol::abi::{MachineRecipeView, MachineSlot, MachineView, RegistryKind, StackData};
use mod_protocol::spec::{MachineRecipeSpec, MachineSlotSpec};

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

    MachineView
    {
        id: machine.m_id.clone(),
        title,
        workstations,
        recipes: machine.m_recipes.iter().map(|spec| recipe(spec, problems)).collect(),
    }
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
