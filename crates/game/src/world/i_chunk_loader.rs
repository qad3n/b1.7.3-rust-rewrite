use crate::world::{Chunk, World};
use jvm::io::IoResult;
use std::rc::Rc;

pub trait IChunkLoader
{
    fn loadChunk(&self, world: &Rc<World>, chunkX: i32, chunkZ: i32) -> IoResult<Option<Rc<Chunk>>>;
    fn saveChunk(&self, world: &Rc<World>, chunk: &Rc<Chunk>) -> IoResult<()>;
    fn saveExtraChunkData(&self, world: &Rc<World>, chunk: &Rc<Chunk>) -> IoResult<()>;
    fn tick(&self);
    fn saveExtraData(&self);
}
