use jvm::logging::Logger;
use server::server::{ConsoleLogManager, MinecraftServer};
use std::path::PathBuf;

const PROBE_ASSET: &str = "lang/en_US.lang";

// the log opens before anything else so startup failures reach logs/server.log
fn main()
{
    ConsoleLogManager::init();
    let probe: PathBuf = game::port::assetRoot().join(PROBE_ASSET);
    if !probe.is_file()
    {
        Logger::getLogger("Minecraft")
            .severe(&format!("missing {}, set B173_ASSETS to the game assets", probe.display()));
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
