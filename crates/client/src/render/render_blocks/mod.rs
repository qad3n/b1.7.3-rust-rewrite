mod bed;
mod faces;
mod fence_stairs_door;
mod fluids;
mod inventory;
mod lever_fire_wire;
mod piston;
mod plants;
mod rail_ladder;
mod standard;
mod standard_ao;

use game::block::Block;
use game::world::IBlockAccess;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

thread_local! {
    static FANCY_GRASS: Cell<bool> = const { Cell::new(true) };
}

#[derive(Default)]
pub(super) struct AoState
{
    pub(super) m_lightValueOwn: f32,
    pub(super) m_aoLightValueXNeg: f32,
    pub(super) m_aoLightValueYNeg: f32,
    pub(super) m_aoLightValueZNeg: f32,
    pub(super) m_aoLightValueXPos: f32,
    pub(super) m_aoLightValueYPos: f32,
    pub(super) m_aoLightValueZPos: f32,
    pub(super) m_bottomNorthWestVertexAmbientOcclusionLight: f32,
    pub(super) m_bottomWestEdgeAmbientOcclusionLight: f32,
    pub(super) m_bottomSouthWestVertexAmbientOcclusionLight: f32,
    pub(super) m_bottomNorthEdgeAmbientOcclusionLight: f32,
    pub(super) m_bottomSouthEdgeAmbientOcclusionLight: f32,
    pub(super) m_bottomNorthEastVertexAmbientOcclusionLight: f32,
    pub(super) m_bottomEastEdgeAmbientOcclusionLight: f32,
    pub(super) m_bottomSouthEastVertexAmbientOcclusionLight: f32,
    pub(super) m_topNorthWestVertexAmbientOcclusionLight: f32,
    pub(super) m_topWestEdgeAmbientOcclusionLight: f32,
    pub(super) m_topSouthWestVertexAmbientOcclusionLight: f32,
    pub(super) m_topNorthEdgeAmbientOcclusionLight: f32,
    pub(super) m_topNorthEastVertexAmbientOcclusionLight: f32,
    pub(super) m_topEastEdgeAmbientOcclusionLight: f32,
    pub(super) m_topSouthEdgeAmbientOcclusionLight: f32,
    pub(super) m_topSouthEastVertexAmbientOcclusionLight: f32,
    pub(super) m_northWestEdgeAmbientOcclusionLight: f32,
    pub(super) m_northEastEdgeAmbientOcclusionLight: f32,
    pub(super) m_southWestEdgeAmbientOcclusionLight: f32,
    pub(super) m_southEastEdgeAmbientOcclusionLight: f32,
    pub(super) m_colorRedTopLeft: f32,
    pub(super) m_colorRedBottomLeft: f32,
    pub(super) m_colorRedBottomRight: f32,
    pub(super) m_colorRedTopRight: f32,
    pub(super) m_colorGreenTopLeft: f32,
    pub(super) m_colorGreenBottomLeft: f32,
    pub(super) m_colorGreenBottomRight: f32,
    pub(super) m_colorGreenTopRight: f32,
    pub(super) m_colorBlueTopLeft: f32,
    pub(super) m_colorBlueBottomLeft: f32,
    pub(super) m_colorBlueBottomRight: f32,
    pub(super) m_colorBlueTopRight: f32,
    pub(super) m_topNorthEdgeTranslucent: bool,
    pub(super) m_topEastEdgeTranslucent: bool,
    pub(super) m_topWestEdgeTranslucent: bool,
    pub(super) m_topSouthEdgeTranslucent: bool,
    pub(super) m_northWestEdgeTranslucent: bool,
    pub(super) m_southEastEdgeTranslucent: bool,
    pub(super) m_southWestEdgeTranslucent: bool,
    pub(super) m_northEastEdgeTranslucent: bool,
    pub(super) m_bottomNorthEdgeTranslucent: bool,
    pub(super) m_bottomEastEdgeTranslucent: bool,
    pub(super) m_bottomWestEdgeTranslucent: bool,
    pub(super) m_bottomSouthEdgeTranslucent: bool,
}

