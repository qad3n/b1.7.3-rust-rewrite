use crate::argo::{ChainedFunctor, ChainedMismatch, Described, Functor};
use std::rc::Rc;

pub struct JsonNodeSelector<T, U>
{
    m_valueGetter: Rc<dyn Functor<T, U>>,
}

impl<T: 'static, U: 'static> JsonNodeSelector<T, U>
{
    pub fn new(valueGetter: Rc<dyn Functor<T, U>>) -> JsonNodeSelector<T, U>
    {
        JsonNodeSelector { m_valueGetter: valueGetter }
    }

    pub fn matches(&self, jsonNode: &T) -> bool { self.m_valueGetter.matchesNode(jsonNode) }

    pub fn getValue(&self, argument: &T) -> Result<U, ChainedMismatch> { self.m_valueGetter.applyTo(argument) }

    pub fn with<V: 'static>(
        self: &Rc<Self>,
        childJsonNodeSelector: &Rc<JsonNodeSelector<U, V>>,
    ) -> Rc<JsonNodeSelector<T, V>>
    {
        Rc::new(JsonNodeSelector::new(Rc::new(ChainedFunctor::new(Rc::clone(self), Rc::clone(childJsonNodeSelector)))))
    }
}

impl<T, U> Described for JsonNodeSelector<T, U>
{
    fn shortForm(&self) -> String { self.m_valueGetter.shortForm() }

    fn describe(&self) -> String { self.m_valueGetter.describe() }
}
