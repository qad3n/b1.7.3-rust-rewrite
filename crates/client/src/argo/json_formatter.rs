use crate::argo::JsonRootNode;

pub trait JsonFormatter
{
    fn format(&self, json: &JsonRootNode) -> String;
}
