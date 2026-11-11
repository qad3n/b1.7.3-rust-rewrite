use crate::client::Minecraft;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::gui::gui_slot::{GUI_SLOT, GuiSlotClass};
use crate::gui::{FontRenderer, GuiButton, GuiScreen, GuiSlot};
use crate::render::{Tessellator, TexturePack, TexturePackList};
use game::locale::StringTranslate;
use game::resources::{Layout, Pack};
use jvm::field::Field;
use jvm::throwable::OrNpe;
use lwjgl::Sys;
use lwjgl::opengl::GL11;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

mod layout
{
    pub const COLUMN: i32 = 200;
    pub const GAP: i32 = 8;
    // room for the scrollbar inside the column
    pub const ROW_MARGIN: i32 = 40;
    pub const ROW_HEIGHT: i32 = 36;
    pub const ICON: i32 = 32;
    pub const TITLE_Y: i32 = 16;
    pub const HEADER_Y: i32 = 32;
    pub const LIST_TOP: i32 = 44;
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
    pub const LABEL_GAP: i32 = 4;
    pub const GLYPH_HALF_HEIGHT: i32 = 4;
}

mod buttons
{
    pub const OPEN_FOLDER: i32 = 5;
    pub const DONE: i32 = 6;
    pub const AVAILABLE_UP: i32 = 7;
    pub const AVAILABLE_DOWN: i32 = 8;
    pub const SELECTED_UP: i32 = 9;
    pub const SELECTED_DOWN: i32 = 10;
}

// the folder is rescanned about once a second
const RESCAN_TICKS: i32 = 20;

mod colors
{
    pub const WHITE: i32 = 0xFFFFFF;
    pub const GRAY: i32 = 0x808080;
    pub const DARK_GRAY: i32 = 0x505050;
    pub const SHADE: i32 = 0xA0000000u32 as i32;
}

