use super::{AoState, RenderBlocks};
use game::block::Block;
use game::world::IBlockAccess;
use std::rc::Rc;

impl RenderBlocks
{
    #[allow(clippy::too_many_arguments)]
    fn setAoColors(&mut self, r: f32, g: f32, b: f32, f: f32, f2: f32, f3: f32, f4: f32)
    {
        let ao: &mut AoState = &mut self.m_ao;
        ao.m_colorRedTopLeft = r * f;
        ao.m_colorGreenTopLeft = g * f;
        ao.m_colorBlueTopLeft = b * f;
        ao.m_colorRedBottomLeft = r * f2;
        ao.m_colorGreenBottomLeft = g * f2;
        ao.m_colorBlueBottomLeft = b * f2;
        ao.m_colorRedBottomRight = r * f3;
        ao.m_colorGreenBottomRight = g * f3;
        ao.m_colorBlueBottomRight = b * f3;
        ao.m_colorRedTopRight = r * f4;
        ao.m_colorGreenTopRight = g * f4;
        ao.m_colorBlueTopRight = b * f4;
    }

    fn tintAoColors(&mut self, r: f32, g: f32, b: f32)
    {
        let ao: &mut AoState = &mut self.m_ao;
        ao.m_colorRedTopLeft *= r;
        ao.m_colorRedBottomLeft *= r;
        ao.m_colorRedBottomRight *= r;
        ao.m_colorRedTopRight *= r;
        ao.m_colorGreenTopLeft *= g;
        ao.m_colorGreenBottomLeft *= g;
        ao.m_colorGreenBottomRight *= g;
        ao.m_colorGreenTopRight *= g;
        ao.m_colorBlueTopLeft *= b;
        ao.m_colorBlueBottomLeft *= b;
        ao.m_colorBlueBottomRight *= b;
        ao.m_colorBlueTopRight *= b;
    }

    fn grassOverlay(&self, n: i32) -> bool { RenderBlocks::fancyGrass() && n == 3 && self.m_overrideBlockTexture < 0 }

