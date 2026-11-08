use crate::argo::JsonNode;
use std::rc::Rc;

pub trait JsonNodeBuilder
{
    fn build(&self) -> Rc<JsonNode>;

    fn buildKey(&self) -> crate::argo::JsonStringNode { jvm::throwable::throwBare(&jvm::throwable::CLASS_CAST) }
}
