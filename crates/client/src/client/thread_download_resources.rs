use crate::client::Minecraft;
use jvm::net::openStream;
use jvm::string::JString;
use jvm::throwable::Throwable;
use std::cell::Cell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const RESOURCES_URL: &str = "http://s3.amazonaws.com/MinecraftResources/";

pub struct ThreadDownloadResources
{
    pub m_resourcesFolder: PathBuf,
    m_mc: Rc<Minecraft>,
    m_closing: Cell<bool>,
    pub m_name: String,
    pub m_daemon: bool,
}

fn decodeEntities(text: &str) -> String
{
    text.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

fn childText(element: &str, tag: &str) -> Result<String, Throwable>
{
    let open: String = format!("<{tag}>");
    let close: String = format!("</{tag}>");
    let start: usize =
        element.find(&open).ok_or_else(|| Throwable::new(&jvm::throwable::NULL_POINTER, None))? + open.len();
    let end: usize =
        element[start..].find(&close).ok_or_else(|| Throwable::new(&jvm::throwable::NULL_POINTER, None))?;
    Ok(decodeEntities(&element[start..start + end]))
}

impl ThreadDownloadResources
{
    pub fn new(gameDir: &Path, minecraft: Rc<Minecraft>) -> ThreadDownloadResources
    {
        let resourcesFolder: PathBuf = gameDir.join("resources/");
        if !resourcesFolder.exists() && std::fs::create_dir_all(&resourcesFolder).is_err()
        {
            jvm::throwable::throw(
                &jvm::throwable::RUNTIME,
                &format!("The working directory could not be created: {}", resourcesFolder.display()),
            );
        }
        ThreadDownloadResources {
            m_resourcesFolder: resourcesFolder,
            m_mc: minecraft,
            m_closing: Cell::new(false),
            m_name: "Resource download thread".to_owned(),
            m_daemon: true,
        }
    }

    fn listContents(&self) -> Result<(), Throwable>
    {
        let document: String = String::from_utf8_lossy(&openStream(RESOURCES_URL)?).into_owned();
        let nodelist: Vec<&str> = document.split("<Contents>").skip(1).collect();
        for i in 0..2
        {
            for node in &nodelist
            {
                let element: &str = node.split("</Contents>").next().unwrap_or("");
                let s: String = childText(element, "Key")?;
                let k: i64 = jvm::integer::parseLong(&JString::from(childText(element, "Size")?.as_str()))?;
                if k > 0
                {
                    self.downloadAndInstallResource(RESOURCES_URL, &s, k, i);
                    if self.m_closing.get()
                    {
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn run(&self)
    {
        if let Err(exception) = self.listContents()
        {
            self.loadResource(&self.m_resourcesFolder, "");
            exception.printStackTrace();
        }
    }

    pub fn reloadResources(&self) { self.loadResource(&self.m_resourcesFolder, ""); }

    fn loadResource(&self, dir: &Path, prefix: &str)
    {
        let afile: Vec<PathBuf> = match std::fs::read_dir(dir)
        {
            Ok(entries) => entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect(),
            Err(_) => jvm::throwable::npe(),
        };
        for file in afile
        {
            let name: String = file.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
            if file.is_dir()
            {
                self.loadResource(&file, &format!("{prefix}{name}/"));
            }
            else if jvm::throwable::catchThrowable(&jvm::throwable::EXCEPTION, || {
                self.m_mc.installResource(&format!("{prefix}{name}"), &file)
            })
            .is_err()
            {
                jvm::system::println(&format!("Failed to add {prefix}{name}"));
            }
        }
    }

    fn installFrom(&self, url: &str, name: &str, size: i64, pass: i32) -> Result<(), Throwable>
    {
        let Some(i) = name.find('/')
        else
        {
            return Err(Throwable::new(
                &jvm::throwable::STRING_INDEX,
                Some("String index out of range: -1".to_owned()),
            ));
        };
        let s: &str = &name[..i];
        if s != "sound" && s != "newsound"
        {
            if pass != 1
            {
                return Ok(());
            }
        }
        else if pass != 0
        {
            return Ok(());
        }
        let file1: PathBuf = self.m_resourcesFolder.join(name);
        let length: i64 = std::fs::metadata(&file1).map_or(0, |metadata| metadata.len() as i64);
        if !file1.exists() || length != size
        {
            if let Some(parent) = file1.parent()
            {
                let _ = std::fs::create_dir_all(parent);
            }
            let s1: String = name.replace(' ', "%20");
            self.downloadResource(&format!("{url}{s1}"), &file1, size)?;
            if self.m_closing.get()
            {
                return Ok(());
            }
        }
        self.m_mc.installResource(name, &file1);
        Ok(())
    }

    fn downloadAndInstallResource(&self, url: &str, name: &str, size: i64, pass: i32)
    {
        if let Err(exception) = self.installFrom(url, name, size, pass)
        {
            exception.printStackTrace();
        }
    }

    fn downloadResource(&self, url: &str, dst: &Path, _size: i64) -> Result<(), Throwable>
    {
        let body: Vec<u8> = openStream(url)?;
        let mut output: std::fs::File =
            std::fs::File::create(dst).map_err(|error| jvm::io::fileNotFound(dst, &error))?;
        for chunk in body.chunks(4096)
        {
            output.write_all(chunk).map_err(|error| jvm::io::fromStd(&error))?;
            if self.m_closing.get()
            {
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn closeMinecraft(&self) { self.m_closing.set(true); }
}
