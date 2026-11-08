use crate::gui::FontRenderer;
use crate::render::entity::render_biped::rendersAsBlock;
use crate::render::entity::{RENDER, Render, RenderClass};
use crate::render::{RenderBlocks, RenderEngine, Tessellator};
use game::block::Block;
use game::entity::{Entity, entity_item};
use game::item::{Item, ItemStack};
use game::math::{AxisAlignedBB, math_helper};
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::opengl::{GL11, GL12};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const ITEM_SEED: i64 = 187;
const TEXTURE_SIZE: f32 = 256.0;
const PISTON_RENDER_TYPE: i32 = 16;
const BLOCK_ITEM_LIMIT: i32 = 256;

pub struct RenderItemExt
{
    m_renderBlocks: RefCell<RenderBlocks>,
    m_random: Random,
    pub m_useCustomDisplayColor: Cell<bool>,
}

fn ext(this: &Render) -> &RenderItemExt { this.ext::<RenderItemExt>() }

pub struct RenderItem
{
    m_render: Rc<Render>,
}

impl RenderItem
{
    pub fn new() -> RenderItem { RenderItem { m_render: Self::createRender() } }

    pub fn render(&self) -> &Rc<Render> { &self.m_render }

    pub fn useCustomDisplayColor(&self) -> &Cell<bool> { &ext(&self.m_render).m_useCustomDisplayColor }

    pub(crate) fn createRender() -> Rc<Render>
    {
        let itemExt: RenderItemExt = RenderItemExt {
            m_renderBlocks: RefCell::new(RenderBlocks::empty()),
            m_random: Random::new(),
            m_useCustomDisplayColor: Cell::new(true),
        };
        let render: Rc<Render> = Render::alloc(&RENDER_ITEM, None, Box::new(itemExt));
        render.m_shadowSize.set(0.15);
        render.m_shadowDarkness.set(0.75);
        render
    }

    pub fn renderAABB(shape: &AxisAlignedBB) { Render::renderAABB(shape); }

    pub fn drawItemIntoGui(
        &self,
        _textRenderer: &FontRenderer,
        textureManager: &RenderEngine,
        item: i32,
        metadata: i32,
        sprite: i32,
        x: i32,
        y: i32,
    )
    {
        let itemExt: &RenderItemExt = ext(&self.m_render);
        if rendersAsBlock(item)
        {
            textureManager.bindTexture(textureManager.getTexture("/terrain.png"));
            let block: Rc<Block> = Block::byId(item).orNpe();
            GL11::glPushMatrix();
            GL11::glTranslatef((x - 2) as f32, (y + 3) as f32, -3.0);
            GL11::glScalef(10.0, 10.0, 10.0);
            GL11::glTranslatef(1.0, 0.5, 1.0);
            GL11::glScalef(1.0, 1.0, -1.0);
            GL11::glRotatef(210.0, 1.0, 0.0, 0.0);
            GL11::glRotatef(45.0, 0.0, 1.0, 0.0);
            let n2: i32 = Item::byIdOrNpe(item).getColorFromDamage(metadata);
            if itemExt.m_useCustomDisplayColor.get()
            {
                GL11::glColor4f(channel(n2, 16), channel(n2, 8), channel(n2, 0), 1.0);
            }
            GL11::glRotatef(-90.0, 0.0, 1.0, 0.0);
            let mut renderBlocks = itemExt.m_renderBlocks.borrow_mut();
            renderBlocks.m_updateColor = itemExt.m_useCustomDisplayColor.get();
            renderBlocks.renderBlockOnInventory(&block, metadata, 1.0);
            renderBlocks.m_updateColor = true;
            GL11::glPopMatrix();
        }
        else if sprite >= 0
        {
            GL11::glDisable(GL11::GL_LIGHTING);
            let sheet: &str = if item < BLOCK_ITEM_LIMIT
            {
                "/terrain.png"
            }
            else
            {
                "/gui/items.png"
            };
            textureManager.bindTexture(textureManager.getTexture(sheet));
            let n: i32 = Item::byIdOrNpe(item).getColorFromDamage(metadata);
            if itemExt.m_useCustomDisplayColor.get()
            {
                GL11::glColor4f(channel(n, 16), channel(n, 8), channel(n, 0), 1.0);
            }
            Self::renderTexturedQuad(x, y, sprite % 16 * 16, sprite / 16 * 16, 16, 16);
            GL11::glEnable(GL11::GL_LIGHTING);
        }
        GL11::glEnable(GL11::GL_CULL_FACE);
    }

