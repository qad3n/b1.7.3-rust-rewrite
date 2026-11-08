use crate::entity::{Entity, EntityList, WatchableObject};
use crate::math::math_helper;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
// watched data crosses the network threads as plain port data
use crate::port::watchable_data::WatchableData;
use jvm::io::IoResult;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Packet24MobSpawn
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_type: i8,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
    pub m_yaw: i8,
    pub m_pitch: i8,
    m_metaData: Option<Vec<WatchableData>>,
    m_receivedMetadata: Option<Vec<WatchableData>>,
}

impl Packet24MobSpawn
{
    pub fn new(entity: &Entity) -> Self
    {
        Self {
            m_entityId: entity.m_entityId.get(),
            m_type: EntityList::getEntityID(entity) as i8,
            m_xPosition: math_helper::floor_double(entity.m_posX.get() * 32.0),
            m_yPosition: math_helper::floor_double(entity.m_posY.get() * 32.0),
            m_zPosition: math_helper::floor_double(entity.m_posZ.get() * 32.0),
            m_yaw: (entity.m_rotationYaw.get() * 256.0 / 360.0) as i32 as i8,
            m_pitch: (entity.m_rotationPitch.get() * 256.0 / 360.0) as i32 as i8,
            m_metaData: WatchableData::fromWatchableObjects(Some(&entity.getDataWatcher().getAllWatched())),
            ..Self::default()
        }
    }

    pub fn getMetadata(&self) -> Option<Vec<Rc<WatchableObject>>>
    {
        WatchableData::toWatchableObjects(self.m_receivedMetadata.as_deref())
    }
}

impl Packet for Packet24MobSpawn
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_type = input.readByte()?;
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = input.readInt()?;
        self.m_zPosition = input.readInt()?;
        self.m_yaw = input.readByte()?;
        self.m_pitch = input.readByte()?;
        self.m_receivedMetadata = WatchableData::readWatchableObjects(input)?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        output.writeByte(self.m_type as i32)?;
        output.writeInt(self.m_xPosition)?;
        output.writeInt(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)?;
        output.writeByte(self.m_yaw as i32)?;
        output.writeByte(self.m_pitch as i32)?;
        match &self.m_metaData
        {
            None => jvm::throwable::npe(),
            Some(metaData) => WatchableData::writeObjectsInListToStream(Some(metaData), output),
        }
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleMobSpawn(self); }

    fn getPacketSize(&self) -> i32 { 20 }
}
