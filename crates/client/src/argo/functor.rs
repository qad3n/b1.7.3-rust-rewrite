use crate::argo::ChainedMismatch;

pub trait Described
{
    fn shortForm(&self) -> String;

    fn describe(&self) -> String;
}

pub trait Functor<T, V>: Described
{
    fn matchesNode(&self, node: &T) -> bool;

    fn applyTo(&self, node: &T) -> Result<V, ChainedMismatch>;
}
