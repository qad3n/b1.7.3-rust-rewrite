use crate::entity::{Entity, entity_player};
use crate::item::ItemStack;
use crate::math::math_helper;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Packet20NamedEntitySpawn
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_name: Option<JString>,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
    pub m_rotation: i8,
    pub m_pitch: i8,
    pub m_currentItem: i32,
}

impl Packet20NamedEntitySpawn
{
    pub fn new(player: &Entity) -> Self
    {
        let itemStack: Option<Rc<ItemStack>> = entity_player::inventory(player).getCurrentItem();
        Self {
            m_entityId: player.m_entityId.get(),
            m_name: entity_player::fields(player).m_username.get(),
            m_xPosition: math_helper::floor_double(player.m_posX.get() * 32.0),
            m_yPosition: math_helper::floor_double(player.m_posY.get() * 32.0),
            m_zPosition: math_helper::floor_double(player.m_posZ.get() * 32.0),
            m_rotation: (player.m_rotationYaw.get() * 256.0 / 360.0) as i32 as i8,
            m_pitch: (player.m_rotationPitch.get() * 256.0 / 360.0) as i32 as i8,
            m_currentItem: match itemStack
            {
                None => 0,
                Some(itemStack) => itemStack.m_itemID.get(),
            },
            ..Self::default()
        }
    }
}

impl Packet for Packet20NamedEntitySpawn
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        self.m_name = Some(packet::readString(input, 16)?);
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = input.readInt()?;
        self.m_zPosition = input.readInt()?;
        self.m_rotation = input.readByte()?;
        self.m_pitch = input.readByte()?;
        self.m_currentItem = input.readShort()? as i32;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_entityId)?;
        packet::writeString(self.m_name.as_ref().orNpe(), output)?;
        output.writeInt(self.m_xPosition)?;
        output.writeInt(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)?;
        output.writeByte(self.m_rotation as i32)?;
        output.writeByte(self.m_pitch as i32)?;
        output.writeShort(self.m_currentItem)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleNamedEntitySpawn(self); }

    fn getPacketSize(&self) -> i32 { 28 }
}
