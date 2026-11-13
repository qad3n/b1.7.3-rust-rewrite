use mod_protocol::abi::{MediaInfo, MediaLimit, MediaStatus};
use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::mpsc::{self, Receiver, TryRecvError};

// yt-dlp takes this prefix as a search, the first hit short enough is kept
const SEARCH: &str = "ytsearch5:";
// youtube's stereo aac track that the game decodes itself so no ffmpeg is needed
const FORMATS: &str = "140/bestaudio[acodec^=mp4a.40.2][audio_channels<=2]";
// yt-dlp exits with this once it reached max downloads
const ENOUGH_DOWNLOADED: i32 = 101;
// most sites give an artist or uploader with a title even when their audio is locked
const YT_DLP_FAILED: &str = "yt-dlp failed: ";
const NAME_TEMPLATE: &str = "%(artist,uploader|)s %(track,title)s";

const WORK_PREFIX: &str = ".import-";
const SOURCE: &str = "source.%(ext)s";
const SONG: &str = "song.ogg";
const SECONDS_PER_MINUTE: i32 = 60;

struct Job
{
    m_id: i32,
    m_result: Receiver<Result<MediaInfo, String>>,
}

thread_local!
{
    static JOBS: RefCell<Vec<Job>> = const { RefCell::new(Vec::new()) };
    static NEXT: Cell<i32> = const { Cell::new(1) };
}

// the download and the encode run off the game thread and the game thread polls the result
pub fn start(dir: PathBuf, cache: PathBuf, text: String, limit: MediaLimit) -> i32
{
    let id: i32 = NEXT.with(|next| next.replace(next.get() + 1));
    let (sender, receiver) = mpsc::channel();

    std::thread::spawn(move ||
    {
        if sender.send(run(&dir, &cache, id, &text, limit)).is_err()
        {
            crate::log::debug(crate::log::LOADER, &format!("song import {id} finished after its world closed"));
        }
    });

    JOBS.with(|jobs| jobs.borrow_mut().push(Job { m_id: id, m_result: receiver }));
    id
}

pub fn status(id: i32) -> MediaStatus
{
    JOBS.with(|jobs|
    {
        let mut jobs: std::cell::RefMut<Vec<Job>> = jobs.borrow_mut();
        let Some(index) = jobs.iter().position(|job| job.m_id == id)
        else
        {
            return MediaStatus::Unknown;
        };

        let status: MediaStatus = match jobs[index].m_result.try_recv()
        {
            Ok(Ok(info)) => MediaStatus::Done(info),
            Ok(Err(reason)) => MediaStatus::Failed(reason),
            Err(TryRecvError::Empty) => return MediaStatus::Running,
            Err(TryRecvError::Disconnected) => MediaStatus::Failed("the import stopped".to_owned()),
        };

        jobs.remove(index);
        status
    })
}

fn run(dir: &Path, cache: &Path, id: i32, text: &str, limit: MediaLimit) -> Result<MediaInfo, String>
{
    let work: PathBuf = dir.join(format!("{WORK_PREFIX}{id}"));
    std::fs::create_dir_all(&work).map_err(|error| format!("could not make {}: {error}", work.display()))?;

    let result: Result<MediaInfo, String> = fetch(dir, cache, &work, text, limit);

    if let Err(error) = std::fs::remove_dir_all(&work)
    {
        crate::log::warn(crate::log::LOADER, &format!("could not remove {}: {error}", work.display()));
    }

    result
}

fn fetch(dir: &Path, cache: &Path, work: &Path, text: &str, limit: MediaLimit) -> Result<MediaInfo, String>
{
    let program: PathBuf = super::tool::ytDlp(cache)?;
    let target: String = target(&program, text)?;
    let (title, seconds, downloaded): (String, i32, PathBuf) = retried(&program, work, &target, limit)?;
    let song: PathBuf = work.join(SONG);
    super::encode::compress(&downloaded, &song)?;

    let bytes: Vec<u8> = std::fs::read(&song).map_err(|error| format!("could not read the song: {error}"))?;
    let size: i64 = i64::try_from(bytes.len()).unwrap_or(i64::MAX);
    if size > limit.bytes
    {
        return Err(format!("the song is {} KB, more than the {} KB allowed", size / 1024, limit.bytes / 1024));
    }

    let hash: String = super::hashOf(&bytes);
    let stored: PathBuf = super::songPath(dir, &hash);
    std::fs::rename(&song, &stored).map_err(|error| format!("could not keep the song: {error}"))?;

    Ok(MediaInfo { hash, title, seconds, bytes: size })
}

