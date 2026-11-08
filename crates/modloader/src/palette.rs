use crate::{registry, states};
use mod_protocol::Identifier;
use mod_protocol::abi::StateData;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

type Properties = Vec<(String, String)>;

const AIR: &str = "minecraft:air";

thread_local! {
    // names this world saved that no loaded mod knows, kept for the id and metadata they loaded as so a save writes them back
    static UNKNOWN: RefCell<HashMap<(i32, i32), (String, Properties)>> = RefCell::new(HashMap::new());
}

pub fn clear() { UNKNOWN.with(|unknown| unknown.borrow_mut().clear()); }

pub fn name(block: i32, meta: i32) -> Option<(String, Properties)>
{
    if block == 0
    {
        return Some((AIR.to_owned(), Vec::new()));
    }
    if let Some((id, index)) = states::stateAt(block, meta)
    {
        let properties: Properties = states::table(&id)?.properties(index).into_iter().collect();
        return Some((id.to_string(), properties));
    }
    if let Some(state) = states::describe(block, meta)
    {
        // a block without named states, or metadata past them, keeps its metadata as a property
        let properties: Properties = state.properties.into_iter().collect();
        return Some((state.block.to_string(), properties));
    }
    // only ids no loaded block owns reach the kept names, a placeholder never shadows real content
    if let Some(kept) = UNKNOWN.with(|unknown| unknown.borrow().get(&(block, meta)).cloned())
    {
        return Some(kept);
    }
    crate::remap::missingName("block", block).map(|id| (id.to_string(), Vec::new()))
}

pub fn resolve(name: &str, properties: &[(String, String)], block: i32, meta: i32) -> Option<(i32, i32)>
{
    if name == AIR
    {
        return Some((0, 0));
    }
    let known: Option<(i32, i32)> = Identifier::parse(name).ok().and_then(|id| {
        let values: BTreeMap<String, String> = properties.iter().cloned().collect();
        if states::table(&id).is_some()
        {
            return states::resolve(&StateData { block: id, properties: values });
        }
        registry::with(|registries| registries.blockId(&id)).map(|numeric| (numeric, meta))
    });
    if known.is_none()
    {
        UNKNOWN.with(|unknown| unknown.borrow_mut().insert((block, meta), (name.to_owned(), properties.to_vec())));
    }
    known
}
