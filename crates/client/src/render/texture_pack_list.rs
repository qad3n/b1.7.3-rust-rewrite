use crate::client::Minecraft;
use crate::render::TexturePack;
use crate::settings::GameSettings;
// java.io.File calls are port stand-ins
use game::port::file;
use game::resources::{Layout, Pack, Source};
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use jvm::{info, warning};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const ZIP: &str = ".zip";
const META: &str = "pack.mcmeta";

pub struct TexturePackList
{
    m_availableTexturePacks: RefCell<Vec<Rc<TexturePack>>>,
    m_defaultTexturePack: Rc<TexturePack>,
    pub m_selectedTexturePack: Field<Option<Rc<TexturePack>>>,
    m_availablePacksByKey: RefCell<HashMap<String, Rc<TexturePack>>>,
    m_mc: Rc<Minecraft>,
    m_texturePackDir: PathBuf,
    m_currentTexturePack: Field<String>,
}

// beta lists zips only and a folder counts once it holds a modern pack.mcmeta
fn listed(path: &Path) -> bool
{
    match path.is_dir()
    {
        true => path.join(META).is_file(),
        false => path.is_file() && JString::from(file::getName(path).as_str()).toLowerCase().to_string().ends_with(ZIP),
    }
}

fn open(path: &Path) -> Pack
{
    match Pack::open(path)
    {
        Ok(pack) => pack,

        Err(reason) =>
        {
            warning!("Texture pack {} could not be read: {reason}", path.display());
            // beta still lists a broken zip and a folder source at a file path reads nothing
            Pack::new(&file::getName(path), &[], Layout::Legacy, Source::Folder(path.to_path_buf()))
        }
    }
}

impl TexturePackList
{
    pub fn new(minecraft: &Rc<Minecraft>, gameDir: &Path) -> Rc<Self>
    {
        let defaultPack: Rc<TexturePack> = Rc::new(TexturePack::new(game::resources::vanilla(), None));
        let texturePackDir: PathBuf = gameDir.join("texturepacks");
        if !file::exists(&texturePackDir)
        {
            file::mkdirs(&texturePackDir);
        }

        let settings: Rc<GameSettings> = minecraft.m_gameSettings.get().orNpe();

        let list: Rc<Self> = Rc::new(Self
        {
            m_availableTexturePacks: RefCell::new(Vec::new()),
            m_defaultTexturePack: defaultPack,
            m_selectedTexturePack: Field::new(None),
            m_availablePacksByKey: RefCell::new(HashMap::new()),
            m_mc: Rc::clone(minecraft),
            m_texturePackDir: texturePackDir,
            m_currentTexturePack: Field::new(settings.m_skin.get()),
        });
        list.updateAvaliableTexturePacks();
        list
    }

    pub fn selected(&self) -> Rc<TexturePack> { self.m_selectedTexturePack.get().orNpe() }

    // the selected pack goes above the vanilla jar for every resource read
    fn install(&self)
    {
        let selected: Rc<TexturePack> = self.selected();
        let packs: Vec<Rc<Pack>> = match Rc::ptr_eq(&selected, &self.m_defaultTexturePack)
        {
            true => Vec::new(),
            false => vec![Rc::clone(selected.pack())],
        };
        game::resources::install(&packs);
    }

    pub fn setTexturePack(&self, pack: &Rc<TexturePack>) -> bool
    {
        if self
            .m_selectedTexturePack
            .with(|selected| selected.as_ref().is_some_and(|selected| Rc::ptr_eq(selected, pack)))
        {
            return false;
        }

        self.m_currentTexturePack.set(pack.pack().name().to_owned());
        info!("Switching texture pack to {}", self.m_currentTexturePack.get());
        self.m_selectedTexturePack.set(Some(Rc::clone(pack)));

        let settings: Rc<GameSettings> = self.m_mc.m_gameSettings.get().orNpe();
        settings.m_skin.set(self.m_currentTexturePack.get());
        settings.saveOptions();
        self.install();
        true
    }

    pub fn updateAvaliableTexturePacks(&self)
    {
        let previous: Option<Rc<TexturePack>> = self.m_selectedTexturePack.get();
        let mut arraylist: Vec<Rc<TexturePack>> = Vec::new();

        self.m_selectedTexturePack.set(None);
        arraylist.push(Rc::clone(&self.m_defaultTexturePack));

        if file::exists(&self.m_texturePackDir) && file::isDirectory(&self.m_texturePackDir)
        {
            let afile: Vec<PathBuf> = file::listFiles(&self.m_texturePackDir).orNpe();
            for file1 in afile.into_iter().filter(|path| listed(path))
            {
                let name: String = file::getName(&file1);
                let s: String = format!("{name}:{}:{}", file::length(&file1), file::lastModified(&file1));
                if !self.m_availablePacksByKey.borrow().contains_key(&s)
                {
                    let pack: Rc<TexturePack> = Rc::new(TexturePack::new(Rc::new(open(&file1)), Some(s.clone())));
                    self.m_availablePacksByKey.borrow_mut().insert(s.clone(), pack);
                }

                let texturepackbase1: Rc<TexturePack> = Rc::clone(&self.m_availablePacksByKey.borrow()[&s]);
                if texturepackbase1.pack().name() == self.m_currentTexturePack.get().as_str()
                {
                    self.m_selectedTexturePack.set(Some(Rc::clone(&texturepackbase1)));
                }

                arraylist.push(texturepackbase1);
            }
        }

        if self.m_selectedTexturePack.with(Option::is_none)
        {
            self.m_selectedTexturePack.set(Some(Rc::clone(&self.m_defaultTexturePack)));
        }

        let removed: Vec<Rc<TexturePack>> = self
            .m_availableTexturePacks
            .borrow()
            .iter()
            .filter(|pack| !arraylist.iter().any(|kept| Rc::ptr_eq(kept, pack)))
            .cloned()
            .collect();

        for texturepackbase in removed
        {
            texturepackbase.unload(&self.m_mc);

            if let Some(key) = texturepackbase.key()
            {
                self.m_availablePacksByKey.borrow_mut().remove(key);
            }
        }
        *self.m_availableTexturePacks.borrow_mut() = arraylist;

        // a changed or removed pack file must not keep serving reads
        let same: bool = previous.is_some_and(|previous| Rc::ptr_eq(&previous, &self.selected()));
        if !same
        {
            self.install();
        }
    }

    pub fn availableTexturePacks(&self) -> Vec<Rc<TexturePack>> { self.m_availableTexturePacks.borrow().clone() }
}
