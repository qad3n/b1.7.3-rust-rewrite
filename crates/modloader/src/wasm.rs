use mod_protocol::abi::{self, GuestCall, GuestReply, HostCall, HostReply, LogLevel, exports, imports};
use mod_protocol::wire;
use sha2::{Digest, Sha256};
use std::cell::{Cell, RefCell};
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use wasmtime::
{
    AsContextMut, Caller, Config, Engine, Extern, Linker, Memory, Module, Store, StoreHookState, StoreLimits,
    StoreLimitsBuilder, TypedFunc, UpdateDeadline,
};

mod budget
{
    use std::time::Duration;

    pub const EPOCH_PERIOD: Duration = Duration::from_millis(10);
    // epochs of its own code a mod may run in one outermost call, time the host spends serving it is not counted
    pub const CALL_EPOCHS: u64 = 50;
    // linear memory one mod may grow to
    pub const MEMORY_LIMIT: usize = 256 << 20;
}

// epochs since the engine started, read on every switch between mod and host so it has to be cheaper than a clock
static EPOCH: AtomicU64 = AtomicU64::new(0);

pub struct HostState
{
    pub m_mod: usize,
    m_limits: StoreLimits,
    // epochs of the mod's own running time in the current outermost call
    m_used: u64,
    // the epoch the mod's code started running at, cleared while the host works for it
    m_since: Option<u64>,
}

impl HostState
{
    fn new(modIndex: usize) -> Self
    {
        Self
        {
            m_mod: modIndex,
            m_limits: StoreLimitsBuilder::new().memory_size(budget::MEMORY_LIMIT).build(),
            m_used: 0,
            m_since: None,
        }
    }

    fn resume(&mut self) { self.m_since = Some(EPOCH.load(Ordering::Relaxed)); }

    fn pause(&mut self)
    {
        if let Some(since) = self.m_since.take()
        {
            self.m_used += EPOCH.load(Ordering::Relaxed) - since;
        }
    }

    fn used(&self) -> u64 { self.m_used + self.m_since.map_or(0, |since| EPOCH.load(Ordering::Relaxed) - since) }
}

