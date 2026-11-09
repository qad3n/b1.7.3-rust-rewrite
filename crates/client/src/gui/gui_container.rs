use crate::client::Minecraft;
use crate::gui::FontRenderer;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreen, GuiScreenClass};
use crate::render::{RenderEngine, RenderHelper, RenderItem, RenderManager};
use game::container::{Container, Slot, container_chest, container_dispenser, container_furnace, container_workbench};
use game::entity::{Entity, entity_player};
use game::inventory::{IInventory, InventoryPlayer};
use game::item::ItemStack;
use game::locale::StringTranslate;
use game::port::text;
use game::stats::AchievementList;
use game::tileentity::{TileEntity, tile_entity_furnace};
use game::world::World;
use jvm::debug;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::input::{Keyboard, Mouse};
use lwjgl::opengl::{GL11, GL12};
use std::cell::{Cell, OnceCell};
use std::rc::Rc;

thread_local! {
    static ITEM_RENDERER: OnceCell<Rc<RenderItem>> = const { OnceCell::new() };
}

fn itemRenderer() -> Rc<RenderItem>
{
    ITEM_RENDERER.with(|renderer| Rc::clone(renderer.get_or_init(|| Rc::new(RenderItem::new()))))
}

pub enum ContainerScreen
{
    Chest
    {
        m_upperChestInventory: Rc<dyn IInventory>,
        m_lowerChestInventory: Rc<dyn IInventory>,
        m_inventoryRows: i32,
    },
    Crafting,
    Furnace
    {
        m_furnaceInventory: Rc<TileEntity>,
    },
    Dispenser,
    Inventory
    {
        m_xSize_lo: Cell<f32>,
        m_ySize_lo: Cell<f32>,
    },
    // a mod container screen, its state belongs to the mod loader glue
    Mod
    {
        m_state: Rc<dyn std::any::Any>,
    },
}

pub struct GuiContainer
{
    pub m_xSize: Cell<i32>,
    pub m_ySize: Cell<i32>,
    pub m_inventorySlots: Rc<Container>,
    pub m_screen: ContainerScreen,
}

pub fn fields(this: &GuiScreen) -> &GuiContainer { this.ext::<GuiContainer>() }

