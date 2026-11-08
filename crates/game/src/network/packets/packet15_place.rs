use crate::item::ItemStack;
use crate::network::net_handler::NetHandler;
use crate::network::packet::{self, Packet, PacketBase, PacketInput, PacketOutput};
// item stacks cross the network threads as plain port data
use crate::port::packet_item_stack::PacketItemStack;
use jvm::io::IoResult;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Packet15Place
{
    m_base: PacketBase,
    pub m_xPosition: i32,
    pub m_yPosition: i32,
    pub m_zPosition: i32,
    pub m_direction: i32,
    pub m_itemStack: Option<PacketItemStack>,
}

impl Packet15Place
{
    pub fn new(x: i32, y: i32, z: i32, face: i32, itemInHand: Option<&Rc<ItemStack>>) -> Self
    {
        Self {
            m_xPosition: x,
            m_yPosition: y,
            m_zPosition: z,
            m_direction: face,
            m_itemStack: PacketItemStack::fromOption(itemInHand),
            ..Self::default()
        }
    }
}

impl Packet for Packet15Place
{
    fn base(&self) -> &PacketBase { &self.m_base }

    fn readPacketData(&mut self, input: &mut PacketInput) -> IoResult<()>
    {
        self.m_xPosition = input.readInt()?;
        self.m_yPosition = packet::read(input)?;
        self.m_zPosition = input.readInt()?;
        self.m_direction = packet::read(input)?;
        self.m_itemStack = PacketItemStack::read(input)?;
        Ok(())
    }

    fn writePacketData(&self, output: &mut PacketOutput) -> IoResult<()>
    {
        output.writeInt(self.m_xPosition)?;
        output.write(self.m_yPosition)?;
        output.writeInt(self.m_zPosition)?;
        output.write(self.m_direction)?;
        PacketItemStack::write(self.m_itemStack.as_ref(), output)
    }

    fn processPacket(&self, handler: &dyn NetHandler) { handler.handlePlace(self); }

    fn getPacketSize(&self) -> i32 { 15 }
}
