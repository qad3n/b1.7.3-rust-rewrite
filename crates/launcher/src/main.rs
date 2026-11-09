mod assets;

use anyhow::{Context, Result, bail};
use jvm::log_files::{self, LogFormatter};
use jvm::logging::logger;
use jvm::{info, severe, warning};
use std::io::{BufRead, IsTerminal, Write};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const GAME_BINARY: &str = "beta173";
const NAME_FILE: &str = "name.txt";
const NO_SESSION: &str = "-";
const USAGE: &str = "usage: beta173_launcher [--client-jar <path>] [--game <path>] [--dev]";

struct Options
{
    m_clientJar: Option<PathBuf>,
    m_game: Option<PathBuf>,
    m_help: bool,
    m_dev: bool,
}

fn main() -> ExitCode
{
    match run()
    {
        Ok(code) => code,
        Err(error) =>
        {
            severe!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode>
{
    let options: Options = parseArgs(std::env::args().skip(1))?;
    if options.m_help
    {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }
    let dataDir: PathBuf = jvm::data_dir::root();
    log_files::install(logger(), &dataDir.join("logs"), "launcher", LogFormatter::boxed);
    let name: String = askName(&dataDir)?;
    assets::install(&jvm::data_dir::assets(), &dataDir, options.m_clientJar)?;
    let game: PathBuf = match options.m_game
    {
        Some(path) => path,
        None => gameNextToLauncher()?,
    };
    info!("Starting the game as {name}");
    let mut command: Command = Command::new(&game);
    command.arg(&name).arg(NO_SESSION);
    // a launcher in a terminal keeps the game attached so its log stays in that window
    let console: bool = std::io::stdout().is_terminal();
    if !(options.m_dev || console)
    {
        detach(&mut command);
        command.spawn().with_context(|| format!("starting {}", game.display()))?;
        return Ok(ExitCode::SUCCESS);
    }

    let status: std::process::ExitStatus = command.status().with_context(|| format!("starting {}", game.display()))?;
    info!("Game exited with {status}");
    if console && !status.success()
    {
        holdWindow();
    }
    Ok(status.code().map_or(ExitCode::FAILURE, |code| ExitCode::from(code as u8)))
}

// a terminal the desktop opened for the launcher closes with it, so a failed run waits until the log is read
fn holdWindow()
{
    print!("Press Enter to close");
    if let Err(error) = std::io::stdout().flush().and_then(|()| std::io::stdin().lock().read_line(&mut String::new()))
    {
        warning!("could not wait for Enter: {error}");
    }
}

// without a terminal the game outlives the launcher, it still writes its log file
fn detach(command: &mut Command)
{
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(command, 0);
    #[cfg(windows)]
    {
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        std::os::windows::process::CommandExt::creation_flags(command, DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
}

fn parseArgs(mut args: impl Iterator<Item = String>) -> Result<Options>
{
    let mut options: Options = Options { m_clientJar: None, m_game: None, m_help: false, m_dev: false };
    while let Some(arg) = args.next()
    {
        match arg.as_str()
        {
            "help" | "--help" | "-h" => options.m_help = true,
            "--client-jar" => options.m_clientJar = Some(flagValue(&mut args, &arg)?),
            "--game" => options.m_game = Some(flagValue(&mut args, &arg)?),
            "--dev" => options.m_dev = true,
            _ => bail!("unknown argument {arg}\n{USAGE}"),
        }
    }
    Ok(options)
}

fn flagValue(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf>
{
    args.next().map(PathBuf::from).with_context(|| format!("{flag} needs a path"))
}

fn askName(dataDir: &Path) -> Result<String>
{
    let path: PathBuf = dataDir.join(NAME_FILE);
    let saved: Option<String> = match path.is_file()
    {
        true => Some(std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?),
        false => None,
    };
    if let Some(last) = saved.map(|text| text.trim().to_owned()).filter(|name| isValidName(name))
    {
        println!("  1. Play as {last}");
        println!("  2. Change name");
        loop
        {
            match ask("Choose 1 or 2 [1]: ")?.as_str()
            {
                "1" | "" => return Ok(last),
                "2" => break,
                _ => println!("Type 1 or 2"),
            }
        }
    }
    let name: String = askNewName()?;
    std::fs::create_dir_all(dataDir).with_context(|| format!("creating {}", dataDir.display()))?;
    std::fs::write(&path, &name).with_context(|| format!("writing {}", path.display()))?;
    Ok(name)
}

fn askNewName() -> Result<String>
{
    loop
    {
        let name: String = ask("Offline name: ")?;
        if isValidName(&name)
        {
            return Ok(name);
        }
        println!("Use 3 to 16 letters, digits or underscores");
    }
}

fn isValidName(name: &str) -> bool
{
    let lengths: RangeInclusive<usize> = 3..=16;
    lengths.contains(&name.len()) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn ask(question: &str) -> Result<String>
{
    print!("{question}");
    std::io::stdout().flush().context("writing the prompt")?;
    let mut line: String = String::new();
    if std::io::stdin().lock().read_line(&mut line).context("reading the answer")? == 0
    {
        bail!("no answer was entered");
    }
    Ok(line.trim().to_owned())
}

fn gameNextToLauncher() -> Result<PathBuf>
{
    let launcher: PathBuf = std::env::current_exe().context("locating the launcher")?;
    Ok(launcher.with_file_name(format!("{GAME_BINARY}{}", std::env::consts::EXE_SUFFIX)))
}
