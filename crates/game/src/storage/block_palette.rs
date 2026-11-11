use crate::world::Chunk;
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList};
use common::world::NibbleArray;
use jvm::string::JString;
use std::collections::HashMap;
use std::rc::Rc;

// rewrite: modded worlds save blocks by name in 16 high sections, so ids a mod set reassigns never corrupt a chunk
mod keys
{
    pub const SECTIONS: &str = "Sections";
    pub const Y: &str = "Y";
    pub const PALETTE: &str = "Palette";
    pub const STATES: &str = "BlockStates";
    pub const NAME: &str = "Name";
    pub const PROPERTIES: &str = "Properties";
    // the numeric id and metadata at save time, what a reader without the names falls back to
    pub const ID: &str = "Id";
    pub const META: &str = "Meta";
}

mod layout
{
    pub const SECTION_HEIGHT: i32 = 16;
    pub const SECTION_VOLUME: usize = 4096;
    // McRegion arrays index blocks as (x << 4 | z) * height + y, sections as y << 8 | z << 4 | x
    pub const CHUNK_X_SHIFT: usize = 4;
    pub const SECTION_Z_SHIFT: usize = 4;
    pub const SECTION_Y_SHIFT: usize = 8;
    pub const AXIS_MASK: usize = 15;
}

mod bits
{
    pub const NIBBLE: usize = 4;
    pub const NIBBLE_MASK: i32 = 15;
    pub const LOW_BYTE_MASK: i32 = 255;
    pub const BYTE: usize = 8;
    pub const BYTE_SHIFT: usize = 3;
    pub const BIT_IN_BYTE: usize = 7;
    // block ids are 12 bits, the Add nibble over the base byte
    pub const BLOCK_ID_MASK: i32 = 0xFFF;
}

type Properties = Vec<(String, String)>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Stored
{
    m_block: i32,
    m_meta: i32,
}

fn chunkIndex(height: usize, section: i32, entry: usize) -> usize
{
    let x: usize = entry & layout::AXIS_MASK;
    let z: usize = (entry >> layout::SECTION_Z_SHIFT) & layout::AXIS_MASK;
    let y: usize = entry >> layout::SECTION_Y_SHIFT;
    ((x << layout::CHUNK_X_SHIFT) | z) * height + (section * layout::SECTION_HEIGHT) as usize + y
}

// two blocks share a byte, the even index in the low nibble as NibbleArray stores them
fn nibbleShift(index: usize) -> usize { (index & 1) * bits::NIBBLE }

fn nibble(array: &[u8], index: usize) -> i32 { (array[index >> 1] >> nibbleShift(index)) as i32 & bits::NIBBLE_MASK }

fn setNibble(array: &mut [u8], index: usize, value: i32)
{
    let shift: usize = nibbleShift(index);
    let mask: u8 = bits::NIBBLE_MASK as u8;
    array[index >> 1] = (array[index >> 1] & !(mask << shift)) | (((value & bits::NIBBLE_MASK) as u8) << shift);
}

fn bitsFor(count: usize) -> usize { (usize::BITS - (count - 1).leading_zeros()).max(1) as usize }

fn pack(indices: &[usize], width: usize) -> Vec<i8>
{
    let mut bytes: Vec<u8> = vec![0; (indices.len() * width).div_ceil(bits::BYTE)];

    for (entry, index) in indices.iter().enumerate()
    {
        for bit in (0..width).filter(|bit| index >> bit & 1 == 1)
        {
            let at: usize = entry * width + bit;
            bytes[at >> bits::BYTE_SHIFT] |= 1 << (at & bits::BIT_IN_BYTE);
        }
    }

    bytes.into_iter().map(|byte| byte as i8).collect()
}

fn unpack(bytes: &[i8], width: usize, entry: usize) -> usize
{
    (0..width).fold(0, |index, bit|
    {
        let at: usize = entry * width + bit;
        let set: usize =
            bytes.get(at >> bits::BYTE_SHIFT).map_or(0, |byte| (*byte as u8 >> (at & bits::BIT_IN_BYTE)) as usize & 1);
        index | set << bit
    })
}

fn entryTag(stored: Stored) -> Rc<NBTBase>
{
    let tag: Rc<NBTBase> = NBTTagCompound::create();
    let compound: &NBTTagCompound = tag.compound();
    let (name, properties): (String, Properties) = crate::hooks::blockName(stored.m_block, stored.m_meta)
        .unwrap_or_else(|| (format!("loader:numeric/{}", stored.m_block), Vec::new()));
    compound.setString(keys::NAME, Some(JString::from(name.as_str())));
    compound.setShort(keys::ID, stored.m_block as i16);
    compound.setByte(keys::META, stored.m_meta as i8);

    if !properties.is_empty()
    {
        let values: Rc<NBTBase> = NBTTagCompound::create();

        for (key, value) in &properties
        {
            values.compound().setString(key, Some(JString::from(value.as_str())));
        }

        compound.setCompoundTag(keys::PROPERTIES, &values);
    }

    tag
}

