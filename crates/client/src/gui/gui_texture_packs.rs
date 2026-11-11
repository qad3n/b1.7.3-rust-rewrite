use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::gui_slot::{GUI_SLOT, GuiSlotClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen, GuiSlot};
use crate::render::{Tessellator, TexturePack, TexturePackList};
use game::locale::StringTranslate;
use game::resources::Pack;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::Sys;
use lwjgl::opengl::GL11;
use std::cell::{Cell, RefCell, RefMut};
use std::rc::{Rc, Weak};

mod layout
{
    pub const ROW_HEIGHT: i32 = 36;
    pub const ICON: i32 = 32;
    pub const TITLE_Y: i32 = 16;
    pub const LIST_TOP: i32 = 32;
    pub const LIST_BOTTOM: i32 = 51;
    pub const BUTTON_ROW: i32 = 48;
    pub const BUTTON_GAP: i32 = 4;
    pub const OPEN_FOLDER_X: i32 = 154;
    pub const FOLDER_INFO_X: i32 = 77;
    pub const FOLDER_INFO_ROW: i32 = 26;
    pub const MODS_BUTTON_INSET: i32 = 64;
    pub const MODS_BUTTON_Y: i32 = 6;
    pub const NAME_Y: i32 = 1;
    pub const DESCRIPTION_Y: i32 = 12;
    pub const LINE_HEIGHT: i32 = 10;
    pub const TEXT_GAP: i32 = 2;
    pub const GLYPH_HALF_HEIGHT: i32 = 4;
}

mod buttons
{
    pub const OPEN_FOLDER: i32 = 5;
    pub const DONE: i32 = 6;
    pub const SCROLL_UP: i32 = 7;
    pub const SCROLL_DOWN: i32 = 8;
}

// the folder is rescanned about once a second
const RESCAN_TICKS: i32 = 20;

