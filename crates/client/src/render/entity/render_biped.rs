use crate::model::{ModelBase, ModelBiped};
use crate::render::RenderBlocks;
use crate::render::entity::render_living::{self, RENDER_LIVING};
use crate::render::entity::{Render, RenderClass};
use game::block::Block;
use game::entity::Entity;
use game::item::{Item, ItemStack};
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::rc::Rc;

const MODEL_SCALE: f32 = 0.0625;
const BLOCK_ITEM_LIMIT: i32 = 256;

pub fn new(model: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    render_living::create(&RENDER_BIPED, model, shadowSize, Box::new(()))
}

pub fn modelBipedMain(this: &Render) -> &ModelBiped
{
    this.living().m_mainModel.biped().expect("RenderBiped holds a ModelBiped")
}

pub fn rendersAsBlock(itemID: i32) -> bool
{
    itemID < BLOCK_ITEM_LIMIT && RenderBlocks::renderItemIn3d(Block::byId(itemID).orNpe().getRenderType())
}

pub fn transformHeldItem(itemstack: &ItemStack, checkRotateAround: bool)
{
    if rendersAsBlock(itemstack.m_itemID.get())
    {
        let mut f2: f32 = 0.5;
        GL11::glTranslatef(0.0, 0.1875, -0.3125);
        f2 *= 0.75;
        GL11::glRotatef(20.0, 1.0, 0.0, 0.0);
        GL11::glRotatef(45.0, 0.0, 1.0, 0.0);
        GL11::glScalef(f2, -f2, f2);
    }
    else if Item::byIdOrNpe(itemstack.m_itemID.get()).isFull3D()
    {
        let f: f32 = 0.625;
        if checkRotateAround && Item::byIdOrNpe(itemstack.m_itemID.get()).shouldRotateAroundWhenRendering()
        {
            GL11::glRotatef(180.0, 0.0, 0.0, 1.0);
            GL11::glTranslatef(0.0, -0.125, 0.0);
        }
        GL11::glTranslatef(0.0, 0.1875, 0.0);
        GL11::glScalef(f, -f, f);
        GL11::glRotatef(-100.0, 1.0, 0.0, 0.0);
        GL11::glRotatef(45.0, 0.0, 1.0, 0.0);
    }
    else
    {
        let f1: f32 = 0.375;
        GL11::glTranslatef(0.25, 0.1875, -0.1875);
        GL11::glScalef(f1, f1, f1);
        GL11::glRotatef(60.0, 0.0, 0.0, 1.0);
        GL11::glRotatef(-90.0, 1.0, 0.0, 0.0);
        GL11::glRotatef(20.0, 0.0, 0.0, 1.0);
    }
}

pub fn moveToRightHand(model: &ModelBiped)
{
    model.m_bipedRightArm.postRender(MODEL_SCALE);
    GL11::glTranslatef(-0.0625, 0.4375, 0.0625);
}

fn renderEquippedItems(this: &Rc<Render>, entity: &Rc<Entity>, _tickDelta: f32)
{
    let Some(itemstack) = entity.getHeldItem()
    else
    {
        return;
    };
    GL11::glPushMatrix();
    moveToRightHand(modelBipedMain(this));
    transformHeldItem(&itemstack, false);
    this.renderManager().m_itemRenderer.get().orNpe().renderItem(entity, &itemstack);
    GL11::glPopMatrix();
}

pub const RENDER_BIPED: RenderClass =
    RenderClass { m_name: "RenderBiped", m_parent: Some(&RENDER_LIVING), renderEquippedItems, ..RENDER_LIVING };
