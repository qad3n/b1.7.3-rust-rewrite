use crate::argo::{ChainedMismatch, Described, Functor, JsonNodeSelector};
use std::rc::Rc;

pub struct ChainedFunctor<T, U, V>
{
    m_parentJsonNodeSelector: Rc<JsonNodeSelector<T, U>>,
    m_childJsonNodeSelector: Rc<JsonNodeSelector<U, V>>,
}

impl<T, U, V> ChainedFunctor<T, U, V>
{
    pub fn new(parent: Rc<JsonNodeSelector<T, U>>, child: Rc<JsonNodeSelector<U, V>>) -> ChainedFunctor<T, U, V>
    {
        ChainedFunctor { m_parentJsonNodeSelector: parent, m_childJsonNodeSelector: child }
    }
}

impl<T, U, V> Described for ChainedFunctor<T, U, V>
{
    fn shortForm(&self) -> String { self.m_childJsonNodeSelector.shortForm() }

    fn describe(&self) -> String
    {
        format!("{}, with {}", self.m_parentJsonNodeSelector.describe(), self.m_childJsonNodeSelector.describe())
    }
}

impl<T: 'static, U: 'static, V: 'static> Functor<T, V> for ChainedFunctor<T, U, V>
{
    fn matchesNode(&self, node: &T) -> bool
    {
        self.m_parentJsonNodeSelector.matches(node)
            && match self.m_parentJsonNodeSelector.getValue(node)
            {
                Ok(value) => self.m_childJsonNodeSelector.matches(&value),
                Err(mismatch) => mismatch.rethrow(),
            }
    }

    fn applyTo(&self, node: &T) -> Result<V, ChainedMismatch>
    {
        let parent: Rc<dyn Described> = Rc::clone(&self.m_parentJsonNodeSelector) as Rc<dyn Described>;
        let u: U = self.m_parentJsonNodeSelector.getValue(node).map_err(|mismatch| {
            ChainedMismatch::createUnchainedJsonNodeDoesNotMatchJsonNodeSelectorException(&mismatch, Rc::clone(&parent))
        })?;
        self.m_childJsonNodeSelector.getValue(&u).map_err(|mismatch| {
            ChainedMismatch::createChainedJsonNodeDoesNotMatchJsonNodeSelectorException(&mismatch, parent)
        })
    }
}
