use crate::argo::{ChainedMismatch, Described, Functor};
use std::rc::Rc;

pub struct LeafFunctor<T, V>
{
    pub m_matchesNode: Box<dyn Fn(&T) -> bool>,
    pub m_typeSafeApplyTo: Box<dyn Fn(&T) -> V>,
    pub m_shortForm: String,
    pub m_toString: String,
}

pub struct FailedNode
{
    m_shortForm: String,
    m_toString: String,
}

impl Described for FailedNode
{
    fn shortForm(&self) -> String { self.m_shortForm.clone() }

    fn describe(&self) -> String { self.m_toString.clone() }
}

impl<T, V> Described for LeafFunctor<T, V>
{
    fn shortForm(&self) -> String { self.m_shortForm.clone() }

    fn describe(&self) -> String { self.m_toString.clone() }
}

impl<T, V> Functor<T, V> for LeafFunctor<T, V>
{
    fn matchesNode(&self, node: &T) -> bool { (self.m_matchesNode)(node) }

    fn applyTo(&self, node: &T) -> Result<V, ChainedMismatch>
    {
        if !self.matchesNode(node)
        {
            let failed: FailedNode = FailedNode { m_shortForm: self.shortForm(), m_toString: self.describe() };
            return Err(ChainedMismatch::createJsonNodeDoesNotMatchJsonNodeSelectorException(Rc::new(failed)));
        }
        Ok((self.m_typeSafeApplyTo)(node))
    }
}
