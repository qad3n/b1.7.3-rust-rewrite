use crate::render::particle::entity_fx::{self, extra, interpPosX, interpPosY, interpPosZ};
use crate::render::{RenderEngine, Tessellator};
use game::entity::{Entity, EntityClass};
use game::math::math_helper::floor_double;
use game::world::World;
use lwjgl::opengl::GL11;
use std::cell::Cell;
use std::rc::Rc;

pub struct FootStepFields
{
    m_age: Cell<i32>,
    m_maxAge: Cell<i32>,
    m_textureManager: Rc<RenderEngine>,
}

fn renderParticle(
    this: &Rc<Entity>,
    target: &dyn std::any::Any,
    tickDelta: f32,
    _dx: f32,
    _dy: f32,
    _dz: f32,
    _forwards: f32,
    _sideways: f32,
)
{
    // the game crate entity vtable passes the tessellator as Any so port downcasts it
    let tessellator: &Tessellator = crate::port::tessellator(target);
    let step: &FootStepFields = extra::<FootStepFields>(this);
    let mut f: f32 = (step.m_age.get() as f32 + tickDelta) / step.m_maxAge.get() as f32;
    f *= f;
    let mut f1: f32 = 2.0 - f * 2.0;
    if f1 > 1.0
    {
        f1 = 1.0;
    }
    f1 *= 0.2;
    GL11::glDisable(2896);
    let f2: f32 = 0.125;
    let f3: f32 = (this.posX() - interpPosX()) as f32;
    let f4: f32 = (this.posY() - interpPosY()) as f32;
    let f5: f32 = (this.posZ() - interpPosZ()) as f32;
    let f6: f32 = this.world().getLightBrightness(
        floor_double(this.posX()),
        floor_double(this.posY()),
        floor_double(this.posZ()),
    );
    step.m_textureManager.bindTexture(step.m_textureManager.getTexture("/misc/footprint.png"));
    GL11::glEnable(3042);
    GL11::glBlendFunc(770, 771);
    tessellator.startDrawingQuads();
    tessellator.setColorRGBA_F(f6, f6, f6, f1);
    tessellator.addVertexWithUV((f3 - f2) as f64, f4 as f64, (f5 + f2) as f64, 0.0, 1.0);
    tessellator.addVertexWithUV((f3 + f2) as f64, f4 as f64, (f5 + f2) as f64, 1.0, 1.0);
    tessellator.addVertexWithUV((f3 + f2) as f64, f4 as f64, (f5 - f2) as f64, 1.0, 0.0);
    tessellator.addVertexWithUV((f3 - f2) as f64, f4 as f64, (f5 - f2) as f64, 0.0, 0.0);
    tessellator.draw();
    GL11::glDisable(3042);
    GL11::glEnable(2896);
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityFootStepFX",
    m_parent: Some(&entity_fx::CLASS),
    renderParticle,
    onUpdate: |this| {
        let step: &FootStepFields = extra::<FootStepFields>(this);
        step.m_age.set(step.m_age.get() + 1);
        if step.m_age.get() == step.m_maxAge.get()
        {
            this.setEntityDead();
        }
    },
    getFXLayer: |_| 3,
    ..entity_fx::CLASS
};

pub fn new(textureManager: &Rc<RenderEngine>, world: &Rc<World>, x: f64, y: f64, z: f64) -> Rc<Entity>
{
    let step: FootStepFields =
        FootStepFields { m_age: Cell::new(0), m_maxAge: Cell::new(0), m_textureManager: Rc::clone(textureManager) };
    let this: Rc<Entity> = entity_fx::construct(&CLASS, Box::new(step), world, x, y, z, 0.0, 0.0, 0.0);
    this.m_motionX.set(0.0);
    this.m_motionY.set(0.0);
    this.m_motionZ.set(0.0);
    extra::<FootStepFields>(&this).m_maxAge.set(200);
    this
}
