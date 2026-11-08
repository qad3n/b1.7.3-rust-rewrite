use crate::client::ChatAllowedCharacters;
use crate::gui::{FontRenderer, Gui, GuiScreen};
use game::port::text;
use jvm::field::Field;
use jvm::string::JString;
use std::cell::Cell;
use std::rc::Rc;

pub(super) fn isAllowedCharacter(chr: u16) -> bool
{
    ChatAllowedCharacters::allowedCharacters().encode_utf16().any(|allowed| allowed == chr)
}

pub struct GuiTextField
{
    pub m_gui: Gui,
    m_fontRenderer: Rc<FontRenderer>,
    m_xPos: i32,
    m_yPos: i32,
    m_width: i32,
    m_height: i32,
    m_text: Field<JString>,
    m_maxStringLength: Cell<i32>,
    m_cursorCounter: Cell<i32>,
    pub m_isFocused: Cell<bool>,
    pub m_isEnabled: Cell<bool>,
    m_parentGuiScreen: Rc<GuiScreen>,
}

impl GuiTextField
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        screen: Rc<GuiScreen>,
        textRenderer: Rc<FontRenderer>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        message: &str,
    ) -> GuiTextField
    {
        let field: GuiTextField = GuiTextField {
            m_gui: Gui::new(),
            m_fontRenderer: textRenderer,
            m_xPos: x,
            m_yPos: y,
            m_width: width,
            m_height: height,
            m_text: Field::new(JString::new()),
            m_maxStringLength: Cell::new(0),
            m_cursorCounter: Cell::new(0),
            m_isFocused: Cell::new(false),
            m_isEnabled: Cell::new(true),
            m_parentGuiScreen: screen,
        };
        field.setText(message);
        field
    }

    pub fn setText(&self, text: &str) { self.m_text.set(JString::from(text)); }

    pub fn getText(&self) -> JString { self.m_text.get() }

    pub fn updateCursorCounter(&self) { self.m_cursorCounter.set(self.m_cursorCounter.get() + 1); }

    pub fn textboxKeyTyped(&self, chr: u16, key: i32)
    {
        if !self.m_isEnabled.get() || !self.m_isFocused.get()
        {
            return;
        }
        if chr == '\t' as u16
        {
            self.m_parentGuiScreen.selectNextField();
        }
        if chr == 0x16
        {
            let clipboard: JString = JString::from(GuiScreen::getClipboardString().unwrap_or_default().as_str());
            let text: JString = self.m_text.get();
            let length: i32 = (32 - text.length()).min(clipboard.length());
            if length > 0
            {
                let mut chars: Vec<u16> = text.chars().to_vec();
                chars.extend_from_slice(&clipboard.chars()[..length as usize]);
                self.m_text.set(JString::fromUtf16(chars));
            }
        }
        let text: JString = self.m_text.get();
        if key == 14 && text.length() > 0
        {
            self.m_text.set(text.substring(0, text.length() - 1));
        }
        let text: JString = self.m_text.get();
        if isAllowedCharacter(chr)
            && (text.length() < self.m_maxStringLength.get() || self.m_maxStringLength.get() == 0)
        {
            // port appendChar appends one utf16 unit like java string concat with a char
            self.m_text.set(text::appendChar(&text, chr));
        }
    }

    pub fn mouseClicked(&self, mouseX: i32, mouseY: i32, _button: i32)
    {
        let bl: bool = self.m_isEnabled.get()
            && mouseX >= self.m_xPos
            && mouseX < self.m_xPos + self.m_width
            && mouseY >= self.m_yPos
            && mouseY < self.m_yPos + self.m_height;
        self.setFocused(bl);
    }

    pub fn setFocused(&self, focused: bool)
    {
        if focused && !self.m_isFocused.get()
        {
            self.m_cursorCounter.set(0);
        }
        self.m_isFocused.set(focused);
    }

    pub fn drawTextBox(&self)
    {
        let (x, y, width, height): (i32, i32, i32, i32) = (self.m_xPos, self.m_yPos, self.m_width, self.m_height);
        self.m_gui.drawRect(x - 1, y - 1, x + width + 1, y + height + 1, -6250336);
        self.m_gui.drawRect(x, y, x + width, y + height, -16777216);
        let text: String = self.getText().to_string();
        if self.m_isEnabled.get()
        {
            let bl: bool = self.m_isFocused.get() && self.m_cursorCounter.get() / 6 % 2 == 0;
            let shown: String = format!("{text}{}", if bl { "_" } else { "" });
            self.m_gui.drawString(&self.m_fontRenderer, &shown, x + 4, y + (height - 8) / 2, 0xE0E0E0);
        }
        else
        {
            self.m_gui.drawString(&self.m_fontRenderer, &text, x + 4, y + (height - 8) / 2, 0x707070);
        }
    }

    pub fn setMaxStringLength(&self, maximumLength: i32) { self.m_maxStringLength.set(maximumLength); }
}
