use crate::locale::StringTranslate;

pub struct StatCollector;

impl StatCollector
{
    pub fn translateToLocal(key: &str) -> String { StringTranslate::getInstance().translateKey(key) }

    pub fn translateToLocalFormatted(key: &str, args: &[String]) -> String
    {
        StringTranslate::getInstance().translateKeyFormat(key, args)
    }
}