pub struct RenderBlocks
{
    m_blockAccess: Option<Rc<dyn IBlockAccess>>,
    m_overrideBlockTexture: i32,
    m_flipTexture: bool,
    m_renderAllFaces: bool,
    pub m_updateColor: bool,
    m_northFaceRotation: i32,
    m_southFaceRotation: i32,
    m_westFaceRotation: i32,
    m_eastFaceRotation: i32,
    m_topFaceRotation: i32,
    m_bottomFaceRotation: i32,
    m_enableAO: bool,
    m_ao: AoState,
    m_ambientOcclusionLevel: i32,
}

pub(super) fn spriteOrigin(sprite: i32) -> (i32, i32) { ((sprite & 0xF) << 4, sprite & 0xF0) }

impl RenderBlocks
{
    fn alloc(world: Option<Rc<dyn IBlockAccess>>) -> RenderBlocks
    {
        RenderBlocks {
            m_blockAccess: world,
            m_overrideBlockTexture: -1,
            m_flipTexture: false,
            m_renderAllFaces: false,
            m_updateColor: true,
            m_northFaceRotation: 0,
            m_southFaceRotation: 0,
            m_westFaceRotation: 0,
            m_eastFaceRotation: 0,
            m_topFaceRotation: 0,
            m_bottomFaceRotation: 0,
            m_enableAO: false,
            m_ao: AoState::default(),
            m_ambientOcclusionLevel: 1,
        }
    }

    pub fn new(world: Rc<dyn IBlockAccess>) -> RenderBlocks { RenderBlocks::alloc(Some(world)) }

    pub fn empty() -> RenderBlocks { RenderBlocks::alloc(None) }

    pub fn fancyGrass() -> bool { FANCY_GRASS.with(Cell::get) }

    pub fn setFancyGrass(value: bool) { FANCY_GRASS.with(|fancy| fancy.set(value)); }

    pub(super) fn world(&self) -> Rc<dyn IBlockAccess> { self.m_blockAccess.clone().orNpe() }

    pub fn renderBlockUsingTexture(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32, sprite: i32)
    {
        self.m_overrideBlockTexture = sprite;
        self.renderBlockByRenderType(block, x, y, z);
        self.m_overrideBlockTexture = -1;
    }

    pub fn renderMovingBlock(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32)
    {
        self.m_renderAllFaces = true;
        self.renderBlockByRenderType(block, x, y, z);
        self.m_renderAllFaces = false;
    }

    pub fn renderBlockByRenderType(&mut self, block: &Rc<Block>, x: i32, y: i32, z: i32) -> bool
    {
        let n: i32 = block.getRenderType();
        block.setBlockBoundsBasedOnState(&*self.world(), x, y, z);
        match n
        {
            0 => self.renderStandardBlock(block, x, y, z),
            4 => self.renderBlockFluids(block, x, y, z),
            13 => self.renderBlockCactus(block, x, y, z),
            1 => self.renderBlockReed(block, x, y, z),
            6 => self.renderBlockCrops(block, x, y, z),
            2 => self.renderBlockTorch(block, x, y, z),
            3 => self.renderBlockFire(block, x, y, z),
            5 => self.renderBlockRedstoneWire(block, x, y, z),
            8 => self.renderBlockLadder(block, x, y, z),
            7 => self.renderBlockDoor(block, x, y, z),
            9 => self.renderBlockMinecartTrack(block, x, y, z),
            10 => self.renderBlockStairs(block, x, y, z),
            11 => self.renderBlockFence(block, x, y, z),
            12 => self.renderBlockLever(block, x, y, z),
            14 => self.renderBlockBed(block, x, y, z),
            15 => self.renderBlockRepeater(block, x, y, z),
            16 => self.tesselatePistonBase(block, x, y, z, false),
            17 => self.tesselatePistonHead(block, x, y, z, true),
            _ => false,
        }
    }

    pub fn renderItemIn3d(kind: i32) -> bool { matches!(kind, 0 | 13 | 10 | 11 | 16) }
}
