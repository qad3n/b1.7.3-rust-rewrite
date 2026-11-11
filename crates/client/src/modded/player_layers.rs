use crate::client::Minecraft;
use crate::model::{ModelBiped, ModelRenderer};
use crate::render::RenderEngine;
use game::entity::Entity;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use modloader::mod_protocol::abi::{BipedPart, ModelOp, PlayerLayer};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

const MODEL_SCALE: f32 = 0.0625;
const CHANNEL: f32 = 255.0;

thread_local!
{
    // one biped per inflation a layer asks for, built once
    static MODELS: RefCell<Vec<(u32, Rc<ModelBiped>)>> = const { RefCell::new(Vec::new()) };
}

fn model(inflate: f32) -> Rc<ModelBiped>
{
    MODELS.with(|models|
    {
        let mut models: RefMut<Vec<(u32, Rc<ModelBiped>)>> = models.borrow_mut();
        if let Some((_, found)) = models.iter().find(|(bits, _)| *bits == inflate.to_bits())
        {
            return Rc::clone(found);
        }

        let built: Rc<ModelBiped> = Rc::new(ModelBiped::new(inflate, 0.0));
        models.push((inflate.to_bits(), Rc::clone(&built)));
        built
    })
}

fn copy(from: &ModelRenderer, to: &ModelRenderer)
{
    to.m_rotationPointX.set(from.m_rotationPointX.get());
    to.m_rotationPointY.set(from.m_rotationPointY.get());
    to.m_rotationPointZ.set(from.m_rotationPointZ.get());
    to.m_rotateAngleX.set(from.m_rotateAngleX.get());
    to.m_rotateAngleY.set(from.m_rotateAngleY.get());
    to.m_rotateAngleZ.set(from.m_rotateAngleZ.get());
}

fn pairs<'a>(main: &'a ModelBiped, layer: &'a ModelBiped) -> [(&'a ModelRenderer, &'a ModelRenderer); 6]
{
    [
        (&main.m_bipedHead, &layer.m_bipedHead),
        (&main.m_bipedBody, &layer.m_bipedBody),
        (&main.m_bipedRightArm, &layer.m_bipedRightArm),
        (&main.m_bipedLeftArm, &layer.m_bipedLeftArm),
        (&main.m_bipedRightLeg, &layer.m_bipedRightLeg),
        (&main.m_bipedLeftLeg, &layer.m_bipedLeftLeg),
    ]
}

fn bind(texture: &str)
{
    let Some(mc) = Minecraft::theMinecraft()
    else
    {
        return;
    };

    let engine: Rc<RenderEngine> = mc.m_renderEngine.get().orNpe();
    super::textures::bindFile(&engine, texture);
}

fn drawLayer(main: &ModelBiped, layer: &PlayerLayer, cloak: &mut dyn FnMut(&ModelBiped))
{
    let biped: Rc<ModelBiped> = model(layer.inflate);
    for (from, to) in pairs(main, &biped)
    {
        copy(from, to);
    }

    bind(&layer.texture);

    let [red, green, blue] = [16, 8, 0].map(|shift| ((layer.color >> shift) & 0xFF) as f32 / CHANNEL);
    GL11::glColor4f(red, green, blue, 1.0);

    if layer.glow
    {
        GL11::glDisable(GL11::GL_LIGHTING);
    }

    if layer.blend
    {
        GL11::glEnable(GL11::GL_BLEND);
        GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
    }

    GL11::glPushMatrix();
    GL11::glScalef(layer.scale, layer.scale, layer.scale);

    for part in &layer.parts
    {
        match part
        {
            BipedPart::Head => biped.m_bipedHead.render(MODEL_SCALE),
            BipedPart::Body => biped.m_bipedBody.render(MODEL_SCALE),
            BipedPart::RightArm => biped.m_bipedRightArm.render(MODEL_SCALE),
            BipedPart::LeftArm => biped.m_bipedLeftArm.render(MODEL_SCALE),
            BipedPart::RightLeg => biped.m_bipedRightLeg.render(MODEL_SCALE),
            BipedPart::LeftLeg => biped.m_bipedLeftLeg.render(MODEL_SCALE),
            BipedPart::Cloak => cloak(&biped),
        }
    }

    attachments(&layer.ops);
    GL11::glPopMatrix();

    if layer.blend
    {
        GL11::glDisable(GL11::GL_BLEND);
    }

    if layer.glow
    {
        GL11::glEnable(GL11::GL_LIGHTING);
    }

    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
}

// every mod layer for this player drawn in the model's space, capes through the vanilla cloak swing
pub fn render(main: &ModelBiped, player: &Rc<Entity>, mut cloak: impl FnMut(&ModelBiped))
{
    if !modloader::play::active()
    {
        return;
    }

    for layer in modloader::screens::playerLayers(player)
    {
        drawLayer(main, &layer, &mut cloak);
    }
}

// geometry a layer adds beside the biped parts, without parts of its own to pose
fn attachments(ops: &[ModelOp])
{
    for op in ops
    {
        match op
        {
            ModelOp::Push => GL11::glPushMatrix(),
            ModelOp::Pop => GL11::glPopMatrix(),
            ModelOp::Translate([x, y, z]) => GL11::glTranslatef(*x, *y, *z),
            ModelOp::Rotate(angle, [x, y, z]) => GL11::glRotatef(*angle, *x, *y, *z),
            ModelOp::Scale([x, y, z]) => GL11::glScalef(*x, *y, *z),
            ModelOp::Normal([x, y, z]) => GL11::glNormal3f(*x, *y, *z),
            ModelOp::Quads(vertices) => super::entity_model::drawQuads(vertices),
            _ =>
            {}
        }
    }
}
