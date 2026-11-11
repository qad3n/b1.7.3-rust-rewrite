use mod_protocol::Identifier;
use mod_protocol::spec::{MachineRecipeSpec, MachineSpec};
use std::cell::{Ref, RefCell};

// vanilla categories mods may only add workstations to
const VANILLA: [&str; 2] = ["crafting", "smelting"];

pub struct Machine
{
    pub m_id: Identifier,
    pub m_spec: MachineSpec,
    pub m_recipes: Vec<MachineRecipeSpec>,
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

        machines.push(Machine { m_id: id, m_spec: spec, m_recipes: Vec::new() });
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
            machines.push(Machine { m_id: id.clone(), m_spec: MachineSpec::default(), m_recipes: Vec::new() });
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
