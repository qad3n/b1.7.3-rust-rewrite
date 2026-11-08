use crate::entity::Entity;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Packet39AttachEntity
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_vehicleEntityId: i32,
}

impl Packet39AttachEntity
{
    pub fn new(entity: &Entity, mount: Option<&Rc<Entity>>) -> Self
    {
        Self {
            m_entityId: entity.m_entityId.get(),
            m_vehicleEntityId: match mount
            {
                Some(mount) => mount.m_entityId.get(),
                None => -1,
            },
            ..Self::default()
        }
    }
}

impl Packet for Packet39AttachEntity
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_vehicleEntityId = input.readInt()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        output.writeInt(self.m_vehicleEntityId)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleRideEntity(self); }

    fn getPacketSize(&self) -> i32 { 8 }
}
