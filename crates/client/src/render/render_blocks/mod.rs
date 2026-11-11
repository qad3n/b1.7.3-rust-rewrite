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

type Draw = fn(&mut RenderBlocks, &Rc<Block>, i32, i32, i32) -> bool;

// how each beta render type draws in the world and whether its item is a 3d block, indexed by the ids in
// game::block::render_type
struct ShapeRow
{
    m_draw: Draw,
    m_item3d: bool,
}

const SHAPES: [ShapeRow; 18] = [
    ShapeRow { m_draw: RenderBlocks::renderStandardBlock, m_item3d: true },
    ShapeRow { m_draw: RenderBlocks::renderBlockReed, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockTorch, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockFire, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockFluids, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockRedstoneWire, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockCrops, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockDoor, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockLadder, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockMinecartTrack, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockStairs, m_item3d: true },
    ShapeRow { m_draw: RenderBlocks::renderBlockFence, m_item3d: true },
    ShapeRow { m_draw: RenderBlocks::renderBlockLever, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockCactus, m_item3d: true },
    ShapeRow { m_draw: RenderBlocks::renderBlockBed, m_item3d: false },
    ShapeRow { m_draw: RenderBlocks::renderBlockRepeater, m_item3d: false },
    ShapeRow { m_draw: |renderer, block, x, y, z| renderer.tesselatePistonBase(block, x, y, z, false), m_item3d: true },
    ShapeRow { m_draw: |renderer, block, x, y, z| renderer.tesselatePistonHead(block, x, y, z, true), m_item3d: false },
];

thread_local!
{
    static FANCY_GRASS: Cell<bool> = const { Cell::new(true) };
}

#[derive(Default)]
pub(crate) struct AoState
{
    pub(crate) m_lightValueOwn: f32,
    pub(crate) m_aoLightValueXNeg: f32,
    pub(crate) m_aoLightValueYNeg: f32,
    pub(crate) m_aoLightValueZNeg: f32,
    pub(crate) m_aoLightValueXPos: f32,
    pub(crate) m_aoLightValueYPos: f32,
    pub(crate) m_aoLightValueZPos: f32,
    pub(crate) m_bottomNorthWestVertexAmbientOcclusionLight: f32,
    pub(crate) m_bottomWestEdgeAmbientOcclusionLight: f32,
    pub(crate) m_bottomSouthWestVertexAmbientOcclusionLight: f32,
    pub(crate) m_bottomNorthEdgeAmbientOcclusionLight: f32,
    pub(crate) m_bottomSouthEdgeAmbientOcclusionLight: f32,
    pub(crate) m_bottomNorthEastVertexAmbientOcclusionLight: f32,
    pub(crate) m_bottomEastEdgeAmbientOcclusionLight: f32,
    pub(crate) m_bottomSouthEastVertexAmbientOcclusionLight: f32,
    pub(crate) m_topNorthWestVertexAmbientOcclusionLight: f32,
    pub(crate) m_topWestEdgeAmbientOcclusionLight: f32,
    pub(crate) m_topSouthWestVertexAmbientOcclusionLight: f32,
    pub(crate) m_topNorthEdgeAmbientOcclusionLight: f32,
    pub(crate) m_topNorthEastVertexAmbientOcclusionLight: f32,
    pub(crate) m_topEastEdgeAmbientOcclusionLight: f32,
    pub(crate) m_topSouthEdgeAmbientOcclusionLight: f32,
    pub(crate) m_topSouthEastVertexAmbientOcclusionLight: f32,
    pub(crate) m_northWestEdgeAmbientOcclusionLight: f32,
    pub(crate) m_northEastEdgeAmbientOcclusionLight: f32,
    pub(crate) m_southWestEdgeAmbientOcclusionLight: f32,
    pub(crate) m_southEastEdgeAmbientOcclusionLight: f32,
    pub(crate) m_colorRedTopLeft: f32,
    pub(crate) m_colorRedBottomLeft: f32,
    pub(crate) m_colorRedBottomRight: f32,
    pub(crate) m_colorRedTopRight: f32,
    pub(crate) m_colorGreenTopLeft: f32,
    pub(crate) m_colorGreenBottomLeft: f32,
    pub(crate) m_colorGreenBottomRight: f32,
    pub(crate) m_colorGreenTopRight: f32,
    pub(crate) m_colorBlueTopLeft: f32,
    pub(crate) m_colorBlueBottomLeft: f32,
    pub(crate) m_colorBlueBottomRight: f32,
    pub(crate) m_colorBlueTopRight: f32,
    pub(crate) m_topNorthEdgeTranslucent: bool,
    pub(crate) m_topEastEdgeTranslucent: bool,
    pub(crate) m_topWestEdgeTranslucent: bool,
    pub(crate) m_topSouthEdgeTranslucent: bool,
    pub(crate) m_northWestEdgeTranslucent: bool,
    pub(crate) m_southEastEdgeTranslucent: bool,
    pub(crate) m_southWestEdgeTranslucent: bool,
    pub(crate) m_northEastEdgeTranslucent: bool,
    pub(crate) m_bottomNorthEdgeTranslucent: bool,
    pub(crate) m_bottomEastEdgeTranslucent: bool,
    pub(crate) m_bottomWestEdgeTranslucent: bool,
    pub(crate) m_bottomSouthEdgeTranslucent: bool,
}

