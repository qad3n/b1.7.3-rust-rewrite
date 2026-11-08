// interop matrix from design 7.3 Phase 3, decided by the pure handshake checks both sides run
use mod_protocol::handshake::{Hello, ModEntry, ModSide};
use modloader::netsync::{clientCheck, serverCheck};

fn entry(id: &str, side: ModSide, network: &str, required: bool) -> ModEntry
{
    ModEntry {
        id: id.to_owned(),
        version: "1.0.0".to_owned(),
        side,
        network_version: network.to_owned(),
        required,
        sha256: String::new(),
    }
}

fn hello(mods: Vec<ModEntry>) -> Hello { Hello { loader_version: "0.1.0".to_owned(), mods, channels: Vec::new() } }

#[test]
fn noModsOnEitherSide()
{
    assert!(serverCheck(&hello(vec![]), &hello(vec![])).is_empty());
    assert!(clientCheck(&hello(vec![]), &hello(vec![])).is_empty());
}

#[test]
fn clientOnlyModsAreNeverChecked()
{
    let client: Hello = hello(vec![entry("minimap", ModSide::Client, "1", true)]);
    assert!(serverCheck(&hello(vec![]), &client).is_empty());
    assert!(clientCheck(&hello(vec![]), &client).is_empty());
}

#[test]
fn serverOnlyModsAreNotRequiredOfClients()
{
    let server: Hello = hello(vec![entry("backup", ModSide::Server, "1", true)]);
    assert!(serverCheck(&server, &hello(vec![])).is_empty());
}

#[test]
fn requiredModMissingOnClientIsNamed()
{
    let server: Hello = hello(vec![entry("ruby", ModSide::Both, "1", true), entry("energy", ModSide::Both, "2", true)]);
    let problems: Vec<String> = serverCheck(&server, &hello(vec![entry("ruby", ModSide::Both, "1", true)]));
    assert_eq!(problems, vec!["missing energy 1.0.0".to_owned()]);
}

#[test]
fn networkVersionMismatchIsNamed()
{
    let server: Hello = hello(vec![entry("ruby", ModSide::Both, "2", true)]);
    let problems: Vec<String> = serverCheck(&server, &hello(vec![entry("ruby", ModSide::Both, "1", true)]));
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("network version 1"));
}

#[test]
fn optionalModsMayDiffer()
{
    let server: Hello = hello(vec![entry("tweaks", ModSide::Both, "1", false)]);
    assert!(serverCheck(&server, &hello(vec![])).is_empty());
}

#[test]
fn clientRequiredModTheServerLacksIsRefusedByTheClient()
{
    let client: Hello = hello(vec![entry("ruby", ModSide::Both, "1", true)]);
    assert_eq!(clientCheck(&hello(vec![]), &client).len(), 1);
}

#[test]
fn framesSurviveCompression()
{
    let data: Vec<u8> = vec![9; 5000];
    let (channel, body) = mod_protocol::wire::compress(3, &data);
    let frame =
        mod_protocol::wire::decompress(channel, &body, mod_protocol::wire::limits::CLIENTBOUND_MAX).expect("inflates");
    assert_eq!(frame.m_channel, 3);
    assert_eq!(frame.m_data, data);
}
