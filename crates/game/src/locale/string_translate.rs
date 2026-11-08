use jvm::io::FileInputStream;
use jvm::properties::Properties;
use jvm::throwable::OrNpe;
use std::cell::OnceCell;
use std::rc::Rc;

thread_local! {
    static INSTANCE: OnceCell<Rc<StringTranslate>> = const { OnceCell::new() };
}

pub struct StringTranslate
{
    m_translateTable: Properties,
}

fn loadTable(table: &mut Properties, path: &str) -> jvm::io::IoResult<()>
{
    // class path resources load from the assets folder through port
    let mut stream: FileInputStream = crate::port::getResourceAsStream(path).orNpe();
    table.load(&mut stream)
}

impl StringTranslate
{
    fn new() -> StringTranslate
    {
        let mut table: Properties = Properties::new();
        let loaded: jvm::io::IoResult<()> =
            loadTable(&mut table, "/lang/en_US.lang").and_then(|()| loadTable(&mut table, "/lang/stats_US.lang"));
        if let Err(error) = loaded
        {
            error.printStackTrace();
        }
        StringTranslate { m_translateTable: table }
    }

    pub fn getInstance() -> Rc<StringTranslate>
    {
        INSTANCE.with(|instance| Rc::clone(instance.get_or_init(|| Rc::new(StringTranslate::new()))))
    }

    pub fn translateKey(&self, key: &str) -> String { self.m_translateTable.getProperty(key, key) }

    pub fn translateKeyFormat(&self, key: &str, args: &[String]) -> String
    {
        let string: String = self.m_translateTable.getProperty(key, key);
        jvm::format::format(&string, args)
    }

    pub fn translateNamedKey(&self, key: &str) -> String
    {
        self.m_translateTable.getProperty(&format!("{key}.name"), "")
    }
}
