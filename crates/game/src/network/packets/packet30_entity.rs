use crate::network::net_handler::NetHandler;
use crate::network::packet::{Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet30Entity
{
    m_base: PacketBase,
    pub m_entityId: i32,
    pub m_xPosition: i8,
    pub m_yPosition: i8,
    pub m_zPosition: i8,
    pub m_yaw: i8,
    pub m_pitch: i8,
    pub m_rotating: bool,
}

impl Packet30Entity
{
    pub fn new(id: i32) -> Self { Self { m_entityId: id, ..Self::default() } }

    fn readMove(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_xPosition = input.readByte()?;
        self.m_yPosition = input.readByte()?;
        self.m_zPosition = input.readByte()?;
        Ok(())
    }

    fn writeMove(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeByte(self.m_xPosition as i32)?;
        output.writeByte(self.m_yPosition as i32)?;
        output.writeByte(self.m_zPosition as i32)
    }

    fn readLook(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_yaw = input.readByte()?;
        self.m_pitch = input.readByte()?;
        Ok(())
    }

    fn writeLook(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeByte(self.m_yaw as i32)?;
        output.writeByte(self.m_pitch as i32)
    }
}

impl Packet for Packet30Entity
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entityId = input.readInt()?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()> { output.writeInt(self.m_entityId) }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleEntity(self); }

    fn getPacketSize(&self) -> i32 { 4 }
}

#[derive(Clone)]
pub struct Packet33RelEntityMoveLook
{
    pub m_entity: Packet30Entity,
}

impl Default for Packet33RelEntityMoveLook
{
    fn default() -> Self { Self { m_entity: Packet30Entity { m_rotating: true, ..Packet30Entity::default() } } }
}

impl Packet33RelEntityMoveLook
{
    pub fn new(id: i32, dx: i8, dy: i8, dz: i8, yaw: i8, pitch: i8) -> Self
    {
        let mut entity: Packet30Entity = Packet30Entity::new(id);
        entity.m_xPosition = dx;
        entity.m_yPosition = dy;
        entity.m_zPosition = dz;
        entity.m_yaw = yaw;
        entity.m_pitch = pitch;
        entity.m_rotating = true;
        Self { m_entity: entity }
    }
}

impl Packet for Packet33RelEntityMoveLook
{
    fn base(&self) -> &PacketBase { &self.m_entity.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entity.readPacketData(input)?;
        self.m_entity.readMove(input)?;
        self.m_entity.readLook(input)
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        self.m_entity.writePacketData(output)?;
        self.m_entity.writeMove(output)?;
        self.m_entity.writeLook(output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { self.m_entity.processPacket(handler); }

    fn getPacketSize(&self) -> i32 { 9 }
}

#[derive(Clone)]
pub struct Packet32EntityLook
{
    pub m_entity: Packet30Entity,
}

impl Default for Packet32EntityLook
{
    fn default() -> Self { Self { m_entity: Packet30Entity { m_rotating: true, ..Packet30Entity::default() } } }
}

impl Packet32EntityLook
{
    pub fn new(id: i32, yaw: i8, pitch: i8) -> Self
    {
        let mut entity: Packet30Entity = Packet30Entity::new(id);
        entity.m_yaw = yaw;
        entity.m_pitch = pitch;
        entity.m_rotating = true;
        Self { m_entity: entity }
    }
}

impl Packet for Packet32EntityLook
{
    fn base(&self) -> &PacketBase { &self.m_entity.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entity.readPacketData(input)?;
        self.m_entity.readLook(input)
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        self.m_entity.writePacketData(output)?;
        self.m_entity.writeLook(output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { self.m_entity.processPacket(handler); }

    fn getPacketSize(&self) -> i32 { 6 }
}

#[derive(Clone, Default)]
pub struct Packet31RelEntityMove
{
    pub m_entity: Packet30Entity,
}

impl Packet31RelEntityMove
{
    pub fn new(id: i32, dx: i8, dy: i8, dz: i8) -> Self
    {
        let mut entity: Packet30Entity = Packet30Entity::new(id);
        entity.m_xPosition = dx;
        entity.m_yPosition = dy;
        entity.m_zPosition = dz;
        Self { m_entity: entity }
    }
}

impl Packet for Packet31RelEntityMove
{
    fn base(&self) -> &PacketBase { &self.m_entity.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_entity.readPacketData(input)?;
        self.m_entity.readMove(input)
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        self.m_entity.writePacketData(output)?;
        self.m_entity.writeMove(output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { self.m_entity.processPacket(handler); }

    fn getPacketSize(&self) -> i32 { 7 }
}