    #[allow(clippy::too_many_arguments)]
    pub fn renderStandardBlockWithAmbientOcclusion(
        &mut self,
        block: &Rc<Block>,
        x: i32,
        y: i32,
        z: i32,
        r: f32,
        g: f32,
        b: f32,
    ) -> bool
    {
        let world: Rc<dyn IBlockAccess> = self.world();
        let light = |x: i32, y: i32, z: i32| block.getBlockBrightness(&*world, x, y, z);
        let grass = |x: i32, y: i32, z: i32| Block::canBlockGrass(world.getBlockId(x, y, z));
        self.m_enableAO = true;
        let mut bl: bool = false;
        let own: f32 = self.m_ao.m_lightValueOwn;
        let (mut f, mut f2, mut f3, mut f4): (f32, f32, f32, f32) = (own, own, own, own);
        let mut bl2: bool = true;
        let bl3: bool = true;
        let mut bl4: bool = true;
        let mut bl5: bool = true;
        let mut bl6: bool = true;
        let mut bl7: bool = true;
        {
            let ao: &mut AoState = &mut self.m_ao;
            ao.m_lightValueOwn = light(x, y, z);
            ao.m_aoLightValueXNeg = light(x - 1, y, z);
            ao.m_aoLightValueYNeg = light(x, y - 1, z);
            ao.m_aoLightValueZNeg = light(x, y, z - 1);
            ao.m_aoLightValueXPos = light(x + 1, y, z);
            ao.m_aoLightValueYPos = light(x, y + 1, z);
            ao.m_aoLightValueZPos = light(x, y, z + 1);
            ao.m_topEastEdgeTranslucent = grass(x + 1, y + 1, z);
            ao.m_bottomEastEdgeTranslucent = grass(x + 1, y - 1, z);
            ao.m_southEastEdgeTranslucent = grass(x + 1, y, z + 1);
            ao.m_northEastEdgeTranslucent = grass(x + 1, y, z - 1);
            ao.m_topWestEdgeTranslucent = grass(x - 1, y + 1, z);
            ao.m_bottomWestEdgeTranslucent = grass(x - 1, y - 1, z);
            ao.m_northWestEdgeTranslucent = grass(x - 1, y, z - 1);
            ao.m_southWestEdgeTranslucent = grass(x - 1, y, z + 1);
            ao.m_topSouthEdgeTranslucent = grass(x, y + 1, z + 1);
            ao.m_topNorthEdgeTranslucent = grass(x, y + 1, z - 1);
            ao.m_bottomSouthEdgeTranslucent = grass(x, y - 1, z + 1);
            ao.m_bottomNorthEdgeTranslucent = grass(x, y - 1, z - 1);
        }
        if block.m_blockIndexInTexture.get() == 3 || self.m_overrideBlockTexture >= 0
        {
            bl2 = false;
            bl4 = false;
            bl5 = false;
            bl6 = false;
            bl7 = false;
        }
        let occlude: bool = self.m_ambientOcclusionLevel > 0;
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y - 1, z, 0)
        {
            let ao: &mut AoState = &mut self.m_ao;
            if occlude
            {
                let y: i32 = y - 1;
                ao.m_bottomWestEdgeAmbientOcclusionLight = light(x - 1, y, z);
                ao.m_bottomNorthEdgeAmbientOcclusionLight = light(x, y, z - 1);
                ao.m_bottomSouthEdgeAmbientOcclusionLight = light(x, y, z + 1);
                ao.m_bottomEastEdgeAmbientOcclusionLight = light(x + 1, y, z);
                ao.m_bottomNorthWestVertexAmbientOcclusionLight = if ao.m_bottomNorthEdgeTranslucent
                    || ao.m_bottomWestEdgeTranslucent
                {
                    light(x - 1, y, z - 1)
                }
                else
                {
                    ao.m_bottomWestEdgeAmbientOcclusionLight
                };
                ao.m_bottomSouthWestVertexAmbientOcclusionLight = if ao.m_bottomSouthEdgeTranslucent
                    || ao.m_bottomWestEdgeTranslucent
                {
                    light(x - 1, y, z + 1)
                }
                else
                {
                    ao.m_bottomWestEdgeAmbientOcclusionLight
                };
                ao.m_bottomNorthEastVertexAmbientOcclusionLight = if ao.m_bottomNorthEdgeTranslucent
                    || ao.m_bottomEastEdgeTranslucent
                {
                    light(x + 1, y, z - 1)
                }
                else
                {
                    ao.m_bottomEastEdgeAmbientOcclusionLight
                };
                ao.m_bottomSouthEastVertexAmbientOcclusionLight = if ao.m_bottomSouthEdgeTranslucent
                    || ao.m_bottomEastEdgeTranslucent
                {
                    light(x + 1, y, z + 1)
                }
                else
                {
                    ao.m_bottomEastEdgeAmbientOcclusionLight
                };
                f = (ao.m_bottomSouthWestVertexAmbientOcclusionLight
                    + ao.m_bottomWestEdgeAmbientOcclusionLight
                    + ao.m_bottomSouthEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueYNeg)
                    / 4.0;
                f4 = (ao.m_bottomSouthEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueYNeg
                    + ao.m_bottomSouthEastVertexAmbientOcclusionLight
                    + ao.m_bottomEastEdgeAmbientOcclusionLight)
                    / 4.0;
                f3 = (ao.m_aoLightValueYNeg
                    + ao.m_bottomNorthEdgeAmbientOcclusionLight
                    + ao.m_bottomEastEdgeAmbientOcclusionLight
                    + ao.m_bottomNorthEastVertexAmbientOcclusionLight)
                    / 4.0;
                f2 = (ao.m_bottomWestEdgeAmbientOcclusionLight
                    + ao.m_bottomNorthWestVertexAmbientOcclusionLight
                    + ao.m_aoLightValueYNeg
                    + ao.m_bottomNorthEdgeAmbientOcclusionLight)
                    / 4.0;
            }
            else
            {
                (f, f2, f3, f4) =
                    (ao.m_aoLightValueYNeg, ao.m_aoLightValueYNeg, ao.m_aoLightValueYNeg, ao.m_aoLightValueYNeg);
            }
            let shade = |color: f32| (if bl2 { color } else { 1.0 }) * 0.5;
            self.setAoColors(shade(r), shade(g), shade(b), f, f2, f3, f4);
            self.renderBottomFace(block, x as f64, y as f64, z as f64, block.getBlockTexture(&*world, x, y, z, 0));
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y + 1, z, 1)
        {
            let ao: &mut AoState = &mut self.m_ao;
            if occlude
            {
                let y: i32 = y + 1;
                ao.m_topWestEdgeAmbientOcclusionLight = light(x - 1, y, z);
                ao.m_topEastEdgeAmbientOcclusionLight = light(x + 1, y, z);
                ao.m_topNorthEdgeAmbientOcclusionLight = light(x, y, z - 1);
                ao.m_topSouthEdgeAmbientOcclusionLight = light(x, y, z + 1);
                ao.m_topNorthWestVertexAmbientOcclusionLight = if ao.m_topNorthEdgeTranslucent
                    || ao.m_topWestEdgeTranslucent
                {
                    light(x - 1, y, z - 1)
                }
                else
                {
                    ao.m_topWestEdgeAmbientOcclusionLight
                };
                ao.m_topNorthEastVertexAmbientOcclusionLight = if ao.m_topNorthEdgeTranslucent
                    || ao.m_topEastEdgeTranslucent
                {
                    light(x + 1, y, z - 1)
                }
                else
                {
                    ao.m_topEastEdgeAmbientOcclusionLight
                };
                ao.m_topSouthWestVertexAmbientOcclusionLight = if ao.m_topSouthEdgeTranslucent
                    || ao.m_topWestEdgeTranslucent
                {
                    light(x - 1, y, z + 1)
                }
                else
                {
                    ao.m_topWestEdgeAmbientOcclusionLight
                };
                ao.m_topSouthEastVertexAmbientOcclusionLight = if ao.m_topSouthEdgeTranslucent
                    || ao.m_topEastEdgeTranslucent
                {
                    light(x + 1, y, z + 1)
                }
                else
                {
                    ao.m_topEastEdgeAmbientOcclusionLight
                };
                f4 = (ao.m_topSouthWestVertexAmbientOcclusionLight
                    + ao.m_topWestEdgeAmbientOcclusionLight
                    + ao.m_topSouthEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueYPos)
                    / 4.0;
                f = (ao.m_topSouthEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueYPos
                    + ao.m_topSouthEastVertexAmbientOcclusionLight
                    + ao.m_topEastEdgeAmbientOcclusionLight)
                    / 4.0;
                f2 = (ao.m_aoLightValueYPos
                    + ao.m_topNorthEdgeAmbientOcclusionLight
                    + ao.m_topEastEdgeAmbientOcclusionLight
                    + ao.m_topNorthEastVertexAmbientOcclusionLight)
                    / 4.0;
                f3 = (ao.m_topWestEdgeAmbientOcclusionLight
                    + ao.m_topNorthWestVertexAmbientOcclusionLight
                    + ao.m_aoLightValueYPos
                    + ao.m_topNorthEdgeAmbientOcclusionLight)
                    / 4.0;
            }
            else
            {
                (f, f2, f3, f4) =
                    (ao.m_aoLightValueYPos, ao.m_aoLightValueYPos, ao.m_aoLightValueYPos, ao.m_aoLightValueYPos);
            }
            let shade = |color: f32| if bl3 { color } else { 1.0 };
            self.setAoColors(shade(r), shade(g), shade(b), f, f2, f3, f4);
            self.renderTopFace(block, x as f64, y as f64, z as f64, block.getBlockTexture(&*world, x, y, z, 1));
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y, z - 1, 2)
        {
            let ao: &mut AoState = &mut self.m_ao;
            if occlude
            {
                let z: i32 = z - 1;
                ao.m_northWestEdgeAmbientOcclusionLight = light(x - 1, y, z);
                ao.m_bottomNorthEdgeAmbientOcclusionLight = light(x, y - 1, z);
                ao.m_topNorthEdgeAmbientOcclusionLight = light(x, y + 1, z);
                ao.m_northEastEdgeAmbientOcclusionLight = light(x + 1, y, z);
                ao.m_bottomNorthWestVertexAmbientOcclusionLight = if ao.m_northWestEdgeTranslucent
                    || ao.m_bottomNorthEdgeTranslucent
                {
                    light(x - 1, y - 1, z)
                }
                else
                {
                    ao.m_northWestEdgeAmbientOcclusionLight
                };
                ao.m_topNorthWestVertexAmbientOcclusionLight = if ao.m_northWestEdgeTranslucent
                    || ao.m_topNorthEdgeTranslucent
                {
                    light(x - 1, y + 1, z)
                }
                else
                {
                    ao.m_northWestEdgeAmbientOcclusionLight
                };
                ao.m_bottomNorthEastVertexAmbientOcclusionLight = if ao.m_northEastEdgeTranslucent
                    || ao.m_bottomNorthEdgeTranslucent
                {
                    light(x + 1, y - 1, z)
                }
                else
                {
                    ao.m_northEastEdgeAmbientOcclusionLight
                };
                ao.m_topNorthEastVertexAmbientOcclusionLight = if ao.m_northEastEdgeTranslucent
                    || ao.m_topNorthEdgeTranslucent
                {
                    light(x + 1, y + 1, z)
                }
                else
                {
                    ao.m_northEastEdgeAmbientOcclusionLight
                };
                f = (ao.m_northWestEdgeAmbientOcclusionLight
                    + ao.m_topNorthWestVertexAmbientOcclusionLight
                    + ao.m_aoLightValueZNeg
                    + ao.m_topNorthEdgeAmbientOcclusionLight)
                    / 4.0;
                f2 = (ao.m_aoLightValueZNeg
                    + ao.m_topNorthEdgeAmbientOcclusionLight
                    + ao.m_northEastEdgeAmbientOcclusionLight
                    + ao.m_topNorthEastVertexAmbientOcclusionLight)
                    / 4.0;
                f3 = (ao.m_bottomNorthEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueZNeg
                    + ao.m_bottomNorthEastVertexAmbientOcclusionLight
                    + ao.m_northEastEdgeAmbientOcclusionLight)
                    / 4.0;
                f4 = (ao.m_bottomNorthWestVertexAmbientOcclusionLight
                    + ao.m_northWestEdgeAmbientOcclusionLight
                    + ao.m_bottomNorthEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueZNeg)
                    / 4.0;
            }
            else
            {
                (f, f2, f3, f4) =
                    (ao.m_aoLightValueZNeg, ao.m_aoLightValueZNeg, ao.m_aoLightValueZNeg, ao.m_aoLightValueZNeg);
            }
            let shade = |color: f32| (if bl4 { color } else { 1.0 }) * 0.8;
            self.setAoColors(shade(r), shade(g), shade(b), f, f2, f3, f4);
            let n: i32 = block.getBlockTexture(&*world, x, y, z, 2);
            self.renderEastFace(block, x as f64, y as f64, z as f64, n);
            if self.grassOverlay(n)
            {
                self.tintAoColors(r, g, b);
                self.renderEastFace(block, x as f64, y as f64, z as f64, 38);
            }
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x, y, z + 1, 3)
        {
            let ao: &mut AoState = &mut self.m_ao;
            if occlude
            {
                let z: i32 = z + 1;
                ao.m_southWestEdgeAmbientOcclusionLight = light(x - 1, y, z);
                ao.m_southEastEdgeAmbientOcclusionLight = light(x + 1, y, z);
                ao.m_bottomSouthEdgeAmbientOcclusionLight = light(x, y - 1, z);
                ao.m_topSouthEdgeAmbientOcclusionLight = light(x, y + 1, z);
                ao.m_bottomSouthWestVertexAmbientOcclusionLight = if ao.m_southWestEdgeTranslucent
                    || ao.m_bottomSouthEdgeTranslucent
                {
                    light(x - 1, y - 1, z)
                }
                else
                {
                    ao.m_southWestEdgeAmbientOcclusionLight
                };
                ao.m_topSouthWestVertexAmbientOcclusionLight = if ao.m_southWestEdgeTranslucent
                    || ao.m_topSouthEdgeTranslucent
                {
                    light(x - 1, y + 1, z)
                }
                else
                {
                    ao.m_southWestEdgeAmbientOcclusionLight
                };
                ao.m_bottomSouthEastVertexAmbientOcclusionLight = if ao.m_southEastEdgeTranslucent
                    || ao.m_bottomSouthEdgeTranslucent
                {
                    light(x + 1, y - 1, z)
                }
                else
                {
                    ao.m_southEastEdgeAmbientOcclusionLight
                };
                ao.m_topSouthEastVertexAmbientOcclusionLight = if ao.m_southEastEdgeTranslucent
                    || ao.m_topSouthEdgeTranslucent
                {
                    light(x + 1, y + 1, z)
                }
                else
                {
                    ao.m_southEastEdgeAmbientOcclusionLight
                };
                f = (ao.m_southWestEdgeAmbientOcclusionLight
                    + ao.m_topSouthWestVertexAmbientOcclusionLight
                    + ao.m_aoLightValueZPos
                    + ao.m_topSouthEdgeAmbientOcclusionLight)
                    / 4.0;
                f4 = (ao.m_aoLightValueZPos
                    + ao.m_topSouthEdgeAmbientOcclusionLight
                    + ao.m_southEastEdgeAmbientOcclusionLight
                    + ao.m_topSouthEastVertexAmbientOcclusionLight)
                    / 4.0;
                f3 = (ao.m_bottomSouthEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueZPos
                    + ao.m_bottomSouthEastVertexAmbientOcclusionLight
                    + ao.m_southEastEdgeAmbientOcclusionLight)
                    / 4.0;
                f2 = (ao.m_bottomSouthWestVertexAmbientOcclusionLight
                    + ao.m_southWestEdgeAmbientOcclusionLight
                    + ao.m_bottomSouthEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueZPos)
                    / 4.0;
            }
            else
            {
                (f, f2, f3, f4) =
                    (ao.m_aoLightValueZPos, ao.m_aoLightValueZPos, ao.m_aoLightValueZPos, ao.m_aoLightValueZPos);
            }
            let shade = |color: f32| (if bl5 { color } else { 1.0 }) * 0.8;
            self.setAoColors(shade(r), shade(g), shade(b), f, f2, f3, f4);
            let n: i32 = block.getBlockTexture(&*world, x, y, z, 3);
            self.renderWestFace(block, x as f64, y as f64, z as f64, block.getBlockTexture(&*world, x, y, z, 3));
            if self.grassOverlay(n)
            {
                self.tintAoColors(r, g, b);
                self.renderWestFace(block, x as f64, y as f64, z as f64, 38);
            }
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x - 1, y, z, 4)
        {
            let ao: &mut AoState = &mut self.m_ao;
            if occlude
            {
                let x: i32 = x - 1;
                ao.m_bottomWestEdgeAmbientOcclusionLight = light(x, y - 1, z);
                ao.m_northWestEdgeAmbientOcclusionLight = light(x, y, z - 1);
                ao.m_southWestEdgeAmbientOcclusionLight = light(x, y, z + 1);
                ao.m_topWestEdgeAmbientOcclusionLight = light(x, y + 1, z);
                ao.m_bottomNorthWestVertexAmbientOcclusionLight = if ao.m_northWestEdgeTranslucent
                    || ao.m_bottomWestEdgeTranslucent
                {
                    light(x, y - 1, z - 1)
                }
                else
                {
                    ao.m_northWestEdgeAmbientOcclusionLight
                };
                ao.m_bottomSouthWestVertexAmbientOcclusionLight = if ao.m_southWestEdgeTranslucent
                    || ao.m_bottomWestEdgeTranslucent
                {
                    light(x, y - 1, z + 1)
                }
                else
                {
                    ao.m_southWestEdgeAmbientOcclusionLight
                };
                ao.m_topNorthWestVertexAmbientOcclusionLight = if ao.m_northWestEdgeTranslucent
                    || ao.m_topWestEdgeTranslucent
                {
                    light(x, y + 1, z - 1)
                }
                else
                {
                    ao.m_northWestEdgeAmbientOcclusionLight
                };
                ao.m_topSouthWestVertexAmbientOcclusionLight = if ao.m_southWestEdgeTranslucent
                    || ao.m_topWestEdgeTranslucent
                {
                    light(x, y + 1, z + 1)
                }
                else
                {
                    ao.m_southWestEdgeAmbientOcclusionLight
                };
                f4 = (ao.m_bottomWestEdgeAmbientOcclusionLight
                    + ao.m_bottomSouthWestVertexAmbientOcclusionLight
                    + ao.m_aoLightValueXNeg
                    + ao.m_southWestEdgeAmbientOcclusionLight)
                    / 4.0;
                f = (ao.m_aoLightValueXNeg
                    + ao.m_southWestEdgeAmbientOcclusionLight
                    + ao.m_topWestEdgeAmbientOcclusionLight
                    + ao.m_topSouthWestVertexAmbientOcclusionLight)
                    / 4.0;
                f2 = (ao.m_northWestEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueXNeg
                    + ao.m_topNorthWestVertexAmbientOcclusionLight
                    + ao.m_topWestEdgeAmbientOcclusionLight)
                    / 4.0;
                f3 = (ao.m_bottomNorthWestVertexAmbientOcclusionLight
                    + ao.m_bottomWestEdgeAmbientOcclusionLight
                    + ao.m_northWestEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueXNeg)
                    / 4.0;
            }
            else
            {
                (f, f2, f3, f4) =
                    (ao.m_aoLightValueXNeg, ao.m_aoLightValueXNeg, ao.m_aoLightValueXNeg, ao.m_aoLightValueXNeg);
            }
            let shade = |color: f32| (if bl6 { color } else { 1.0 }) * 0.6;
            self.setAoColors(shade(r), shade(g), shade(b), f, f2, f3, f4);
            let n: i32 = block.getBlockTexture(&*world, x, y, z, 4);
            self.renderNorthFace(block, x as f64, y as f64, z as f64, n);
            if self.grassOverlay(n)
            {
                self.tintAoColors(r, g, b);
                self.renderNorthFace(block, x as f64, y as f64, z as f64, 38);
            }
            bl = true;
        }
        if self.m_renderAllFaces || block.shouldSideBeRendered(&*world, x + 1, y, z, 5)
        {
            let ao: &mut AoState = &mut self.m_ao;
            if occlude
            {
                let x: i32 = x + 1;
                ao.m_bottomEastEdgeAmbientOcclusionLight = light(x, y - 1, z);
                ao.m_northEastEdgeAmbientOcclusionLight = light(x, y, z - 1);
                ao.m_southEastEdgeAmbientOcclusionLight = light(x, y, z + 1);
                ao.m_topEastEdgeAmbientOcclusionLight = light(x, y + 1, z);
                ao.m_bottomNorthEastVertexAmbientOcclusionLight = if ao.m_bottomEastEdgeTranslucent
                    || ao.m_northEastEdgeTranslucent
                {
                    light(x, y - 1, z - 1)
                }
                else
                {
                    ao.m_northEastEdgeAmbientOcclusionLight
                };
                ao.m_bottomSouthEastVertexAmbientOcclusionLight = if ao.m_bottomEastEdgeTranslucent
                    || ao.m_southEastEdgeTranslucent
                {
                    light(x, y - 1, z + 1)
                }
                else
                {
                    ao.m_southEastEdgeAmbientOcclusionLight
                };
                ao.m_topNorthEastVertexAmbientOcclusionLight = if ao.m_topEastEdgeTranslucent
                    || ao.m_northEastEdgeTranslucent
                {
                    light(x, y + 1, z - 1)
                }
                else
                {
                    ao.m_northEastEdgeAmbientOcclusionLight
                };
                ao.m_topSouthEastVertexAmbientOcclusionLight = if ao.m_topEastEdgeTranslucent
                    || ao.m_southEastEdgeTranslucent
                {
                    light(x, y + 1, z + 1)
                }
                else
                {
                    ao.m_southEastEdgeAmbientOcclusionLight
                };
                f = (ao.m_bottomEastEdgeAmbientOcclusionLight
                    + ao.m_bottomSouthEastVertexAmbientOcclusionLight
                    + ao.m_aoLightValueXPos
                    + ao.m_southEastEdgeAmbientOcclusionLight)
                    / 4.0;
                f4 = (ao.m_aoLightValueXPos
                    + ao.m_southEastEdgeAmbientOcclusionLight
                    + ao.m_topEastEdgeAmbientOcclusionLight
                    + ao.m_topSouthEastVertexAmbientOcclusionLight)
                    / 4.0;
                f3 = (ao.m_northEastEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueXPos
                    + ao.m_topNorthEastVertexAmbientOcclusionLight
                    + ao.m_topEastEdgeAmbientOcclusionLight)
                    / 4.0;
                f2 = (ao.m_bottomNorthEastVertexAmbientOcclusionLight
                    + ao.m_bottomEastEdgeAmbientOcclusionLight
                    + ao.m_northEastEdgeAmbientOcclusionLight
                    + ao.m_aoLightValueXPos)
                    / 4.0;
            }
            else
            {
                (f, f2, f3, f4) =
                    (ao.m_aoLightValueXPos, ao.m_aoLightValueXPos, ao.m_aoLightValueXPos, ao.m_aoLightValueXPos);
            }
            let shade = |color: f32| (if bl7 { color } else { 1.0 }) * 0.6;
            self.setAoColors(shade(r), shade(g), shade(b), f, f2, f3, f4);
            let n: i32 = block.getBlockTexture(&*world, x, y, z, 5);
            self.renderSouthFace(block, x as f64, y as f64, z as f64, n);
            if self.grassOverlay(n)
            {
                self.tintAoColors(r, g, b);
                self.renderSouthFace(block, x as f64, y as f64, z as f64, 38);
            }
            bl = true;
        }
        self.m_enableAO = false;
        bl
    }
}
