use crate::world::{Chunk, ChunkProviderLoadOrGenerate};
use common::world::IProgressUpdate;
use std::rc::Rc;

pub trait IChunkProvider
{
    fn chunkExists(&self, chunkX: i32, chunkZ: i32) -> bool;
    fn provideChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>;
    fn prepareChunk(&self, chunkX: i32, chunkZ: i32) -> Rc<Chunk>;
    fn populate(&self, source: &Rc<dyn IChunkProvider>, chunkX: i32, chunkZ: i32);
    fn saveChunks(&self, saveEntities: bool, listener: Option<&dyn IProgressUpdate>) -> bool;
    fn unload100OldestChunks(&self) -> bool;
    fn canSave(&self) -> bool;
    fn makeString(&self) -> String;

    fn asLoadOrGenerate(&self) -> Option<Rc<ChunkProviderLoadOrGenerate>> { None }

    fn releaseChunks(&self) -> Vec<Rc<Chunk>> { Vec::new() }
}