pub struct RenderBlocks
{
    m_blockAccess: Option<Rc<dyn IBlockAccess>>,
    pub m_overrideBlockTexture: i32,
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

// rewrite: the row keeps every bit so sprites past slot 255 land in the grown terrain sheet
pub(crate) fn spriteOrigin(sprite: i32) -> (i32, i32) { ((sprite & 0xF) << 4, sprite & !0xF) }

impl RenderBlocks
{
    fn alloc(world: Option<Rc<dyn IBlockAccess>>) -> Self
    {
        Self
        {
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

    pub fn new(world: Rc<dyn IBlockAccess>) -> Self { Self::alloc(Some(world)) }

    pub fn empty() -> Self { Self::alloc(None) }

    pub fn fancyGrass() -> bool { FANCY_GRASS.with(Cell::get) }

    pub fn setFancyGrass(value: bool) { FANCY_GRASS.with(|fancy| fancy.set(value)); }

    pub(crate) fn world(&self) -> Rc<dyn IBlockAccess> { self.m_blockAccess.clone().orNpe() }

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
        // rewrite: a mod block may turn the sprites of its faces where it stands
        let turns: Option<[i32; 6]> = modloader::behavior::faceRotations(block, &*self.world(), x, y, z);
        if let Some(turns) = turns
        {
            self.setFaceRotations(turns);
        }

        let drawn: bool = match n
        {
            // mod block models are lists of cuboids
            modloader::content::blocks::MODEL_RENDER_TYPE => crate::modded::renderModel(self, block, x, y, z),
            _ => usize::try_from(n)
                .ok()
                .and_then(|index| SHAPES.get(index))
                .is_some_and(|row| (row.m_draw)(self, block, x, y, z)),
        };

        if turns.is_some()
        {
            self.setFaceRotations([0; 6]);
        }
        // rewrite: a mod block may lay sprites over its faces after it is drawn
        crate::modded::renderOverlays(self, block, x, y, z) || drawn
    }

    // the uvRotate fields by face index, face 2 draws through renderEastFace and its north rotation and so on
    fn setFaceRotations(&mut self, turns: [i32; 6])
    {
        [
            self.m_bottomFaceRotation,
            self.m_topFaceRotation,
            self.m_northFaceRotation,
            self.m_southFaceRotation,
            self.m_eastFaceRotation,
            self.m_westFaceRotation,
        ] = turns;
    }

    pub fn renderItemIn3d(kind: i32) -> bool
    {
        usize::try_from(kind).ok().and_then(|index| SHAPES.get(index)).is_some_and(|row| row.m_item3d)
    }
}
