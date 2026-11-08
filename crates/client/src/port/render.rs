use crate::render::entity::Render;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn castToRenderPlayer(render: Option<Rc<Render>>) -> Rc<Render>
{
    let render: Rc<Render> = render.orNpe();
    if !render.instanceOf("RenderPlayer")
    {
        jvm::throwable::throw(
            &jvm::throwable::CLASS_CAST,
            &format!("{} cannot be cast to RenderPlayer", render.className()),
        );
    }
    render
}
