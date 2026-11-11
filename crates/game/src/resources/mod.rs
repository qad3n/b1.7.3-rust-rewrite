mod animation;
mod export;
mod legacy;
mod location;
mod pack;
mod sheet;
mod source;
mod sprites;
mod stack;

pub use animation::{Frame, animate, painted};
pub use export::export;
use jvm::image::BufferedImage;
pub use location::{Location, MINECRAFT};
pub use pack::{Layout, Pack};
pub use source::Source;
pub(crate) use stack::Stack;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const VANILLA_NAME: &str = "Default";
const VANILLA_DESCRIPTION: &str = "The default look of Minecraft";

thread_local!
{
    static VANILLA: Rc<Pack> = Rc::new(Pack::new(
        VANILLA_NAME,
        &[VANILLA_DESCRIPTION],
        Layout::Jar,
        Source::Folder(crate::port::assetRoot()),
    ));
    static MODS: RefCell<Vec<Rc<Pack>>> = const { RefCell::new(Vec::new()) };
    static USER: RefCell<Vec<Rc<Pack>>> = const { RefCell::new(Vec::new()) };
    static STACK: RefCell<Stack> = RefCell::new(Stack::new(vec![vanilla()], 1));
}

// the user's extracted b1.7.3 jar read in place
pub fn vanilla() -> Rc<Pack> { VANILLA.with(Rc::clone) }

fn rebuild()
{
    let mods: Vec<Rc<Pack>> = MODS.with(|mods| mods.borrow().clone());
    let user: Vec<Rc<Pack>> = USER.with(|user| user.borrow().clone());
    let first: usize = 1 + mods.len();
    let stack: Vec<Rc<Pack>> = std::iter::once(vanilla()).chain(mods).chain(user).collect();
    STACK.with(|current| *current.borrow_mut() = Stack::new(stack, first));
}

// one pack per active mod in load order between the jar and the texture packs
pub fn setMods(packs: Vec<Rc<Pack>>)
{
    MODS.with(|mods| *mods.borrow_mut() = packs);
    rebuild();
}

// texture packs the player picked lowest first
pub fn install(packs: &[Rc<Pack>])
{
    USER.with(|user| *user.borrow_mut() = packs.to_vec());
    rebuild();
}

pub fn read(location: &Location) -> Option<Vec<u8>> { STACK.with(|stack| stack.borrow().read(location)) }

// a jar path such as /lang/en_US.lang
pub fn readBeta(path: &str) -> Option<Vec<u8>> { read(&legacy::location(path)) }

// beta read some files from the jar even with a texture pack selected
pub fn readJar(path: &str) -> Option<Vec<u8>> { vanilla().read(&legacy::location(path)) }

// a beta sheet such as /terrain.png with sprites from modern packs above the pack that supplied it
pub fn overlay(path: &str, image: Rc<BufferedImage>) -> Rc<BufferedImage>
{
    STACK.with(|stack| sheet::overlay(&stack.borrow(), path, image))
}

// copies of a merged file such as sounds.json lowest first
pub fn readAll(location: &Location) -> Vec<Vec<u8>>
{
    readEach(std::slice::from_ref(location)).into_iter().map(|(_, bytes)| bytes).collect()
}

// copies of several merged files lowest pack first with the index of the location each one is for
pub fn readEach(locations: &[Location]) -> Vec<(usize, Vec<u8>)>
{
    STACK.with(|stack| stack.borrow().readEach(locations, false))
}

// the same copies from the texture packs the player picked only
pub fn readUser(locations: &[Location]) -> Vec<(usize, Vec<u8>)>
{
    STACK.with(|stack| stack.borrow().readEach(locations, true))
}

// the stack index of the pack serving a location
pub fn owner(location: &Location) -> Option<usize> { STACK.with(|stack| stack.borrow().owner(location)) }

// minecraft and every mod namespace
pub fn namespaces() -> Vec<String> { STACK.with(|stack| stack.borrow().namespaces()) }

// where a modern pack keeps a beta jar file such as /lang/en_US.lang
pub fn betaLocation(path: &str) -> Location { legacy::location(path) }

// the pack holding a location above the floor pack and the file on disk for it
pub fn disk(location: &Location, floor: Option<&str>) -> Option<(String, PathBuf)>
{
    STACK.with(|stack| stack.borrow().disk(location, floor))
}
