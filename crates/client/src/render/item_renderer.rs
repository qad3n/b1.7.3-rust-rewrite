use crate::client::Minecraft;
use crate::port::render::castToRenderPlayer;
use crate::render::entity::{Render, render_player};
use crate::render::{MapItemRenderer, RenderBlocks, RenderEngine, RenderHelper, RenderManager, Tessellator};
use game::block::{Block, Material};
use game::entity::{Entity, entity_living, entity_player};
use game::item::{Item, ItemStack};
use game::math::math_helper::{cos, floor_double, floor_float, sin, sqrt_float};
use game::world::World;
use jvm::field::Field;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::{Cell, RefCell};
use std::f32::consts::PI;
use std::rc::Rc;

pub struct ItemRenderer
{
    m_mc: Rc<Minecraft>,
    m_itemToRender: Field<Option<Rc<ItemStack>>>,
    m_equippedProgress: Cell<f32>,
    m_prevEquippedProgress: Cell<f32>,
    m_renderBlocksInstance: RefCell<RenderBlocks>,
    m_mapRenderer: MapItemRenderer,
    m_selectedSlot: Cell<i32>,
}

fn sameStack(a: &Option<Rc<ItemStack>>, b: &Option<Rc<ItemStack>>) -> bool
{
    match (a, b)
    {
        (None, None) => true,
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

impl ItemRenderer
{
    pub fn new(minecraft: Rc<Minecraft>) -> ItemRenderer
    {
        let mapRenderer: MapItemRenderer = MapItemRenderer::new(
            minecraft.m_fontRenderer.get().orNpe(),
            minecraft.m_gameSettings.get().orNpe(),
            &minecraft.m_renderEngine.get().orNpe(),
        );
        ItemRenderer {
            m_mc: minecraft,
            m_itemToRender: Field::new(None),
            m_equippedProgress: Cell::new(0.0),
            m_prevEquippedProgress: Cell::new(0.0),
            m_renderBlocksInstance: RefCell::new(RenderBlocks::empty()),
            m_mapRenderer: mapRenderer,
            m_selectedSlot: Cell::new(-1),
        }
    }

    fn renderEngine(&self) -> Rc<RenderEngine> { self.m_mc.m_renderEngine.get().orNpe() }

    fn thePlayer(&self) -> Rc<Entity> { self.m_mc.m_thePlayer.get().orNpe() }

    fn theWorld(&self) -> Rc<World> { self.m_mc.m_theWorld.get().orNpe() }

    pub fn renderItem(&self, mob: &Rc<Entity>, itemInHand: &Rc<ItemStack>)
    {
        GL11::glPushMatrix();
        let itemID: i32 = itemInHand.m_itemID.get();
        if itemID < 256 && RenderBlocks::renderItemIn3d(Block::byId(itemID).orNpe().getRenderType())
        {
            GL11::glBindTexture(3553, self.renderEngine().getTexture("/terrain.png"));
            self.m_renderBlocksInstance.borrow_mut().renderBlockOnInventory(
                &Block::byId(itemID).orNpe(),
                itemInHand.getItemDamage(),
                mob.getEntityBrightness(1.0),
            );
        }
        else
        {
            let path: &str = if itemID < 256
            {
                "/terrain.png"
            }
            else
            {
                "/gui/items.png"
            };
            GL11::glBindTexture(3553, self.renderEngine().getTexture(path));
            self.renderFlatItem(mob.getItemIcon(itemInHand));
        }
        GL11::glPopMatrix();
    }

    fn renderFlatItem(&self, i: i32)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let f: f32 = ((i % 16 * 16) as f32 + 0.0) / 256.0;
        let f1: f32 = ((i % 16 * 16) as f32 + 15.99) / 256.0;
        let f2: f32 = ((i / 16 * 16) as f32 + 0.0) / 256.0;
        let f3: f32 = ((i / 16 * 16) as f32 + 15.99) / 256.0;
        let f4: f32 = 1.0;
        let f5: f32 = 0.0;
        let f6: f32 = 0.3;
        GL11::glEnable(32826);
        GL11::glTranslatef(-f5, -f6, 0.0);
        let f7: f32 = 1.5;
        GL11::glScalef(f7, f7, f7);
        GL11::glRotatef(50.0, 0.0, 1.0, 0.0);
        GL11::glRotatef(335.0, 0.0, 0.0, 1.0);
        GL11::glTranslatef(-0.9375, -0.0625, 0.0);
        let f8: f32 = 0.0625;
        let back: f64 = (0.0 - f8) as f64;
        let v =
            |x: f32, y: f64, z: f64, u: f32, w: f32| tessellator.addVertexWithUV(x as f64, y, z, u as f64, w as f64);
        tessellator.startDrawingQuads();
        tessellator.setNormal(0.0, 0.0, 1.0);
        v(0.0, 0.0, 0.0, f1, f3);
        v(f4, 0.0, 0.0, f, f3);
        v(f4, 1.0, 0.0, f, f2);
        v(0.0, 1.0, 0.0, f1, f2);
        tessellator.draw();
        tessellator.startDrawingQuads();
        tessellator.setNormal(0.0, 0.0, -1.0);
        v(0.0, 1.0, back, f1, f2);
        v(f4, 1.0, back, f, f2);
        v(f4, 0.0, back, f, f3);
        v(0.0, 0.0, back, f1, f3);
        tessellator.draw();
        tessellator.startDrawingQuads();
        tessellator.setNormal(-1.0, 0.0, 0.0);
        for j in 0..16
        {
            let f9: f32 = j as f32 / 16.0;
            let f10: f32 = f1 + (f - f1) * f9 - 0.001953125;
            let f11: f32 = f4 * f9;
            v(f11, 0.0, back, f10, f3);
            v(f11, 0.0, 0.0, f10, f3);
            v(f11, 1.0, 0.0, f10, f2);
            v(f11, 1.0, back, f10, f2);
        }
        tessellator.draw();
        tessellator.startDrawingQuads();
        tessellator.setNormal(1.0, 0.0, 0.0);
        for k in 0..16
        {
            let f12: f32 = k as f32 / 16.0;
            let f15: f32 = f1 + (f - f1) * f12 - 0.001953125;
            let f18: f32 = f4 * f12 + 0.0625;
            v(f18, 1.0, back, f15, f2);
            v(f18, 1.0, 0.0, f15, f2);
            v(f18, 0.0, 0.0, f15, f3);
            v(f18, 0.0, back, f15, f3);
        }
        tessellator.draw();
        let h = |x: f32, y: f32, z: f64, u: f32, w: f32| {
            tessellator.addVertexWithUV(x as f64, y as f64, z, u as f64, w as f64)
        };
        tessellator.startDrawingQuads();
        tessellator.setNormal(0.0, 1.0, 0.0);
        for l in 0..16
        {
            let f13: f32 = l as f32 / 16.0;
            let f16: f32 = f3 + (f2 - f3) * f13 - 0.001953125;
            let f19: f32 = f4 * f13 + 0.0625;
            h(0.0, f19, 0.0, f1, f16);
            h(f4, f19, 0.0, f, f16);
            h(f4, f19, back, f, f16);
            h(0.0, f19, back, f1, f16);
        }
        tessellator.draw();
        tessellator.startDrawingQuads();
        tessellator.setNormal(0.0, -1.0, 0.0);
        for i1 in 0..16
        {
            let f14: f32 = i1 as f32 / 16.0;
            let f17: f32 = f3 + (f2 - f3) * f14 - 0.001953125;
            let f20: f32 = f4 * f14;
            h(f4, f20, 0.0, f, f17);
            h(0.0, f20, 0.0, f1, f17);
            h(0.0, f20, back, f1, f17);
            h(f4, f20, back, f, f17);
        }
        tessellator.draw();
        GL11::glDisable(32826);
    }

    fn bindPlayerSkin(&self)
    {
        let player: Rc<Entity> = self.thePlayer();
        let skin: String = JString::display(player.m_skinUrl.get().as_ref());
        let texture: Option<String> = player.getEntityTexture().map(|texture| JString::display(Some(&texture)));
        GL11::glBindTexture(3553, self.renderEngine().getTextureForDownloadableImage(&skin, texture.as_deref()));
    }

    fn swingTranslate(&self, player: &Entity, tickDelta: f32, side: f32, lift: f32, depth: f32)
    {
        let f10: f32 = entity_living::getSwingProgress(player, tickDelta);
        let f13: f32 = sin(f10 * PI);
        let f16: f32 = sin(sqrt_float(f10) * PI);
        GL11::glTranslatef(-f16 * side, sin(sqrt_float(f10) * PI * 2.0) * lift, -f13 * depth);
    }

    pub fn renderItemInFirstPerson(&self, tickDelta: f32)
    {
        let f: f32 = self.m_prevEquippedProgress.get()
            + (self.m_equippedProgress.get() - self.m_prevEquippedProgress.get()) * tickDelta;
        let player: Rc<Entity> = self.thePlayer();
        let f1: f32 = player.m_prevRotationPitch.get()
            + (player.m_rotationPitch.get() - player.m_prevRotationPitch.get()) * tickDelta;
        GL11::glPushMatrix();
        GL11::glRotatef(f1, 1.0, 0.0, 0.0);
        GL11::glRotatef(
            player.m_prevRotationYaw.get() + (player.m_rotationYaw.get() - player.m_prevRotationYaw.get()) * tickDelta,
            0.0,
            1.0,
            0.0,
        );
        RenderHelper::enableStandardItemLighting();
        GL11::glPopMatrix();
        let itemstack: Option<Rc<ItemStack>> = self.m_itemToRender.get();
        let f2: f32 = self.theWorld().getLightBrightness(
            floor_double(player.posX()),
            floor_double(player.posY()),
            floor_double(player.posZ()),
        );
        match &itemstack
        {
            Some(itemstack) =>
            {
                let i: i32 = Item::byIdOrNpe(itemstack.m_itemID.get()).getColorFromDamage(itemstack.getItemDamage());
                let f3: f32 = (i >> 16 & 0xFF) as f32 / 255.0;
                let f4: f32 = (i >> 8 & 0xFF) as f32 / 255.0;
                let f5: f32 = (i & 0xFF) as f32 / 255.0;
                GL11::glColor4f(f2 * f3, f2 * f4, f2 * f5, 1.0);
            }
            None => GL11::glColor4f(f2, f2, f2, 1.0),
        }
        match &itemstack
        {
            Some(itemstack) if itemstack.m_itemID.get() == Item::mapItem().m_shiftedIndex =>
            {
                self.renderMapInFirstPerson(&player, itemstack, tickDelta, f, f1);
            }
            Some(itemstack) =>
            {
                GL11::glPushMatrix();
                let f7: f32 = 0.8;
                self.swingTranslate(&player, tickDelta, 0.4, 0.2, 0.2);
                GL11::glTranslatef(0.7 * f7, -0.65 * f7 - (1.0 - f) * 0.6, -0.9 * f7);
                GL11::glRotatef(45.0, 0.0, 1.0, 0.0);
                GL11::glEnable(32826);
                let f10: f32 = entity_living::getSwingProgress(&player, tickDelta);
                let f13: f32 = sin(f10 * f10 * PI);
                let f16: f32 = sin(sqrt_float(f10) * PI);
                GL11::glRotatef(-f13 * 20.0, 0.0, 1.0, 0.0);
                GL11::glRotatef(-f16 * 20.0, 0.0, 0.0, 1.0);
                GL11::glRotatef(-f16 * 80.0, 1.0, 0.0, 0.0);
                let scale: f32 = 0.4;
                GL11::glScalef(scale, scale, scale);
                if itemstack.getItem().shouldRotateAroundWhenRendering()
                {
                    GL11::glRotatef(180.0, 0.0, 1.0, 0.0);
                }
                self.renderItem(&player, itemstack);
                GL11::glPopMatrix();
            }
            None =>
            {
                GL11::glPushMatrix();
                let f8: f32 = 0.8;
                self.swingTranslate(&player, tickDelta, 0.3, 0.4, 0.4);
                GL11::glTranslatef(0.8 * f8, -0.75 * f8 - (1.0 - f) * 0.6, -0.9 * f8);
                GL11::glRotatef(45.0, 0.0, 1.0, 0.0);
                GL11::glEnable(32826);
                let f11: f32 = entity_living::getSwingProgress(&player, tickDelta);
                let f14: f32 = sin(f11 * f11 * PI);
                let f17: f32 = sin(sqrt_float(f11) * PI);
                GL11::glRotatef(f17 * 70.0, 0.0, 1.0, 0.0);
                GL11::glRotatef(-f14 * 20.0, 0.0, 0.0, 1.0);
                self.bindPlayerSkin();
                GL11::glTranslatef(-1.0, 3.6, 3.5);
                GL11::glRotatef(120.0, 0.0, 0.0, 1.0);
                GL11::glRotatef(200.0, 1.0, 0.0, 0.0);
                GL11::glRotatef(-135.0, 0.0, 1.0, 0.0);
                GL11::glScalef(1.0, 1.0, 1.0);
                GL11::glTranslatef(5.6, 0.0, 0.0);
                GL11::glScalef(1.0, 1.0, 1.0);
                // the java (RenderPlayer) cast runs in port because Render has no subclass types to downcast
                let renderplayer: Rc<Render> =
                    castToRenderPlayer(RenderManager::instance().getEntityRenderObject(&player));
                render_player::drawFirstPersonHand(&renderplayer);
                GL11::glPopMatrix();
            }
        }
        GL11::glDisable(32826);
        RenderHelper::disableStandardItemLighting();
    }

    fn renderMapInFirstPerson(&self, player: &Rc<Entity>, itemstack: &Rc<ItemStack>, tickDelta: f32, f: f32, f1: f32)
    {
        GL11::glPushMatrix();
        let f9: f32 = 0.8;
        self.swingTranslate(player, tickDelta, 0.4, 0.2, 0.2);
        let mut f12: f32 = (1.0 - f1 / 45.0 + 0.1).clamp(0.0, 1.0);
        f12 = -cos(f12 * PI) * 0.5 + 0.5;
        GL11::glTranslatef(0.0, 0.0 * f9 - (1.0 - f) * 1.2 - f12 * 0.5 + 0.04, -0.9 * f9);
        GL11::glRotatef(90.0, 0.0, 1.0, 0.0);
        GL11::glRotatef(f12 * -85.0, 0.0, 0.0, 1.0);
        GL11::glEnable(32826);
        self.bindPlayerSkin();
        for j in 0..2
        {
            let k: i32 = j * 2 - 1;
            GL11::glPushMatrix();
            GL11::glTranslatef(-0.0, -0.6, 1.1 * k as f32);
            GL11::glRotatef((-45 * k) as f32, 1.0, 0.0, 0.0);
            GL11::glRotatef(-90.0, 0.0, 0.0, 1.0);
            GL11::glRotatef(59.0, 0.0, 0.0, 1.0);
            GL11::glRotatef((-65 * k) as f32, 0.0, 1.0, 0.0);
            GL11::glScalef(1.0, 1.0, 1.0);
            // the java (RenderPlayer) cast runs in port because Render has no subclass types to downcast
            let renderplayer: Rc<Render> = castToRenderPlayer(RenderManager::instance().getEntityRenderObject(player));
            render_player::drawFirstPersonHand(&renderplayer);
            GL11::glPopMatrix();
        }
        let f15: f32 = entity_living::getSwingProgress(player, tickDelta);
        let f18: f32 = sin(f15 * f15 * PI);
        let f19: f32 = sin(sqrt_float(f15) * PI);
        GL11::glRotatef(-f18 * 20.0, 0.0, 1.0, 0.0);
        GL11::glRotatef(-f19 * 20.0, 0.0, 0.0, 1.0);
        GL11::glRotatef(-f19 * 80.0, 1.0, 0.0, 0.0);
        let scale: f32 = 0.38;
        GL11::glScalef(scale, scale, scale);
        GL11::glRotatef(90.0, 0.0, 1.0, 0.0);
        GL11::glRotatef(180.0, 0.0, 0.0, 1.0);
        GL11::glTranslatef(-1.0, -1.0, 0.0);
        let pixel: f32 = 0.015625;
        GL11::glScalef(pixel, pixel, pixel);
        let renderEngine: Rc<RenderEngine> = self.renderEngine();
        renderEngine.bindTexture(renderEngine.getTexture("/misc/mapbg.png"));
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glNormal3f(0.0, 0.0, -1.0);
        tessellator.startDrawingQuads();
        let b0: i32 = 7;
        tessellator.addVertexWithUV((0 - b0) as f64, (128 + b0) as f64, 0.0, 0.0, 1.0);
        tessellator.addVertexWithUV((128 + b0) as f64, (128 + b0) as f64, 0.0, 1.0, 1.0);
        tessellator.addVertexWithUV((128 + b0) as f64, (0 - b0) as f64, 0.0, 1.0, 0.0);
        tessellator.addVertexWithUV((0 - b0) as f64, (0 - b0) as f64, 0.0, 0.0, 0.0);
        tessellator.draw();
        let mapdata: Rc<game::world::MapDataBase> = game::item::item_map::getSavedMapData(itemstack, &self.theWorld());
        self.m_mapRenderer.render(player, &renderEngine, &mapdata);
        GL11::glPopMatrix();
    }

    pub fn renderOverlays(&self, tickDelta: f32)
    {
        GL11::glDisable(3008);
        let player: Rc<Entity> = self.thePlayer();
        let renderEngine: Rc<RenderEngine> = self.renderEngine();
        if player.isBurning()
        {
            GL11::glBindTexture(3553, renderEngine.getTexture("/terrain.png"));
            self.renderFireInFirstPerson(tickDelta);
        }
        if player.isEntityInsideOpaqueBlock()
        {
            let world: Rc<World> = self.theWorld();
            let j2: i32 = floor_double(player.posX());
            let j: i32 = floor_double(player.posY());
            let k: i32 = floor_double(player.posZ());
            GL11::glBindTexture(3553, renderEngine.getTexture("/terrain.png"));
            let mut i1: i32 = world.getBlockId(j2, j, k);
            if world.isBlockNormalCube(j2, j, k)
            {
                self.renderInsideOfBlock(tickDelta, Block::byId(i1).orNpe().getBlockTextureFromSide(2));
            }
            else
            {
                for j1 in 0..8
                {
                    let f: f32 = (((j1 >> 0) % 2) as f32 - 0.5) * player.m_width.get() * 0.9;
                    let f1: f32 = (((j1 >> 1) % 2) as f32 - 0.5) * player.m_height.get() * 0.2;
                    let f2: f32 = (((j1 >> 2) % 2) as f32 - 0.5) * player.m_width.get() * 0.9;
                    let k1: i32 = floor_float(j2 as f32 + f);
                    let l1: i32 = floor_float(j as f32 + f1);
                    let i2: i32 = floor_float(k as f32 + f2);
                    if world.isBlockNormalCube(k1, l1, i2)
                    {
                        i1 = world.getBlockId(k1, l1, i2);
                    }
                }
            }
            if let Some(block) = Block::byId(i1)
            {
                self.renderInsideOfBlock(tickDelta, block.getBlockTextureFromSide(2));
            }
        }
        if player.isInsideOfMaterial(&Material::water())
        {
            GL11::glBindTexture(3553, renderEngine.getTexture("/misc/water.png"));
            self.renderWarpedTextureOverlay(tickDelta);
        }
        GL11::glEnable(3008);
    }

    fn renderInsideOfBlock(&self, tickDelta: f32, sprite: i32)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        self.thePlayer().getEntityBrightness(tickDelta);
        let f: f32 = 0.1;
        GL11::glColor4f(f, f, f, 0.5);
        GL11::glPushMatrix();
        let (f1, f2, f3, f4, f5): (f32, f32, f32, f32, f32) = (-1.0, 1.0, -1.0, 1.0, -0.5);
        let f6: f32 = 0.0078125;
        let f7: f32 = (sprite % 16) as f32 / 256.0 - f6;
        let f8: f32 = ((sprite % 16) as f32 + 15.99) / 256.0 + f6;
        let f9: f32 = (sprite / 16) as f32 / 256.0 - f6;
        let f10: f32 = ((sprite / 16) as f32 + 15.99) / 256.0 + f6;
        let v = |x: f32, y: f32, z: f32, u: f32, w: f32| {
            tessellator.addVertexWithUV(x as f64, y as f64, z as f64, u as f64, w as f64)
        };
        tessellator.startDrawingQuads();
        v(f1, f3, f5, f8, f10);
        v(f2, f3, f5, f7, f10);
        v(f2, f4, f5, f7, f9);
        v(f1, f4, f5, f8, f9);
        tessellator.draw();
        GL11::glPopMatrix();
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    }

    fn renderWarpedTextureOverlay(&self, tickDelta: f32)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let player: Rc<Entity> = self.thePlayer();
        let f: f32 = player.getEntityBrightness(tickDelta);
        GL11::glColor4f(f, f, f, 0.5);
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        GL11::glPushMatrix();
        let f1: f32 = 4.0;
        let (f2, f3, f4, f5, f6): (f32, f32, f32, f32, f32) = (-1.0, 1.0, -1.0, 1.0, -0.5);
        let f7: f32 = -player.m_rotationYaw.get() / 64.0;
        let f8: f32 = player.m_rotationPitch.get() / 64.0;
        let v = |x: f32, y: f32, z: f32, u: f32, w: f32| {
            tessellator.addVertexWithUV(x as f64, y as f64, z as f64, u as f64, w as f64)
        };
        tessellator.startDrawingQuads();
        v(f2, f4, f6, f1 + f7, f1 + f8);
        v(f3, f4, f6, 0.0 + f7, f1 + f8);
        v(f3, f5, f6, 0.0 + f7, 0.0 + f8);
        v(f2, f5, f6, f1 + f7, 0.0 + f8);
        tessellator.draw();
        GL11::glPopMatrix();
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glDisable(3042);
    }

    fn renderFireInFirstPerson(&self, _tickDelta: f32)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        GL11::glColor4f(1.0, 1.0, 1.0, 0.9);
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        let f: f32 = 1.0;
        for i in 0..2
        {
            GL11::glPushMatrix();
            let j: i32 = Block::fire().m_blockIndexInTexture.get() + i * 16;
            let k: i32 = (j & 15) << 4;
            let l: i32 = j & 240;
            let f1: f32 = k as f32 / 256.0;
            let f2: f32 = (k as f32 + 15.99) / 256.0;
            let f3: f32 = l as f32 / 256.0;
            let f4: f32 = (l as f32 + 15.99) / 256.0;
            let f5: f32 = (0.0 - f) / 2.0;
            let f6: f32 = f5 + f;
            let f7: f32 = 0.0 - f / 2.0;
            let f8: f32 = f7 + f;
            let f9: f32 = -0.5;
            GL11::glTranslatef(-(i * 2 - 1) as f32 * 0.24, -0.3, 0.0);
            GL11::glRotatef((i * 2 - 1) as f32 * 10.0, 0.0, 1.0, 0.0);
            let v = |x: f32, y: f32, z: f32, u: f32, w: f32| {
                tessellator.addVertexWithUV(x as f64, y as f64, z as f64, u as f64, w as f64)
            };
            tessellator.startDrawingQuads();
            v(f5, f7, f9, f2, f4);
            v(f6, f7, f9, f1, f4);
            v(f6, f8, f9, f1, f3);
            v(f5, f8, f9, f2, f3);
            tessellator.draw();
            GL11::glPopMatrix();
        }
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glDisable(3042);
    }

    pub fn updateEquippedItem(&self)
    {
        self.m_prevEquippedProgress.set(self.m_equippedProgress.get());
        let player: Rc<Entity> = self.thePlayer();
        let inventory: Rc<game::inventory::InventoryPlayer> = entity_player::inventory(&player);
        let itemstack1: Option<Rc<ItemStack>> = inventory.getCurrentItem();
        let toRender: Option<Rc<ItemStack>> = self.m_itemToRender.get();
        let mut flag: bool =
            self.m_selectedSlot.get() == inventory.m_currentItem.get() && sameStack(&itemstack1, &toRender);
        if toRender.is_none() && itemstack1.is_none()
        {
            flag = true;
        }
        if let (Some(stack), Some(current)) = (&itemstack1, &toRender)
            && !Rc::ptr_eq(stack, current)
            && stack.m_itemID.get() == current.m_itemID.get()
            && stack.getItemDamage() == current.getItemDamage()
        {
            self.m_itemToRender.set(itemstack1.clone());
            flag = true;
        }
        let f: f32 = 0.4;
        let f1: f32 = if flag { 1.0 } else { 0.0 };
        let f2: f32 = (f1 - self.m_equippedProgress.get()).clamp(-f, f);
        self.m_equippedProgress.set(self.m_equippedProgress.get() + f2);
        if self.m_equippedProgress.get() < 0.1
        {
            self.m_itemToRender.set(itemstack1);
            self.m_selectedSlot.set(inventory.m_currentItem.get());
        }
    }

    pub fn onBlockUsed(&self) { self.m_equippedProgress.set(0.0); }

    pub fn onItemUsed(&self) { self.m_equippedProgress.set(0.0); }
}
