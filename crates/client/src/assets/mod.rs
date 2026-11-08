pub use game::port::{assetRoot, getResourceAsStream};

// files every complete extraction holds, checked before the game starts
const PROBE_FILES: &[&str] =
    &["terrain.png", "gui/gui.png", "font/default.png", "title/mojang.png", "lang/en_US.lang", "misc/grasscolor.png"];

pub fn isComplete() -> bool { PROBE_FILES.iter().all(|file| assetRoot().join(file).is_file()) }
