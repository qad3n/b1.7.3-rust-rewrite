use mod_protocol::abi::{self, GuestCall, GuestReply, HostCall, HostReply, LogLevel, exports, imports};
use mod_protocol::wire;
use sha2::{Digest, Sha256};
use std::cell::{Cell, RefCell};
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use wasmtime::{AsContextMut, Caller, Config, Engine, Extern, Linker, Memory, Module, Store, TypedFunc};

mod budget
{
    use std::time::Duration;

    pub const EPOCH_PERIOD: Duration = Duration::from_millis(10);
    // a single guest call may run this many epochs before it traps
    pub const CALL_EPOCHS: u64 = 50;
}

pub struct HostState
{
    pub m_mod: usize,
}

static ENGINE: OnceLock<Engine> = OnceLock::new();

// the epoch thread only starts when the first mod with code loads
pub fn engine() -> &'static Engine
{
    ENGINE.get_or_init(|| {
        let mut config: Config = Config::new();
        config.epoch_interruption(true);
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        let engine: Engine = Engine::new(&config).expect("the wasmtime config is valid on supported hosts");
        let ticker: Engine = engine.clone();
        let spawned: std::io::Result<std::thread::JoinHandle<()>> =
            std::thread::Builder::new().name("Mod watchdog".to_owned()).spawn(move || {
                loop
                {
                    std::thread::sleep(budget::EPOCH_PERIOD);
                    ticker.increment_epoch();
                }
            });
        if let Err(error) = spawned
        {
            crate::log::error(crate::log::LOADER, &format!("mod watchdog thread did not start: {error}"));
        }
        engine
    })
}

fn compile(wasm: &[u8], cacheDir: &Path) -> Result<Module, String>
{
    let engine: &Engine = engine();
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(wasm);
    let mut compatibility: std::collections::hash_map::DefaultHasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&engine.precompile_compatibility_hash(), &mut compatibility);
    hasher.update(std::hash::Hasher::finish(&compatibility).to_le_bytes());
    let cached: std::path::PathBuf = cacheDir.join(format!("{}.cwasm", crate::discovery::hex(&hasher.finalize())));
    if cached.is_file()
    {
        // SAFETY: the file name hashes the wasm bytes and the engine compatibility key, and only this loader writes the cache
        if let Ok(module) = unsafe { Module::deserialize_file(engine, &cached) }
        {
            return Ok(module);
        }
    }
    let module: Module = Module::new(engine, wasm).map_err(|error| format!("mod.wasm does not compile: {error:#}"))?;
    if let Ok(bytes) = module.serialize()
    {
        let written: std::io::Result<()> =
            std::fs::create_dir_all(cacheDir).and_then(|()| std::fs::write(&cached, bytes));
        if let Err(error) = written
        {
            crate::log::warn(
                crate::log::LOADER,
                &format!("could not cache compiled mod at {}: {error}", cached.display()),
            );
        }
    }
    Ok(module)
}

