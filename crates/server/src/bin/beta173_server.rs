use server::server::MinecraftServer;

fn main()
{
    game::port::setSide(game::port::Side::Server);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let started: std::io::Result<std::thread::JoinHandle<()>> =
        jvm::thread::start(jvm::string::JString::from("Server thread"), move || MinecraftServer::main(&args));
    match started
    {
        Ok(thread) =>
        {
            if thread.join().is_err()
            {
                std::process::exit(1);
            }
        }
        Err(error) => jvm::io::fromStd(&error).printStackTrace(),
    }
}
