use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::FluidContainerSpec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// a peer listing this channel takes the content message while it joins
pub const CONTENT: &str = "loader:content";

// what a server's content is beyond numeric ids: names to check against and stores the client mirrors
#[derive(Serialize, Deserialize)]
pub struct Served
{
    pub names: BTreeMap<String, Vec<Identifier>>,
    pub registries: BTreeMap<Identifier, Vec<Identifier>>,
    pub custom: Vec<(String, Identifier, String)>,
    pub loot: super::loot::LootWire,
    pub machines: super::machines::MachineWire,
    pub containers: Vec<FluidContainerSpec>,
    pub modify: crate::modify::ModifyWire,
}

// sounds and vanilla particles only exist on the client, so they never take part in the check
fn checked(kind: RegistryKind) -> bool
{
    !kind.numbered() && !matches!(kind, RegistryKind::Particle | RegistryKind::Sound)
}

// content of client-only and server-only mods exists on one side by design
fn shared(id: &Identifier) -> bool
{
    crate::state::byId(id.namespace())
        .is_none_or(|runtime| runtime.m_manifest.side.wire() == mod_protocol::handshake::ModSide::Both)
}

fn ownNames() -> BTreeMap<String, Vec<Identifier>>
{
    crate::registry::with(|registries|
    {
        RegistryKind::ROWS
            .iter()
            .filter(|row| checked(row.kind))
            .map(|row|
            {
                let ids: Vec<Identifier> =
                    registries.list(row.kind).into_iter().filter(|id| !id.isVanilla() && shared(id)).collect();
                (row.singular.to_owned(), ids)
            })
            .collect()
    })
}

fn sharedRegistries() -> BTreeMap<Identifier, Vec<Identifier>>
{
    super::custom::names().into_iter().filter(|(registry, _)| shared(registry)).collect()
}

pub fn build() -> Served
{
    Served
    {
        names: ownNames(),
        registries: sharedRegistries(),
        custom: super::custom::synced(),
        loot: super::loot::snapshot(),
        machines: super::machines::snapshot(),
        containers: super::fluids::containers(),
        modify: crate::modify::snapshot(),
    }
}

fn differences(label: &str, server: &[Identifier], own: &[Identifier], problems: &mut Vec<String>)
{
    problems.extend(server.iter().filter(|id| !own.contains(id)).map(|id| format!("{label} {id} missing here")));
    problems.extend(own.iter().filter(|id| !server.contains(id)).map(|id| format!("{label} {id} not on the server")));
}

// every name the server and this client disagree on, a client that differs cannot follow the server's content
pub fn mismatches(server: &Served) -> Vec<String>
{
    let mut problems: Vec<String> = Vec::new();
    let own: BTreeMap<String, Vec<Identifier>> = ownNames();

    for (kind, ids) in &server.names
    {
        differences(kind, ids, own.get(kind).map_or(&[], Vec::as_slice), &mut problems);
    }

    let registries: BTreeMap<Identifier, Vec<Identifier>> = sharedRegistries();
    for (registry, ids) in &server.registries
    {
        differences(
            &format!("{registry} entry"),
            ids,
            registries.get(registry).map_or(&[], Vec::as_slice),
            &mut problems,
        );
    }

    problems
}

// the server's stores stand in for the client's own until it disconnects
pub fn apply(server: Served)
{
    super::custom::replaceSynced(server.custom);
    super::loot::replace(server.loot);
    super::machines::replace(server.machines);
    super::fluids::replaceContainers(server.containers);
    crate::modify::replace(server.modify);
}

pub fn restore()
{
    super::custom::restoreSynced();
    super::loot::restore();
    super::machines::restore();
    super::fluids::restoreContainers();
    crate::modify::restore();
}
