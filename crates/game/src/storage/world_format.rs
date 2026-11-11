use jvm::calendar::{self, DateFields};
use jvm::{info, warning};
use std::path::{Path, PathBuf};

// rewrite: kept beside level.dat so level.dat itself stays byte for byte vanilla
const STAMP_FILE: &str = "rewrite.format";
const BACKUP_DIR: &str = "backups";
// raise when a new step joins upgrade so worlds stamped earlier convert and back up again
pub(crate) const CURRENT: i32 = 1;

fn stampPath(world: &Path) -> PathBuf { world.join(STAMP_FILE) }

pub(crate) fn stamped(world: &Path) -> i32
{
    let path: PathBuf = stampPath(world);
    let Ok(text) = std::fs::read_to_string(&path)
    else
    {
        return 0;
    };

    text.trim().parse().unwrap_or_else(|_|
    {
        warning!("Unreadable world format stamp in {}, treating the world as unconverted", path.display());
        0
    })
}

pub(crate) fn current(world: &Path) -> bool { stamped(world) >= CURRENT }

pub(crate) fn stamp(world: &Path) -> std::io::Result<()> { std::fs::write(stampPath(world), CURRENT.to_string()) }

fn copyTree(from: &Path, to: &Path) -> std::io::Result<u64>
{
    std::fs::create_dir_all(to)?;

    let mut bytes: u64 = 0;

    for entry in std::fs::read_dir(from)?
    {
        let entry: std::fs::DirEntry = entry?;
        let target: PathBuf = to.join(entry.file_name());

        bytes += if entry.file_type()?.is_dir()
        {
            copyTree(&entry.path(), &target)?
        }
        else
        {
            std::fs::copy(entry.path(), &target)?
        };
    }

    Ok(bytes)
}

fn backupName(saveName: &str) -> String
{
    let date: DateFields = calendar::dateFields(jvm::system::currentTimeMillis());
    format!(
        "{saveName}-{:04}-{:02}-{:02}_{:02}-{:02}-{:02}",
        date.m_year, date.m_month, date.m_day, date.m_hour, date.m_minute, date.m_second
    )
}

// backups sit in their own folder so the world list never shows them
pub(crate) fn backup(savesDir: &Path, saveName: &str) -> std::io::Result<PathBuf>
{
    let target: PathBuf = savesDir.join(BACKUP_DIR).join(backupName(saveName));
    if target.exists()
    {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, format!("{} exists", target.display())));
    }

    let bytes: u64 = copyTree(&savesDir.join(saveName), &target)?;
    info!("Backed up {saveName} ({bytes} bytes) to {}", target.display());
    Ok(target)
}
