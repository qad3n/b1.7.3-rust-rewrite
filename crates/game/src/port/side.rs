use std::sync::OnceLock;

// which program runs, for the methods whose client and server bodies differ
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side
{
    Client,
    Server,
}

static SIDE: OnceLock<Side> = OnceLock::new();

// each binary calls this once before any game code runs
pub fn setSide(side: Side)
{
    if SIDE.set(side).is_err() && SIDE.get() != Some(&side)
    {
        panic!("the side was already set to {:?}", SIDE.get());
    }
}

pub fn side() -> Side { *SIDE.get().expect("the binary sets the side before game code runs") }
