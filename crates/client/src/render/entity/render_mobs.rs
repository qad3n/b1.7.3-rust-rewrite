use crate::model::{ModelBase, ModelCreeper, ModelGhast, ModelSpider};
use crate::render::entity::render_living::{self, RENDER_LIVING, setRenderPassModel};
use crate::render::entity::{Render, RenderClass};
use game::entity::{
    entity_chicken, entity_creeper, entity_ghast, entity_pig, entity_sheep, entity_slime, entity_squid, entity_wolf,
};
use game::math::math_helper;
use lwjgl::opengl::GL11;
use std::rc::Rc;

fn lerp(previous: f32, current: f32, tickDelta: f32) -> f32 { previous + (current - previous) * tickDelta }

struct CreeperRenderExt
{
    m_model: Rc<dyn ModelBase>,
}

pub fn newCreeper() -> Rc<Render>
{
    let ext: CreeperRenderExt = CreeperRenderExt { m_model: Rc::new(ModelCreeper::new(2.0)) };
    render_living::create(&RENDER_CREEPER, Rc::new(ModelCreeper::new(0.0)), 0.5, Box::new(ext))
}

pub const RENDER_CREEPER: RenderClass = RenderClass {
    m_name: "RenderCreeper",
    m_parent: Some(&RENDER_LIVING),
    preRenderCallback: |_, entity, tickDelta| {
        let mut f2: f32 = entity_creeper::setCreeperFlashTime(entity, tickDelta);
        let f3: f32 = 1.0 + math_helper::sin(f2 * 100.0) * f2 * 0.01;
        f2 = f2.clamp(0.0, 1.0);
        f2 *= f2;
        f2 *= f2;
        let f4: f32 = (1.0 + f2 * 0.4) * f3;
        let f5: f32 = (1.0 + f2 * 0.1) / f3;
        GL11::glScalef(f4, f5, f4);
    },
    getColorMultiplier: |_, entity, _, tickDelta| {
        let f3: f32 = entity_creeper::setCreeperFlashTime(entity, tickDelta);
        if (f3 * 10.0) as i32 % 2 == 0
        {
            return 0;
        }
        let n: i32 = ((f3 * 0.2 * 255.0) as i32).clamp(0, 255);
        n << 24 | 255 << 16 | 255 << 8 | 255
    },
    shouldRenderPass: |this, entity, layer, tickDelta| {
        if !entity_creeper::getPowered(entity)
        {
            return false;
        }
        if layer == 1
        {
            let f2: f32 = entity.m_ticksExisted.get() as f32 + tickDelta;
            this.loadTexture("/armor/power.png");
            GL11::glMatrixMode(GL11::GL_TEXTURE);
            GL11::glLoadIdentity();
            let f3: f32 = f2 * 0.01;
            let f4: f32 = f2 * 0.01;
            GL11::glTranslatef(f3, f4, 0.0);
            setRenderPassModel(this, Rc::clone(&this.ext::<CreeperRenderExt>().m_model));
            GL11::glMatrixMode(GL11::GL_MODELVIEW);
            GL11::glEnable(GL11::GL_BLEND);
            let f5: f32 = 0.5;
            GL11::glColor4f(f5, f5, f5, 1.0);
            GL11::glDisable(GL11::GL_LIGHTING);
            GL11::glBlendFunc(GL11::GL_ONE, GL11::GL_ONE);
            return true;
        }
        if layer == 2
        {
            GL11::glMatrixMode(GL11::GL_TEXTURE);
            GL11::glLoadIdentity();
            GL11::glMatrixMode(GL11::GL_MODELVIEW);
            GL11::glEnable(GL11::GL_LIGHTING);
            GL11::glDisable(GL11::GL_BLEND);
        }
        false
    },
    bindDecorationTexture: |_, _, _, _| false,
    ..RENDER_LIVING
};

pub fn newGhast() -> Rc<Render> { render_living::create(&RENDER_GHAST, Rc::new(ModelGhast::new()), 0.5, Box::new(())) }

