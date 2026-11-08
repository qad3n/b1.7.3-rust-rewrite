use client::assets;
use client::client::Minecraft;
use std::process::ExitCode;

// virtual size of the game thread stack, near a deep java -Xss so recursion depth matches more closely
const GAME_STACK_BYTES: usize = 512 * 1024 * 1024;

// everything runs on the big stack thread, so the thread_local registries are first built there
fn main() -> ExitCode
{
    jvm::throwable::installHook();
    game::port::setSide(game::port::Side::Client);
    client::port::sound::install();
    let spawned: std::io::Result<std::thread::JoinHandle<ExitCode>> =
        std::thread::Builder::new().name("main".to_owned()).stack_size(GAME_STACK_BYTES).spawn(run);
    let handle: std::thread::JoinHandle<ExitCode> = match spawned
    {
        Ok(handle) => handle,
        Err(error) =>
        {
            eprintln!("beta173: could not start the game thread: {error}");
            return ExitCode::FAILURE;
        }
    };
    match handle.join()
    {
        Ok(code) => code,
        Err(payload) =>
        {
            eprintln!("Exception in thread \"main\" {}", jvm::throwable::fromPayload(payload));
            ExitCode::FAILURE
        }
    }
}

fn run() -> ExitCode
{
    if !assets::isComplete()
    {
        eprintln!("beta173: no game assets in {}, start the game with beta173_launcher", assets::assetRoot().display());
        return ExitCode::FAILURE;
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    Minecraft::main(&args);
    ExitCode::SUCCESS
}
