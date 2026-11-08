use crate::client::{ChatLine, Minecraft};
use crate::entity::entity_player_sp;
use crate::gui::{FontRenderer, Gui, ScaledResolution};
use crate::port::{awt, runtime};
use crate::render::{RenderEngine, RenderHelper, RenderItem, Tessellator};
use crate::settings::GameSettings;
use game::block::{Block, Material};
use game::entity::{Entity, entity_living, entity_player};
use game::inventory::InventoryPlayer;
use game::item::ItemStack;
use game::locale::StringTranslate;
use game::math::math_helper::floor_double;
use jvm::field::Field;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

thread_local! {
    static ITEM_RENDERER: OnceCell<Rc<RenderItem>> = const { OnceCell::new() };
}

fn itemRenderer() -> Rc<RenderItem>
{
    ITEM_RENDERER.with(|renderer| Rc::clone(renderer.get_or_init(|| Rc::new(RenderItem::new()))))
}

pub struct GuiIngame
{
    pub m_gui: Gui,
    m_chatMessageList: RefCell<Vec<ChatLine>>,
    m_rand: Random,
    m_mc: Rc<Minecraft>,
    pub m_selectedName: Field<Option<String>>,
    m_updateCounter: Cell<i32>,
    m_recordPlaying: Field<String>,
    m_recordPlayingUpFor: Cell<i32>,
    m_overlayMessageTinted: Cell<bool>,
    pub m_damageGuiPartialTime: Cell<f32>,
    m_prevVignetteBrightness: Cell<f32>,
}

impl GuiIngame
{
    pub fn new(minecraft: Rc<Minecraft>) -> GuiIngame
    {
        GuiIngame {
            m_gui: Gui::new(),
            m_chatMessageList: RefCell::new(Vec::new()),
            m_rand: Random::new(),
            m_mc: minecraft,
            m_selectedName: Field::new(None),
            m_updateCounter: Cell::new(0),
            m_recordPlaying: Field::new(String::new()),
            m_recordPlayingUpFor: Cell::new(0),
            m_overlayMessageTinted: Cell::new(false),
            m_damageGuiPartialTime: Cell::new(0.0),
            m_prevVignetteBrightness: Cell::new(1.0),
        }
    }

    fn player(&self) -> Rc<Entity> { self.m_mc.m_thePlayer.get().orNpe() }

    fn renderEngine(&self) -> Rc<RenderEngine> { self.m_mc.m_renderEngine.get().orNpe() }

    fn settings(&self) -> Rc<GameSettings> { self.m_mc.m_gameSettings.get().orNpe() }