thread_local! {
    // the caller of an active host call per mod, so game code reached from it can call that mod again
    static REENTRY: RefCell<Vec<(usize, *mut Caller<'static, HostState>)>> = const { RefCell::new(Vec::new()) };
}

struct Exports
{
    m_memory: Memory,
    m_alloc: TypedFunc<u32, u32>,
    m_free: TypedFunc<(u32, u32), ()>,
    m_dispatch: TypedFunc<(u32, u32), u64>,
}

pub struct ModInstance
{
    m_modIndex: usize,
    m_store: RefCell<Store<HostState>>,
    m_exports: Exports,
    m_nanos: Cell<u64>,
    m_calls: Cell<u64>,
}

fn readGuest(memory: &Memory, context: impl wasmtime::AsContext, pointer: u32, length: u32) -> Result<Vec<u8>, String>
{
    let mut bytes: Vec<u8> = vec![0; length as usize];
    memory
        .read(context, pointer as usize, &mut bytes)
        .map_err(|error| format!("guest buffer out of bounds: {error}"))?;
    Ok(bytes)
}

fn hostCall(mut caller: Caller<'_, HostState>, pointer: u32, length: u32) -> wasmtime::Result<u64>
{
    let memory: Memory = caller
        .get_export(exports::MEMORY)
        .and_then(Extern::into_memory)
        .ok_or_else(|| wasmtime::Error::msg("guest exports no memory"))?;
    let bytes: Vec<u8> = readGuest(&memory, &caller, pointer, length).map_err(wasmtime::Error::msg)?;
    let modIndex: usize = caller.data().m_mod;
    let reply: HostReply = match wire::decode::<HostCall>(&bytes)
    {
        Ok(call) =>
        {
            let pointer: *mut Caller<'_, HostState> = &mut caller;
            // SAFETY: the pointer is popped before this function touches caller again, so it never outlives the borrow
            REENTRY.with(|stack| stack.borrow_mut().push((modIndex, pointer.cast())));
            let reply: HostReply = crate::host::handle(modIndex, call);
            REENTRY.with(|stack| stack.borrow_mut().pop());
            reply
        }
        Err(error) => HostReply::Error(format!("bad host call: {error}")),
    };
    let out: Vec<u8> = wire::encode(&reply);
    let alloc: TypedFunc<u32, u32> = caller
        .get_export(exports::ALLOC)
        .and_then(Extern::into_func)
        .ok_or_else(|| wasmtime::Error::msg("guest exports no allocator"))?
        .typed(&caller)?;
    let outPointer: u32 = alloc.call(&mut caller, out.len() as u32)?;
    memory.write(&mut caller, outPointer as usize, &out)?;
    Ok(abi::packBuffer(outPointer, out.len() as u32))
}

fn hostLog(mut caller: Caller<'_, HostState>, level: u32, pointer: u32, length: u32)
{
    let Some(memory) = caller.get_export(exports::MEMORY).and_then(Extern::into_memory)
    else
    {
        return;
    };
    let text: String = match readGuest(&memory, &mut caller, pointer, length)
    {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(error) => error,
    };
    let modId: String = crate::state::modId(caller.data().m_mod);
    match level
    {
        0 => crate::log::info(&modId, &text),
        1 => crate::log::warn(&modId, &text),
        _ => crate::log::error(&modId, &text),
    }
}

pub fn logLevel(level: LogLevel) -> u32
{
    match level
    {
        LogLevel::Info => 0,
        LogLevel::Warn => 1,
        LogLevel::Error => 2,
    }
}

fn invoke(
    context: &mut impl AsContextMut<Data = HostState>,
    exports: &Exports,
    request: &[u8],
) -> wasmtime::Result<Vec<u8>>
{
    let pointer: u32 = exports.m_alloc.call(&mut *context, request.len() as u32)?;
    exports.m_memory.write(&mut *context, pointer as usize, request)?;
    let packed: u64 = exports.m_dispatch.call(&mut *context, (pointer, request.len() as u32))?;
    exports.m_free.call(&mut *context, (pointer, request.len() as u32))?;
    let (outPointer, outLength): (u32, u32) = abi::unpackBuffer(packed);
    if outLength == 0
    {
        return Ok(Vec::new());
    }
    let reply: Vec<u8> =
        readGuest(&exports.m_memory, &mut *context, outPointer, outLength).map_err(wasmtime::Error::msg)?;
    exports.m_free.call(&mut *context, (outPointer, outLength))?;
    Ok(reply)
}

impl ModInstance
{
    pub fn load(modIndex: usize, wasm: &[u8], cacheDir: &Path) -> Result<ModInstance, String>
    {
        let module: Module = compile(wasm, cacheDir)?;
        let engine: &Engine = engine();
        let mut linker: Linker<HostState> = Linker::new(engine);
        linker.func_wrap(abi::IMPORT_MODULE, imports::CALL, hostCall).map_err(|error| error.to_string())?;
        linker.func_wrap(abi::IMPORT_MODULE, imports::LOG, hostLog).map_err(|error| error.to_string())?;
        let mut store: Store<HostState> = Store::new(engine, HostState { m_mod: modIndex });
        store.set_epoch_deadline(budget::CALL_EPOCHS);
        // only b173 imports exist, so a mod has no file system, clock or socket access
        let instance: wasmtime::Instance =
            linker.instantiate(&mut store, &module).map_err(|error| format!("mod.wasm does not link: {error:#}"))?;
        let abiVersion: TypedFunc<(), u32> = instance
            .get_typed_func(&mut store, exports::ABI)
            .map_err(|_| format!("mod.wasm exports no {}, build it with the b173 SDK", exports::ABI))?;
        let version: u32 = abiVersion.call(&mut store, ()).map_err(|error| format!("{}: {error:#}", exports::ABI))?;
        if version != abi::ABI_VERSION
        {
            return Err(format!("mod.wasm uses ABI {version}, this loader speaks {}", abi::ABI_VERSION));
        }
        let missing = |name: &str| format!("mod.wasm exports no {name}");
        let exports: Exports = Exports {
            m_memory: instance.get_memory(&mut store, exports::MEMORY).ok_or_else(|| missing(exports::MEMORY))?,
            m_alloc: instance.get_typed_func(&mut store, exports::ALLOC).map_err(|_| missing(exports::ALLOC))?,
            m_free: instance.get_typed_func(&mut store, exports::FREE).map_err(|_| missing(exports::FREE))?,
            m_dispatch: instance
                .get_typed_func(&mut store, exports::DISPATCH)
                .map_err(|_| missing(exports::DISPATCH))?,
        };
        Ok(ModInstance {
            m_modIndex: modIndex,
            m_store: RefCell::new(store),
            m_exports: exports,
            m_nanos: Cell::new(0),
            m_calls: Cell::new(0),
        })
    }

    fn reentrantCaller(&self) -> Option<*mut Caller<'static, HostState>>
    {
        REENTRY.with(|stack| {
            stack.borrow().iter().rev().find(|(index, _)| *index == self.m_modIndex).map(|(_, caller)| *caller)
        })
    }

    pub fn call(&self, call: &GuestCall) -> Result<GuestReply, String>
    {
        let request: Vec<u8> = wire::encode(call);
        let started: Instant = Instant::now();
        let result: wasmtime::Result<Vec<u8>> = match self.m_store.try_borrow_mut()
        {
            Ok(mut store) =>
            {
                store.set_epoch_deadline(budget::CALL_EPOCHS);
                invoke(&mut *store, &self.m_exports, &request)
            }
            Err(_) => match self.reentrantCaller()
            {
                // SAFETY: the pointer belongs to a host call of this mod that is still on the stack below us
                Some(caller) => invoke(unsafe { &mut *caller }, &self.m_exports, &request),
                None => Err(wasmtime::Error::msg("mod store is busy and no host call of this mod is active")),
            },
        };
        self.m_nanos.set(self.m_nanos.get() + started.elapsed().as_nanos() as u64);
        self.m_calls.set(self.m_calls.get() + 1);
        let bytes: Vec<u8> = result.map_err(|error| format!("{error:#}"))?;
        if bytes.is_empty()
        {
            return Ok(GuestReply::Unit);
        }
        wire::decode(&bytes).map_err(|error| format!("bad guest reply: {error}"))
    }

    // total guest time and call count, read and reset by the profiler
    pub fn takeProfile(&self) -> (Duration, u64)
    {
        (Duration::from_nanos(self.m_nanos.replace(0)), self.m_calls.replace(0))
    }
}
