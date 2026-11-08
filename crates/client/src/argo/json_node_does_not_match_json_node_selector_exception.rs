use crate::argo::Described;
use jvm::throwable::{self, ILLEGAL_ARGUMENT, ThrowableClass};
use std::rc::Rc;

pub static JSON_NODE_DOES_NOT_MATCH_JSON_NODE_SELECTOR_EXCEPTION: ThrowableClass = ThrowableClass {
    m_name: "net.minecraft.src.J_JsonNodeDoesNotMatchJsonNodeSelectorException",
    m_parent: Some(&ILLEGAL_ARGUMENT),
};

pub static CHAINED_EXCEPTION: ThrowableClass = ThrowableClass {
    m_name: "net.minecraft.src.J_JsonNodeDoesNotMatchChainedJsonNodeSelectorException",
    m_parent: Some(&JSON_NODE_DOES_NOT_MATCH_JSON_NODE_SELECTOR_EXCEPTION),
};

pub static PATH_ELEMENTS_EXCEPTION: ThrowableClass = ThrowableClass {
    m_name: "net.minecraft.src.J_JsonNodeDoesNotMatchPathElementsException",
    m_parent: Some(&JSON_NODE_DOES_NOT_MATCH_JSON_NODE_SELECTOR_EXCEPTION),
};

pub struct ChainedMismatch
{
    pub m_failedNode: Rc<dyn Described>,
    pub m_failPath: Vec<Rc<dyn Described>>,
}

impl ChainedMismatch
{
    pub fn createJsonNodeDoesNotMatchJsonNodeSelectorException(failedNode: Rc<dyn Described>) -> ChainedMismatch
    {
        ChainedMismatch { m_failedNode: failedNode, m_failPath: Vec::new() }
    }

    pub fn createChainedJsonNodeDoesNotMatchJsonNodeSelectorException(
        e: &ChainedMismatch,
        parentJsonNodeSelector: Rc<dyn Described>,
    ) -> ChainedMismatch
    {
        let mut failPath: Vec<Rc<dyn Described>> = e.m_failPath.clone();
        failPath.push(parentJsonNodeSelector);
        ChainedMismatch { m_failedNode: Rc::clone(&e.m_failedNode), m_failPath: failPath }
    }

    pub fn createUnchainedJsonNodeDoesNotMatchJsonNodeSelectorException(
        e: &ChainedMismatch,
        parentJsonNodeSelector: Rc<dyn Described>,
    ) -> ChainedMismatch
    {
        ChainedMismatch { m_failedNode: Rc::clone(&e.m_failedNode), m_failPath: vec![parentJsonNodeSelector] }
    }

    pub fn getShortFormFailPath(&self) -> String
    {
        let parts: Vec<String> = self.m_failPath.iter().rev().map(|selector| selector.shortForm()).collect();
        parts.join(".")
    }

    pub fn getMessage(&self) -> String { format!("Failed to match any JSON node at [{}]", self.getShortFormFailPath()) }

    pub fn rethrow(&self) -> ! { throwable::throw(&CHAINED_EXCEPTION, &self.getMessage()) }
}