    pub fn renderGameOverlay(&self, tickDelta: f32, _screenOpen: bool, _mouseX: i32, _mouseY: i32)
    {
        let mc: &Rc<Minecraft> = &self.m_mc;
        let scaledresolution: ScaledResolution =
            ScaledResolution::new(&self.settings(), mc.m_displayWidth.get(), mc.m_displayHeight.get());
        let i: i32 = scaledresolution.getScaledWidth();
        let j: i32 = scaledresolution.getScaledHeight();
        let fontrenderer: Rc<FontRenderer> = mc.m_fontRenderer.get().orNpe();
        mc.m_entityRenderer.get().orNpe().setupGuiState();
        GL11::glEnable(3042);
        let player: Rc<Entity> = self.player();
        if Minecraft::isFancyGraphicsEnabled()
        {
            self.renderVignette(player.getEntityBrightness(tickDelta), i, j);
        }
        let inventoryplayer: Rc<InventoryPlayer> = entity_player::inventory(&player);
        let itemstack: Option<Rc<ItemStack>> = inventoryplayer.armorItemInSlot(3);
        if !self.settings().m_thirdPersonView.get()
            && itemstack.is_some_and(|stack| stack.m_itemID.get() == Block::pumpkin().m_blockID)
        {
            self.renderPumpkinBlur(i, j);
        }
        let playerFields: &entity_player::PlayerFields = entity_player::fields(&player);
        let f: f32 = playerFields.m_prevTimeInPortal.get()
            + (playerFields.m_timeInPortal.get() - playerFields.m_prevTimeInPortal.get()) * tickDelta;
        if f > 0.0
        {
            self.renderPortalOverlay(f, i, j);
        }
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        let renderEngine: Rc<RenderEngine> = self.renderEngine();
        GL11::glBindTexture(3553, renderEngine.getTexture("/gui/gui.png"));
        self.m_gui.m_zLevel.set(-90.0);
        self.m_gui.drawTexturedModalRect(i / 2 - 91, j - 22, 0, 0, 182, 22);
        self.m_gui.drawTexturedModalRect(
            i / 2 - 91 - 1 + inventoryplayer.m_currentItem.get() * 20,
            j - 22 - 1,
            0,
            22,
            24,
            22,
        );
        GL11::glBindTexture(3553, renderEngine.getTexture("/gui/icons.png"));
        GL11::glEnable(3042);
        GL11::glBlendFunc(775, 769);
        self.m_gui.drawTexturedModalRect(i / 2 - 7, j / 2 - 7, 0, 0, 16, 16);
        GL11::glDisable(3042);
        self.renderStatusBars(&player, i, j);
        GL11::glDisable(3042);
        GL11::glEnable(32826);
        GL11::glPushMatrix();
        GL11::glRotatef(120.0, 1.0, 0.0, 0.0);
        RenderHelper::enableStandardItemLighting();
        GL11::glPopMatrix();
        for i3 in 0..9
        {
            let i4: i32 = i / 2 - 90 + i3 * 20 + 2;
            let l4: i32 = j - 16 - 3;
            self.renderInventorySlot(i3, i4, l4, tickDelta);
        }
        RenderHelper::disableStandardItemLighting();
        GL11::glDisable(32826);
        let sleepingTime: i32 = entity_player::getSleepingTime(&player);
        if sleepingTime > 0
        {
            GL11::glDisable(2929);
            GL11::glDisable(3008);
            let mut f2: f32 = sleepingTime as f32 / 100.0;
            if f2 > 1.0
            {
                f2 = 1.0 - (sleepingTime - 100) as f32 / 10.0;
            }
            let i5: i32 = ((220.0 * f2) as i32) << 24 | 1052704;
            self.m_gui.drawRect(0, 0, i, j, i5);
            GL11::glEnable(3008);
            GL11::glEnable(2929);
        }
        if self.settings().m_showDebugInfo.get()
        {
            self.renderDebugInfo(&fontrenderer, &player, i);
        }
        self.renderRecordMessage(&fontrenderer, tickDelta, i, j);
        self.renderChat(&fontrenderer, j);
    }

    fn renderStatusBars(&self, player: &Rc<Entity>, i: i32, j: i32)
    {
        let mut flag: bool = player.m_heartsLife.get() / 3 % 2 == 1;
        if player.m_heartsLife.get() < 10
        {
            flag = false;
        }
        let living: &entity_living::LivingFields = entity_living::fields(player);
        let k: i32 = living.m_health.get();
        let l: i32 = living.m_prevHealth.get();
        self.m_rand.setSeed((self.m_updateCounter.get().wrapping_mul(312871)) as i64);
        if !self.m_mc.m_playerController.get().orNpe().shouldDrawHUD()
        {
            return;
        }
        let i1: i32 = entity_player_sp::getPlayerArmorValue(player);
        let gui: &Gui = &self.m_gui;
        for j1 in 0..10
        {
            let mut k1: i32 = j - 32;
            if i1 > 0
            {
                let l1: i32 = i / 2 + 91 - j1 * 8 - 9;
                if j1 * 2 + 1 < i1
                {
                    gui.drawTexturedModalRect(l1, k1, 34, 9, 9, 9);
                }
                if j1 * 2 + 1 == i1
                {
                    gui.drawTexturedModalRect(l1, k1, 25, 9, 9, 9);
                }
                if j1 * 2 + 1 > i1
                {
                    gui.drawTexturedModalRect(l1, k1, 16, 9, 9, 9);
                }
            }
            let b1: i32 = if flag { 1 } else { 0 };
            let i2: i32 = i / 2 - 91 + j1 * 8;
            if k <= 4
            {
                k1 += self.m_rand.nextIntBounded(2);
            }
            gui.drawTexturedModalRect(i2, k1, 16 + b1 * 9, 0, 9, 9);
            if flag
            {
                if j1 * 2 + 1 < l
                {
                    gui.drawTexturedModalRect(i2, k1, 70, 0, 9, 9);
                }
                if j1 * 2 + 1 == l
                {
                    gui.drawTexturedModalRect(i2, k1, 79, 0, 9, 9);
                }
            }
            if j1 * 2 + 1 < k
            {
                gui.drawTexturedModalRect(i2, k1, 52, 0, 9, 9);
            }
            if j1 * 2 + 1 == k
            {
                gui.drawTexturedModalRect(i2, k1, 61, 0, 9, 9);
            }
        }
        if player.isInsideOfMaterial(&Material::water())
        {
            let air: i32 = player.m_air.get();
            let l3: i32 = ((air - 2) as f64 * 10.0 / 300.0).ceil() as i32;
            let k4: i32 = (air as f64 * 10.0 / 300.0).ceil() as i32 - l3;
            for i6 in 0..l3 + k4
            {
                let u: i32 = if i6 < l3 { 16 } else { 25 };
                gui.drawTexturedModalRect(i / 2 - 91 + i6 * 8, j - 32 - 9, u, 18, 9, 9);
            }
        }
    }

