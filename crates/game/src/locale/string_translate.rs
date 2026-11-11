use jvm::io::ByteArrayInputStream;
use jvm::properties::Properties;
use jvm::throwable::OrNpe;
use std::cell::OnceCell;
use std::rc::Rc;

thread_local!
{
    static INSTANCE: OnceCell<Rc<StringTranslate>> = const { OnceCell::new() };
}

// rewrite options the vanilla lang files never had
const REWRITE_KEYS: [(&str, &str); 2] = [("options.vsync", "Use VSync"), ("options.fullscreen", "Fullscreen")];

pub struct StringTranslate
{
    m_translateTable: Properties,
}

fn loadTable(table: &mut Properties, path: &str) -> jvm::io::IoResult<()>
{
    // class path resources load from the assets folder through port
    let mut stream: ByteArrayInputStream = crate::port::getResourceAsStream(path).orNpe();
    table.load(&mut stream)
}

impl StringTranslate
{
    fn new() -> Self
    {
        let mut table: Properties = Properties::new();
        let loaded: jvm::io::IoResult<()> =
            loadTable(&mut table, "/lang/en_US.lang").and_then(|()| loadTable(&mut table, "/lang/stats_US.lang"));

        if let Err(error) = loaded
        {
            error.printStackTrace();
        }

        for (key, text) in REWRITE_KEYS
        {
            if table.getPropertyOrNull(key).is_none()
            {
                table.setProperty(key, text);
            }
        }

        Self { m_translateTable: table }
    }

    pub fn getInstance() -> Rc<Self>
    {
        INSTANCE.with(|instance| Rc::clone(instance.get_or_init(|| Rc::new(Self::new()))))
    }

    // mod lang files are consulted before the vanilla table
    fn lookup(&self, key: &str) -> Option<String>
    {
        crate::hooks::translate(key).or_else(|| self.m_translateTable.getPropertyOrNull(key))
    }

    pub fn translateKey(&self, key: &str) -> String { self.lookup(key).unwrap_or_else(|| key.to_owned()) }

    pub fn translateKeyFormat(&self, key: &str, args: &[String]) -> String
    {
        let string: String = self.lookup(key).unwrap_or_else(|| key.to_owned());
        jvm::format::format(&string, args)
    }

    pub fn translateNamedKey(&self, key: &str) -> String { self.lookup(&format!("{key}.name")).unwrap_or_default() }
}
