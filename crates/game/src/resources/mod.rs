mod legacy;
mod location;
mod pack;
mod sheet;
mod source;
mod sprites;
mod stack;

use jvm::image::BufferedImage;
pub(crate) use location::{Location, MINECRAFT};
pub use pack::{Layout, Pack};
pub use source::Source;
pub(crate) use stack::Stack;
use std::cell::RefCell;
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
    static STACK: RefCell<Stack> = RefCell::new(Stack::new(vec![vanilla()]));
}

// the user's extracted b1.7.3 jar read in place
pub fn vanilla() -> Rc<Pack> { VANILLA.with(Rc::clone) }

// texture packs above the vanilla jar, lowest first
pub fn install(packs: &[Rc<Pack>])
{
    let stack: Vec<Rc<Pack>> = std::iter::once(vanilla()).chain(packs.iter().cloned()).collect();
    STACK.with(|current| *current.borrow_mut() = Stack::new(stack));
}

pub(crate) fn read(location: &Location) -> Option<Vec<u8>> { STACK.with(|stack| stack.borrow().read(location)) }

// a jar path such as /lang/en_US.lang
pub fn readBeta(path: &str) -> Option<Vec<u8>> { read(&legacy::location(path)) }

// beta read some files from the jar even with a texture pack selected
pub fn readJar(path: &str) -> Option<Vec<u8>> { vanilla().read(&legacy::location(path)) }

// a beta sheet such as /terrain.png with sprites from modern packs above the pack that supplied it
pub fn overlay(path: &str, image: Rc<BufferedImage>) -> Rc<BufferedImage>
{
    STACK.with(|stack| sheet::overlay(stack.borrow().packs(), path, image))
}