    fn renderDebugInfo(&self, fontrenderer: &FontRenderer, player: &Entity, i: i32)
    {
        let mc: &Rc<Minecraft> = &self.m_mc;
        GL11::glPushMatrix();
        if Minecraft::hasPaidCheckTime() > 0
        {
            GL11::glTranslatef(0.0, 32.0, 0.0);
        }
        fontrenderer.drawStringWithShadow(&format!("Minecraft Beta 1.7.3 ({})", mc.m_debug.get()), 2, 2, 16777215);
        fontrenderer.drawStringWithShadow(&mc.getRenderChunkDebugInfo(), 2, 12, 16777215);
        fontrenderer.drawStringWithShadow(&mc.getRenderEntityDebugInfo(), 2, 22, 16777215);
        fontrenderer.drawStringWithShadow(&mc.getWorldDebugInfo(), 2, 32, 16777215);
        fontrenderer.drawStringWithShadow(&mc.getChunkSourceDebugInfo(), 2, 42, 16777215);
        // port runtimeMemory stands in for java Runtime memory figures
        let (k3, j5, j6): (i64, i64, i64) = runtime::runtimeMemory();
        let k2: i64 = j5 - j6;
        let mut s: String =
            format!("Used memory: {}% ({}MB) of {}MB", k2 * 100 / k3, k2 / 1024 / 1024, k3 / 1024 / 1024);
        self.m_gui.drawString(fontrenderer, &s, i - fontrenderer.getStringWidth(&s) - 2, 2, 14737632);
        s = format!("Allocated memory: {}% ({}MB)", j5 * 100 / k3, j5 / 1024 / 1024);
        self.m_gui.drawString(fontrenderer, &s, i - fontrenderer.getStringWidth(&s) - 2, 12, 14737632);
        let double: fn(f64) -> String = jvm::floating_decimal::doubleToString;
        self.m_gui.drawString(fontrenderer, &format!("x: {}", double(player.posX())), 2, 64, 14737632);
        self.m_gui.drawString(fontrenderer, &format!("y: {}", double(player.posY())), 2, 72, 14737632);
        self.m_gui.drawString(fontrenderer, &format!("z: {}", double(player.posZ())), 2, 80, 14737632);
        let facing: i32 = floor_double((player.m_rotationYaw.get() * 4.0 / 360.0) as f64 + 0.5) & 3;
        self.m_gui.drawString(fontrenderer, &format!("f: {facing}"), 2, 88, 14737632);
        GL11::glPopMatrix();
    }

    fn renderRecordMessage(&self, fontrenderer: &FontRenderer, tickDelta: f32, i: i32, j: i32)
    {
        if self.m_recordPlayingUpFor.get() <= 0
        {
            return;
        }
        let f1: f32 = self.m_recordPlayingUpFor.get() as f32 - tickDelta;
        let j4: i32 = ((f1 * 256.0 / 20.0) as i32).min(255);
        if j4 <= 0
        {
            return;
        }
        GL11::glPushMatrix();
        GL11::glTranslatef((i / 2) as f32, (j - 48) as f32, 0.0);
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        let mut k5: i32 = 16777215;
        if self.m_overlayMessageTinted.get()
        {
            // port hsbToRgb is java Color.HSBtoRGB
            k5 = awt::hsbToRgb(f1 / 50.0, 0.7, 0.6) & 16777215;
        }
        let recordPlaying: String = self.m_recordPlaying.get();
        fontrenderer.drawString(&recordPlaying, -fontrenderer.getStringWidth(&recordPlaying) / 2, -4, k5 + (j4 << 24));
        GL11::glDisable(3042);
        GL11::glPopMatrix();
    }

