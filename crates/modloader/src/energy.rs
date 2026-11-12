use mod_protocol::Identifier;
use mod_protocol::spec::EnergySpec;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

thread_local!
{
    // every energy kind and its worth in one shared base unit, so a generator of one mod powers a machine of another
    static KINDS: RefCell<BTreeMap<Identifier, i64>> = const { RefCell::new(BTreeMap::new()) };
    // kind pairs already reported as unable to convert, each logged once
    static REPORTED: RefCell<BTreeSet<(Identifier, Identifier)>> = const { RefCell::new(BTreeSet::new()) };
}

pub fn register(owner: &str, id: Identifier, spec: EnergySpec) -> Result<(), String>
{
    if spec.worth < 0
    {
        return Err(format!("energy kind {id} has a negative worth {}", spec.worth));
    }

    if spec.worth == 0
    {
        crate::log::warn(
            owner,
            &format!("energy kind {id} has no worth, it only moves between blocks of its own kind"),
        );
    }

    crate::registry::claim(mod_protocol::abi::RegistryKind::Energy, &id)?;
    KINDS.with(|kinds| kinds.borrow_mut().insert(id, spec.worth));
    Ok(())
}

fn worth(id: &Identifier) -> Option<i64>
{
    KINDS.with(|kinds| kinds.borrow().get(id).copied().filter(|worth| *worth > 0))
}

// both worths in base units, None when the kinds differ and either has none
fn rates(from: &Identifier, to: &Identifier) -> Option<(i64, i64)>
{
    if from == to
    {
        return Some((1, 1));
    }

    let rates: Option<(i64, i64)> = worth(from).zip(worth(to));
    if rates.is_none()
    {
        report(from, to);
    }

    rates
}

fn report(from: &Identifier, to: &Identifier)
{
    let fresh: bool = REPORTED.with(|reported| reported.borrow_mut().insert((from.clone(), to.clone())));
    if !fresh
    {
        return;
    }

    let known: &str = match (worth(from).is_some(), worth(to).is_some())
    {
        (false, false) => "neither kind is registered with a worth",
        (false, true) => "the sending kind is not registered with a worth",
        _ => "the receiving kind is not registered with a worth",
    };
    crate::log::warn("loader", &format!("energy {from} cannot power a block taking {to}: {known}"));
}

// an amount of one kind in another, rounded down so conversion never makes energy
pub fn convert(amount: i64, from: &Identifier, to: &Identifier) -> Option<i64>
{
    let (fromWorth, toWorth): (i64, i64) = rates(from, to)?;
    Some(amount.saturating_mul(fromWorth) / toWorth)
}

// what an amount taken in the receiver's kind cost the sender, rounded up for the same reason
pub fn cost(taken: i64, from: &Identifier, to: &Identifier) -> Option<i64>
{
    let (fromWorth, toWorth): (i64, i64) = rates(from, to)?;
    let base: i64 = taken.saturating_mul(toWorth);
    Some((base + fromWorth - 1) / fromWorth)
}
