use crate::settings::GameSettings;
use crate::sound::{CodecMus, SoundPool, SoundPoolEntry};
use game::entity::Entity;
use game::math::math_helper::{cos, sin};
use jvm::field::Field;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use paulscode::sound::codecs::{CodecJOrbis, CodecWav};
use paulscode::sound::libraries::LibraryLWJGLOpenAL;
use paulscode::sound::{SoundSystem, SoundSystemConfig};
use std::cell::Cell;
use std::f32::consts::PI;
use std::path::Path;
use std::rc::Rc;

thread_local! {
    static SND_SYSTEM: Field<Option<Rc<SoundSystem>>> = Field::new(None);
    static LOADED: Cell<bool> = const { Cell::new(false) };
}

fn sndSystem() -> Rc<SoundSystem> { SND_SYSTEM.with(Field::get).orNpe() }

fn loaded() -> bool { LOADED.with(Cell::get) }

pub struct SoundManager
{
    m_soundPoolSounds: SoundPool,
    m_soundPoolStreaming: SoundPool,
    m_soundPoolMusic: SoundPool,
    m_nextSoundEventIndex: Cell<i32>,
    m_options: Field<Option<Rc<GameSettings>>>,
    m_rand: Random,
    m_ticksBeforeMusic: Cell<i32>,
}

impl SoundManager
{
    pub fn new() -> SoundManager
    {
        let rand: Random = Random::new();
        let ticksBeforeMusic: i32 = rand.nextIntBounded(12000);
        SoundManager {
            m_soundPoolSounds: SoundPool::new(),
            m_soundPoolStreaming: SoundPool::new(),
            m_soundPoolMusic: SoundPool::new(),
            m_nextSoundEventIndex: Cell::new(0),
            m_options: Field::new(None),
            m_rand: rand,
            m_ticksBeforeMusic: Cell::new(ticksBeforeMusic),
        }
    }

    fn options(&self) -> Rc<GameSettings> { self.m_options.get().orNpe() }

    pub fn loadSoundSettings(&self, options: Option<Rc<GameSettings>>)
    {
        self.m_soundPoolStreaming.m_trimTrailingDigits.set(false);
        let silent: bool = options
            .as_ref()
            .is_some_and(|options| options.m_soundVolume.get() == 0.0 && options.m_musicVolume.get() == 0.0);
        self.m_options.set(options);
        if !(loaded() || silent)
        {
            self.tryToSetLibraryAndCodecs();
        }
    }

    fn tryToSetLibraryAndCodecs(&self)
    {
        let result: Result<(), jvm::throwable::Throwable> =
            jvm::throwable::catchThrowable(&jvm::throwable::THROWABLE, || {
                let options: Rc<GameSettings> = self.options();
                let f: f32 = options.m_soundVolume.get();
                let f2: f32 = options.m_musicVolume.get();
                options.m_soundVolume.set(0.0);
                options.m_musicVolume.set(0.0);
                options.saveOptions();
                SoundSystemConfig::addLibrary(LibraryLWJGLOpenAL::CLASS_NAME);
                SoundSystemConfig::setCodec("ogg", CodecJOrbis::CLASS_NAME);
                SoundSystemConfig::setCodec("mus", CodecMus::CLASS_NAME);
                SoundSystemConfig::setCodec("wav", CodecWav::CLASS_NAME);
                SND_SYSTEM.with(|system| system.set(Some(Rc::new(SoundSystem::new()))));
                options.m_soundVolume.set(f);
                options.m_musicVolume.set(f2);
                options.saveOptions();
            });
        if let Err(throwable) = result
        {
            throwable.printStackTrace();
            jvm::system::errPrintln("error linking with the LibraryJavaSound plug-in");
        }
        LOADED.with(|loaded| loaded.set(true));
    }

    pub fn onSoundOptionsChanged(&self)
    {
        let options: Rc<GameSettings> = self.options();
        if !(loaded() || options.m_soundVolume.get() == 0.0 && options.m_musicVolume.get() == 0.0)
        {
            self.tryToSetLibraryAndCodecs();
        }
        if loaded()
        {
            if options.m_musicVolume.get() == 0.0
            {
                sndSystem().stop("BgMusic");
            }
            else
            {
                sndSystem().setVolume("BgMusic", options.m_musicVolume.get());
            }
        }
    }

    pub fn closeMinecraft(&self)
    {
        if loaded()
        {
            sndSystem().cleanup();
        }
    }

    pub fn addSound(&self, path: &str, dir: &Path) { self.m_soundPoolSounds.addSound(path, dir); }

    pub fn addStreaming(&self, path: &str, file: &Path) { self.m_soundPoolStreaming.addSound(path, file); }

    pub fn addMusic(&self, path: &str, file: &Path) { self.m_soundPoolMusic.addSound(path, file); }

    pub fn playRandomMusicIfReady(&self)
    {
        if !loaded() || self.options().m_musicVolume.get() == 0.0
        {
            return;
        }
        let system: Rc<SoundSystem> = sndSystem();
        if system.playing("BgMusic") || system.playing("streaming")
        {
            return;
        }
        if self.m_ticksBeforeMusic.get() > 0
        {
            self.m_ticksBeforeMusic.set(self.m_ticksBeforeMusic.get() - 1);
            return;
        }
        if let Some(soundPoolEntry) = self.m_soundPoolMusic.getRandomSound()
        {
            self.m_ticksBeforeMusic.set(self.m_rand.nextIntBounded(12000) + 12000);
            system.backgroundMusic("BgMusic", &soundPoolEntry.m_soundUrl, &soundPoolEntry.m_soundName, false);
            system.setVolume("BgMusic", self.options().m_musicVolume.get());
            system.play("BgMusic");
        }
    }