    fn renderChat(&self, fontrenderer: &FontRenderer, j: i32)
    {
        let mut b0: usize = 10;
        let mut flag1: bool = false;
        if self.m_mc.m_currentScreen.get().is_some_and(|screen| screen.instanceOf("GuiChat"))
        {
            b0 = 20;
            flag1 = true;
        }
        GL11::glEnable(3042);
        GL11::glBlendFunc(770, 771);
        GL11::glDisable(3008);
        GL11::glPushMatrix();
        GL11::glTranslatef(0.0, (j - 48) as f32, 0.0);
        for (l5, line) in self.m_chatMessageList.borrow().iter().enumerate().take(b0)
        {
            if line.m_updateCounter.get() >= 200 && !flag1
            {
                continue;
            }
            let mut d0: f64 = line.m_updateCounter.get() as f64 / 200.0;
            d0 = ((1.0 - d0) * 10.0).clamp(0.0, 1.0);
            d0 *= d0;
            let mut j2: i32 = (255.0 * d0) as i32;
            if flag1
            {
                j2 = 255;
            }
            if j2 > 0
            {
                let b2: i32 = 2;
                let l2: i32 = -(l5 as i32) * 9;
                self.m_gui.drawRect(b2, l2 - 1, b2 + 320, l2 + 8, (j2 / 2) << 24);
                GL11::glEnable(3042);
                fontrenderer.drawStringWithShadow(&line.m_message, b2, l2, 16777215 + (j2 << 24));
            }
        }
        GL11::glPopMatrix();
        GL11::glEnable(3008);
        GL11::glDisable(3042);
    }

    fn drawOverlayQuad(width: i32, height: i32, u0: f64, v0: f64, u1: f64, v1: f64)
    {
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        tessellator.addVertexWithUV(0.0, height as f64, -90.0, u0, v1);
        tessellator.addVertexWithUV(width as f64, height as f64, -90.0, u1, v1);
        tessellator.addVertexWithUV(width as f64, 0.0, -90.0, u1, v0);
        tessellator.addVertexWithUV(0.0, 0.0, -90.0, u0, v0);
        tessellator.draw();
    }

    fn renderPumpkinBlur(&self, width: i32, height: i32)
    {
        GL11::glDisable(2929);
        GL11::glDepthMask(false);
        GL11::glBlendFunc(770, 771);
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glDisable(3008);
        GL11::glBindTexture(3553, self.renderEngine().getTexture("%blur%/misc/pumpkinblur.png"));
        GuiIngame::drawOverlayQuad(width, height, 0.0, 0.0, 1.0, 1.0);
        GL11::glDepthMask(true);
        GL11::glEnable(2929);
        GL11::glEnable(3008);
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    }

    fn renderVignette(&self, brightnessAtEyes: f32, width: i32, height: i32)
    {
        let brightnessAtEyes: f32 = (1.0 - brightnessAtEyes).clamp(0.0, 1.0);
        let prev: f32 = self.m_prevVignetteBrightness.get();
        self.m_prevVignetteBrightness.set((prev as f64 + (brightnessAtEyes - prev) as f64 * 0.01) as f32);
        let brightness: f32 = self.m_prevVignetteBrightness.get();
        GL11::glDisable(2929);
        GL11::glDepthMask(false);
        GL11::glBlendFunc(0, 769);
        GL11::glColor4f(brightness, brightness, brightness, 1.0);
        GL11::glBindTexture(3553, self.renderEngine().getTexture("%blur%/misc/vignette.png"));
        GuiIngame::drawOverlayQuad(width, height, 0.0, 0.0, 1.0, 1.0);
        GL11::glDepthMask(true);
        GL11::glEnable(2929);
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        GL11::glBlendFunc(770, 771);
    }

