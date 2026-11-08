use crate::block::Block;
use std::sync::OnceLock;

fn table() -> &'static [i8; 256]
{
    static BLOCKS: OnceLock<[i8; 256]> = OnceLock::new();
    BLOCKS.get_or_init(|| {
        let mut blocks: [i8; 256] = [0; 256];
        for i in 0..256
        {
            let mut by: i8 = i as i8;
            if by != 0 && !Block::exists(by as i32 & 0xFF)
            {
                by = 0;
            }
            blocks[i] = by;
        }
        blocks
    })
}

pub struct ChunkBlockMap;

impl ChunkBlockMap
{
    pub fn fill(blocks: &mut [i8])
    {
        let map: &[i8; 256] = table();
        for value in blocks.iter_mut()
        {
            *value = map[(*value as i32 & 0xFF) as usize];
        }
    }
}
