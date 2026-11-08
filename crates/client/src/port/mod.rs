pub mod awt;
pub mod awt_frame;
pub mod calendar;
pub mod file_url;
pub mod network;
pub mod render;
pub mod runtime;
pub mod sound;
pub mod system;
pub mod world_release;

use crate::render::Tessellator;
use std::any::Any;

pub fn tessellator(target: &dyn Any) -> &Tessellator
{
    target.downcast_ref().expect("renderParticle is only called with the tessellator")
}