pub const RENDER_GHAST: RenderClass = RenderClass {
    m_name: "RenderGhast",
    m_parent: Some(&RENDER_LIVING),
    preRenderCallback: |_, entity, tickDelta| {
        let ghast: &entity_ghast::GhastExt = entity_ghast::fields(entity);
        let previous: i32 = ghast.m_prevAttackCounter.get();
        let mut f2: f32 = (previous as f32 + (ghast.m_attackCounter.get() - previous) as f32 * tickDelta) / 20.0;
        if f2 < 0.0
        {
            f2 = 0.0;
        }
        f2 = 1.0 / (f2 * f2 * f2 * f2 * f2 * 2.0 + 1.0);
        let f3: f32 = (8.0 + f2) / 2.0;
        let f4: f32 = (8.0 + 1.0 / f2) / 2.0;
        GL11::glScalef(f4, f3, f4);
        GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
    },
    ..RENDER_LIVING
};

struct SlimeRenderExt
{
    m_scaleAmount: Rc<dyn ModelBase>,
}

pub fn newSlime(model: Rc<dyn ModelBase>, innerModel: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    render_living::create(&RENDER_SLIME, model, shadowSize, Box::new(SlimeRenderExt { m_scaleAmount: innerModel }))
}

pub const RENDER_SLIME: RenderClass = RenderClass {
    m_name: "RenderSlime",
    m_parent: Some(&RENDER_LIVING),
    shouldRenderPass: |this, _, layer, _| {
        if layer == 0
        {
            setRenderPassModel(this, Rc::clone(&this.ext::<SlimeRenderExt>().m_scaleAmount));
            GL11::glEnable(GL11::GL_NORMALIZE);
            GL11::glEnable(GL11::GL_BLEND);
            GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
            return true;
        }
        if layer == 1
        {
            GL11::glDisable(GL11::GL_BLEND);
            GL11::glColor4f(1.0, 1.0, 1.0, 1.0);
        }
        false
    },
    preRenderCallback: |_, entity, tickDelta| {
        let n: i32 = entity_slime::getSlimeSize(entity);
        let slime: &entity_slime::SlimeExt = entity_slime::fields(entity);
        let f2: f32 = lerp(slime.m_lastStretch.get(), slime.m_stretch.get(), tickDelta) / (n as f32 * 0.5 + 1.0);
        let f3: f32 = 1.0 / (f2 + 1.0);
        let f4: f32 = n as f32;
        GL11::glScalef(f3 * f4, 1.0 / f3 * f4, f3 * f4);
    },
    ..RENDER_LIVING
};

pub fn newSpider() -> Rc<Render>
{
    let render: Rc<Render> = render_living::create(&RENDER_SPIDER, Rc::new(ModelSpider::new()), 1.0, Box::new(()));
    setRenderPassModel(&render, Rc::new(ModelSpider::new()));
    render
}

pub const RENDER_SPIDER: RenderClass = RenderClass {
    m_name: "RenderSpider",
    m_parent: Some(&RENDER_LIVING),
    getDeathMaxRotation: |_, _| 180.0,
    shouldRenderPass: |this, entity, layer, _| {
        if layer != 0
        {
            return false;
        }
        this.loadTexture("/mob/spider_eyes.png");
        let f2: f32 = (1.0 - entity.getEntityBrightness(1.0)) * 0.5;
        GL11::glEnable(GL11::GL_BLEND);
        GL11::glDisable(GL11::GL_ALPHA_TEST);
        GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
        GL11::glColor4f(1.0, 1.0, 1.0, f2);
        true
    },
    ..RENDER_LIVING
};

pub fn newSquid(model: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    render_living::create(&RENDER_SQUID, model, shadowSize, Box::new(()))
}

pub const RENDER_SQUID: RenderClass = RenderClass {
    m_name: "RenderSquid",
    m_parent: Some(&RENDER_LIVING),
    rotateCorpse: |_, entity, _, bodyYaw, tickDelta| {
        let squid: &entity_squid::SquidExt = entity_squid::fields(entity);
        let f4: f32 = lerp(squid.m_lastBodyPitch.get(), squid.m_bodyPitch.get(), tickDelta);
        let f5: f32 = lerp(squid.m_lastBodyRoll.get(), squid.m_bodyRoll.get(), tickDelta);
        GL11::glTranslatef(0.0, 0.5, 0.0);
        GL11::glRotatef(180.0 - bodyYaw, 0.0, 1.0, 0.0);
        GL11::glRotatef(f4, 1.0, 0.0, 0.0);
        GL11::glRotatef(f5, 0.0, 1.0, 0.0);
        GL11::glTranslatef(0.0, -1.2, 0.0);
    },
    getBob: |_, entity, tickDelta| {
        let squid: &entity_squid::SquidExt = entity_squid::fields(entity);
        lerp(squid.m_lastTentacleRotation.get(), squid.m_tentacleRotation.get(), tickDelta)
    },
    ..RENDER_LIVING
};

