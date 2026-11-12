use game::tileentity::TileEntity;
use mod_protocol::Identifier;
use mod_protocol::abi::BlockPos;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

pub const TILE_DATA: &str = "loader:tile_data";

#[derive(Serialize, Deserialize)]
pub struct TileData
{
    pub world: i32,
    pub pos: BlockPos,
    pub data: Vec<u8>,
}

pub fn tileChannel() -> Identifier { Identifier::parse(TILE_DATA).expect("the loader channel name is valid") }

// the bytes of a mod tile entity marked sync, none for any other tile
fn message(tile: &TileEntity) -> Option<TileData>
{
    let ext: &crate::tiles::ModTileExt = tile.tryExt::<crate::tiles::ModTileExt>()?;
    let world: Rc<game::world::World> = tile.m_worldObj.get().filter(|_| ext.m_type.m_spec.sync)?;
    Some(TileData
    {
        world: crate::host::context::dimension(&world),
        pos: BlockPos::new(tile.m_xCoord.get(), tile.m_yCoord.get(), tile.m_zCoord.get()),
        data: ext.m_data.borrow().clone(),
    })
}

// mod tile entities marked sync send their bytes to players near them, vanilla only syncs signs
pub fn tileChanged(tile: &Rc<TileEntity>)
{
    if !crate::side::isServer()
    {
        return;
    }
    let (Some(message), Some(world)) = (message(tile), tile.m_worldObj.get())
    else
    {
        return;
    };

    let pos: [f64; 3] = [message.pos.x as f64, message.pos.y as f64, message.pos.z as f64];
    super::sendNear(&tileChannel(), &world, pos, super::Reach::Column, &mod_protocol::wire::encode(&message));
}

// a player who starts seeing a chunk gets the data of its synced mod tiles, which before only followed changes
pub fn chunkTiles(player: &Rc<game::entity::Entity>, chunk: &game::world::Chunk)
{
    for tile in chunk.m_chunkTileEntityMap.get().values()
    {
        if let Some(message) = message(&tile)
        {
            super::sendToPlayer(&tileChannel(), player, mod_protocol::wire::encode(&message));
        }
    }
}