    fn renderPortalOverlay(&self, mut portalTime: f32, width: i32, height: i32)
    {
        if portalTime < 1.0
        {
            portalTime *= portalTime;
            portalTime *= portalTime;
            portalTime = portalTime * 0.8 + 0.2;
        }
        GL11::glDisable(3008);
        GL11::glDisable(2929);
        GL11::glDepthMask(false);
        GL11::glBlendFunc(770, 771);
        GL11::glColor4f(1.0, 1.0, 1.0, portalTime);
        GL11::glBindTexture(3553, self.renderEngine().getTexture("/terrain.png"));
        let sprite: i32 = Block::portal().m_blockIndexInTexture.get();
        let f: f32 = (sprite % 16) as f32 / 16.0;
        let f1: f32 = (sprite / 16) as f32 / 16.0;
        let f2: f32 = (sprite % 16 + 1) as f32 / 16.0;
        let f3: f32 = (sprite / 16 + 1) as f32 / 16.0;
        GuiIngame::drawOverlayQuad(width, height, f as f64, f1 as f64, f2 as f64, f3 as f64);
        GL11::glDepthMask(true);
        GL11::glEnable(2929);
        GL11::glEnable(3008);
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    }

    fn renderInventorySlot(&self, slot: i32, x: i32, z: i32, tickDelta: f32)
    {
        let inventory: Rc<InventoryPlayer> = entity_player::inventory(&self.player());
        let itemStack: Option<Rc<ItemStack>> = inventory.m_mainInventory.borrow()[slot as usize].clone();
        let Some(itemStack) = itemStack
        else
        {
            return;
        };
        let fontRenderer: Rc<FontRenderer> = self.m_mc.m_fontRenderer.get().orNpe();
        let renderEngine: Rc<RenderEngine> = self.renderEngine();
        let f: f32 = itemStack.m_animationsToGo.get() as f32 - tickDelta;
        if f > 0.0
        {
            GL11::glPushMatrix();
            let f2: f32 = 1.0 + f / 5.0;
            GL11::glTranslatef((x + 8) as f32, (z + 12) as f32, 0.0);
            GL11::glScalef(1.0 / f2, (f2 + 1.0) / 2.0, 1.0);
            GL11::glTranslatef(-(x + 8) as f32, -(z + 12) as f32, 0.0);
        }
        itemRenderer().renderItemIntoGUI(&fontRenderer, &renderEngine, Some(&itemStack), x, z);
        if f > 0.0
        {
            GL11::glPopMatrix();
        }
        itemRenderer().renderItemOverlayIntoGUI(&fontRenderer, &renderEngine, Some(&itemStack), x, z);
    }

    pub fn updateTick(&self)
    {
        if self.m_recordPlayingUpFor.get() > 0
        {
            self.m_recordPlayingUpFor.set(self.m_recordPlayingUpFor.get() - 1);
        }
        self.m_updateCounter.set(self.m_updateCounter.get() + 1);
        for line in self.m_chatMessageList.borrow().iter()
        {
            line.m_updateCounter.set(line.m_updateCounter.get() + 1);
        }
    }

    pub fn clearChatMessages(&self) { self.m_chatMessageList.borrow_mut().clear(); }

    pub fn addChatMessage(&self, text: &str)
    {
        let fontRenderer: Rc<FontRenderer> = self.m_mc.m_fontRenderer.get().orNpe();
        let mut text: Vec<u16> = text.encode_utf16().collect();
        let width = |units: &[u16]| fontRenderer.getStringWidth(&String::from_utf16_lossy(units));
        while width(&text) > 320
        {
            let mut i: usize = 1;
            while i < text.len() && width(&text[..i + 1]) <= 320
            {
                i += 1;
            }
            self.addChatMessage(&String::from_utf16_lossy(&text[..i]));
            text = text[i..].to_vec();
        }
        let mut list: std::cell::RefMut<Vec<ChatLine>> = self.m_chatMessageList.borrow_mut();
        list.insert(0, ChatLine::new(String::from_utf16_lossy(&text)));
        while list.len() > 50
        {
            list.pop();
        }
    }

    pub fn setRecordPlayingMessage(&self, record: &str)
    {
        self.m_recordPlaying.set(format!("Now playing: {record}"));
        self.m_recordPlayingUpFor.set(60);
        self.m_overlayMessageTinted.set(true);
    }

    pub fn addChatMessageTranslate(&self, s: &str)
    {
        let stringtranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        self.addChatMessage(&stringtranslate.translateKey(s));
    }
}