    pub fn update(&self, listener: Option<&Rc<Entity>>, tickDelta: f32)
    {
        if !loaded() || self.options().m_soundVolume.get() == 0.0
        {
            return;
        }
        let Some(listener) = listener
        else
        {
            return;
        };
        let prevYaw: f32 = listener.m_prevRotationYaw.get();
        let f: f32 = prevYaw + (listener.m_rotationYaw.get() - prevYaw) * tickDelta;
        let d: f64 = listener.m_prevPosX.get() + (listener.posX() - listener.m_prevPosX.get()) * tickDelta as f64;
        let d2: f64 = listener.m_prevPosY.get() + (listener.posY() - listener.m_prevPosY.get()) * tickDelta as f64;
        let d3: f64 = listener.m_prevPosZ.get() + (listener.posZ() - listener.m_prevPosZ.get()) * tickDelta as f64;
        let f2: f32 = cos(-f * (PI / 180.0) - PI);
        let f3: f32 = sin(-f * (PI / 180.0) - PI);
        let f4: f32 = -f3;
        let f5: f32 = 0.0;
        let f6: f32 = -f2;
        let f7: f32 = 0.0;
        let f8: f32 = 1.0;
        let f9: f32 = 0.0;
        let system: Rc<SoundSystem> = sndSystem();
        system.setListenerPosition(d as f32, d2 as f32, d3 as f32);
        system.setListenerOrientation(f4, f5, f6, f7, f8, f9);
    }

    pub fn playStreaming(&self, name: Option<&str>, x: f32, y: f32, z: f32, volume: f32, _pitch: f32)
    {
        if !loaded() || self.options().m_soundVolume.get() == 0.0
        {
            return;
        }
        let string: &str = "streaming";
        let system: Rc<SoundSystem> = sndSystem();
        if system.playing("streaming")
        {
            system.stop("streaming");
        }
        let Some(name) = name
        else
        {
            return;
        };
        let soundPoolEntry: Option<Rc<SoundPoolEntry>> = self.m_soundPoolStreaming.getRandomSoundFromSoundPool(name);
        if let Some(soundPoolEntry) = soundPoolEntry
            && volume > 0.0
        {
            if system.playing("BgMusic")
            {
                system.stop("BgMusic");
            }
            let f: f32 = 16.0;
            system.newStreamingSource(
                true,
                string,
                &soundPoolEntry.m_soundUrl,
                &soundPoolEntry.m_soundName,
                false,
                x,
                y,
                z,
                2,
                f * 4.0,
            );
            system.setVolume(string, 0.5 * self.options().m_soundVolume.get());
            system.play(string);
        }
    }

    fn nextSourceName(&self) -> String
    {
        self.m_nextSoundEventIndex.set((self.m_nextSoundEventIndex.get() + 1) % 256);
        format!("sound_{}", self.m_nextSoundEventIndex.get())
    }

    pub fn playSound(&self, id: &str, x: f32, y: f32, z: f32, mut volume: f32, pitch: f32)
    {
        if !loaded() || self.options().m_soundVolume.get() == 0.0
        {
            return;
        }
        let soundPoolEntry: Option<Rc<SoundPoolEntry>> = self.m_soundPoolSounds.getRandomSoundFromSoundPool(id);
        if let Some(soundPoolEntry) = soundPoolEntry
            && volume > 0.0
        {
            let string: String = self.nextSourceName();
            let mut f: f32 = 16.0;
            if volume > 1.0
            {
                f *= volume;
            }
            let system: Rc<SoundSystem> = sndSystem();
            system.newSource(
                volume > 1.0,
                &string,
                &soundPoolEntry.m_soundUrl,
                &soundPoolEntry.m_soundName,
                false,
                x,
                y,
                z,
                2,
                f,
            );
            system.setPitch(&string, pitch);
            if volume > 1.0
            {
                volume = 1.0;
            }
            system.setVolume(&string, volume * self.options().m_soundVolume.get());
            system.play(&string);
        }
    }

    pub fn playSoundFX(&self, id: &str, mut volume: f32, pitch: f32)
    {
        if !loaded() || self.options().m_soundVolume.get() == 0.0
        {
            return;
        }
        if let Some(soundPoolEntry) = self.m_soundPoolSounds.getRandomSoundFromSoundPool(id)
        {
            let string: String = self.nextSourceName();
            let system: Rc<SoundSystem> = sndSystem();
            system.newSource(
                false,
                &string,
                &soundPoolEntry.m_soundUrl,
                &soundPoolEntry.m_soundName,
                false,
                0.0,
                0.0,
                0.0,
                0,
                0.0,
            );
            if volume > 1.0
            {
                volume = 1.0;
            }
            volume *= 0.25;
            system.setPitch(&string, pitch);
            system.setVolume(&string, volume * self.options().m_soundVolume.get());
            system.play(&string);
        }
    }
}

impl Default for SoundManager
{
    fn default() -> Self { Self::new() }
}
