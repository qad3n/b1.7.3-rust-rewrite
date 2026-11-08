use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
use jvm::io::IoResult;

#[derive(Clone, Default)]
pub struct Packet10Flying
{
    m_base: PacketBase,
    pub m_xPosition: f64,
    pub m_yPosition: f64,
    pub m_zPosition: f64,
    pub m_stance: f64,
    pub m_yaw: f32,
    pub m_pitch: f32,
    pub m_onGround: bool,
    pub m_moving: bool,
    pub m_rotating: bool,
}

impl Packet10Flying
{
    pub fn new(onGround: bool) -> Self { Self { m_onGround: onGround, ..Self::default() } }

    pub fn intoPacket(self) -> Box<dyn Packet>
    {
        match (self.m_moving, self.m_rotating)
        {
            (false, false) => Box::new(self),
            (true, false) => Box::new(Packet11PlayerPosition { m_flying: self }),
            (false, true) => Box::new(Packet12PlayerLook { m_flying: self }),
            (true, true) => Box::new(Packet13PlayerLookMove { m_flying: self }),
        }
    }

    fn readPosition(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_xPosition = input.readDouble()?;
        self.m_yPosition = input.readDouble()?;
        self.m_stance = input.readDouble()?;
        self.m_zPosition = input.readDouble()?;
        Ok(())
    }

    fn writePosition(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeDouble(self.m_xPosition)?;
        output.writeDouble(self.m_yPosition)?;
        output.writeDouble(self.m_stance)?;
        output.writeDouble(self.m_zPosition)
    }

    fn readLook(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_yaw = input.readFloat()?;
        self.m_pitch = input.readFloat()?;
        Ok(())
    }

    fn writeLook(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeFloat(self.m_yaw)?;
        output.writeFloat(self.m_pitch)
    }
}

impl Packet for Packet10Flying
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_onGround = packet::read(input)? != 0;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.write(if self.m_onGround { 1 } else { 0 })
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handleFlying(self); }

    fn getPacketSize(&self) -> i32 { 1 }
}

#[derive(Clone)]
pub struct Packet11PlayerPosition
{
    pub m_flying: Packet10Flying,
}

impl Default for Packet11PlayerPosition
{
    fn default() -> Self { Self { m_flying: Packet10Flying { m_moving: true, ..Packet10Flying::default() } } }
}

impl Packet11PlayerPosition
{
    pub fn new(x: f64, minY: f64, y: f64, z: f64, onGround: bool) -> Self
    {
        Self {
            m_flying: Packet10Flying {
                m_xPosition: x,
                m_yPosition: minY,
                m_stance: y,
                m_zPosition: z,
                m_onGround: onGround,
                m_moving: true,
                ..Packet10Flying::default()
            },
        }
    }
}

impl Packet for Packet11PlayerPosition
{
    fn base(&self) -> &PacketBase { &self.m_flying.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_flying.readPosition(input)?;
        self.m_flying.readPacketData(input)
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        self.m_flying.writePosition(output)?;
        self.m_flying.writePacketData(output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { self.m_flying.processPacket(handler); }

    fn getPacketSize(&self) -> i32 { 33 }
}

#[derive(Clone)]
pub struct Packet13PlayerLookMove
{
    pub m_flying: Packet10Flying,
}

impl Default for Packet13PlayerLookMove
{
    fn default() -> Self
    {
        Self { m_flying: Packet10Flying { m_rotating: true, m_moving: true, ..Packet10Flying::default() } }
    }
}

impl Packet13PlayerLookMove
{
    pub fn new(x: f64, minY: f64, y: f64, z: f64, yaw: f32, pitch: f32, onGround: bool) -> Self
    {
        Self {
            m_flying: Packet10Flying {
                m_xPosition: x,
                m_yPosition: minY,
                m_stance: y,
                m_zPosition: z,
                m_yaw: yaw,
                m_pitch: pitch,
                m_onGround: onGround,
                m_rotating: true,
                m_moving: true,
                ..Packet10Flying::default()
            },
        }
    }
}

impl Packet for Packet13PlayerLookMove
{
    fn base(&self) -> &PacketBase { &self.m_flying.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_flying.readPosition(input)?;
        self.m_flying.readLook(input)?;
        self.m_flying.readPacketData(input)
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        self.m_flying.writePosition(output)?;
        self.m_flying.writeLook(output)?;
        self.m_flying.writePacketData(output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { self.m_flying.processPacket(handler); }

    fn getPacketSize(&self) -> i32 { 41 }
}

#[derive(Clone)]
pub struct Packet12PlayerLook
{
    pub m_flying: Packet10Flying,
}

impl Default for Packet12PlayerLook
{
    fn default() -> Self { Self { m_flying: Packet10Flying { m_rotating: true, ..Packet10Flying::default() } } }
}

impl Packet12PlayerLook
{
    pub fn new(yaw: f32, pitch: f32, onGround: bool) -> Self
    {
        Self {
            m_flying: Packet10Flying {
                m_yaw: yaw,
                m_pitch: pitch,
                m_onGround: onGround,
                m_rotating: true,
                ..Packet10Flying::default()
            },
        }
    }
}

impl Packet for Packet12PlayerLook
{
    fn base(&self) -> &PacketBase { &self.m_flying.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_flying.readLook(input)?;
        self.m_flying.readPacketData(input)
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        self.m_flying.writeLook(output)?;
        self.m_flying.writePacketData(output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { self.m_flying.processPacket(handler); }

    fn getPacketSize(&self) -> i32 { 9 }
}
