use crate::client::Minecraft;
use crate::gui::GuiButton;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreen, GuiScreenClass};
use crate::gui::gui_text_field::isAllowedCharacter;
use crate::render::TileEntityRenderer;
use game::block::Block;
use game::network::packets::packet130_update_sign::Packet130UpdateSign;
use game::port::text;
use game::tileentity::{TileEntity, tile_entity_sign};
use jvm::string::JString;
use jvm::throwable::OrNpe;
use lwjgl::input::Keyboard;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

pub struct GuiEditSign
{
    m_screenTitle: String,
    m_entitySign: Rc<TileEntity>,
    m_updateCounter: Cell<i32>,
    m_editLine: Cell<i32>,
}

fn fields(this: &GuiScreen) -> &GuiEditSign { this.ext::<GuiEditSign>() }

fn signLine(sign: &TileEntity, line: i32) -> JString
{
    tile_entity_sign::fields(sign).m_signText.with(|text| text[line as usize].clone()).orNpe()
}

fn setSignLine(sign: &TileEntity, line: i32, value: JString)
{
    tile_entity_sign::fields(sign).m_signText.update(|text| text[line as usize] = Some(value));
}

pub const GUI_EDIT_SIGN: GuiScreenClass = GuiScreenClass {
    m_name: "GuiEditSign",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    initGui: |this| {
        this.m_controlList.clear();
        Keyboard::enableRepeatEvents(true);
        this.m_controlList.add(GuiButton::new(0, this.m_width.get() / 2 - 100, this.m_height.get() / 4 + 120, "Done"));
    },
    onGuiClosed: |this| {
        Keyboard::enableRepeatEvents(false);
        let mc: Rc<Minecraft> = this.mc();
        if mc.m_theWorld.get().orNpe().m_multiplayerWorld.get()
        {
            let sign: &Rc<TileEntity> = &fields(this).m_entitySign;
            let lines: Vec<Option<JString>> = tile_entity_sign::fields(sign).m_signText.get().to_vec();
            mc.getSendQueue().addToSendQueue(Box::new(Packet130UpdateSign::new(
                sign.m_xCoord.get(),
                sign.m_yCoord.get(),
                sign.m_zCoord.get(),
                lines,
            )));
        }
    },
    updateScreen: |this| fields(this).m_updateCounter.set(fields(this).m_updateCounter.get() + 1),
    actionPerformed: |this, button| {
        if !button.m_enabled.get()
        {
            return;
        }
        if button.m_id.get() == 0
        {
            fields(this).m_entitySign.onInventoryChanged();
            this.mc().displayGuiScreen(None);
        }
    },
    keyTyped: |this, chr, key| {
        let edit: &GuiEditSign = fields(this);
        if key == 200
        {
            edit.m_editLine.set((edit.m_editLine.get() - 1) & 3);
        }
        if key == 208 || key == 28
        {
            edit.m_editLine.set((edit.m_editLine.get() + 1) & 3);
        }
        let sign: &TileEntity = &edit.m_entitySign;
        let line: i32 = edit.m_editLine.get();
        let text: JString = signLine(sign, line);
        if key == 14 && text.length() > 0
        {
            setSignLine(sign, line, text.substring(0, text.length() - 1));
        }
        let text: JString = signLine(sign, line);
        if isAllowedCharacter(chr) && text.length() < 15
        {
            // port appendChar appends one utf16 unit like java string concat with a char
            setSignLine(sign, line, text::appendChar(&text, chr));
        }
    },
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        let edit: &GuiEditSign = fields(this);
        this.drawDefaultBackground();
        this.m_gui.drawCenteredString(&this.fontRenderer(), &edit.m_screenTitle, this.m_width.get() / 2, 40, 0xFFFFFF);
        GL11::glPushMatrix();
        GL11::glTranslatef((this.m_width.get() / 2) as f32, 0.0, 50.0);
        let f: f32 = 93.75;
        GL11::glScalef(-f, -f, -f);
        GL11::glRotatef(180.0, 0.0, 1.0, 0.0);
        let sign: &Rc<TileEntity> = &edit.m_entitySign;
        let block: Option<Rc<Block>> = sign.getBlockType();
        if block.is_some_and(|block| Rc::ptr_eq(&block, &Block::signPost()))
        {
            let f2: f32 = (sign.getBlockMetadata() * 360) as f32 / 16.0;
            GL11::glRotatef(f2, 0.0, 1.0, 0.0);
            GL11::glTranslatef(0.0, -1.0625, 0.0);
        }
        else
        {
            let f3: f32 = match sign.getBlockMetadata()
            {
                2 => 180.0,
                4 => 90.0,
                5 => -90.0,
                _ => 0.0,
            };
            GL11::glRotatef(f3, 0.0, 1.0, 0.0);
            GL11::glTranslatef(0.0, -1.0625, 0.0);
        }
        let signFields: &tile_entity_sign::SignExt = tile_entity_sign::fields(sign);
        if edit.m_updateCounter.get() / 6 % 2 == 0
        {
            signFields.m_lineBeingEdited.set(edit.m_editLine.get());
        }
        TileEntityRenderer::instance().renderTileEntityAt(sign, -0.5, -0.75, -0.5, 0.0);
        signFields.m_lineBeingEdited.set(-1);
        GL11::glPopMatrix();
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

impl GuiEditSign
{
    pub fn new(sign: &Rc<TileEntity>) -> Rc<GuiScreen>
    {
        GuiScreen::construct(
            &GUI_EDIT_SIGN,
            Box::new(GuiEditSign {
                m_screenTitle: "Edit sign message:".to_owned(),
                m_entitySign: Rc::clone(sign),
                m_updateCounter: Cell::new(0),
                m_editLine: Cell::new(0),
            }),
        )
    }
}