// runs when the epoch deadline passes: a mod still inside its own time budget gets the rest of it
fn deadline(state: StoreHookState<'_, HostState>) -> wasmtime::Result<UpdateDeadline>
{
    let used: u64 = state.data().used();
    if used >= budget::CALL_EPOCHS
    {
        return Err(wasmtime::Error::msg(format!(
            "the mod ran for more than {} ms of its own time in one call",
            budget::CALL_EPOCHS * budget::EPOCH_PERIOD.as_millis() as u64
        )));
    }

    Ok(UpdateDeadline::Continue(budget::CALL_EPOCHS - used))
}

static ENGINE: OnceLock<Engine> = OnceLock::new();

// the epoch thread only starts when the first mod with code loads
pub fn engine() -> &'static Engine
{
    ENGINE.get_or_init(||
    {
        let mut config: Config = Config::new();
        config.epoch_interruption(true);
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);

        let engine: Engine = Engine::new(&config).expect("the wasmtime config is valid on supported hosts");
        let ticker: Engine = engine.clone();
        let spawned: std::io::Result<std::thread::JoinHandle<()>> =
            std::thread::Builder::new().name("Mod watchdog".to_owned()).spawn(move ||
            {
                loop
                {
                    std::thread::sleep(budget::EPOCH_PERIOD);
                    EPOCH.fetch_add(1, Ordering::Relaxed);
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
        match unsafe { Module::deserialize_file(engine, &cached) }
        {
            Ok(module) => return Ok(module),
            Err(error) => crate::log::warn(
                crate::log::LOADER,
                &format!("compiled mod cache {} is unusable, compiling again: {error}", cached.display()),
            ),
        }
    }

    let compiling: Instant = Instant::now();
    let module: Module = Module::new(engine, wasm).map_err(|error| format!("mod.wasm does not compile: {error:#}"))?;

    crate::log::debug(crate::log::LOADER, &format!("compiled mod.wasm in {}", jvm::logging::took(compiling)));

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

thread_local!
{
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
    // the length comes from the mod, so it is checked against its memory before the host allocates for it
    let size: usize = memory.data_size(&context);
    if pointer as usize + length as usize > size
    {
        return Err(format!("guest buffer out of bounds: {length} bytes at {pointer} in a memory of {size}"));
    }

    let mut bytes: Vec<u8> = vec![0; length as usize];
    memory
        .read(context, pointer as usize, &mut bytes)
        .map_err(|error| format!("guest buffer out of bounds: {error}"))?;
    Ok(bytes)
}

// longest call text a crash trace shows
const TRACE_CALL_CHARS: usize = 300;

// pops the reentry slot of a host call even when it unwinds, and names the call in the crash trace
struct HostFrame<'a>
{
    m_mod: usize,
    m_call: &'a [u8],
}

impl Drop for HostFrame<'_>
{
    fn drop(&mut self)
    {
        REENTRY.with(|stack| stack.borrow_mut().pop());

        if !std::thread::panicking()
        {
            return;
        }

        jvm::throwable::noteUnwind(||
        {
            let call: String = wire::decode::<HostCall>(self.m_call)
                .map_or_else(|error| error.to_string(), |call| format!("{call:?}"));
            format!("host call from mod {}: {}", crate::state::modId(self.m_mod), clip(&call))
        });
    }
}

// names a call into a mod in the crash trace when it unwinds
struct GuestFrame<'a>
{
    m_mod: usize,
    m_call: &'a GuestCall,
}

impl Drop for GuestFrame<'_>
{
    fn drop(&mut self)
    {
        if !std::thread::panicking()
        {
            return;
        }

        jvm::throwable::noteUnwind(||
        {
            format!("call into mod {}: {}", crate::state::modId(self.m_mod), clip(&format!("{:?}", self.m_call)))
        });
    }
}

fn clip(text: &str) -> &str { text.char_indices().nth(TRACE_CALL_CHARS).map_or(text, |(end, _)| &text[..end]) }

fn hostCall(mut caller: Caller<'_, HostState>, pointer: u32, length: u32) -> wasmtime::Result<u64>
{
    let memory: Memory = caller
        .get_export(exports::MEMORY)
        .and_then(Extern::into_memory)
        .ok_or_else(|| wasmtime::Error::msg("guest exports no memory"))?;
    let bytes: Vec<u8> = readGuest(&memory, &caller, pointer, length).map_err(wasmtime::Error::msg)?;
    let modIndex: usize = caller.data().m_mod;
    caller.data_mut().pause();

    let reply: HostReply = match wire::decode::<HostCall>(&bytes)
    {
        Ok(call) =>
        {
            // the frame pops the pointer before this function touches caller again so it never outlives the borrow
            let pointer: *mut Caller<'_, HostState> = &mut caller;
            REENTRY.with(|stack| stack.borrow_mut().push((modIndex, pointer.cast())));

            let _frame: HostFrame<'_> = HostFrame { m_mod: modIndex, m_call: &bytes };

            crate::host::handle(modIndex, call)
        }

        Err(error) => HostReply::Error(format!("bad host call: {error}")),
    };
    let out: Vec<u8> = wire::encode(&reply);
    caller.data_mut().resume();

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

// the mod's clock runs for the length of the exchange, a nested call resumes it inside a paused host call
fn invoke(
    context: &mut impl AsContextMut<Data = HostState>,
    exports: &Exports,
    request: &[u8],
) -> wasmtime::Result<Vec<u8>>
{
    context.as_context_mut().data_mut().resume();

    let reply: wasmtime::Result<Vec<u8>> = exchange(context, exports, request);

    context.as_context_mut().data_mut().pause();
    reply
}

fn exchange(
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
    pub fn load(modIndex: usize, wasm: &[u8], cacheDir: &Path) -> Result<Self, String>
    {
        let module: Module = compile(wasm, cacheDir)?;
        let engine: &Engine = engine();
        let mut linker: Linker<HostState> = Linker::new(engine);
        linker.func_wrap(abi::IMPORT_MODULE, imports::CALL, hostCall).map_err(|error| error.to_string())?;
        linker.func_wrap(abi::IMPORT_MODULE, imports::LOG, hostLog).map_err(|error| error.to_string())?;

        let mut store: Store<HostState> = Store::new(engine, HostState::new(modIndex));
        store.limiter(|state| &mut state.m_limits);
        store.epoch_deadline_callback(deadline);
        store.set_epoch_deadline(budget::CALL_EPOCHS);
        // only b173 imports exist, so a mod has no file system, clock or socket access
        let instance: wasmtime::Instance =
            linker.instantiate(&mut store, &module).map_err(|error| format!("mod.wasm does not link: {error:#}"))?;
        let abiVersion: TypedFunc<(), u32> = instance
            .get_typed_func(&mut store, exports::ABI)
            .map_err(|_| format!("mod.wasm exports no {}, build it with the b173 SDK", exports::ABI))?;
        let version: u32 = abiVersion.call(&mut store, ()).map_err(|error| format!("{}: {error:#}", exports::ABI))?;
        if !(abi::ABI_MIN_VERSION..=abi::ABI_VERSION).contains(&version)
        {
            return Err(format!(
                "mod.wasm uses ABI {version}, this loader speaks {} to {}",
                abi::ABI_MIN_VERSION,
                abi::ABI_VERSION
            ));
        }

        let missing = |name: &str| format!("mod.wasm exports no {name}");

        let exports: Exports = Exports
        {
            m_memory: instance.get_memory(&mut store, exports::MEMORY).ok_or_else(|| missing(exports::MEMORY))?,
            m_alloc: instance.get_typed_func(&mut store, exports::ALLOC).map_err(|_| missing(exports::ALLOC))?,
            m_free: instance.get_typed_func(&mut store, exports::FREE).map_err(|_| missing(exports::FREE))?,
            m_dispatch: instance
                .get_typed_func(&mut store, exports::DISPATCH)
                .map_err(|_| missing(exports::DISPATCH))?,
        };

        Ok(Self
        {
            m_modIndex: modIndex,
            m_store: RefCell::new(store),
            m_exports: exports,
            m_nanos: Cell::new(0),
            m_calls: Cell::new(0),
        })
    }

    fn reentrantCaller(&self) -> Option<*mut Caller<'static, HostState>>
    {
        REENTRY.with(|stack|
        {
            stack.borrow().iter().rev().find(|(index, _)| *index == self.m_modIndex).map(|(_, caller)| *caller)
        })
    }

    pub fn call(&self, call: &GuestCall) -> Result<GuestReply, String>
    {
        let _frame: GuestFrame<'_> = GuestFrame { m_mod: self.m_modIndex, m_call: call };
        let request: Vec<u8> = wire::encode(call);
        let started: Instant = Instant::now();

        let result: wasmtime::Result<Vec<u8>> = match self.m_store.try_borrow_mut()
        {
            Ok(mut store) =>
            {
                store.data_mut().m_used = 0;
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
