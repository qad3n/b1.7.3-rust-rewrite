use game::block::Block;
use jvm::array_list::JArrayList;
use jvm::field::Field;
use std::cell::OnceCell;
use std::rc::Rc;

thread_local! {
    static REGISTERED_BLOCKS_LIST: OnceCell<Rc<JArrayList<Rc<Block>>>> = const { OnceCell::new() };
}

pub struct Session
{
    pub m_username: Field<Option<String>>,
    pub m_sessionId: Field<Option<String>>,
    pub m_mpPassParameter: Field<Option<String>>,
}

fn initRegisteredBlocks() -> Rc<JArrayList<Rc<Block>>>
{
    let list: Rc<JArrayList<Rc<Block>>> = Rc::new(JArrayList::new());
    for block in [
        Block::stone(),
        Block::cobblestone(),
        Block::brick(),
        Block::dirt(),
        Block::planks(),
        Block::wood(),
        Block::leaves(),
        Block::torchWood(),
        Block::stairSingle(),
        Block::glass(),
        Block::cobblestoneMossy(),
        Block::sapling(),
        Block::plantYellow(),
        Block::plantRed(),
        Block::mushroomBrown(),
        Block::mushroomRed(),
        Block::sand(),
        Block::gravel(),
        Block::sponge(),
        Block::cloth(),
        Block::oreCoal(),
        Block::oreIron(),
        Block::oreGold(),
        Block::blockSteel(),
        Block::blockGold(),
        Block::bookShelf(),
        Block::tnt(),
        Block::obsidian(),
    ]
    {
        list.add(block);
    }
    list
}

impl Session
{
    pub fn registeredBlocksList() -> Rc<JArrayList<Rc<Block>>>
    {
        REGISTERED_BLOCKS_LIST.with(|list| Rc::clone(list.get_or_init(initRegisteredBlocks)))
    }

    pub fn new(username: Option<String>, id: Option<String>) -> Session
    {
        Session::registeredBlocksList();
        Session { m_username: Field::new(username), m_sessionId: Field::new(id), m_mpPassParameter: Field::new(None) }
    }
}
