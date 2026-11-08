use crate::port::{parseInt, text};
use jvm::io::InputStream;
use jvm::throwable::Throwable;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    static INSTANCE: OnceCell<Rc<AchievementMap>> = const { OnceCell::new() };
}

pub struct AchievementMap
{
    m_guidMap: RefCell<HashMap<i32, String>>,
}

fn element<'a>(parts: &[&'a str], index: usize) -> Result<&'a str, Throwable>
{
    match parts.get(index)
    {
        Some(part) => Ok(part),
        None => Err(Throwable::new(&jvm::throwable::ARRAY_INDEX, Some(index.to_string()))),
    }
}

fn load(guidMap: &mut HashMap<i32, String>) -> Result<(), Throwable>
{
    // port getResourceAsStream reads the assets folder in place of the jar
    let Some(mut stream) = crate::port::getResourceAsStream("/achievement/map.txt")
    else
    {
        return Err(Throwable::new(&jvm::throwable::NULL_POINTER, None));
    };
    let mut bytes: Vec<u8> = Vec::new();
    loop
    {
        let value: i32 = stream.read()?;
        if value < 0
        {
            break;
        }
        bytes.push(value as u8);
    }
    // port decodePlatform decodes in the platform charset like a java Reader
    let text: String = text::decodePlatform(&bytes);
    // port readLines splits lines like BufferedReader.readLine
    for line in text::readLines(&text)
    {
        // port split drops trailing empty parts like java String.split
        let parts: Vec<&str> = text::split(line, ',');
        // port parseInt runs java Integer.parseInt on rust text
        let id: i32 = parseInt(element(&parts, 0)?)?;
        guidMap.insert(id, element(&parts, 1)?.to_owned());
    }
    Ok(())
}

impl AchievementMap
{
    fn new() -> AchievementMap
    {
        let mut guidMap: HashMap<i32, String> = HashMap::new();
        if let Err(exception) = load(&mut guidMap)
        {
            exception.printStackTrace();
        }
        AchievementMap { m_guidMap: RefCell::new(guidMap) }
    }

    pub fn instance() -> Rc<AchievementMap>
    {
        INSTANCE.with(|instance| Rc::clone(instance.get_or_init(|| Rc::new(AchievementMap::new()))))
    }

    pub fn getGuid(id: i32) -> Option<String> { AchievementMap::instance().m_guidMap.borrow().get(&id).cloned() }
}