    pub fn renderItemIntoGUI(
        &self,
        textRenderer: &FontRenderer,
        textureManager: &RenderEngine,
        item: Option<&Rc<ItemStack>>,
        x: i32,
        y: i32,
    )
    {
        let Some(item) = item
        else
        {
            return;
        };
        self.drawItemIntoGui(
            textRenderer,
            textureManager,
            item.m_itemID.get(),
            item.getItemDamage(),
            item.getIconIndex(),
            x,
            y,
        );
    }

    pub fn renderItemOverlayIntoGUI(
        &self,
        textRenderer: &FontRenderer,
        _textureManager: &RenderEngine,
        item: Option<&Rc<ItemStack>>,
        x: i32,
        y: i32,
    )
    {
        let Some(item) = item
        else
        {
            return;
        };
        if item.m_stackSize.get() > 1
        {
            let string: String = item.m_stackSize.get().to_string();
            GL11::glDisable(GL11::GL_LIGHTING);
            GL11::glDisable(GL11::GL_DEPTH_TEST);
            textRenderer.drawStringWithShadow(
                &string,
                x + 19 - 2 - textRenderer.getStringWidth(&string),
                y + 6 + 3,
                0xFFFFFF,
            );
            GL11::glEnable(GL11::GL_LIGHTING);
            GL11::glEnable(GL11::GL_DEPTH_TEST);
        }
        if item.isItemDamaged()
        {
            let damage: f64 = item.getItemDamageForDisplay() as f64;
            let maxDamage: f64 = item.getMaxDamage() as f64;
            let n: i32 = jvm::math::roundDouble(13.0 - damage * 13.0 / maxDamage) as i32;
            let n2: i32 = jvm::math::roundDouble(255.0 - damage * 255.0 / maxDamage) as i32;
            GL11::glDisable(GL11::GL_LIGHTING);
            GL11::glDisable(GL11::GL_DEPTH_TEST);
            GL11::glDisable(GL11::GL_TEXTURE_2D);
            let tessellator: Rc<Tessellator> = Tessellator::instance();
            let n3: i32 = (255 - n2) << 16 | n2 << 8;
            let n4: i32 = (255 - n2) / 4 << 16 | 0x3F00;
            Self::renderQuad(&tessellator, x + 2, y + 13, 13, 2, 0);
            Self::renderQuad(&tessellator, x + 2, y + 13, 12, 1, n4);
            Self::renderQuad(&tessellator, x + 2, y + 13, n, 1, n3);
            GL11::glEnable(GL11::GL_TEXTURE_2D);
            GL11::glEnable(GL11::GL_LIGHTING);
            GL11::glEnable(GL11::GL_DEPTH_TEST);
            GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        }
    }

    fn renderQuad(tesselator: &Tessellator, x: i32, y: i32, width: i32, height: i32, color: i32)
    {
        tesselator.startDrawingQuads();
        tesselator.setColorOpaque_I(color);
        tesselator.addVertex(x as f64, y as f64, 0.0);
        tesselator.addVertex(x as f64, (y + height) as f64, 0.0);
        tesselator.addVertex((x + width) as f64, (y + height) as f64, 0.0);
        tesselator.addVertex((x + width) as f64, y as f64, 0.0);
        tesselator.draw();
    }

    pub fn renderTexturedQuad(x: i32, y: i32, u: i32, v: i32, width: i32, height: i32)
    {
        let f: f64 = 0.0;
        let f2: f32 = 0.00390625;
        let f3: f32 = 0.00390625;
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        tessellator.startDrawingQuads();
        let vertex = |px: i32, py: i32, tu: i32, tv: i32| {
            tessellator.addVertexWithUV(px as f64, py as f64, f, (tu as f32 * f2) as f64, (tv as f32 * f3) as f64);
        };
        vertex(x, y + height, u, v + height);
        vertex(x + width, y + height, u + width, v + height);
        vertex(x + width, y, u + width, v);
        vertex(x, y, u, v);
        tessellator.draw();
    }
}

fn channel(color: i32, shift: i32) -> f32 { (color >> shift & 0xFF) as f32 / 255.0 }