pub fn newWolf(model: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    render_living::create(&RENDER_WOLF, model, shadowSize, Box::new(()))
}

pub const RENDER_WOLF: RenderClass = RenderClass {
    m_name: "RenderWolf",
    m_parent: Some(&RENDER_LIVING),
    getBob: |_, entity, _| entity_wolf::setTailRotation(entity),
    ..RENDER_LIVING
};

pub fn newSheep(model: Rc<dyn ModelBase>, furModel: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    let render: Rc<Render> = render_living::create(&RENDER_SHEEP, model, shadowSize, Box::new(()));
    setRenderPassModel(&render, furModel);
    render
}

pub const RENDER_SHEEP: RenderClass = RenderClass {
    m_name: "RenderSheep",
    m_parent: Some(&RENDER_LIVING),
    shouldRenderPass: |this, entity, layer, tickDelta| {
        if layer == 0 && !entity_sheep::getSheared(entity)
        {
            this.loadTexture("/mob/sheep_fur.png");
            let f2: f32 = entity.getEntityBrightness(tickDelta);
            let color: [f32; 3] = entity_sheep::FLEECE_COLOR_TABLE[entity_sheep::getFleeceColor(entity) as usize];
            GL11::glColor3f(f2 * color[0], f2 * color[1], f2 * color[2]);
            return true;
        }
        false
    },
    ..RENDER_LIVING
};

pub fn newPig(model: Rc<dyn ModelBase>, saddleModel: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    let render: Rc<Render> = render_living::create(&RENDER_PIG, model, shadowSize, Box::new(()));
    setRenderPassModel(&render, saddleModel);
    render
}

pub const RENDER_PIG: RenderClass = RenderClass {
    m_name: "RenderPig",
    m_parent: Some(&RENDER_LIVING),
    shouldRenderPass: |this, entity, layer, _| {
        this.loadTexture("/mob/saddle.png");
        layer == 0 && entity_pig::getSaddled(entity)
    },
    ..RENDER_LIVING
};

pub fn newCow(model: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    render_living::create(&RENDER_COW, model, shadowSize, Box::new(()))
}

pub const RENDER_COW: RenderClass =
    RenderClass { m_name: "RenderCow", m_parent: Some(&RENDER_LIVING), ..RENDER_LIVING };

pub fn newChicken(model: Rc<dyn ModelBase>, shadowSize: f32) -> Rc<Render>
{
    render_living::create(&RENDER_CHICKEN, model, shadowSize, Box::new(()))
}

pub const RENDER_CHICKEN: RenderClass = RenderClass {
    m_name: "RenderChicken",
    m_parent: Some(&RENDER_LIVING),
    getBob: |_, entity, tickDelta| {
        let chicken: &entity_chicken::ChickenExt = entity_chicken::fields(entity);
        let f2: f32 = lerp(chicken.m_lastFlapProgress.get(), chicken.m_flapProgress.get(), tickDelta);
        let f3: f32 = lerp(chicken.m_lastFlapSpeed.get(), chicken.m_destPos.get(), tickDelta);
        (math_helper::sin(f2) + 1.0) * f3
    },
    ..RENDER_LIVING
};

struct GiantRenderExt
{
    m_scale: f32,
}

pub fn newGiantZombie(model: Rc<dyn ModelBase>, shadowSize: f32, scale: f32) -> Rc<Render>
{
    render_living::create(&RENDER_GIANT_ZOMBIE, model, shadowSize * scale, Box::new(GiantRenderExt { m_scale: scale }))
}

pub const RENDER_GIANT_ZOMBIE: RenderClass = RenderClass {
    m_name: "RenderGiantZombie",
    m_parent: Some(&RENDER_LIVING),
    preRenderCallback: |this, _, _| {
        let scale: f32 = this.ext::<GiantRenderExt>().m_scale;
        GL11::glScalef(scale, scale, scale);
    },
    ..RENDER_LIVING
};
