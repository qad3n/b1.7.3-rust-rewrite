use super::{getIsPowered, isRailBlockAt};
use crate::block::Block;
use crate::world::World;
use common::world::ChunkPosition;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub struct RailLogic
{
    m_worldObj: Rc<World>,
    m_trackX: i32,
    m_trackY: i32,
    m_trackZ: i32,
    m_isPoweredRail: bool,
    m_connectedTracks: Vec<ChunkPosition>,
}

impl RailLogic
{
    pub fn new(world: &Rc<World>, x: i32, y: i32, z: i32) -> Self
    {
        let blockId: i32 = world.getBlockId(x, y, z);
        let mut blockMetadata: i32 = world.getBlockMetadata(x, y, z);
        let block: Rc<Block> = Block::byId(blockId).orNpe();
        if !block.instanceOf("BlockRail")
        {
            jvm::throwable::throw(
                &jvm::throwable::CLASS_CAST,
                &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.BlockRail", block.className()),
            );
        }
        let isPoweredRail: bool;
        if getIsPowered(&block)
        {
            isPoweredRail = true;
            blockMetadata &= 0xFFFFFFF7u32 as i32;
        }
        else
        {
            isPoweredRail = false;
        }
        let mut logic: Self = Self {
            m_worldObj: Rc::clone(world),
            m_trackX: x,
            m_trackY: y,
            m_trackZ: z,
            m_isPoweredRail: isPoweredRail,
            m_connectedTracks: Vec::new(),
        };
        logic.setConnections(blockMetadata);
        logic
    }

    fn setConnections(&mut self, metadata: i32)
    {
        let (x, y, z): (i32, i32, i32) = (self.m_trackX, self.m_trackY, self.m_trackZ);
        self.m_connectedTracks.clear();
        let pair: Option<[ChunkPosition; 2]> = match metadata
        {
            0 => Some([ChunkPosition::new(x, y, z - 1), ChunkPosition::new(x, y, z + 1)]),
            1 => Some([ChunkPosition::new(x - 1, y, z), ChunkPosition::new(x + 1, y, z)]),
            2 => Some([ChunkPosition::new(x - 1, y, z), ChunkPosition::new(x + 1, y + 1, z)]),
            3 => Some([ChunkPosition::new(x - 1, y + 1, z), ChunkPosition::new(x + 1, y, z)]),
            4 => Some([ChunkPosition::new(x, y + 1, z - 1), ChunkPosition::new(x, y, z + 1)]),
            5 => Some([ChunkPosition::new(x, y, z - 1), ChunkPosition::new(x, y + 1, z + 1)]),
            6 => Some([ChunkPosition::new(x + 1, y, z), ChunkPosition::new(x, y, z + 1)]),
            7 => Some([ChunkPosition::new(x - 1, y, z), ChunkPosition::new(x, y, z + 1)]),
            8 => Some([ChunkPosition::new(x - 1, y, z), ChunkPosition::new(x, y, z - 1)]),
            9 => Some([ChunkPosition::new(x + 1, y, z), ChunkPosition::new(x, y, z - 1)]),
            _ => None,
        };
        if let Some(pair) = pair
        {
            self.m_connectedTracks.extend(pair);
        }
    }

    fn removeSoftConnections(&mut self)
    {
        let mut i: i32 = 0;
        while (i as usize) < self.m_connectedTracks.len()
        {
            let minecartTrackLogic: Option<RailLogic> = self.getMinecartTrackLogic(self.m_connectedTracks[i as usize]);
            match minecartTrackLogic
            {
                Some(logic) if logic.isConnectedTo(self) =>
                {
                    self.m_connectedTracks[i as usize] =
                        ChunkPosition::new(logic.m_trackX, logic.m_trackY, logic.m_trackZ);
                }
                _ =>
                {
                    self.m_connectedTracks.remove(i as usize);
                    i -= 1;
                }
            }
            i += 1;
        }
    }

    fn isMinecartTrack(&self, x: i32, y: i32, z: i32) -> bool
    {
        isRailBlockAt(&self.m_worldObj, x, y, z)
            || isRailBlockAt(&self.m_worldObj, x, y + 1, z)
            || isRailBlockAt(&self.m_worldObj, x, y - 1, z)
    }

