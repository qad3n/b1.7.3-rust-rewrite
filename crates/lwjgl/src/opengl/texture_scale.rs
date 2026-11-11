use super::gl11::GL11;
use std::cell::{Cell, RefCell, RefMut};
use std::collections::HashMap;

const IDENTITY: f32 = 1.0;

thread_local!
{
    static SCALES: RefCell<HashMap<i32, f32>> = RefCell::new(HashMap::new());
    static APPLIED: Cell<f32> = const { Cell::new(IDENTITY) };
    static MATRIX_MODE: Cell<i32> = const { Cell::new(GL11::GL_MODELVIEW) };
}

pub struct TextureScale;

impl TextureScale
{
    // a sheet taller than it is wide keeps its square tile math, the texture matrix squeezes v to fit
    pub fn set(texture: i32, scale: f32)
    {
        SCALES.with(|scales|
        {
            let mut scales: RefMut<HashMap<i32, f32>> = scales.borrow_mut();

            if scale == IDENTITY
            {
                scales.remove(&texture);
            }
            else
            {
                scales.insert(texture, scale);
            }
        });
    }

    pub(crate) fn bound(texture: i32)
    {
        let scale: f32 = SCALES.with(|scales| scales.borrow().get(&texture).copied().unwrap_or(IDENTITY));
        if scale == IDENTITY && APPLIED.get() == IDENTITY
        {
            return;
        }

        let mode: i32 = MATRIX_MODE.get();

        GL11::glMatrixMode(GL11::GL_TEXTURE);
        GL11::glLoadIdentity();

        if scale != IDENTITY
        {
            GL11::glScalef(1.0, scale, 1.0);
        }

        GL11::glMatrixMode(mode);
        APPLIED.set(scale);
    }

    pub(crate) fn matrixMode(mode: i32) { MATRIX_MODE.set(mode); }
}