mod text
{
    pub const AVAILABLE: &str = "Available";
    pub const SELECTED: &str = "Selected";
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Column
{
    Available,
    Selected,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action
{
    Add,
    Remove,
    Up,
    Down,
}

struct Arrow
{
    m_action: Action,
    m_glyph: &'static str,
}

const ARROWS: &[Arrow] = &[
    Arrow { m_action: Action::Add, m_glyph: ">" },
    Arrow { m_action: Action::Remove, m_glyph: "<" },
    Arrow { m_action: Action::Up, m_glyph: "^" },
    Arrow { m_action: Action::Down, m_glyph: "v" },
];

struct Format
{
    m_layout: Layout,
    m_label: &'static str,
}

// the jar has no label since it is always at the bottom
const FORMATS: &[Format] =
    &[Format { m_layout: Layout::Legacy, m_label: "Beta" }, Format { m_layout: Layout::Modern, m_label: "Modern" }];

struct TexturePacksExt
{
    m_guiScreen: Rc<GuiScreen>,
    m_cooldown: Cell<i32>,
    m_fileLocation: Field<String>,
    m_available: Field<Option<Rc<GuiSlot>>>,
    m_chosen: Field<Option<Rc<GuiSlot>>>,
    // the choice made on this screen lowest first and applied on done
    m_pending: RefCell<Vec<Rc<TexturePack>>>,
    m_mouse: Cell<(i32, i32)>,
    m_hover: Cell<Option<(Column, i32, Action)>>,
    // slots handle clicks before drawing rows so a click sees the hover of the last frame
    m_lastHover: Cell<Option<(Column, i32, Action)>>,
}

fn fields(this: &GuiScreen) -> &TexturePacksExt { this.ext::<TexturePacksExt>() }

fn texturePackList(mc: &Minecraft) -> Rc<TexturePackList> { mc.m_texturePackList.get().orNpe() }

struct TexturePackSlotExt
{
    m_screen: Weak<GuiScreen>,
    m_column: Column,
}

fn screen(slot: &GuiSlot) -> Rc<GuiScreen> { slot.ext::<TexturePackSlotExt>().m_screen.upgrade().orNpe() }

fn column(slot: &GuiSlot) -> Column { slot.ext::<TexturePackSlotExt>().m_column }

// available rows in folder order and selected rows top first with the jar last
fn rows(this: &GuiScreen, column: Column) -> Vec<Rc<TexturePack>>
{
    let list: Rc<TexturePackList> = texturePackList(&this.mc());
    let pending: Vec<Rc<TexturePack>> = fields(this).m_pending.borrow().clone();
    let chosen = |pack: &Rc<TexturePack>| pending.iter().any(|picked| Rc::ptr_eq(picked, pack));

    match column
    {
        Column::Available => list.availableTexturePacks().into_iter().skip(1).filter(|pack| !chosen(pack)).collect(),
        Column::Selected => pending.iter().rev().cloned().chain(std::iter::once(list.defaultPack())).collect(),
    }
}

fn rowWidth(slotWidth: i32) -> i32 { slotWidth - layout::ROW_MARGIN }

// cuts text to fit a width in pixels
fn fit(font: &FontRenderer, line: &str, width: i32) -> String
{
    let mut text: String = line.to_owned();
    while !text.is_empty() && font.getStringWidth(&text) > width
    {
        text.pop();
    }

    text
}

// the action under the mouse on a row icon
fn iconAction(column: Column, index: i32, size: i32, local: (i32, i32)) -> Option<Action>
{
    let (x, y): (i32, i32) = local;
    let inside: bool = x >= 0 && x < layout::ICON && y >= 0 && y < layout::ICON;
    if !inside
    {
        return None;
    }

    // the jar is the last selected row and cannot move or leave
    let pending: i32 = size - 1;
    match column
    {
        Column::Available => Some(Action::Add),
        Column::Selected if index >= pending => None,
        Column::Selected if x < layout::ICON / 2 => Some(Action::Remove),
        Column::Selected if y < layout::ICON / 2 && index > 0 => Some(Action::Up),
        Column::Selected if y >= layout::ICON / 2 && index < pending - 1 => Some(Action::Down),
        Column::Selected => None,
    }
}

fn drawArrow(this: &GuiScreen, font: &FontRenderer, action: Action, x: i32, y: i32)
{
    let Some(arrow) = ARROWS.iter().find(|arrow| arrow.m_action == action)
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

fn drawText(this: &GuiScreen, pack: &Pack, x: i32, y: i32, width: i32)
{
    let font: Rc<FontRenderer> = this.fontRenderer();
    let label: Option<&str> =
        FORMATS.iter().find(|format| format.m_layout == pack.layout()).map(|format| format.m_label);
    let labelWidth: i32 = label.map_or(0, |label| font.getStringWidth(label) + layout::LABEL_GAP);
    if let Some(label) = label
    {
        this.m_gui.drawString(
            &font,
            label,
            x + width - labelWidth + layout::LABEL_GAP,
            y + layout::NAME_Y,
            colors::DARK_GRAY,
        );
    }

    this.m_gui.drawString(&font, &fit(&font, pack.name(), width - labelWidth), x, y + layout::NAME_Y, colors::WHITE);

    let description: &[String] = pack.description();
    let lines: [(Option<&String>, i32); 2] = [
        (description.first(), layout::DESCRIPTION_Y),
        (description.get(1), layout::DESCRIPTION_Y + layout::LINE_HEIGHT),
    ];
    for (line, offset) in lines
    {
        if let Some(line) = line
        {
            this.m_gui.drawString(&font, &fit(&font, line, width), x, y + offset, colors::GRAY);
        }
    }
}

fn drawSlot(slot: &Rc<GuiSlot>, index: i32, x: i32, y: i32, height: i32, tesselator: &Rc<Tessellator>)
{
    let this: Rc<GuiScreen> = screen(slot);
    let mc: Rc<Minecraft> = this.mc();
    let column: Column = column(slot);
    let packs: Vec<Rc<TexturePack>> = rows(&this, column);
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
    if let Some(action) = iconAction(column, index, packs.len() as i32, (mouseX - x, mouseY - y)).filter(|_| inList)
    {
        ext.m_hover.set(Some((column, index, action)));
        drawArrow(&this, &this.fontRenderer(), action, x, y);
    }

    let textX: i32 = x + layout::ICON + layout::TEXT_GAP;
    drawText(
        &this,
        texturePack.pack(),
        textX,
        y,
        rowWidth(slot.right() - slot.left()) - layout::ICON - layout::TEXT_GAP,
    );
}

fn apply(this: &GuiScreen, column: Column, index: i32, action: Action)
{
    let packs: Vec<Rc<TexturePack>> = rows(this, column);
    let Some(pack) = packs.get(index as usize)
    else
    {
        return;
    };

    let mut pending: std::cell::RefMut<Vec<Rc<TexturePack>>> = fields(this).m_pending.borrow_mut();
    let position: Option<usize> = pending.iter().position(|picked| Rc::ptr_eq(picked, pack));

    match (action, position)
    {
        (Action::Add, None) => pending.push(Rc::clone(pack)),
        (Action::Remove, Some(position)) =>
        {
            pending.remove(position);
        }

        // the list shows the top pack first so up means later in the stack
        (Action::Up, Some(position)) if position + 1 < pending.len() => pending.swap(position, position + 1),
        (Action::Down, Some(position)) if position > 0 => pending.swap(position, position - 1),
        _ =>
        {}
    }
}

fn elementClicked(slot: &Rc<GuiSlot>, index: i32)
{
    let this: Rc<GuiScreen> = screen(slot);
    let column: Column = column(slot);
    let hover: Option<(Column, i32, Action)> = fields(&this).m_lastHover.get();

    match hover
    {
        Some((hovered, row, action)) if hovered == column && row == index => apply(&this, column, index, action),
        // a click anywhere on an available row adds it
        _ if column == Column::Available => apply(&this, column, index, Action::Add),
        _ =>
        {}
    }
}

const TEXTURE_PACK_SLOT: GuiSlotClass = GuiSlotClass
{
    m_name: "GuiTexturePackSlot",
    m_parent: Some(&GUI_SLOT),
    m_interfaces: &[],
    getSize: |slot| rows(&screen(slot), column(slot)).len() as i32,
    elementClicked: |slot, index, _| elementClicked(slot, index),
    isSelected: |_, _| false,
    getContentHeight: |slot| slot.getSize() * layout::ROW_HEIGHT,
    // the left list paints the screen background once for both
    drawBackground: |slot|
    {
        if column(slot) == Column::Available
        {
            screen(slot).drawDefaultBackground();
        }
    },
    drawSlot,
    ..GUI_SLOT
};

// keeps the pending choice on the packs a rescan just loaded
fn refreshPending(this: &GuiScreen)
{
    let list: Rc<TexturePackList> = texturePackList(&this.mc());
    list.updateAvaliableTexturePacks();

    let available: Vec<Rc<TexturePack>> = list.availableTexturePacks();
    let mut pending: std::cell::RefMut<Vec<Rc<TexturePack>>> = fields(this).m_pending.borrow_mut();
    let names: Vec<String> = pending.iter().map(|pack| pack.pack().name().to_owned()).collect();
    *pending = names
        .iter()
        .filter_map(|name| available.iter().skip(1).find(|pack| pack.pack().name() == name.as_str()).cloned())
        .collect();
}

fn slot(this: &GuiScreen, column: Column, left: i32, right: i32) -> Rc<GuiSlot>
{
    let height: i32 = this.m_height.get();
    let slot: Rc<GuiSlot> = GuiSlot::construct(
        &TEXTURE_PACK_SLOT,
        Box::new(TexturePackSlotExt { m_screen: Rc::downgrade(&this.rc()), m_column: column }),
        this.mc(),
        this.m_width.get(),
        height,
        layout::LIST_TOP,
        height - layout::LIST_BOTTOM,
        layout::ROW_HEIGHT,
    );
    slot.setColumn(left, right);
    slot.setRowWidth(rowWidth(right - left));
    slot.setRenderSelectionHighlight(false);
    slot
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

        let width: i32 = layout::COLUMN.min((this.m_width.get() - layout::GAP * 3) / 2);
        let half: i32 = layout::GAP / 2;
        let available: Rc<GuiSlot> = slot(this, Column::Available, center - half - width, center - half);
        let chosen: Rc<GuiSlot> = slot(this, Column::Selected, center + half, center + half + width);
        available.registerScrollButtons(&this.m_controlList, buttons::AVAILABLE_UP, buttons::AVAILABLE_DOWN);
        chosen.registerScrollButtons(&this.m_controlList, buttons::SELECTED_UP, buttons::SELECTED_DOWN);
        ext.m_available.set(Some(available));
        ext.m_chosen.set(Some(chosen));
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
            _ =>
            {
                ext.m_available.get().orNpe().actionPerformed(button);
                ext.m_chosen.get().orNpe().actionPerformed(button);
            }
        }
    },
    mouseClicked: |this, mouseX, mouseY, button| (GUI_SCREEN.mouseClicked)(this, mouseX, mouseY, button),
    mouseMovedOrUp: |this, mouseX, mouseY, button| (GUI_SCREEN.mouseMovedOrUp)(this, mouseX, mouseY, button),
    drawScreen: |this, mouseX, mouseY, tickDelta|
    {
        let ext: &TexturePacksExt = fields(this);
        ext.m_mouse.set((mouseX, mouseY));
        ext.m_lastHover.set(ext.m_hover.take());

        let available: Rc<GuiSlot> = ext.m_available.get().orNpe();
        let chosen: Rc<GuiSlot> = ext.m_chosen.get().orNpe();
        available.drawScreen(mouseX, mouseY, tickDelta);
        chosen.drawScreen(mouseX, mouseY, tickDelta);

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

        for (slot, title) in [(&available, text::AVAILABLE), (&chosen, text::SELECTED)]
        {
            let middle: i32 = (slot.left() + slot.right()) / 2;
            this.m_gui.drawCenteredString(&font, title, middle, layout::HEADER_Y, colors::WHITE);
        }

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
            m_available: Field::new(None),
            m_chosen: Field::new(None),
            m_pending: RefCell::new(Vec::new()),
            m_mouse: Cell::new((0, 0)),
            m_hover: Cell::new(None),
            m_lastHover: Cell::new(None),
        };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