    fn getMinecartTrackLogic(&self, pos: ChunkPosition) -> Option<RailLogic>
    {
        if isRailBlockAt(&self.m_worldObj, pos.m_x, pos.m_y, pos.m_z)
        {
            return Some(RailLogic::new(&self.m_worldObj, pos.m_x, pos.m_y, pos.m_z));
        }
        if isRailBlockAt(&self.m_worldObj, pos.m_x, pos.m_y + 1, pos.m_z)
        {
            return Some(RailLogic::new(&self.m_worldObj, pos.m_x, pos.m_y + 1, pos.m_z));
        }
        if isRailBlockAt(&self.m_worldObj, pos.m_x, pos.m_y - 1, pos.m_z)
        {
            return Some(RailLogic::new(&self.m_worldObj, pos.m_x, pos.m_y - 1, pos.m_z));
        }
        None
    }

    fn isConnectedTo(&self, rail: &RailLogic) -> bool
    {
        for chunkPosition in &self.m_connectedTracks
        {
            if chunkPosition.m_x == rail.m_trackX && chunkPosition.m_z == rail.m_trackZ
            {
                return true;
            }
        }
        false
    }

    fn isInTrack(&self, x: i32, _y: i32, z: i32) -> bool
    {
        for chunkPosition in &self.m_connectedTracks
        {
            if chunkPosition.m_x == x && chunkPosition.m_z == z
            {
                return true;
            }
        }
        false
    }

    pub fn getAdjacentTracks(&self) -> i32
    {
        let mut n: i32 = 0;
        if self.isMinecartTrack(self.m_trackX, self.m_trackY, self.m_trackZ - 1)
        {
            n += 1;
        }
        if self.isMinecartTrack(self.m_trackX, self.m_trackY, self.m_trackZ + 1)
        {
            n += 1;
        }
        if self.isMinecartTrack(self.m_trackX - 1, self.m_trackY, self.m_trackZ)
        {
            n += 1;
        }
        if self.isMinecartTrack(self.m_trackX + 1, self.m_trackY, self.m_trackZ)
        {
            n += 1;
        }
        n
    }

    fn handleKeyPress(&self, rail: &RailLogic) -> bool
    {
        if self.isConnectedTo(rail)
        {
            return true;
        }
        if self.m_connectedTracks.len() == 2
        {
            return false;
        }
        if self.m_connectedTracks.is_empty()
        {
            return true;
        }
        let chunkPosition: ChunkPosition = self.m_connectedTracks[0];
        if rail.m_trackY == self.m_trackY && chunkPosition.m_y == self.m_trackY
        {
            return true;
        }
        true
    }

    fn addConnection(&mut self, rail: &RailLogic)
    {
        self.m_connectedTracks.push(ChunkPosition::new(rail.m_trackX, rail.m_trackY, rail.m_trackZ));
        let inTrack: bool = self.isInTrack(self.m_trackX, self.m_trackY, self.m_trackZ - 1);
        let inTrack2: bool = self.isInTrack(self.m_trackX, self.m_trackY, self.m_trackZ + 1);
        let inTrack3: bool = self.isInTrack(self.m_trackX - 1, self.m_trackY, self.m_trackZ);
        let inTrack4: bool = self.isInTrack(self.m_trackX + 1, self.m_trackY, self.m_trackZ);
        let mut n: i32 = -1;
        if inTrack || inTrack2
        {
            n = 0;
        }
        if inTrack3 || inTrack4
        {
            n = 1;
        }
        if !self.m_isPoweredRail
        {
            if inTrack2 && inTrack4 && !inTrack && !inTrack3
            {
                n = 6;
            }
            if inTrack2 && inTrack3 && !inTrack && !inTrack4
            {
                n = 7;
            }
            if inTrack && inTrack3 && !inTrack2 && !inTrack4
            {
                n = 8;
            }
            if inTrack && inTrack4 && !inTrack2 && !inTrack3
            {
                n = 9;
            }
        }
        n = self.slope(n);
        if n < 0
        {
            n = 0;
        }
        let mut metadata: i32 = n;
        if self.m_isPoweredRail
        {
            metadata = (self.m_worldObj.getBlockMetadata(self.m_trackX, self.m_trackY, self.m_trackZ) & 0x8) | n;
        }
        self.m_worldObj.setBlockMetadataWithNotify(self.m_trackX, self.m_trackY, self.m_trackZ, metadata);
    }

    fn slope(&self, mut n: i32) -> i32
    {
        let world: &Rc<World> = &self.m_worldObj;
        if n == 0
        {
            if isRailBlockAt(world, self.m_trackX, self.m_trackY + 1, self.m_trackZ - 1)
            {
                n = 4;
            }
            if isRailBlockAt(world, self.m_trackX, self.m_trackY + 1, self.m_trackZ + 1)
            {
                n = 5;
            }
        }
        if n == 1
        {
            if isRailBlockAt(world, self.m_trackX + 1, self.m_trackY + 1, self.m_trackZ)
            {
                n = 2;
            }
            if isRailBlockAt(world, self.m_trackX - 1, self.m_trackY + 1, self.m_trackZ)
            {
                n = 3;
            }
        }
        n
    }

