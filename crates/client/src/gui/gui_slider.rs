use crate::client::Minecraft;
use crate::gui::gui_button::{GUI_BUTTON, GuiButton, GuiButtonClass};
use crate::settings::{EnumOptions, GameSettings};
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

pub struct GuiSlider
{
    pub m_sliderValue: Cell<f32>,
    pub m_dragging: Cell<bool>,
    m_idFloat: Option<EnumOptions>,
}

fn fields(this: &GuiButton) -> &GuiSlider { this.ext::<GuiSlider>() }

fn slideTo(this: &GuiButton, minecraft: &Minecraft, mouseX: i32)
{
    let slider: &GuiSlider = fields(this);
    let value: f32 = (mouseX - (this.m_xPosition.get() + 4)) as f32 / (this.m_width.get() - 8) as f32;
    slider.m_sliderValue.set(value.clamp(0.0, 1.0));
    let settings: Rc<GameSettings> = minecraft.m_gameSettings.get().orNpe();
    let option: EnumOptions = slider.m_idFloat.orNpe();
    settings.setOptionFloatValue(option, slider.m_sliderValue.get());
    this.m_displayString.set(Some(settings.getKeyBinding(option)));
}

pub const GUI_SLIDER: GuiButtonClass = GuiButtonClass {
    m_name: "GuiSlider",
    m_parent: Some(&GUI_BUTTON),
    m_interfaces: &[],
    getHoverState: |_, _| 0,
    mouseDragged: |this, minecraft, mouseX, _| {
        if !this.m_enabled2.get()
        {
            return;
        }
        let slider: &GuiSlider = fields(this);
        if slider.m_dragging.get()
        {
            slideTo(this, minecraft, mouseX);
        }
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        let offset: i32 = (slider.m_sliderValue.get() * (this.m_width.get() - 8) as f32) as i32;
        this.m_gui.drawTexturedModalRect(this.m_xPosition.get() + offset, this.m_yPosition.get(), 0, 66, 4, 20);
        this.m_gui.drawTexturedModalRect(this.m_xPosition.get() + offset + 4, this.m_yPosition.get(), 196, 66, 4, 20);
    },
    mousePressed: |this, minecraft, mouseX, mouseY| {
        if (GUI_BUTTON.mousePressed)(this, minecraft, mouseX, mouseY)
        {
            slideTo(this, minecraft, mouseX);
            fields(this).m_dragging.set(true);
            return true;
        }
        false
    },
    mouseReleased: |this, _, _| fields(this).m_dragging.set(false),
};

impl GuiSlider
{
    pub fn new(id: i32, x: i32, y: i32, option: Option<EnumOptions>, message: &str, value: f32) -> Rc<GuiButton>
    {
        let ext: Box<GuiSlider> =
            Box::new(GuiSlider { m_sliderValue: Cell::new(value), m_dragging: Cell::new(false), m_idFloat: option });
        GuiButton::construct(&GUI_SLIDER, ext, id, x, y, 150, 20, message)
    }
}
