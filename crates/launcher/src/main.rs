mod assets;

use anyhow::{Context, Result, bail};
use std::io::{BufRead, Write};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const ASSETS_ENV: &str = "B173_ASSETS";
const GAME_DIR_ENV: &str = "B173_MINECRAFT_DIR";
const GAME_BINARY: &str = "beta173";
const DATA_FOLDER: &str = "beta173";
const NAME_FILE: &str = "name.txt";
const NO_SESSION: &str = "-";
const USAGE: &str = "usage: beta173_launcher [--client-jar <path>] [--game <path>]";

struct Options
{
    m_clientJar: Option<PathBuf>,
    m_game: Option<PathBuf>,
    m_help: bool,
}

fn main() -> ExitCode
{
    match run()
    {
        Ok(code) => code,
        Err(error) =>
        {
            eprintln!("beta173_launcher: {error:#}");
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
    let dataDir: PathBuf = dataDir()?;
    let name: String = askName(&dataDir)?;
    let root: PathBuf = assetRoot(&dataDir);
    assets::install(&root, &gameDir()?, options.m_clientJar)?;
    let game: PathBuf = match options.m_game
    {
        Some(path) => path,
        None => gameNextToLauncher()?,
    };
    println!("Starting the game as {name}");
    let status: std::process::ExitStatus = std::process::Command::new(&game)
        .arg(&name)
        .arg(NO_SESSION)
        .env(ASSETS_ENV, &root)
        .status()
        .with_context(|| format!("starting {}", game.display()))?;
    Ok(status.code().map_or(ExitCode::FAILURE, |code| ExitCode::from(code as u8)))
}

fn parseArgs(mut args: impl Iterator<Item = String>) -> Result<Options>
{
    let mut options: Options = Options { m_clientJar: None, m_game: None, m_help: false };
    while let Some(arg) = args.next()
    {
        match arg.as_str()
        {
            "help" | "--help" | "-h" => options.m_help = true,
            "--client-jar" => options.m_clientJar = Some(flagValue(&mut args, &arg)?),
            "--game" => options.m_game = Some(flagValue(&mut args, &arg)?),
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

fn dataDir() -> Result<PathBuf>
{
    let fromEnv = |name: &str| std::env::var_os(name).map(PathBuf::from).with_context(|| format!("{name} is not set"));
    let base: PathBuf = match std::env::consts::OS
    {
        "windows" => fromEnv("APPDATA")?,
        "macos" => fromEnv("HOME")?.join("Library/Application Support"),
        _ => match std::env::var_os("XDG_DATA_HOME").filter(|path| !path.is_empty())
        {
            Some(path) => PathBuf::from(path),
            None => fromEnv("HOME")?.join(".local/share"),
        },
    };
    Ok(base.join(DATA_FOLDER))
}

fn gameDir() -> Result<PathBuf>
{
    if let Some(dir) = std::env::var_os(GAME_DIR_ENV)
    {
        return Ok(PathBuf::from(dir));
    }
    let home: PathBuf = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .context("no home folder is set")?;
    Ok(match std::env::consts::OS
    {
        "linux" | "solaris" => home.join(".minecraft"),
        "windows" => std::env::var_os("APPDATA")
            .map_or_else(|| home.join(".minecraft"), |appData| PathBuf::from(appData).join(".minecraft")),
        "macos" => home.join("Library/Application Support/minecraft"),
        _ => home.join("minecraft"),
    })
}

fn assetRoot(dataDir: &Path) -> PathBuf
{
    match std::env::var_os(ASSETS_ENV)
    {
        Some(path) => PathBuf::from(path),
        None => dataDir.join("assets"),
    }
}

fn gameNextToLauncher() -> Result<PathBuf>
{
    let launcher: PathBuf = std::env::current_exe().context("locating the launcher")?;
    Ok(launcher.with_file_name(format!("{GAME_BINARY}{}", std::env::consts::EXE_SUFFIX)))
}
