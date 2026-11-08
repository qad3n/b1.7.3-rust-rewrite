use crate::argo::{EnumJsonNodeType, JsonFormatter, JsonNode, JsonRootNode, JsonStringNode};
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct CompactJsonFormatter;

fn escape(text: &str) -> String
{
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\u{8}', "\\b")
        .replace('\u{c}', "\\f")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

impl CompactJsonFormatter
{
    fn formatNode(&self, jsonNode: &Rc<JsonNode>, writer: &mut String)
    {
        let mut bl: bool = true;
        match jsonNode.getType()
        {
            EnumJsonNodeType::Array =>
            {
                writer.push('[');
                for element in jsonNode.getElements()
                {
                    if !bl
                    {
                        writer.push(',');
                    }
                    bl = false;
                    self.formatNode(&element, writer);
                }
                writer.push(']');
            }
            EnumJsonNodeType::Object =>
            {
                writer.push('{');
                let mut keys: Vec<JsonStringNode> = jsonNode.getFields().keys();
                keys.sort_by(|a, b| a.compareTo(b));
                keys.dedup_by(|a, b| a.compareTo(b).is_eq());
                for key in keys
                {
                    if !bl
                    {
                        writer.push(',');
                    }
                    bl = false;
                    writer.push_str(&format!("\"{}\"", escape(&key.getText().to_string())));
                    writer.push(':');
                    self.formatNode(&jsonNode.getFields().get(&key).orNpe(), writer);
                }
                writer.push('}');
            }
            EnumJsonNodeType::String =>
            {
                writer.push_str(&format!("\"{}\"", escape(&jsonNode.getText().to_string())));
            }
            EnumJsonNodeType::Number => writer.push_str(&jsonNode.getText().to_string()),
            EnumJsonNodeType::False => writer.push_str("false"),
            EnumJsonNodeType::True => writer.push_str("true"),
            EnumJsonNodeType::Null => writer.push_str("null"),
        }
    }
}

impl JsonFormatter for CompactJsonFormatter
{
    fn format(&self, json: &JsonRootNode) -> String
    {
        let mut writer: String = String::new();
        self.formatNode(json, &mut writer);
        writer
    }
}
