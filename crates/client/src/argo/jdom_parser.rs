use crate::argo::{JsonListenerToJdomAdapter, JsonRootNode, SajParser};
use jvm::string::JString;
use jvm::throwable::{self, Throwable};

pub struct JdomParser;

impl JdomParser
{
    pub fn parse(&self, json: &JString) -> Result<JsonRootNode, Throwable>
    {
        let mut adapter: JsonListenerToJdomAdapter = JsonListenerToJdomAdapter::new();
        match throwable::catchThrowable(&throwable::IO, || SajParser.parse(json, &mut adapter))
        {
            Ok(parsed) => parsed?,
            Err(_) =>
            {
                throwable::throw(&throwable::RUNTIME, "Coding failure in Argo:  StringWriter gave an IOException")
            }
        }
        Ok(adapter.getDocument())
    }
}