pub fn construct(class: &'static GuiScreenClass, menu: Rc<Container>, screen: ContainerScreen) -> Rc<GuiScreen>
{
    GuiScreen::construct(
        class,
        Box::new(GuiContainer {
            m_xSize: Cell::new(176),
            m_ySize: Cell::new(166),
            m_inventorySlots: menu,
            m_screen: screen,
        }),
    )
}

pub fn origin(this: &GuiScreen) -> (i32, i32)
{
    let container: &GuiContainer = fields(this);
    ((this.m_width.get() - container.m_xSize.get()) / 2, (this.m_height.get() - container.m_ySize.get()) / 2)
}

fn player(this: &GuiScreen) -> Rc<Entity> { this.mc().m_thePlayer.get().orNpe() }

fn renderEngine(this: &GuiScreen) -> Rc<RenderEngine> { this.mc().m_renderEngine.get().orNpe() }

fn getIsMouseOverSlot(this: &GuiScreen, slot: &Slot, mut mouseX: i32, mut mouseY: i32) -> bool
{
    let (i, j): (i32, i32) = origin(this);
    mouseX -= i;
    mouseY -= j;
    let (x, y): (i32, i32) = (slot.m_xDisplayPosition.get(), slot.m_yDisplayPosition.get());
    mouseX >= x - 1 && mouseX < x + 16 + 1 && mouseY >= y - 1 && mouseY < y + 16 + 1
}

fn getSlotAtPosition(this: &GuiScreen, x: i32, y: i32) -> Option<Rc<Slot>>
{
    let slots: &jvm::array_list::JArrayList<Rc<Slot>> = &fields(this).m_inventorySlots.m_slots;
    (0..slots.size()).map(|i| slots.get(i)).find(|slot| getIsMouseOverSlot(this, slot, x, y))
}

// rewrite: the stack under the mouse for mod overlay key input, keyTyped carries no mouse position
fn hoveredStack(this: &GuiScreen) -> Option<Rc<ItemStack>>
{
    let mc: Rc<Minecraft> = this.mc();
    let x: i32 = Mouse::getX() * this.m_width.get() / mc.m_displayWidth.get();
    let y: i32 = this.m_height.get() - Mouse::getY() * this.m_height.get() / mc.m_displayHeight.get() - 1;
    let (left, top): (i32, i32) = origin(this);
    if crate::modded::overlayCovered(this, x - left, y - top)
    {
        return None;
    }
    getSlotAtPosition(this, x, y).and_then(|slot| slot.getStack())
}

fn drawSlotInventory(this: &GuiScreen, slot: &Slot)
{
    let i: i32 = slot.m_xDisplayPosition.get();
    let j: i32 = slot.m_yDisplayPosition.get();
    let itemstack: Option<Rc<ItemStack>> = slot.getStack();
    let renderEngine: Rc<RenderEngine> = renderEngine(this);
    if itemstack.is_none()
    {
        let k: i32 = slot.getBackgroundIconIndex();
        if k >= 0
        {
            GL11::glDisable(2896);
            renderEngine.bindTexture(renderEngine.getTexture("/gui/items.png"));
            this.m_gui.drawTexturedModalRect(i, j, k % 16 * 16, k / 16 * 16, 16, 16);
            GL11::glEnable(2896);
            return;
        }
    }
    let fontRenderer: Rc<FontRenderer> = this.fontRenderer();
    itemRenderer().renderItemIntoGUI(&fontRenderer, &renderEngine, itemstack.as_ref(), i, j);
    itemRenderer().renderItemOverlayIntoGUI(&fontRenderer, &renderEngine, itemstack.as_ref(), i, j);
}

fn drawScreen(this: &Rc<GuiScreen>, mouseX: i32, mouseY: i32, tickDelta: f32)
{
    this.drawDefaultBackground();
    let (i, j): (i32, i32) = origin(this);
    this.drawGuiContainerBackgroundLayer(tickDelta);
    GL11::glPushMatrix();
    GL11::glRotatef(120.0, 1.0, 0.0, 0.0);
    RenderHelper::enableStandardItemLighting();
    GL11::glPopMatrix();
    GL11::glPushMatrix();
    GL11::glTranslatef(i as f32, j as f32, 0.0);
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    GL11::glEnable(32826);
    let mut slot: Option<Rc<Slot>> = None;
    // rewrite: a mod overlay over the container hides the slot under the mouse from hover and tooltip
    let covered: bool = crate::modded::overlayCovered(this, mouseX - i, mouseY - j);
    let slots: &jvm::array_list::JArrayList<Rc<Slot>> = &fields(this).m_inventorySlots.m_slots;
    let mut k: i32 = 0;
    while k < slots.size()
    {
        let slot1: Rc<Slot> = slots.get(k);
        drawSlotInventory(this, &slot1);
        if !covered && getIsMouseOverSlot(this, &slot1, mouseX, mouseY)
        {
            GL11::glDisable(2896);
            GL11::glDisable(2929);
            let l: i32 = slot1.m_xDisplayPosition.get();
            let i1: i32 = slot1.m_yDisplayPosition.get();
            this.m_gui.drawGradientRect(l, i1, l + 16, i1 + 16, -2130706433, -2130706433);
            GL11::glEnable(2896);
            GL11::glEnable(2929);
            slot = Some(slot1);
        }
        k += 1;
    }
    let inventoryplayer: Rc<InventoryPlayer> = entity_player::inventory(&player(this));
    GL11::glDisable(32826);
    RenderHelper::disableStandardItemLighting();
    GL11::glDisable(2896);
    GL11::glDisable(2929);
    this.drawGuiContainerForegroundLayer();
    let tooltip: Option<Rc<Slot>> = slot.filter(|slot| inventoryplayer.getItemStack().is_none() && slot.getHasStack());
    GL11::glPopMatrix();
    (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    // rewrite: mod overlays draw over the finished screen at the container's corner, under the slot tooltip
    crate::modded::drawOverlays(
        this,
        origin(this),
        [fields(this).m_xSize.get(), fields(this).m_ySize.get()],
        [mouseX, mouseY],
    );
    // rewrite: the held stack draws after mod overlays so overlay panels never cover it
    drawHeld(this, &inventoryplayer, mouseX, mouseY);
    if let Some(slot) = tooltip
    {
        GL11::glPushMatrix();
        GL11::glTranslatef(i as f32, j as f32, 0.0);
        drawTooltip(this, &slot, mouseX - i, mouseY - j);
        GL11::glPopMatrix();
    }
    GL11::glEnable(2896);
    GL11::glEnable(2929);
}

fn drawHeld(this: &Rc<GuiScreen>, inventory: &InventoryPlayer, mouseX: i32, mouseY: i32)
{
    let Some(held) = inventory.getItemStack()
    else
    {
        return;
    };

    // overlay items leave depth that would hide the held stack
    GL11::glClear(GL11::GL_DEPTH_BUFFER_BIT);
    GL11::glPushMatrix();
    GL11::glRotatef(120.0, 1.0, 0.0, 0.0);
    RenderHelper::enableStandardItemLighting();
    GL11::glPopMatrix();
    GL11::glEnable(GL12::GL_RESCALE_NORMAL);
    GL11::glEnable(GL11::GL_DEPTH_TEST);

    let fontRenderer: Rc<FontRenderer> = this.fontRenderer();
    let renderEngine: Rc<RenderEngine> = renderEngine(this);
    itemRenderer().renderItemIntoGUI(&fontRenderer, &renderEngine, Some(&held), mouseX - 8, mouseY - 8);
    itemRenderer().renderItemOverlayIntoGUI(&fontRenderer, &renderEngine, Some(&held), mouseX - 8, mouseY - 8);

    GL11::glDisable(GL12::GL_RESCALE_NORMAL);
    RenderHelper::disableStandardItemLighting();
    GL11::glDisable(GL11::GL_LIGHTING);
    GL11::glDisable(GL11::GL_DEPTH_TEST);
}

fn drawTooltip(this: &GuiScreen, slot: &Slot, x: i32, y: i32)
{
    let stack: Rc<ItemStack> = slot.getStack().orNpe();
    let name: Option<String> = stack.getItemName();
    let translated: String = StringTranslate::getInstance().translateNamedKey(name.as_deref().unwrap_or("null"));
    // port trim strips chars up to space like java String.trim
    let s: String = text::trim(&JString::from(translated.as_str())).to_string();
    if s.is_empty()
    {
        return;
    }
    // rewrite: mod lines sit under the name spaced as later versions space tooltip lines
    const LINE_SPACING: i32 = 10;
    const NAME_GAP: i32 = 2;
    let lines: Vec<String> = crate::modded::tooltipLines(&stack);
    let fontRenderer: Rc<FontRenderer> = this.fontRenderer();
    let k1: i32 = x + 12;
    let l1: i32 = y - 12;
    let j1: i32 =
        lines.iter().map(|line| fontRenderer.getStringWidth(line)).fold(fontRenderer.getStringWidth(&s), i32::max);
    let extra: i32 = if lines.is_empty()
    {
        0
    }
    else
    {
        NAME_GAP + lines.len() as i32 * LINE_SPACING
    };
    this.m_gui.drawGradientRect(k1 - 3, l1 - 3, k1 + j1 + 3, l1 + 8 + extra + 3, -1073741824, -1073741824);
    fontRenderer.drawStringWithShadow(&s, k1, l1, -1);
    for (index, line) in lines.iter().enumerate()
    {
        fontRenderer.drawStringWithShadow(line, k1, l1 + NAME_GAP + (index as i32 + 1) * LINE_SPACING, -1);
    }
}

fn mouseClicked(this: &Rc<GuiScreen>, mouseX: i32, mouseY: i32, button: i32)
{
    // rewrite: a mod overlay may take the click before the container sees it
    let (left, top): (i32, i32) = origin(this);
    if modloader::screens::clickOverlays(this.className(), mouseX - left, mouseY - top, button)
    {
        debug!("A mod overlay took mouse button {button} on {}", this.className());
        return;
    }
    (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button);
    if button != 0 && button != 1
    {
        return;
    }
    let slot: Option<Rc<Slot>> = getSlotAtPosition(this, mouseX, mouseY);
    let (i, j): (i32, i32) = origin(this);
    let container: &GuiContainer = fields(this);
    let flag: bool =
        mouseX < i || mouseY < j || mouseX >= i + container.m_xSize.get() || mouseY >= j + container.m_ySize.get();
    let mut k: i32 = -1;
    if let Some(slot) = &slot
    {
        k = slot.m_slotNumber.get();
    }
    if flag
    {
        k = -999;
    }
    if k != -1
    {
        let flag1: bool = k != -999 && (Keyboard::isKeyDown(42) || Keyboard::isKeyDown(54));
        debug!(
            "Clicked slot {k} with mouse button {button}{} on {}",
            if flag1 { " and shift" } else { "" },
            this.className()
        );
        let mc: Rc<Minecraft> = this.mc();
        mc.m_playerController.get().orNpe().clickSlot(
            container.m_inventorySlots.m_windowId.get(),
            k,
            button,
            flag1,
            &player(this),
        );
    }
}

pub const GUI_CONTAINER: GuiScreenClass = GuiScreenClass {
    m_name: "GuiContainer",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        (GUI_SCREEN.initGui)(this);
        entity_player::fields(&player(this)).m_craftingInventory.set(Some(Rc::clone(&fields(this).m_inventorySlots)));
    },
    drawScreen,
    drawGuiContainerForegroundLayer: |_| {},
    mouseClicked,
    // rewrite: the mouse wheel reaches mod overlays, vanilla containers ignore it
    mouseMovedOrUp: |this, mouseX, mouseY, _| {
        let delta: i32 = Mouse::getEventDWheel();
        let (left, top): (i32, i32) = origin(this);
        if delta != 0
        {
            crate::modded::overlayScroll(this, mouseX - left, mouseY - top, delta.signum());
        }
    },
    keyTyped: |this, character, key| {
        // rewrite: a mod overlay may take the key before the container sees it
        if crate::modded::overlayKey(this, character, key, hoveredStack(this))
        {
            return;
        }
        let mc: Rc<Minecraft> = this.mc();
        if key == 1 || key == mc.m_gameSettings.get().orNpe().m_keyBindInventory.m_keyCode.get()
        {
            player(this).closeScreen();
        }
    },
    onGuiClosed: |this| {
        let mc: Rc<Minecraft> = this.mc();
        let Some(player) = mc.m_thePlayer.get()
        else
        {
            return;
        };
        mc.m_playerController.get().orNpe().closeMenu(fields(this).m_inventorySlots.m_windowId.get(), &player);
    },
    doesGuiPauseGame: |_| false,
    updateScreen: |this| {
        (GUI_SCREEN.updateScreen)(this);
        let player: Rc<Entity> = player(this);
        if !player.isEntityAlive() || player.m_isDead.get()
        {
            player.closeScreen();
        }
    },
    ..GUI_SCREEN
};

fn drawBackgroundTexture(this: &GuiScreen, texture: &str)
{
    let renderEngine: Rc<RenderEngine> = renderEngine(this);
    let n: i32 = renderEngine.getTexture(texture);
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    renderEngine.bindTexture(n);
    let (n2, n3): (i32, i32) = origin(this);
    let container: &GuiContainer = fields(this);
    this.m_gui.drawTexturedModalRect(n2, n3, 0, 0, container.m_xSize.get(), container.m_ySize.get());
}

fn drawTitles(this: &GuiScreen, title: &str, titleX: i32)
{
    let fontRenderer: Rc<FontRenderer> = this.fontRenderer();
    fontRenderer.drawString(title, titleX, 6, 0x404040);
    fontRenderer.drawString("Inventory", 8, fields(this).m_ySize.get() - 96 + 2, 0x404040);
}

pub struct GuiChest;

pub const GUI_CHEST: GuiScreenClass = GuiScreenClass {
    m_name: "GuiChest",
    m_parent: Some(&GUI_CONTAINER),
    m_interfaces: &[],
    drawGuiContainerForegroundLayer: |this| {
        let ContainerScreen::Chest { m_upperChestInventory: upper, m_lowerChestInventory: lower, .. } =
            &fields(this).m_screen
        else
        {
            return;
        };
        let fontRenderer: Rc<FontRenderer> = this.fontRenderer();
        fontRenderer.drawString(&lower.getInvName(), 8, 6, 0x404040);
        fontRenderer.drawString(&upper.getInvName(), 8, fields(this).m_ySize.get() - 96 + 2, 0x404040);
    },
    drawGuiContainerBackgroundLayer: |this, _| {
        let ContainerScreen::Chest { m_inventoryRows: rows, .. } = &fields(this).m_screen
        else
        {
            return;
        };
        let renderEngine: Rc<RenderEngine> = renderEngine(this);
        let n: i32 = renderEngine.getTexture("/gui/container.png");
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        renderEngine.bindTexture(n);
        let (n2, n3): (i32, i32) = origin(this);
        let xSize: i32 = fields(this).m_xSize.get();
        this.m_gui.drawTexturedModalRect(n2, n3, 0, 0, xSize, rows * 18 + 17);
        this.m_gui.drawTexturedModalRect(n2, n3 + rows * 18 + 17, 0, 126, xSize, 96);
    },
    ..GUI_CONTAINER
};

impl GuiChest
{
    pub fn new(playerInventory: Rc<dyn IInventory>, inventory: Option<Rc<dyn IInventory>>) -> Rc<GuiScreen>
    {
        let inventory: Rc<dyn IInventory> = inventory.orNpe();
        let menu: Rc<Container> = container_chest::create(Rc::clone(&playerInventory), Rc::clone(&inventory));
        let rows: i32 = inventory.getSizeInventory() / 9;
        let screen: ContainerScreen = ContainerScreen::Chest {
            m_upperChestInventory: playerInventory,
            m_lowerChestInventory: inventory,
            m_inventoryRows: rows,
        };
        let gui: Rc<GuiScreen> = construct(&GUI_CHEST, menu, screen);
        gui.m_passEvents.set(false);
        let n: i32 = 222;
        let n2: i32 = n - 108;
        fields(&gui).m_ySize.set(n2 + rows * 18);
        gui
    }
}

pub struct GuiCrafting;

pub const GUI_CRAFTING: GuiScreenClass = GuiScreenClass {
    m_name: "GuiCrafting",
    m_parent: Some(&GUI_CONTAINER),
    m_interfaces: &[],
    onGuiClosed: |this| {
        (GUI_CONTAINER.onGuiClosed)(this);
        fields(this).m_inventorySlots.onCraftGuiClosed(&this.mc().m_thePlayer.get().orNpe());
    },
    drawGuiContainerForegroundLayer: |this| {
        let fontRenderer: Rc<FontRenderer> = this.fontRenderer();
        fontRenderer.drawString("Crafting", 28, 6, 0x404040);
        fontRenderer.drawString("Inventory", 8, fields(this).m_ySize.get() - 96 + 2, 0x404040);
    },
    drawGuiContainerBackgroundLayer: |this, _| drawBackgroundTexture(this, "/gui/crafting.png"),
    ..GUI_CONTAINER
};

impl GuiCrafting
{
    pub fn new(playerInventory: &Rc<InventoryPlayer>, world: &Rc<World>, x: i32, y: i32, z: i32) -> Rc<GuiScreen>
    {
        construct(
            &GUI_CRAFTING,
            container_workbench::create(playerInventory, world, x, y, z),
            ContainerScreen::Crafting,
        )
    }
}

pub struct GuiFurnace;

pub const GUI_FURNACE: GuiScreenClass = GuiScreenClass {
    m_name: "GuiFurnace",
    m_parent: Some(&GUI_CONTAINER),
    m_interfaces: &[],
    drawGuiContainerForegroundLayer: |this| drawTitles(this, "Furnace", 60),
    drawGuiContainerBackgroundLayer: |this, _| {
        drawBackgroundTexture(this, "/gui/furnace.png");
        let ContainerScreen::Furnace { m_furnaceInventory: furnace } = &fields(this).m_screen
        else
        {
            return;
        };
        let (n3, n4): (i32, i32) = origin(this);
        if tile_entity_furnace::isBurning(furnace)
        {
            let n: i32 = tile_entity_furnace::getBurnTimeRemainingScaled(furnace, 12);
            this.m_gui.drawTexturedModalRect(n3 + 56, n4 + 36 + 12 - n, 176, 12 - n, 14, n + 2);
        }
        let n: i32 = tile_entity_furnace::getCookProgressScaled(furnace, 24);
        this.m_gui.drawTexturedModalRect(n3 + 79, n4 + 34, 176, 14, n + 1, 16);
    },
    ..GUI_CONTAINER
};

impl GuiFurnace
{
    pub fn new(playerInventory: &Rc<InventoryPlayer>, furnace: Option<Rc<TileEntity>>) -> Rc<GuiScreen>
    {
        let furnace: Rc<TileEntity> = furnace.orNpe();
        let menu: Rc<Container> = container_furnace::create(playerInventory, &furnace);
        construct(&GUI_FURNACE, menu, ContainerScreen::Furnace { m_furnaceInventory: furnace })
    }
}

pub struct GuiDispenser;

pub const GUI_DISPENSER: GuiScreenClass = GuiScreenClass {
    m_name: "GuiDispenser",
    m_parent: Some(&GUI_CONTAINER),
    m_interfaces: &[],
    drawGuiContainerForegroundLayer: |this| drawTitles(this, "Dispenser", 60),
    drawGuiContainerBackgroundLayer: |this, _| drawBackgroundTexture(this, "/gui/trap.png"),
    ..GUI_CONTAINER
};

impl GuiDispenser
{
    pub fn new(playerInventory: &Rc<InventoryPlayer>, dispenser: Option<Rc<TileEntity>>) -> Rc<GuiScreen>
    {
        let inventory: Rc<dyn IInventory> = Rc::clone(playerInventory) as Rc<dyn IInventory>;
        let menu: Rc<Container> = container_dispenser::create(inventory, &dispenser.orNpe());
        construct(&GUI_DISPENSER, menu, ContainerScreen::Dispenser)
    }
}

pub struct GuiInventory;

fn drawInventoryBackground(this: &Rc<GuiScreen>, _tickDelta: f32)
{
    drawBackgroundTexture(this, "/gui/inventory.png");
    let ContainerScreen::Inventory { m_xSize_lo: xSizeLo, m_ySize_lo: ySizeLo } = &fields(this).m_screen
    else
    {
        return;
    };
    let (n2, n3): (i32, i32) = origin(this);
    GL11::glEnable(32826);
    GL11::glEnable(2903);
    GL11::glPushMatrix();
    GL11::glTranslatef((n2 + 51) as f32, (n3 + 75) as f32, 50.0);
    let f: f32 = 30.0;
    GL11::glScalef(-f, f, f);
    GL11::glRotatef(180.0, 0.0, 0.0, 1.0);
    let player: Rc<Entity> = player(this);
    let living: &game::entity::entity_living::LivingFields = game::entity::entity_living::fields(&player);
    let f2: f32 = living.m_renderYawOffset.get();
    let f3: f32 = player.m_rotationYaw.get();
    let f4: f32 = player.m_rotationPitch.get();
    let f5: f32 = (n2 + 51) as f32 - xSizeLo.get();
    let f6: f32 = (n3 + 75 - 50) as f32 - ySizeLo.get();
    let atan = |value: f32| jvm::math::atan(value as f64) as f32;
    GL11::glRotatef(135.0, 0.0, 1.0, 0.0);
    RenderHelper::enableStandardItemLighting();
    GL11::glRotatef(-135.0, 0.0, 1.0, 0.0);
    GL11::glRotatef(-atan(f6 / 40.0) * 20.0, 1.0, 0.0, 0.0);
    living.m_renderYawOffset.set(atan(f5 / 40.0) * 20.0);
    player.m_rotationYaw.set(atan(f5 / 40.0) * 40.0);
    player.m_rotationPitch.set(-atan(f6 / 40.0) * 20.0);
    player.m_entityBrightness.set(1.0);
    GL11::glTranslatef(0.0, player.m_yOffset.get(), 0.0);
    RenderManager::instance().m_playerViewY.set(180.0);
    RenderManager::instance().renderEntityWithPosYaw(&player, 0.0, 0.0, 0.0, 0.0, 1.0);
    player.m_entityBrightness.set(0.0);
    living.m_renderYawOffset.set(f2);
    player.m_rotationYaw.set(f3);
    player.m_rotationPitch.set(f4);
    GL11::glPopMatrix();
    RenderHelper::disableStandardItemLighting();
    GL11::glDisable(32826);
}

pub const GUI_INVENTORY: GuiScreenClass = GuiScreenClass {
    m_name: "GuiInventory",
    m_parent: Some(&GUI_CONTAINER),
    m_interfaces: &[],
    initGui: |this| this.m_controlList.clear(),
    drawGuiContainerForegroundLayer: |this| this.fontRenderer().drawString("Crafting", 86, 16, 0x404040),
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        (GUI_CONTAINER.drawScreen)(this, mouseX, mouseY, tickDelta);
        if let ContainerScreen::Inventory { m_xSize_lo: xSizeLo, m_ySize_lo: ySizeLo } = &fields(this).m_screen
        {
            xSizeLo.set(mouseX as f32);
            ySizeLo.set(mouseY as f32);
        }
    },
    drawGuiContainerBackgroundLayer: drawInventoryBackground,
    actionPerformed: |this, button| {
        let mc: Rc<Minecraft> = this.mc();
        if button.m_id.get() == 0
        {
            mc.displayGuiScreen(Some(crate::gui::GuiAchievements::new(mc.m_statFileWriter.get().orNpe())));
        }
        if button.m_id.get() == 1
        {
            mc.displayGuiScreen(Some(crate::gui::GuiStats::new(this.rc(), mc.m_statFileWriter.get().orNpe())));
        }
    },
    ..GUI_CONTAINER
};

impl GuiInventory
{
    pub fn new(player: &Rc<Entity>) -> Rc<GuiScreen>
    {
        let menu: Rc<Container> = entity_player::fields(player).m_inventorySlots.get().orNpe();
        let screen: ContainerScreen =
            ContainerScreen::Inventory { m_xSize_lo: Cell::new(0.0), m_ySize_lo: Cell::new(0.0) };
        let gui: Rc<GuiScreen> = construct(&GUI_INVENTORY, menu, screen);
        gui.m_passEvents.set(true);
        player.addStat(AchievementList::openInventory().as_ref(), 1);
        gui
    }
}