fn doRender(this: &Rc<Render>, entity: &Rc<Entity>, d: f64, d2: f64, d3: f64, _f: f32, f2: f32)
{
    let itemExt: &RenderItemExt = ext(this);
    let random: &Random = &itemExt.m_random;
    random.setSeed(ITEM_SEED);
    let fields: &entity_item::ItemFields = entity_item::fields(entity);
    let itemStack: Rc<ItemStack> = fields.m_item.get().orNpe();
    GL11::glPushMatrix();
    let age: f32 = fields.m_age.get() as f32 + f2;
    let f3: f32 = math_helper::sin(age / 10.0 + fields.m_bobOffset.get()) * 0.1 + 0.1;
    let f4: f32 = (age / 20.0 + fields.m_bobOffset.get()) * 57.295776;
    let stackSize: i32 = itemStack.m_stackSize.get();
    let n: i32 = match stackSize
    {
        size if size > 20 => 4,
        size if size > 5 => 3,
        size if size > 1 => 2,
        _ => 1,
    };
    GL11::glTranslatef(d as f32, d2 as f32 + f3, d3 as f32);
    GL11::glEnable(GL12::GL_RESCALE_NORMAL);
    let itemID: i32 = itemStack.m_itemID.get();
    if rendersAsBlock(itemID)
    {
        GL11::glRotatef(f4, 0.0, 1.0, 0.0);
        this.loadTexture("/terrain.png");
        let block: Rc<Block> = Block::byId(itemID).orNpe();
        let mut f5: f32 = 0.25;
        if !block.renderAsNormalBlock()
            && itemID != Block::stairSingle().m_blockID
            && block.getRenderType() != PISTON_RENDER_TYPE
        {
            f5 = 0.5;
        }
        GL11::glScalef(f5, f5, f5);
        for i in 0..n
        {
            GL11::glPushMatrix();
            if i > 0
            {
                let f6: f32 = (random.nextFloat() * 2.0 - 1.0) * 0.2 / f5;
                let f7: f32 = (random.nextFloat() * 2.0 - 1.0) * 0.2 / f5;
                let f8: f32 = (random.nextFloat() * 2.0 - 1.0) * 0.2 / f5;
                GL11::glTranslatef(f6, f7, f8);
            }
            itemExt.m_renderBlocks.borrow_mut().renderBlockOnInventory(
                &block,
                itemStack.getItemDamage(),
                entity.getEntityBrightness(f2),
            );
            GL11::glPopMatrix();
        }
    }
    else
    {
        GL11::glScalef(0.5, 0.5, 0.5);
        let n3: i32 = itemStack.getIconIndex();
        this.loadTexture(
            if itemID < BLOCK_ITEM_LIMIT
            {
                "/terrain.png"
            }
            else
            {
                "/gui/items.png"
            },
        );
        let tessellator: Rc<Tessellator> = Tessellator::instance();
        let f12: f64 = ((n3 % 16 * 16 + 0) as f32 / TEXTURE_SIZE) as f64;
        let f13: f64 = ((n3 % 16 * 16 + 16) as f32 / TEXTURE_SIZE) as f64;
        let f14: f64 = ((n3 / 16 * 16 + 0) as f32 / TEXTURE_SIZE) as f64;
        let f15: f64 = ((n3 / 16 * 16 + 16) as f32 / TEXTURE_SIZE) as f64;
        let f16: f32 = 1.0;
        let f17: f32 = 0.5;
        let f18: f32 = 0.25;
        if itemExt.m_useCustomDisplayColor.get()
        {
            let n2: i32 = Item::byIdOrNpe(itemID).getColorFromDamage(itemStack.getItemDamage());
            let f19: f32 = entity.getEntityBrightness(f2);
            GL11::glColor4f(channel(n2, 16) * f19, channel(n2, 8) * f19, channel(n2, 0) * f19, 1.0);
        }
        for n2 in 0..n
        {
            GL11::glPushMatrix();
            if n2 > 0
            {
                let f11: f32 = (random.nextFloat() * 2.0 - 1.0) * 0.3;
                let f10: f32 = (random.nextFloat() * 2.0 - 1.0) * 0.3;
                let f9: f32 = (random.nextFloat() * 2.0 - 1.0) * 0.3;
                GL11::glTranslatef(f11, f10, f9);
            }
            GL11::glRotatef(180.0 - this.renderManager().m_playerViewY.get(), 0.0, 1.0, 0.0);
            tessellator.startDrawingQuads();
            tessellator.setNormal(0.0, 1.0, 0.0);
            tessellator.addVertexWithUV((0.0 - f17) as f64, (0.0 - f18) as f64, 0.0, f12, f15);
            tessellator.addVertexWithUV((f16 - f17) as f64, (0.0 - f18) as f64, 0.0, f13, f15);
            tessellator.addVertexWithUV((f16 - f17) as f64, (1.0 - f18) as f64, 0.0, f13, f14);
            tessellator.addVertexWithUV((0.0 - f17) as f64, (1.0 - f18) as f64, 0.0, f12, f14);
            tessellator.draw();
            GL11::glPopMatrix();
        }
    }
    GL11::glDisable(GL12::GL_RESCALE_NORMAL);
    GL11::glPopMatrix();
}

pub const RENDER_ITEM: RenderClass = RenderClass { m_name: "RenderItem", m_parent: Some(&RENDER), doRender, ..RENDER };
