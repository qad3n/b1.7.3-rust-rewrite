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
    // packs above the jar lowest first
    m_selected: RefCell<Vec<Rc<TexturePack>>>,
    m_availablePacksByKey: RefCell<HashMap<String, Rc<TexturePack>>>,
    m_mc: Rc<Minecraft>,
    m_texturePackDir: PathBuf,
    m_names: Field<Vec<String>>,
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

fn samePacks(left: &[Rc<TexturePack>], right: &[Rc<TexturePack>]) -> bool
{
    left.len() == right.len() && left.iter().zip(right).all(|(old, new)| Rc::ptr_eq(old, new))
}

// the beta skin option names one pack and Default means none
fn savedNames(settings: &GameSettings) -> Vec<String>
{
    settings.m_texturePacks.get().unwrap_or_else(||
    {
        let skin: String = settings.m_skin.get();
        if skin == game::resources::vanilla().name()
        {
            Vec::new()
        }
        else
        {
            vec![skin]
        }
    })
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
            m_selected: RefCell::new(Vec::new()),
            m_availablePacksByKey: RefCell::new(HashMap::new()),
            m_mc: Rc::clone(minecraft),
            m_texturePackDir: texturePackDir,
            m_names: Field::new(savedNames(&settings)),
        });
        list.updateAvaliableTexturePacks();
        list
    }

    pub fn defaultPack(&self) -> Rc<TexturePack> { Rc::clone(&self.m_defaultTexturePack) }

    pub fn selected(&self) -> Vec<Rc<TexturePack>> { self.m_selected.borrow().clone() }

    // the selected packs go above the vanilla jar for every resource read
    fn install(&self)
    {
        let packs: Vec<Rc<Pack>> = self.m_selected.borrow().iter().map(|pack| Rc::clone(pack.pack())).collect();
        game::resources::install(&packs);
    }

    // true when the stack changed and resources need a reload
    pub fn setSelection(&self, packs: Vec<Rc<TexturePack>>) -> bool
    {
        if samePacks(&self.m_selected.borrow(), &packs)
        {
            return false;
        }

        let names: Vec<String> = packs.iter().map(|pack| pack.pack().name().to_owned()).collect();
        let top: String = names.last().cloned().unwrap_or_else(|| game::resources::vanilla().name().to_owned());
        info!(
            "Switching texture packs to {}",
            if names.is_empty()
            {
                top.clone()
            }
            else
            {
                names.join(", ")
            }
        );
        self.m_names.set(names.clone());
        *self.m_selected.borrow_mut() = packs;

        let settings: Rc<GameSettings> = self.m_mc.m_gameSettings.get().orNpe();
        settings.m_skin.set(top);
        settings.m_texturePacks.set(Some(names));
        settings.saveOptions();
        self.install();
        true
    }

    pub fn updateAvaliableTexturePacks(&self)
    {
        let previous: Vec<Rc<TexturePack>> = self.selected();
        let mut arraylist: Vec<Rc<TexturePack>> = Vec::new();

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

                arraylist.push(Rc::clone(&self.m_availablePacksByKey.borrow()[&s]));
            }
        }

        // saved names keep their order and a pack whose file is gone drops out
        let selected: Vec<Rc<TexturePack>> = self
            .m_names
            .get()
            .iter()
            .filter_map(|name| arraylist.iter().skip(1).find(|pack| pack.pack().name() == name.as_str()).cloned())
            .collect();

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
        let same: bool = samePacks(&previous, &selected);
        *self.m_selected.borrow_mut() = selected;

        if !same
        {
            self.install();
        }
    }

    pub fn availableTexturePacks(&self) -> Vec<Rc<TexturePack>> { self.m_availableTexturePacks.borrow().clone() }
}