// a youtube link downloads as is, any other link becomes a search for the song it names and plain text is a search
fn target(program: &Path, text: &str) -> Result<String, String>
{
    if !super::page::isLink(text)
    {
        return Ok(format!("{SEARCH}{text}"));
    }

    if super::page::youtube(text)
    {
        return Ok(text.to_owned());
    }

    let named: String = match named(program, text)
    {
        Ok(named) => named,
        Err(reason) =>
        {
            crate::log::debug(crate::log::LOADER, &format!("yt-dlp could not name {text}: {reason}"));
            super::page::title(text)?
        }
    };
    Ok(format!("{SEARCH}{named}"))
}

fn named(program: &Path, link: &str) -> Result<String, String>
{
    let output: Output = super::tool::command(program)
        .args(["--no-playlist", "--no-warnings", "--quiet", "--skip-download", "--encoding", "utf-8"])
        .args(["--playlist-items", "1", "--print", NAME_TEMPLATE, "--", link])
        .output()
        .map_err(|error| format!("yt-dlp could not start: {error}"))?;
    checked(&output)?;

    let name: String = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or_default().trim().to_owned();
    if name.is_empty()
    {
        return Err("no title".to_owned());
    }

    Ok(name)
}

// youtube refuses a download now and then and lets the same one through a moment later
fn retried(program: &Path, work: &Path, target: &str, limit: MediaLimit) -> Result<(String, i32, PathBuf), String>
{
    const ATTEMPTS: u32 = 3;
    const PAUSE: std::time::Duration = std::time::Duration::from_secs(2);

    let mut attempt: u32 = 1;
    loop
    {
        match download(program, work, target, limit)
        {
            Err(reason) if attempt < ATTEMPTS && reason.starts_with(YT_DLP_FAILED) =>
            {
                crate::log::debug(crate::log::LOADER, &format!("download attempt {attempt} failed: {reason}"));
                std::thread::sleep(PAUSE);
                attempt += 1;
            }
            result => return result,
        }
    }
}

// yt-dlp prints the title and length before the download and the file path after it
fn download(program: &Path, work: &Path, target: &str, limit: MediaLimit) -> Result<(String, i32, PathBuf), String>
{
    let filter: String = format!("duration <= {}", limit.seconds);
    let output: Output = super::tool::command(program)
        .args(["--no-playlist", "--no-warnings", "--quiet", "--no-simulate", "--fixup", "never", "--encoding", "utf-8"])
        .args(["-f", FORMATS])
        .args(["--match-filter", &filter, "--max-downloads", "1"])
        .args(["--print", "title", "--print", "duration", "--print", "after_move:filepath", "-o"])
        .arg(work.join(SOURCE))
        .arg("--")
        .arg(target)
        .output()
        .map_err(|error| format!("yt-dlp could not start: {error}"))?;
    checked(&output)?;

    let text: String = String::from_utf8_lossy(&output.stdout).into_owned();
    let mut lines: std::str::Lines = text.lines();
    let (Some(title), Some(duration), Some(path)) = (lines.next(), lines.next(), lines.next())
    else
    {
        let minutes: i32 = limit.seconds / SECONDS_PER_MINUTE;
        return Err(format!("nothing was downloaded, the song may be longer than {minutes} minutes"));
    };

    let seconds: i32 = duration.trim().parse::<f64>().map_or(0, |seconds| seconds.ceil() as i32);
    Ok((title.trim().to_owned(), seconds, PathBuf::from(path.trim())))
}

fn checked(output: &Output) -> Result<(), String>
{
    if output.status.success() || output.status.code() == Some(ENOUGH_DOWNLOADED)
    {
        return Ok(());
    }

    let reason: String = String::from_utf8_lossy(&output.stderr).lines().last().unwrap_or_default().trim().to_owned();
    Err(format!("{YT_DLP_FAILED}{reason}"))
}
