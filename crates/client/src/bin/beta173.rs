use client::assets;
use client::client::Minecraft;
use jvm::logging::logger;
use std::process::ExitCode;

// virtual size of the game thread stack, near a deep java -Xss so recursion depth matches more closely
const GAME_STACK_BYTES: usize = 512 * 1024 * 1024;
// crates whose debug lines reach the game log, every other library only logs warnings and errors
const LIBRARY_LOGS: &[&str] = &["pudscode"];

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
            jvm::system::errPrintln(&format!("Exception in thread \"main\" {}", jvm::throwable::fromPayload(payload)));
            ExitCode::FAILURE
        }
    }
}

fn run() -> ExitCode
{
    if let Err(error) = jvm::data_dir::createClient()
    {
        jvm::system::errPrintln(&format!(
            "could not set up the game folder {}: {error}",
            jvm::data_dir::root().display()
        ));
        return ExitCode::FAILURE;
    }
    let logs: std::path::PathBuf = Minecraft::getMinecraftDir().join("logs");
    jvm::log_files::install(
        jvm::logging::Logger::getLogger("Minecraft"),
        &logs,
        "client",
        jvm::log_files::LogFormatter::boxed,
    );
    jvm::logging::bridge(logger(), LIBRARY_LOGS);
    logger().info(&format!(
        "Starting beta173 client {} on {} {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    logger().debug(&format!("Game folder {}", jvm::data_dir::root().display()));
    if !assets::isComplete()
    {
        logger().severe(&format!(
            "no game assets in {}, start the game with beta173_launcher",
            assets::assetRoot().display()
        ));
        return ExitCode::FAILURE;
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    Minecraft::main(&args);
    ExitCode::SUCCESS
}
