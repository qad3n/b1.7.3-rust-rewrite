use crate::client::Minecraft;
use crate::render::{TexturePackBase, TexturePackCustom, TexturePackDefault};
use crate::settings::GameSettings;
// java.io.File calls are port stand-ins
use game::port::file;
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct TexturePackList
{
    m_availableTexturePacks: RefCell<Vec<Rc<dyn TexturePackBase>>>,
    m_defaultTexturePack: Rc<dyn TexturePackBase>,
    pub m_selectedTexturePack: Field<Option<Rc<dyn TexturePackBase>>>,
    m_availablePacksByKey: RefCell<HashMap<String, Rc<dyn TexturePackBase>>>,
    m_mc: Rc<Minecraft>,
    m_texturePackDir: PathBuf,
    m_currentTexturePack: Field<String>,
}

impl TexturePackList
{
    pub fn new(minecraft: &Rc<Minecraft>, gameDir: &Path) -> Rc<TexturePackList>
    {
        let defaultPack: Rc<dyn TexturePackBase> = Rc::new(TexturePackDefault::new());
        let texturePackDir: PathBuf = gameDir.join("texturepacks");
        if !file::exists(&texturePackDir)
        {
            file::mkdirs(&texturePackDir);
        }
        let settings: Rc<GameSettings> = minecraft.m_gameSettings.get().orNpe();
        let list: Rc<TexturePackList> = Rc::new(TexturePackList {
            m_availableTexturePacks: RefCell::new(Vec::new()),
            m_defaultTexturePack: defaultPack,
            m_selectedTexturePack: Field::new(None),
            m_availablePacksByKey: RefCell::new(HashMap::new()),
            m_mc: Rc::clone(minecraft),
            m_texturePackDir: texturePackDir,
            m_currentTexturePack: Field::new(settings.m_skin.get()),
        });
        list.updateAvaliableTexturePacks();
        list.selected().open();
        list
    }

    pub fn selected(&self) -> Rc<dyn TexturePackBase> { self.m_selectedTexturePack.get().orNpe() }

    pub fn setTexturePack(&self, pack: &Rc<dyn TexturePackBase>) -> bool
    {
        if self
            .m_selectedTexturePack
            .with(|selected| selected.as_ref().is_some_and(|selected| Rc::ptr_eq(selected, pack)))
        {
            return false;
        }
        self.selected().closeTexturePackFile();
        self.m_currentTexturePack.set(pack.pack().m_texturePackFileName.get().orNpe());
        self.m_selectedTexturePack.set(Some(Rc::clone(pack)));
        let settings: Rc<GameSettings> = self.m_mc.m_gameSettings.get().orNpe();
        settings.m_skin.set(self.m_currentTexturePack.get());
        settings.saveOptions();
        self.selected().open();
        true
    }

    pub fn updateAvaliableTexturePacks(&self)
    {
        let mut arraylist: Vec<Rc<dyn TexturePackBase>> = Vec::new();
        self.m_selectedTexturePack.set(None);
        arraylist.push(Rc::clone(&self.m_defaultTexturePack));
        if file::exists(&self.m_texturePackDir) && file::isDirectory(&self.m_texturePackDir)
        {
            let afile: Vec<PathBuf> = file::listFiles(&self.m_texturePackDir).orNpe();
            for file1 in afile
            {
                let name: String = file::getName(&file1);
                if !file1.is_file() || !JString::from(name.as_str()).toLowerCase().to_string().ends_with(".zip")
                {
                    continue;
                }
                let s: String = format!("{name}:{}:{}", file::length(&file1), file::lastModified(&file1));
                if !self.m_availablePacksByKey.borrow().contains_key(&s)
                {
                    let texturepackcustom: Rc<dyn TexturePackBase> = Rc::new(TexturePackCustom::new(&file1));
                    texturepackcustom.pack().m_key.set(Some(s.clone()));
                    self.m_availablePacksByKey.borrow_mut().insert(s.clone(), Rc::clone(&texturepackcustom));
                    if let Err(error) = texturepackcustom.load(&self.m_mc)
                    {
                        error.printStackTrace();
                        continue;
                    }
                }
                let texturepackbase1: Rc<dyn TexturePackBase> = Rc::clone(&self.m_availablePacksByKey.borrow()[&s]);
                if texturepackbase1.pack().m_texturePackFileName.get().as_deref()
                    == Some(self.m_currentTexturePack.get().as_str())
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
        let removed: Vec<Rc<dyn TexturePackBase>> = self
            .m_availableTexturePacks
            .borrow()
            .iter()
            .filter(|pack| !arraylist.iter().any(|kept| Rc::ptr_eq(kept, pack)))
            .cloned()
            .collect();
        for texturepackbase in removed
        {
            texturepackbase.unload(&self.m_mc);
            if let Some(key) = texturepackbase.pack().m_key.get()
            {
                self.m_availablePacksByKey.borrow_mut().remove(&key);
            }
        }
        *self.m_availableTexturePacks.borrow_mut() = arraylist;
    }

    pub fn availableTexturePacks(&self) -> Vec<Rc<dyn TexturePackBase>>
    {
        self.m_availableTexturePacks.borrow().clone()
    }
}
