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

// mod tile entities marked sync send their bytes to players near them, vanilla only syncs signs
pub fn tileChanged(tile: &Rc<TileEntity>)
{
    let Some(ext) = tile.tryExt::<crate::tiles::ModTileExt>()
    else
    {
        return;
    };
    if !ext.m_type.m_spec.sync || !crate::side::isServer()
    {
        return;
    }
    let Some(world) = tile.m_worldObj.get()
    else
    {
        return;
    };
    let message: TileData = TileData {
        world: crate::host::context::dimension(&world),
        pos: BlockPos::new(tile.m_xCoord.get(), tile.m_yCoord.get(), tile.m_zCoord.get()),
        data: ext.m_data.borrow().clone(),
    };
    let bytes: Vec<u8> = mod_protocol::wire::encode(&message);
    for player in world.m_playerEntities.toVec()
    {
        let dx: f64 = player.m_posX.get() - message.pos.x as f64;
        let dz: f64 = player.m_posZ.get() - message.pos.z as f64;
        if dx * dx + dz * dz < 64.0 * 64.0
        {
            super::sendToPlayer(&tileChannel(), &player, bytes.clone());
        }
    }
}