    fn hasNeighborRail(&self, x: i32, y: i32, z: i32) -> bool
    {
        let Some(mut minecartTrackLogic) = self.getMinecartTrackLogic(ChunkPosition::new(x, y, z))
        else
        {
            return false;
        };
        minecartTrackLogic.removeSoftConnections();
        minecartTrackLogic.handleKeyPress(self)
    }

    pub fn updateState(&mut self, powered: bool, force: bool)
    {
        let hasNeighborRail: bool = self.hasNeighborRail(self.m_trackX, self.m_trackY, self.m_trackZ - 1);
        let hasNeighborRail2: bool = self.hasNeighborRail(self.m_trackX, self.m_trackY, self.m_trackZ + 1);
        let hasNeighborRail3: bool = self.hasNeighborRail(self.m_trackX - 1, self.m_trackY, self.m_trackZ);
        let hasNeighborRail4: bool = self.hasNeighborRail(self.m_trackX + 1, self.m_trackY, self.m_trackZ);
        let mut connections: i32 = -1;
        if (hasNeighborRail || hasNeighborRail2) && !hasNeighborRail3 && !hasNeighborRail4
        {
            connections = 0;
        }
        if (hasNeighborRail3 || hasNeighborRail4) && !hasNeighborRail && !hasNeighborRail2
        {
            connections = 1;
        }
        if !self.m_isPoweredRail
        {
            if hasNeighborRail2 && hasNeighborRail4 && !hasNeighborRail && !hasNeighborRail3
            {
                connections = 6;
            }
            if hasNeighborRail2 && hasNeighborRail3 && !hasNeighborRail && !hasNeighborRail4
            {
                connections = 7;
            }
            if hasNeighborRail && hasNeighborRail3 && !hasNeighborRail2 && !hasNeighborRail4
            {
                connections = 8;
            }
            if hasNeighborRail && hasNeighborRail4 && !hasNeighborRail2 && !hasNeighborRail3
            {
                connections = 9;
            }
        }
        if connections == -1
        {
            if hasNeighborRail || hasNeighborRail2
            {
                connections = 0;
            }
            if hasNeighborRail3 || hasNeighborRail4
            {
                connections = 1;
            }
            if !self.m_isPoweredRail
            {
                if powered
                {
                    if hasNeighborRail2 && hasNeighborRail4
                    {
                        connections = 6;
                    }
                    if hasNeighborRail3 && hasNeighborRail2
                    {
                        connections = 7;
                    }
                    if hasNeighborRail4 && hasNeighborRail
                    {
                        connections = 9;
                    }
                    if hasNeighborRail && hasNeighborRail3
                    {
                        connections = 8;
                    }
                }
                else
                {
                    if hasNeighborRail && hasNeighborRail3
                    {
                        connections = 8;
                    }
                    if hasNeighborRail4 && hasNeighborRail
                    {
                        connections = 9;
                    }
                    if hasNeighborRail3 && hasNeighborRail2
                    {
                        connections = 7;
                    }
                    if hasNeighborRail2 && hasNeighborRail4
                    {
                        connections = 6;
                    }
                }
            }
        }
        connections = self.slope(connections);
        if connections < 0
        {
            connections = 0;
        }
        self.setConnections(connections);
        let mut metadata: i32 = connections;
        if self.m_isPoweredRail
        {
            metadata =
                (self.m_worldObj.getBlockMetadata(self.m_trackX, self.m_trackY, self.m_trackZ) & 0x8) | connections;
        }
        if force || self.m_worldObj.getBlockMetadata(self.m_trackX, self.m_trackY, self.m_trackZ) != metadata
        {
            self.m_worldObj.setBlockMetadataWithNotify(self.m_trackX, self.m_trackY, self.m_trackZ, metadata);
            let mut i: usize = 0;
            while i < self.m_connectedTracks.len()
            {
                if let Some(mut minecartTrackLogic) = self.getMinecartTrackLogic(self.m_connectedTracks[i])
                {
                    minecartTrackLogic.removeSoftConnections();
                    if minecartTrackLogic.handleKeyPress(self)
                    {
                        minecartTrackLogic.addConnection(self);
                    }
                }
                i += 1;
            }
        }
    }
}
