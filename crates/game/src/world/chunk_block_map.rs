use crate::block::Block;
use std::cell::Cell;

const LOW_IDS: usize = 256;

thread_local!
{
    // rebuilt whenever the block list changes, mods register and remap blocks after the first chunk loads
    static TABLE: Cell<Option<(u32, [i8; LOW_IDS])>> = const { Cell::new(None) };
}

fn build() -> [i8; LOW_IDS]
{
    let mut blocks: [i8; LOW_IDS] = [0; LOW_IDS];

    for i in 0..LOW_IDS
    {
        let mut by: i8 = i as i8;
        if by != 0 && !Block::exists(by as i32 & 0xFF)
        {
            by = 0;
        }

        blocks[i] = by;
    }

    blocks
}

fn table() -> [i8; LOW_IDS]
{
    let generation: u32 = Block::generation();
    if let Some((built, blocks)) = TABLE.get()
        && built == generation
    {
        return blocks;
    }

    let blocks: [i8; LOW_IDS] = build();
    TABLE.set(Some((generation, blocks)));
    blocks
}

pub struct ChunkBlockMap;

impl ChunkBlockMap
{
    pub fn fill(blocks: &mut [i8])
    {
        let map: [i8; LOW_IDS] = table();

        for value in blocks.iter_mut()
        {
            *value = map[(*value as i32 & 0xFF) as usize];
        }
    }
}