mod colors
{
    pub const WHITE: i32 = 0xFFFFFF;
    pub const GRAY: i32 = 0x808080;
    pub const SHADE: i32 = 0xA0000000u32 as i32;
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Move
{
    Up,
    Down,
}

struct Arrow
{
    m_move: Move,
    m_glyph: &'static str,
}

const ARROWS: &[Arrow] = &[Arrow { m_move: Move::Up, m_glyph: "^" }, Arrow { m_move: Move::Down, m_glyph: "v" }];

struct TexturePacksExt
{
    m_guiScreen: Rc<GuiScreen>,
    m_cooldown: Cell<i32>,
    m_fileLocation: Field<String>,
    m_guiTexturePackSlot: Field<Option<Rc<GuiSlot>>>,
    // the choice made on this screen lowest first and applied on done
    m_pending: RefCell<Vec<Rc<TexturePack>>>,
    m_mouse: Cell<(i32, i32)>,
    m_hover: Cell<Option<(i32, Move)>>,
    // slots handle clicks before drawing rows so a click sees the hover of the last frame
    m_lastHover: Cell<Option<(i32, Move)>>,
}

fn fields(this: &GuiScreen) -> &TexturePacksExt { this.ext::<TexturePacksExt>() }

fn texturePackList(mc: &Minecraft) -> Rc<TexturePackList> { mc.m_texturePackList.get().orNpe() }

struct TexturePackSlotExt
{
    m_screen: Weak<GuiScreen>,
}

fn screen(slot: &GuiSlot) -> Rc<GuiScreen> { slot.ext::<TexturePackSlotExt>().m_screen.upgrade().orNpe() }

// selected packs top first then the jar then the rest in folder order
fn rows(this: &GuiScreen) -> Vec<Rc<TexturePack>>
{
    let list: Rc<TexturePackList> = texturePackList(&this.mc());
    let pending: Vec<Rc<TexturePack>> = fields(this).m_pending.borrow().clone();
    let chosen = |pack: &Rc<TexturePack>| pending.iter().any(|picked| Rc::ptr_eq(picked, pack));
    let rest: Vec<Rc<TexturePack>> =
        list.availableTexturePacks().into_iter().skip(1).filter(|pack| !chosen(pack)).collect();

    pending.iter().rev().cloned().chain(std::iter::once(list.defaultPack())).chain(rest).collect()
}

// the move under the mouse on the icon of a selected row
fn iconMove(index: i32, selected: i32, local: (i32, i32)) -> Option<Move>
{
    let (x, y): (i32, i32) = local;
    let inside: bool = x >= 0 && x < layout::ICON && y >= 0 && y < layout::ICON;
    if !inside || index >= selected
    {
        return None;
    }

    match y < layout::ICON / 2
    {
        true if index > 0 => Some(Move::Up),
        false if index < selected - 1 => Some(Move::Down),
        _ => None,
    }
}

fn drawArrow(this: &GuiScreen, font: &FontRenderer, step: Move, x: i32, y: i32)
{
    let Some(arrow) = ARROWS.iter().find(|arrow| arrow.m_move == step)
    else
    {
        return;
    };

    this.m_gui.drawRect(x, y, x + layout::ICON, y + layout::ICON, colors::SHADE);
    this.m_gui.drawCenteredString(
        font,
        arrow.m_glyph,
        x + layout::ICON / 2,
        y + layout::ICON / 2 - layout::GLYPH_HALF_HEIGHT,
        colors::WHITE,
    );
}

fn drawText(this: &GuiScreen, pack: &Pack, x: i32, y: i32)
{
    let font: Rc<FontRenderer> = this.fontRenderer();
    let description: &[String] = pack.description();
    let lines: [(Option<&str>, i32, i32); 3] = [
        (Some(pack.name()), layout::NAME_Y, colors::WHITE),
        (description.first().map(String::as_str), layout::DESCRIPTION_Y, colors::GRAY),
        (description.get(1).map(String::as_str), layout::DESCRIPTION_Y + layout::LINE_HEIGHT, colors::GRAY),
    ];

    for (line, offset, color) in lines
    {
        if let Some(line) = line
        {
            this.m_gui.drawString(&font, line, x, y + offset, color);
        }
    }
}

fn drawSlot(slot: &Rc<GuiSlot>, index: i32, x: i32, y: i32, height: i32, tesselator: &Rc<Tessellator>)
{
    let this: Rc<GuiScreen> = screen(slot);
    let mc: Rc<Minecraft> = this.mc();
    let packs: Vec<Rc<TexturePack>> = rows(&this);
    let texturePack: &Rc<TexturePack> = &packs[index as usize];
    texturePack.bindThumbnail(&mc);
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    tesselator.startDrawingQuads();
    tesselator.setColorOpaque_I(colors::WHITE);
    tesselator.addVertexWithUV(x as f64, (y + height) as f64, 0.0, 0.0, 1.0);
    tesselator.addVertexWithUV((x + layout::ICON) as f64, (y + height) as f64, 0.0, 1.0, 1.0);
    tesselator.addVertexWithUV((x + layout::ICON) as f64, y as f64, 0.0, 1.0, 0.0);
    tesselator.addVertexWithUV(x as f64, y as f64, 0.0, 0.0, 0.0);
    tesselator.draw();

    let ext: &TexturePacksExt = fields(&this);
    let (mouseX, mouseY): (i32, i32) = ext.m_mouse.get();
    let inList: bool = mouseY >= slot.m_top && mouseY <= slot.m_bottom;
    let selected: i32 = ext.m_pending.borrow().len() as i32;
    if let Some(step) = iconMove(index, selected, (mouseX - x, mouseY - y)).filter(|_| inList)
    {
        ext.m_hover.set(Some((index, step)));
        drawArrow(&this, &this.fontRenderer(), step, x, y);
    }

    drawText(&this, texturePack.pack(), x + layout::ICON + layout::TEXT_GAP, y);
}

fn position(pending: &[Rc<TexturePack>], pack: &Rc<TexturePack>) -> Option<usize>
{
    pending.iter().position(|picked| Rc::ptr_eq(picked, pack))
}

fn elementClicked(slot: &Rc<GuiSlot>, index: i32)
{
    let this: Rc<GuiScreen> = screen(slot);
    let packs: Vec<Rc<TexturePack>> = rows(&this);
    let Some(pack) = packs.get(index as usize)
    else
    {
        return;
    };

    let ext: &TexturePacksExt = fields(&this);
    let step: Option<Move> = ext.m_lastHover.get().filter(|(row, _)| *row == index).map(|(_, step)| step);
    let mut pending: RefMut<Vec<Rc<TexturePack>>> = ext.m_pending.borrow_mut();

    // the list shows the top pack first so up means later in the stack
    match (position(&pending, pack), step)
    {
        (Some(at), Some(Move::Up)) if at + 1 < pending.len() => pending.swap(at, at + 1),
        (Some(at), Some(Move::Down)) if at > 0 => pending.swap(at, at - 1),
        (Some(at), None) =>
        {
            pending.remove(at);
        }

        // the jar always sits under the stack
        (None, None) if !Rc::ptr_eq(pack, &texturePackList(&this.mc()).defaultPack()) => pending.push(Rc::clone(pack)),
        _ =>
        {}
    }
}

fn isSelected(slot: &Rc<GuiSlot>, index: i32) -> bool
{
    let this: Rc<GuiScreen> = screen(slot);
    let selected: usize = fields(&this).m_pending.borrow().len();
    index as usize <= selected
}

const TEXTURE_PACK_SLOT: GuiSlotClass = GuiSlotClass
{
    m_name: "GuiTexturePackSlot",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    getSize: |slot| rows(&screen(slot)).len() as i32,
    elementClicked: |slot, index, _| elementClicked(slot, index),
    isSelected,
    getContentHeight: |slot| slot.getSize() * layout::ROW_HEIGHT,
    drawBackground: |slot| screen(slot).drawDefaultBackground(),
    drawSlot,
    ..GUI_SLOT
};

// keeps the pending choice on the packs a rescan just loaded
fn refreshPending(this: &GuiScreen)
{
    let list: Rc<TexturePackList> = texturePackList(&this.mc());
    list.updateAvaliableTexturePacks();

    let available: Vec<Rc<TexturePack>> = list.availableTexturePacks();
    let mut pending: RefMut<Vec<Rc<TexturePack>>> = fields(this).m_pending.borrow_mut();
    let names: Vec<String> = pending.iter().map(|pack| pack.pack().name().to_owned()).collect();
    *pending = names
        .iter()
        .filter_map(|name| available.iter().skip(1).find(|pack| pack.pack().name() == name.as_str()).cloned())
        .collect();
}

const CLASS: GuiScreenClass = GuiScreenClass
{
    m_name: "GuiTexturePacks",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this|
    {
        let ext: &TexturePacksExt = fields(this);
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let center: i32 = this.m_width.get() / 2;
        let height: i32 = this.m_height.get();
        this.m_controlList.add(GuiButton::small(
            buttons::OPEN_FOLDER,
            center - layout::OPEN_FOLDER_X,
            height - layout::BUTTON_ROW,
            &stringTranslate.translateKey("texturePack.openFolder"),
        ));
        this.m_controlList.add(GuiButton::small(
            buttons::DONE,
            center + layout::BUTTON_GAP,
            height - layout::BUTTON_ROW,
            &stringTranslate.translateKey("gui.done"),
        ));
        // the mod list opens from the top right corner since this screen is titled mods and texture packs
        crate::modded::addModsButton(this, this.m_width.get() - layout::MODS_BUTTON_INSET, layout::MODS_BUTTON_Y);

        let mc: Rc<Minecraft> = this.mc();
        let list: Rc<TexturePackList> = texturePackList(&mc);
        list.updateAvaliableTexturePacks();
        *ext.m_pending.borrow_mut() = list.selected();

        let folder: std::path::PathBuf = Minecraft::getMinecraftDir().join("texturepacks");
        let absolute: std::path::PathBuf = std::path::absolute(&folder).unwrap_or(folder);
        ext.m_fileLocation.set(absolute.display().to_string());

        let slot: Rc<GuiSlot> = GuiSlot::construct(
            &TEXTURE_PACK_SLOT,
            Box::new(TexturePackSlotExt { m_screen: Rc::downgrade(&this.rc()) }),
            mc,
            this.m_width.get(),
            height,
            layout::LIST_TOP,
            height - layout::LIST_BOTTOM,
            layout::ROW_HEIGHT,
        );
        slot.registerScrollButtons(&this.m_controlList, buttons::SCROLL_UP, buttons::SCROLL_DOWN);
        ext.m_guiTexturePackSlot.set(Some(slot));
    },

    actionPerformed: |this, button|
    {
        if !button.m_enabled.get()
        {
            return;
        }

        let ext: &TexturePacksExt = fields(this);

        match button.m_id.get()
        {
            buttons::OPEN_FOLDER =>
            {
                Sys::openURL(&format!("file://{}", ext.m_fileLocation.get()));
            }

            buttons::DONE =>
            {
                let mc: Rc<Minecraft> = this.mc();
                let pending: Vec<Rc<TexturePack>> = ext.m_pending.borrow().clone();
                if texturePackList(&mc).setSelection(pending)
                {
                    mc.reloadTexturePacks();
                }

                mc.displayGuiScreen(Some(Rc::clone(&ext.m_guiScreen)));
            }
            // the mod list returns here when closed
            crate::modded::MODS_BUTTON => this.mc().displayGuiScreen(Some(crate::modded::modsScreen(&this.rc()))),
            _ => ext.m_guiTexturePackSlot.get().orNpe().actionPerformed(button),
        }
    },
    mouseClicked: |this, mouseX, mouseY, button| (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button),
    mouseMovedOrUp: |this, mouseX, mouseY, button| (GUI_SCREEN.mouseMovedOrUp)(this, mouseX, mouseY, button),
    drawScreen: |this, mouseX, mouseY, tickDelta|
    {
        let ext: &TexturePacksExt = fields(this);
        ext.m_mouse.set((mouseX, mouseY));
        ext.m_lastHover.set(ext.m_hover.take());
        ext.m_guiTexturePackSlot.get().orNpe().drawScreen(mouseX, mouseY, tickDelta);

        if ext.m_cooldown.get() <= 0
        {
            refreshPending(this);
            ext.m_cooldown.set(ext.m_cooldown.get() + RESCAN_TICKS);
        }

        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        let font: Rc<FontRenderer> = this.fontRenderer();
        let center: i32 = this.m_width.get() / 2;
        this.m_gui.drawCenteredString(
            &font,
            &stringTranslate.translateKey("texturePack.title"),
            center,
            layout::TITLE_Y,
            colors::WHITE,
        );
        this.m_gui.drawCenteredString(
            &font,
            &stringTranslate.translateKey("texturePack.folderInfo"),
            center - layout::FOLDER_INFO_X,
            this.m_height.get() - layout::FOLDER_INFO_ROW,
            colors::GRAY,
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },

    updateScreen: |this|
    {
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
        let ext: TexturePacksExt = TexturePacksExt
        {
            m_guiScreen: Rc::clone(parent),
            m_cooldown: Cell::new(-1),
            m_fileLocation: Field::new(String::new()),
            m_guiTexturePackSlot: Field::new(None),
            m_pending: RefCell::new(Vec::new()),
            m_mouse: Cell::new((0, 0)),
            m_hover: Cell::new(None),
            m_lastHover: Cell::new(None),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
