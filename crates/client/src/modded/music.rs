use crate::client::Minecraft;
use crate::sound::SoundManager;
use game::entity::entity_player;
use jvm::random::Random;
use modloader::mod_protocol::abi::MusicSpec;
use std::cell::{Cell, RefCell};

// the first mod track starts soon after arriving rather than after vanilla's long wait
const FIRST_DELAY: i32 = 100;

thread_local! {
    static DIMENSION: Cell<Option<i32>> = const { Cell::new(None) };
    static UNTIL_NEXT: Cell<i32> = const { Cell::new(FIRST_DELAY) };
    static RANDOM: RefCell<Random> = RefCell::new(Random::new());
}

fn musicOf(dimension: i32) -> Option<MusicSpec> { modloader::dimensions::dimension(dimension)?.m_spec.music }

// true when the player's dimension has mod music, which then replaces SoundManager.playRandomMusicIfReady
pub fn tick(manager: &SoundManager) -> bool
{
    let Some(dimension) = Minecraft::theMinecraft()
        .and_then(|mc| mc.m_thePlayer.get())
        .map(|player| entity_player::fields(&player).m_dimension.get())
    else
    {
        return false;
    };
    let previous: Option<i32> = DIMENSION.replace(Some(dimension));
    let music: Option<MusicSpec> = musicOf(dimension);
    if previous.is_some_and(|previous| previous != dimension && (music.is_some() || musicOf(previous).is_some()))
    {
        manager.stopMusic();
        UNTIL_NEXT.set(FIRST_DELAY);
    }
    let Some(music) = music
    else
    {
        return false;
    };
    if manager.musicPlaying()
    {
        return true;
    }
    if UNTIL_NEXT.get() > 0
    {
        UNTIL_NEXT.set(UNTIL_NEXT.get() - 1);
        return true;
    }
    if manager.playMusicNamed(&music.sound)
    {
        let [low, high] = music.delay;
        let spread: i32 = (high - low + 1).max(1);
        UNTIL_NEXT.set(low + RANDOM.with(|random| random.borrow().nextIntBounded(spread)));
    }
    true
}
