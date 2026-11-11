use jvm::logging::logger;
use jvm::{debug, severe};
use server::server::{ConsoleLogManager, MinecraftServer};
use std::path::PathBuf;

const PROBE_ASSET: &str = "lang/en_US.lang";

// the server runs in the data folder server folder and opens the log first so startup failures reach logs/server.log
fn main()
{
    if let Err(error) = jvm::data_dir::createServer().and_then(std::env::set_current_dir)
    {
        jvm::system::errPrintln(&format!(
            "could not set up the server folder {}: {error}",
            jvm::data_dir::server().display()
        ));
        std::process::exit(1);
    }

    ConsoleLogManager::init();
    // wasmtime and other libraries only reach the server log with warnings and errors
    jvm::logging::bridge(logger(), &[]);
    debug!("Server folder {}", jvm::data_dir::server().display());

    let probe: PathBuf = game::port::assetRoot().join(PROBE_ASSET);
    if !probe.is_file()
    {
        severe!("Missing {}, run beta173_launcher once or set B173_ASSETS to the game assets", probe.display());
        std::process::exit(1);
    }

    game::port::setSide(game::port::Side::Server);

    let args: Vec<String> = std::env::args().skip(1).collect();
    let started: std::io::Result<std::thread::JoinHandle<()>> =
        jvm::thread::start(jvm::string::JString::from("Server thread"), move || MinecraftServer::main(&args));

    match started
    {
        Ok(thread) =>
        {
            if let Err(payload) = thread.join()
            {
                jvm::system::errPrintln(&format!(
                    "Exception in thread \"Server thread\" {}",
                    jvm::throwable::fromPayload(payload)
                ));
                std::process::exit(1);
            }
        }

        Err(error) => jvm::io::fromStd(&error).printStackTrace(),
    }
}
