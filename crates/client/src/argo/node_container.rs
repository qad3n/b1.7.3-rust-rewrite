use crate::argo::{JsonFieldBuilder, JsonNodeBuilder};
use std::rc::Rc;

pub trait NodeContainer
{
    fn addNode(&self, node: Rc<dyn JsonNodeBuilder>);

    fn addField(&self, field: Rc<JsonFieldBuilder>);
}