// a name the loader knows wins, otherwise the saved id and metadata come back as they were
fn entryState(compound: &NBTTagCompound) -> Stored
{
    let hint: Stored = Stored
    {
        m_block: compound.getShort(keys::ID) as i32 & bits::BLOCK_ID_MASK,
        m_meta: compound.getByte(keys::META) as i32 & bits::NIBBLE_MASK,
    };

    let name: String = compound.getString(keys::NAME).map(|name| name.to_string()).unwrap_or_default();

    let properties: Properties = if compound.hasKey(keys::PROPERTIES)
    {
        let values: Rc<NBTBase> = compound.getCompoundTag(keys::PROPERTIES);
        values
            .compound()
            .getValues()
            .iter()
            .map(|tag| tag.getKey().to_string())
            .map(|key|
            {
                let value: String =
                    values.compound().getString(&key).map(|value| value.to_string()).unwrap_or_default();
                (key, value)
            })
            .collect()
    }
    else
    {
        Vec::new()
    };

    match crate::hooks::blockByName(&name, &properties, hint.m_block, hint.m_meta)
    {
        Some((block, meta)) => Stored { m_block: block, m_meta: meta },
        None => hint,
    }
}

pub(crate) fn present(nbt: &NBTTagCompound) -> bool { nbt.hasKey(keys::SECTIONS) }

pub(crate) fn write(chunk: &Chunk, nbt: &NBTTagCompound)
{
    let blocks: Vec<i8> = chunk.m_blocks.get();
    let add: Option<Vec<u8>> = chunk.m_addBlocks.get();
    let data: Vec<u8> = chunk.data().m_data.get().into_iter().map(|byte| byte as u8).collect();
    let height: usize = chunk.height() as usize;
    let sections: Rc<NBTBase> = NBTTagList::create();

    for section in 0..chunk.height() / layout::SECTION_HEIGHT
    {
        let mut palette: Vec<Stored> = Vec::new();
        let mut lookup: HashMap<Stored, usize> = HashMap::new();
        let indices: Vec<usize> = (0..layout::SECTION_VOLUME)
            .map(|entry|
            {
                let at: usize = chunkIndex(height, section, entry);
                let high: i32 = add.as_deref().map_or(0, |add| nibble(add, at));
                let low: i32 = blocks[at] as i32 & bits::LOW_BYTE_MASK;
                let stored: Stored = Stored { m_block: low | high << bits::BYTE, m_meta: nibble(&data, at) };
                *lookup.entry(stored).or_insert_with(||
                {
                    palette.push(stored);
                    palette.len() - 1
                })
            })
            .collect();

        if palette == [Stored { m_block: 0, m_meta: 0 }]
        {
            continue;
        }

        let tag: Rc<NBTBase> = NBTTagCompound::create();
        tag.compound().setByte(keys::Y, section as i8);

        let entries: Rc<NBTBase> = NBTTagList::create();

        for stored in &palette
        {
            entries.list().setTag(entryTag(*stored));
        }

        tag.compound().setTag(keys::PALETTE, &entries);

        if palette.len() > 1
        {
            tag.compound().setByteArray(keys::STATES, Rc::new(pack(&indices, bitsFor(palette.len()))));
        }

        sections.list().setTag(tag);
    }

    nbt.setTag(keys::SECTIONS, &sections);
}

pub(crate) fn read(chunk: &Chunk, nbt: &NBTTagCompound)
{
    let height: usize = chunk.height() as usize;
    let volume: usize = chunk.volume() as usize;
    let mut blocks: Vec<i8> = vec![0; volume];
    let mut add: Vec<u8> = vec![0; volume / 2];
    let mut data: Vec<u8> = vec![0; volume / 2];
    let mut high: bool = false;
    let sections: Rc<NBTBase> = nbt.getTagList(keys::SECTIONS);
    for slot in 0..sections.list().tagCount()
    {
        let tag: Rc<NBTBase> = sections.list().tagAt(slot);
        let section: i32 = tag.compound().getByte(keys::Y) as i32;
        if !(0..chunk.height() / layout::SECTION_HEIGHT).contains(&section)
        {
            continue;
        }
        let entries: Rc<NBTBase> = tag.compound().getTagList(keys::PALETTE);
        let palette: Vec<Stored> =
            (0..entries.list().tagCount()).map(|entry| entryState(entries.list().tagAt(entry).compound())).collect();

        let Some(first) = palette.first().copied()
        else
        {
            continue;
        };

        let packed: Option<Rc<Vec<i8>>> = tag.compound().getByteArray(keys::STATES);
        let width: usize = bitsFor(palette.len());

        for entry in 0..layout::SECTION_VOLUME
        {
            let stored: Stored =
                packed.as_ref().and_then(|packed| palette.get(unpack(packed, width, entry)).copied()).unwrap_or(first);
            let at: usize = chunkIndex(height, section, entry);
            blocks[at] = stored.m_block as i8;
            setNibble(&mut add, at, stored.m_block >> bits::BYTE);
            setNibble(&mut data, at, stored.m_meta);
            high |= stored.m_block > bits::LOW_BYTE_MASK;
        }
    }

    chunk.m_blocks.set(blocks);
    chunk.m_addBlocks.set(high.then_some(add));
    chunk.m_data.set(Some(Rc::new(NibbleArray::withData(data.into_iter().map(|byte| byte as i8).collect()))));
}
