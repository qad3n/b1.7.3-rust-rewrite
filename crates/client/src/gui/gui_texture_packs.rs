use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::gui_slot::{GUI_SLOT, GuiSlotClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen, GuiSlot};
use crate::render::{Tessellator, TexturePackBase, TexturePackFields, TexturePackList};
use game::locale::StringTranslate;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::Sys;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::{Rc, Weak};

struct TexturePacksExt
{
    m_guiScreen: Rc<GuiScreen>,
    m_cooldown: Cell<i32>,
    m_fileLocation: Field<String>,
    m_guiTexturePackSlot: Field<Option<Rc<GuiSlot>>>,
}

fn fields(this: &GuiScreen) -> &TexturePacksExt { this.ext::<TexturePacksExt>() }

fn texturePackList(mc: &Minecraft) -> Rc<TexturePackList> { mc.m_texturePackList.get().orNpe() }

struct TexturePackSlotExt
{
    m_screen: Weak<GuiScreen>,
}

fn screen(slot: &GuiSlot) -> Rc<GuiScreen> { slot.ext::<TexturePackSlotExt>().m_screen.upgrade().orNpe() }

fn packs(slot: &GuiSlot) -> Vec<Rc<dyn TexturePackBase>> { texturePackList(&screen(slot).mc()).availableTexturePacks() }

fn drawSlot(slot: &Rc<GuiSlot>, index: i32, x: i32, y: i32, height: i32, tesselator: &Rc<Tessellator>)
{
    let this: Rc<GuiScreen> = screen(slot);
    let mc: Rc<Minecraft> = this.mc();
    let texturePackBase: Rc<dyn TexturePackBase> = packs(slot)[index as usize].clone();
    texturePackBase.bindThumbnailTexture(&mc);
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    tesselator.startDrawingQuads();
    tesselator.setColorOpaque_I(0xFFFFFF);
    tesselator.addVertexWithUV(x as f64, (y + height) as f64, 0.0, 0.0, 1.0);
    tesselator.addVertexWithUV((x + 32) as f64, (y + height) as f64, 0.0, 1.0, 1.0);
    tesselator.addVertexWithUV((x + 32) as f64, y as f64, 0.0, 1.0, 0.0);
    tesselator.addVertexWithUV(x as f64, y as f64, 0.0, 0.0, 0.0);
    tesselator.draw();
    let font: Rc<FontRenderer> = this.fontRenderer();
    let pack: &TexturePackFields = texturePackBase.pack();
    let lines: [(Option<String>, i32, i32); 3] = [
        (pack.m_texturePackFileName.get(), 1, 0xFFFFFF),
        (pack.m_firstDescriptionLine.get(), 12, 0x808080),
        (pack.m_secondDescriptionLine.get(), 12 + 10, 0x808080),
    ];
    for (line, offset, color) in lines
    {
        if let Some(line) = line
        {
            this.m_gui.drawString(&font, &line, x + 32 + 2, y + offset, color);
        }
    }
}

const TEXTURE_PACK_SLOT: GuiSlotClass = GuiSlotClass {
    m_name: "GuiTexturePackSlot",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    getSize: |slot| packs(slot).len() as i32,
    elementClicked: |slot, index, _| {
        let mc: Rc<Minecraft> = screen(slot).mc();
        let list: Vec<Rc<dyn TexturePackBase>> = packs(slot);
        texturePackList(&mc).setTexturePack(&list[index as usize]);
        mc.m_renderEngine.get().orNpe().refreshTextures();
    },
    isSelected: |slot, index| {
        let list: Vec<Rc<dyn TexturePackBase>> = packs(slot);
        texturePackList(&screen(slot).mc())
            .m_selectedTexturePack
            .get()
            .is_some_and(|selected| Rc::ptr_eq(&selected, &list[index as usize]))
    },
    getContentHeight: |slot| slot.getSize() * 36,
    drawBackground: |slot| screen(slot).drawDefaultBackground(),
    drawSlot,
    ..GUI_SLOT
};

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiTexturePacks",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        let ext: &TexturePacksExt = fields(this);
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let center: i32 = this.m_width.get() / 2;
        let height: i32 = this.m_height.get();
        this.m_controlList.add(GuiButton::small(
            5,
            center - 154,
            height - 48,
            &stringTranslate.translateKey("texturePack.openFolder"),
        ));
        this.m_controlList.add(GuiButton::small(6, center + 4, height - 48, &stringTranslate.translateKey("gui.done")));
        let mc: Rc<Minecraft> = this.mc();
        texturePackList(&mc).updateAvaliableTexturePacks();
        let folder: std::path::PathBuf = Minecraft::getMinecraftDir().join("texturepacks");
        let absolute: std::path::PathBuf = std::path::absolute(&folder).unwrap_or(folder);
        ext.m_fileLocation.set(absolute.display().to_string());
        let slot: Rc<GuiSlot> = GuiSlot::construct(
            &TEXTURE_PACK_SLOT,
            Box::new(TexturePackSlotExt { m_screen: Rc::downgrade(&this.rc()) }),
            mc,
            this.m_width.get(),
            height,
            32,
            height - 55 + 4,
            36,
        );
        slot.registerScrollButtons(&this.m_controlList, 7, 8);
        ext.m_guiTexturePackSlot.set(Some(slot));
    },
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        let ext: &TexturePacksExt = fields(this);
        match button.m_id.get()
        {
            5 =>
            {
                Sys::openURL(&format!("file://{}", ext.m_fileLocation.get()));
            }
            6 =>
            {
                let mc: Rc<Minecraft> = this.mc();
                mc.m_renderEngine.get().orNpe().refreshTextures();
                mc.displayGuiScreen(Some(Rc::clone(&ext.m_guiScreen)));
            }
            _ => ext.m_guiTexturePackSlot.get().orNpe().actionPerformed(button),
        }
    },
    mouseClicked: |this, mouseX, mouseY, button| (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button),
    mouseMovedOrUp: |this, mouseX, mouseY, button| (GUI_SCREEN.mouseMovedOrUp)(this, mouseX, mouseY, button),
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let ext: &TexturePacksExt = fields(this);
        ext.m_guiTexturePackSlot.get().orNpe().drawScreen(mouseX, mouseY, tickDelta);
        if ext.m_cooldown.get() <= 0
        {
            texturePackList(&this.mc()).updateAvaliableTexturePacks();
            ext.m_cooldown.set(ext.m_cooldown.get() + 20);
        }
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        this.m_gui.drawCenteredString(&font, &stringTranslate.translateKey("texturePack.title"), center, 16, 0xFFFFFF);
        this.m_gui.drawCenteredString(
            &font,
            &stringTranslate.translateKey("texturePack.folderInfo"),
            center - 77,
            this.m_height.get() - 26,
            0x808080,
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    updateScreen: |this| {
        (GUI_SCREEN.updateScreen)(this);
        let ext: &TexturePacksExt = fields(this);
        ext.m_cooldown.set(ext.m_cooldown.get() - 1);
    },
    ..GUI_SCREEN
};

pub struct GuiTexturePacks;

impl GuiTexturePacks
{
    pub fn new(parent: &Rc<GuiScreen>) -> Rc<GuiScreen>
    {
        let ext: TexturePacksExt = TexturePacksExt {
            m_guiScreen: Rc::clone(parent),
            m_cooldown: Cell::new(-1),
            m_fileLocation: Field::new(String::new()),
            m_guiTexturePackSlot: Field::new(None),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
