use crate::render::entity::render_biped::rendersAsBlock;
use crate::render::{RenderBlocks, RenderManager, Tessellator, channel};
use game::block::Block;
use game::item::{Item, ItemStack};
use jvm::random::Random;
use lwjgl::opengl::{GL11, GL12};
use modloader::mod_protocol::abi::StackData;
use std::cell::RefCell;
use std::rc::Rc;

// RenderItem.doRender's numbers, without the bob and spin of an item lying in the world
mod look
{
    pub const SEED: i64 = 187;
    pub const TEXTURE_SIZE: f32 = 256.0;
    pub const TILE: i32 = 16;
    pub const BLOCK_SCALE: f32 = 0.25;
    pub const SHAPED_BLOCK_SCALE: f32 = 0.5;
    pub const SPRITE_SCALE: f32 = 0.5;
    pub const BLOCK_SPREAD: f32 = 0.2;
    pub const SPRITE_SPREAD: f32 = 0.3;
    pub const SPRITE_WIDTH: f32 = 1.0;
    pub const SPRITE_CENTER: f32 = 0.5;
    pub const SPRITE_FOOT: f32 = 0.25;
    pub const PISTON_RENDER_TYPE: i32 = 16;
    // stack sizes past which another copy shows
    pub const COPY_STEPS: [i32; 3] = [1, 5, 20];
    pub const WHITE: i32 = 0xFFFFFF;
    pub const RED_SHIFT: i32 = 16;
    pub const GREEN_SHIFT: i32 = 8;
    pub const FACE_TURN: f32 = 180.0;
}

thread_local!
{
    static BLOCKS: RefCell<Option<RenderBlocks>> = const { RefCell::new(None) };
}

fn copies(count: i32) -> i32 { 1 + look::COPY_STEPS.iter().filter(|step| count > **step).count() as i32 }

// a random offset between minus and plus amount
fn spread(random: &Random, amount: f32) -> f32
{
    (random.nextFloat() - look::SPRITE_CENTER) * amount / look::SPRITE_CENTER
}

fn bind(path: &str)
{
    if let Some(engine) = RenderManager::instance().m_renderEngine.get()
    {
        engine.bindTexture(engine.getTexture(path));
    }
}

fn drawBlock(stack: &Rc<ItemStack>, count: i32, brightness: f32, random: &Random)
{
    let Some(block) = Block::byId(stack.m_itemID.get())
    else
    {
        return;
    };
    let shaped: bool = !block.renderAsNormalBlock()
        && block.m_blockID != Block::STAIR_SINGLE
        && block.getRenderType() != look::PISTON_RENDER_TYPE;
    let scale: f32 = if shaped
    {
        look::SHAPED_BLOCK_SCALE
    }
    else
    {
        look::BLOCK_SCALE
    };

    bind(modloader::atlas::TERRAIN_PATH);
    GL11::glScalef(scale, scale, scale);

    BLOCKS.with(|renderer|
    {
        let mut renderer = renderer.borrow_mut();
        let renderer: &mut RenderBlocks = renderer.get_or_insert_with(RenderBlocks::empty);

        for copy in 0..copies(count)
        {
            GL11::glPushMatrix();

            if copy > 0
            {
                let amount: f32 = look::BLOCK_SPREAD / scale;
                GL11::glTranslatef(spread(random, amount), spread(random, amount), spread(random, amount));
            }

            renderer.renderBlockOnInventory(&block, stack.getItemDamage(), brightness);
            GL11::glPopMatrix();
        }
    });
}

fn drawSprite(stack: &Rc<ItemStack>, count: i32, brightness: f32, random: &Random)
{
    let id: i32 = stack.m_itemID.get();
    let icon: i32 = stack.getIconIndex();
    let path: &str = if Block::isBlockItem(id)
    {
        modloader::atlas::TERRAIN_PATH
    }
    else
    {
        modloader::atlas::ITEMS_PATH
    };
    let left: f64 = ((icon % look::TILE * look::TILE) as f32 / look::TEXTURE_SIZE) as f64;
    let right: f64 = ((icon % look::TILE * look::TILE + look::TILE) as f32 / look::TEXTURE_SIZE) as f64;
    let top: f64 = ((icon / look::TILE * look::TILE) as f32 / look::TEXTURE_SIZE) as f64;
    let bottom: f64 = ((icon / look::TILE * look::TILE + look::TILE) as f32 / look::TEXTURE_SIZE) as f64;
    let color: i32 = Item::byId(id).map_or(look::WHITE, |item| item.getColorFromDamage(stack.getItemDamage()));
    let tessellator: Rc<Tessellator> = Tessellator::instance();
    let viewYaw: f32 = RenderManager::instance().m_playerViewY.get();
    let [x0, x1]: [f64; 2] = [(-look::SPRITE_CENTER) as f64, (look::SPRITE_WIDTH - look::SPRITE_CENTER) as f64];
    let [y0, y1]: [f64; 2] = [(-look::SPRITE_FOOT) as f64, (look::SPRITE_WIDTH - look::SPRITE_FOOT) as f64];

    bind(path);
    GL11::glScalef(look::SPRITE_SCALE, look::SPRITE_SCALE, look::SPRITE_SCALE);
    let [red, green, blue]: [f32; 3] =
        [look::RED_SHIFT, look::GREEN_SHIFT, 0].map(|shift| channel(color, shift) * brightness);
    GL11::glColor4f(red, green, blue, 1.0);

    for copy in 0..copies(count)
    {
        GL11::glPushMatrix();

        if copy > 0
        {
            let amount: f32 = look::SPRITE_SPREAD;
            GL11::glTranslatef(spread(random, amount), spread(random, amount), spread(random, amount));
        }

        GL11::glRotatef(look::FACE_TURN - viewYaw, 0.0, 1.0, 0.0);
        tessellator.startDrawingQuads();
        tessellator.setNormal(0.0, 1.0, 0.0);
        tessellator.addVertexWithUV(x0, y0, 0.0, left, bottom);
        tessellator.addVertexWithUV(x1, y0, 0.0, right, bottom);
        tessellator.addVertexWithUV(x1, y1, 0.0, right, top);
        tessellator.addVertexWithUV(x0, y1, 0.0, left, top);
        tessellator.draw();
        GL11::glPopMatrix();
    }
}

// ModelOp::Item, a stack as an item lying in the world draws it
pub(crate) fn draw(data: &StackData, brightness: f32)
{
    let Some(stack) = modloader::host::context::stackFrom(data)
    else
    {
        return;
    };
    let random: Random = Random::new();
    random.setSeed(look::SEED);

    GL11::glPushMatrix();
    GL11::glEnable(GL12::GL_RESCALE_NORMAL);

    let count: i32 = stack.m_stackSize.get();
    if rendersAsBlock(stack.m_itemID.get())
    {
        drawBlock(&stack, count, brightness, &random);
    }
    else
    {
        drawSprite(&stack, count, brightness, &random);
    }

    GL11::glDisable(GL12::GL_RESCALE_NORMAL);
    GL11::glPopMatrix();
    GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
}
