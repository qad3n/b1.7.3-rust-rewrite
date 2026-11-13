use crate::port::file_url;
use crate::settings::GameSettings;
use crate::sound::texture_pack_sounds::{self, Event, Install, Origin, Pool, Sound};
use crate::sound::{CodecMus, SoundPool, SoundPoolEntry};
use game::entity::Entity;
use game::math::math_helper::{cos, sin};
use jvm::field::Field;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use jvm::{debug, info};
use paulscode::sound::codecs::{CodecJOrbis, CodecWav};
use paulscode::sound::libraries::LibraryLWJGLOpenAL;
use paulscode::sound::{SoundSystem, SoundSystemConfig};
use std::cell::{Cell, RefCell};
use std::f32::consts::PI;
use std::path::Path;
use std::rc::Rc;

// the one source records stream through
const STREAMING: &str = "streaming";

thread_local!
{
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
    // every file added in order so a texture pack change can rebuild the pools
    m_installs: RefCell<Vec<Install>>,
}

impl SoundManager
{
    pub fn new() -> Self
    {
        let rand: Random = Random::new();
        let ticksBeforeMusic: i32 = rand.nextIntBounded(12000);

        Self
        {
            m_soundPoolSounds: SoundPool::new(),
            m_soundPoolStreaming: SoundPool::new(),
            m_soundPoolMusic: SoundPool::new(),
            m_nextSoundEventIndex: Cell::new(0),
            m_options: Field::new(None),
            m_rand: rand,
            m_ticksBeforeMusic: Cell::new(ticksBeforeMusic),
            m_installs: RefCell::new(Vec::new()),
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

        if silent
        {
            info!("Sound and music are muted, the sound system starts once either is turned up");
        }

        if !(loaded() || silent)
        {
            self.tryToSetLibraryAndCodecs();
        }
    }

    pub fn logPools(&self)
    {
        info!(
            "Sound pools hold {} sounds, {} records and {} music tracks",
            self.m_soundPoolSounds.m_numberOfSoundPoolEntries.get(),
            self.m_soundPoolStreaming.m_numberOfSoundPoolEntries.get(),
            self.m_soundPoolMusic.m_numberOfSoundPoolEntries.get()
        );
    }

    fn tryToSetLibraryAndCodecs(&self)
    {
        let result: Result<(), jvm::throwable::Throwable> =
            jvm::throwable::catchThrowable(&jvm::throwable::THROWABLE, ||
            {
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

        match result
        {
            Ok(()) => debug!("Sound system linked"),

            Err(throwable) =>
            {
                throwable.printStackTrace();
                jvm::system::errPrintln("error linking with the LibraryJavaSound plug-in");
            }
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

    pub fn addSound(&self, path: &str, dir: &Path) { self.install(Pool::Sounds, path, dir, Origin::Mod); }

    pub fn addStreaming(&self, path: &str, file: &Path) { self.install(Pool::Streaming, path, file, Origin::Mod); }

    pub fn addMusic(&self, path: &str, file: &Path) { self.install(Pool::Music, path, file, Origin::Mod); }

    fn pool(&self, pool: Pool) -> &SoundPool
    {
        match pool
        {
            Pool::Sounds => &self.m_soundPoolSounds,
            Pool::Streaming => &self.m_soundPoolStreaming,
            Pool::Music => &self.m_soundPoolMusic,
        }
    }

    // a texture pack above the origin of a file may replace it
    pub fn install(&self, pool: Pool, path: &str, file: &Path, origin: Origin)
    {
        self.pool(pool).addSound(path, file);
        nameSound(pool, origin, &self.pool(pool).key(path));

        let install: Install =
            Install { m_pool: pool, m_path: path.to_owned(), m_file: file.to_path_buf(), m_origin: origin };
        self.m_installs.borrow_mut().push(install);
    }

    // adds every file again in its first order with texture pack files and sounds.json events on top
    pub fn rebuild(&self)
    {
        for pool in [Pool::Sounds, Pool::Streaming, Pool::Music]
        {
            self.pool(pool).clear();
        }

        for install in self.m_installs.borrow().iter()
        {
            let target: &SoundPool = self.pool(install.m_pool);
            let Sound { m_name: name, m_file: file } = texture_pack_sounds::resolve(install);
            target.add(target.key(&install.m_path), name, &file);
        }

        for Event { m_pool: pool, m_key: key, m_replace: replace, m_sounds: sounds } in texture_pack_sounds::events()
        {
            let target: &SoundPool = self.pool(pool);
            if replace
            {
                target.remove(&key);
            }

            for Sound { m_name: name, m_file: file } in sounds
            {
                target.add(key.clone(), name, &file);
            }
        }
    }

    pub fn playRandomMusicIfReady(&self)
    {
        // rewrite: a mod dimension with its own music plays it instead of the vanilla tracks
        if crate::modded::music::tick(self)
        {
            return;
        }
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

    // a background track by sound pool name, for mod dimension music, false while sound is off or busy
    pub fn playMusicNamed(&self, name: &str) -> bool
    {
        if !loaded() || self.options().m_musicVolume.get() == 0.0
        {
            return false;
        }
        let Some(entry) = self.m_soundPoolSounds.getRandomSoundFromSoundPool(name)
        else
        {
            return false;
        };

        let system: Rc<SoundSystem> = sndSystem();
        system.backgroundMusic("BgMusic", &entry.m_soundUrl, &entry.m_soundName, false);
        system.setVolume("BgMusic", self.options().m_musicVolume.get());
        system.play("BgMusic");
        true
    }

    pub fn musicPlaying(&self) -> bool
    {
        loaded() && (sndSystem().playing("BgMusic") || sndSystem().playing("streaming"))
    }

    pub fn stopMusic(&self)
    {
        if loaded() && sndSystem().playing("BgMusic")
        {
            sndSystem().stop("BgMusic");
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
        if !self.stopStreaming()
        {
            return;
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
            self.stream(&soundPoolEntry.m_soundUrl, &soundPoolEntry.m_soundName, [x, y, z], 0.0);
        }
    }

    // rewrite: a song file a mod streams through the record source starting offset seconds in
    pub fn playFile(&self, file: &Path, x: f32, y: f32, z: f32, offset: f32)
    {
        if !self.stopStreaming()
        {
            return;
        }

        let identifier: String = file.file_name().map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        self.stream(&file_url::fileUrl(file), &identifier, [x, y, z], offset);
    }

    // false while sound is off
    fn stopStreaming(&self) -> bool
    {
        if !loaded() || self.options().m_soundVolume.get() == 0.0
        {
            return false;
        }

        let system: Rc<SoundSystem> = sndSystem();
        if system.playing(STREAMING)
        {
            system.stop(STREAMING);
        }

        true
    }

    fn stream(&self, url: &str, identifier: &str, pos: [f32; 3], offset: f32)
    {
        let system: Rc<SoundSystem> = sndSystem();
        if system.playing("BgMusic")
        {
            system.stop("BgMusic");
        }

        let f: f32 = 16.0;
        system.newStreamingSource(true, STREAMING, url, identifier, false, pos[0], pos[1], pos[2], 2, f * 4.0);
        system.setVolume(STREAMING, 0.5 * self.options().m_soundVolume.get());
        system.playFrom(STREAMING, offset);
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

// sound events join the sound registry as minecraft:<key> or <modid>:<rest of key>, music stays out
fn nameSound(pool: Pool, origin: Origin, key: &str)
{
    if matches!(pool, Pool::Music)
    {
        return;
    }

    let name: String = match (origin, key.split_once('.'))
    {
        (Origin::Mod, Some((modid, rest))) => format!("{modid}:{rest}"),
        _ => format!("minecraft:{key}"),
    };
    match modloader::Identifier::parse(&name)
    {
        Ok(id) => modloader::registry::addClientName(modloader::mod_protocol::abi::RegistryKind::Sound, id),
        Err(error) => debug!("sound {name} has no registry name: {error}"),
    }
}
