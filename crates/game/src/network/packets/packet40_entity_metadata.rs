use crate::entity::{DataWatcher, WatchableObject};
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
// watched data crosses the network threads as plain port data
use crate::port::watchable_data::WatchableData;
use jvm::io::IoResult;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Packet40EntityMetadata
{
    m_base: PacketBase,
    pub m_entityId: i32,
    m_dataEntries: Option<Vec<WatchableData>>,
}

impl Packet40EntityMetadata
{
    pub fn new(id: i32, syncedData: &DataWatcher) -> Self
    {
        Self {
            m_entityId: id,
            m_dataEntries: WatchableData::fromWatchableObjects(syncedData.getChangedObjects().as_deref()),
            ..Self::default()
        }
    }

    pub fn getDataEntries(&self) -> Option<Vec<Rc<WatchableObject>>>
    {
        WatchableData::toWatchableObjects(self.m_dataEntries.as_deref())
    }
}

impl Packet for Packet40EntityMetadata
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_dataEntries = WatchableData::readWatchableObjects(input)?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        WatchableData::writeObjectsInListToStream(self.m_dataEntries.as_deref(), output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleEntityData(self); }

    fn getPacketSize(&self) -> i32 { 5 }
}
