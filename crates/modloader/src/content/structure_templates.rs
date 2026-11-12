use game::block::Block;
use game::world::World;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, StackData, StateData};
use mod_protocol::spec::StructureTemplateSpec;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

// a quarter turn count wraps at four
const TURNS: i32 = 4;
// characters a template leaves as the world already is
const KEEP: char = ' ';

thread_local!
{
    static TEMPLATES: RefCell<BTreeMap<Identifier, StructureTemplateSpec>> = const { RefCell::new(BTreeMap::new()) };
}

fn key(text: &str) -> Result<char, String>
{
    let mut chars = text.chars();
    match (chars.next(), chars.next())
    {
        (Some(single), None) if single != KEEP => Ok(single),
        _ => Err(format!("template palette key '{text}' must be one character other than a space")),
    }
}

pub fn register(id: Identifier, spec: StructureTemplateSpec) -> Result<(), String>
{
    for text in spec.palette.keys().chain(spec.chests.keys())
    {
        key(text)?;
    }

    let width: Option<usize> = spec.layers.first().and_then(|layer| layer.first()).map(|row| row.chars().count());
    let even: bool = spec
        .layers
        .iter()
        .all(|layer| layer.len() == spec.layers[0].len() && layer.iter().all(|row| Some(row.chars().count()) == width));
    if !even
    {
        return Err(format!("template {id} needs every layer and row the same size"));
    }

    TEMPLATES.with(|templates|
    {
        let mut templates = templates.borrow_mut();
        if templates.contains_key(&id)
        {
            return Err(format!("template {id} already exists"));
        }

        templates.insert(id, spec);
        Ok(())
    })
}

// x and z of a template cell after rotation quarter turns clockwise about the origin
fn turn(x: i32, z: i32, rotation: i32) -> (i32, i32)
{
    match rotation.rem_euclid(TURNS)
    {
        1 => (-z, x),
        2 => (-x, -z),
        3 => (z, -x),
        _ => (x, z),
    }
}

// the placed block of each palette character, unknown blocks named in the problem
fn palette(spec: &StructureTemplateSpec) -> Result<BTreeMap<char, (i32, i32)>, String>
{
    spec.palette
        .iter()
        .map(|(text, block)|
        {
            let state: StateData = StateData { block: block.block.clone(), properties: block.properties.clone() };
            let placed: (i32, i32) = crate::states::resolve(&state)
                .ok_or_else(|| format!("template block {} is not registered", block.block))?;
            Ok((key(text)?, placed))
        })
        .collect()
}

pub fn place(world: &Rc<World>, id: &Identifier, origin: BlockPos, rotation: i32, seed: i64) -> Result<bool, String>
{
    let spec: StructureTemplateSpec =
        TEMPLATES.with(|templates| templates.borrow().get(id).cloned()).ok_or_else(|| format!("no template {id}"))?;
    let blocks: BTreeMap<char, (i32, i32)> = palette(&spec)?;
    let chests: BTreeMap<char, Identifier> =
        spec.chests.iter().map(|(text, table)| Ok((key(text)?, table.clone()))).collect::<Result<_, String>>()?;
    let mut placed: bool = false;
    let mut filled: i64 = 0;

    for (y, layer) in spec.layers.iter().enumerate()
    {
        for (z, row) in layer.iter().enumerate()
        {
            for (x, cell) in row.chars().enumerate()
            {
                let (dx, dz): (i32, i32) = turn(x as i32, z as i32, rotation);
                let at: [i32; 3] = [origin.x + dx, origin.y + y as i32, origin.z + dz];

                if let Some((block, meta)) = blocks.get(&cell)
                {
                    placed |= world.setBlockAndMetadataWithNotify(at[0], at[1], at[2], *block, *meta);
                }
                else if let Some(table) = chests.get(&cell)
                {
                    placed |= world.setBlockWithNotify(at[0], at[1], at[2], Block::CHEST);
                    // each chest of one placement rolls its own loot from the placement seed
                    fill(world, at, table, seed.wrapping_add(filled))?;
                    filled += 1;
                }
            }
        }
    }

    Ok(placed)
}

fn fill(world: &Rc<World>, at: [i32; 3], table: &Identifier, seed: i64) -> Result<(), String>
{
    let stacks: Vec<StackData> = super::loot::rollChest(table, seed)?;
    let Some(inventory) = game::transfer::inventoryAt(world, at[0], at[1], at[2])
    else
    {
        return Err(format!("no chest at {at:?} for loot {table}"));
    };

    for (slot, stack) in stacks.iter().enumerate().take(inventory.getSizeInventory() as usize)
    {
        inventory.setInventorySlotContents(slot as i32, crate::host::context::stackFrom(stack));
    }

    Ok(())
}
