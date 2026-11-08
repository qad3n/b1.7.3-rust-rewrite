use crate::model::{ModelBase, ModelBaseFields, ModelRenderer};
use crate::render::RenderManager;
use crate::render::entity::{Render, RenderClass, render_living};
use game::entity::{Entity, entity_living};
use lwjgl::opengl::GL11;
use modloader::mod_protocol::abi::{ModelOp, Pose, PoseInput};
use modloader::mod_protocol::spec::{EntityModelSpec, ModelPart};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const RADIANS_TO_DEGREES: f32 = 57.295776;
// RenderLiving's one sixteenth model unit
const MODEL_SCALE: f32 = 0.0625;

struct Part
{
    m_spec: ModelPart,
    m_boxes: Vec<ModelRenderer>,
    m_children: Vec<usize>,
    m_root: bool,
    m_rotation: Cell<[f32; 3]>,
    m_pivot: Cell<[f32; 3]>,
}

// a mod entity's ModelBase built from its spec and driven by the draw program its pose callback returns
pub struct ModModel
{
    m_base: ModelBaseFields,
    m_parts: Vec<Part>,
    m_pose: RefCell<Pose>,
    // later draws in one frame are the hurt and color overlays, which keep their own color and texture
    m_draws: Cell<u32>,
}

impl ModModel
{
    pub fn new(spec: &EntityModelSpec) -> ModModel
    {
        let mut parts: Vec<Part> = spec
            .parts
            .iter()
            .map(|part| {
                let [width, height] = part.texture_size.unwrap_or(spec.texture_size);
                let boxes: Vec<ModelRenderer> = part
                    .boxes
                    .iter()
                    .map(|cuboid| {
                        let [u, v] = cuboid.uv.unwrap_or(part.uv);
                        let mut renderer: ModelRenderer = ModelRenderer::sized(u, v, width, height, part.mirror);
                        let [x, y, z] = cuboid.origin;
                        let [sizeX, sizeY, sizeZ] = cuboid.size;
                        renderer.addBox(x, y, z, sizeX, sizeY, sizeZ, cuboid.inflate);
                        renderer
                    })
                    .collect();
                Part {
                    m_spec: part.clone(),
                    m_boxes: boxes,
                    m_children: Vec::new(),
                    m_root: part.parent.is_none(),
                    m_rotation: Cell::new(part.rotation),
                    m_pivot: Cell::new(part.pivot),
                }
            })
            .collect();
        for index in 0..parts.len()
        {
            let parent: Option<usize> = parts[index]
                .m_spec
                .parent
                .as_ref()
                .and_then(|name| parts.iter().position(|part| part.m_spec.name == *name));
            if let Some(parent) = parent
            {
                parts[parent].m_children.push(index);
            }
        }
        ModModel {
            m_base: ModelBaseFields::default(),
            m_parts: parts,
            m_pose: RefCell::new(Pose::default()),
            m_draws: Cell::new(0),
        }
    }

    fn drawPart(&self, index: usize, scale: f32)
    {
        let part: &Part = &self.m_parts[index];
        let [rotateX, rotateY, rotateZ] = part.m_rotation.get();
        let [pivotX, pivotY, pivotZ] = part.m_pivot.get();
        for renderer in &part.m_boxes
        {
            renderer.setRotationPoint(pivotX, pivotY, pivotZ);
            renderer.m_rotateAngleX.set(rotateX);
            renderer.m_rotateAngleY.set(rotateY);
            renderer.m_rotateAngleZ.set(rotateZ);
            renderer.render(scale);
        }
        if part.m_children.is_empty()
        {
            return;
        }
        GL11::glPushMatrix();
        GL11::glTranslatef(pivotX * scale, pivotY * scale, pivotZ * scale);
        GL11::glRotatef(rotateZ * RADIANS_TO_DEGREES, 0.0, 0.0, 1.0);
        GL11::glRotatef(rotateY * RADIANS_TO_DEGREES, 0.0, 1.0, 0.0);
        GL11::glRotatef(rotateX * RADIANS_TO_DEGREES, 1.0, 0.0, 0.0);
        for child in &part.m_children
        {
            self.drawPart(*child, scale);
        }
        GL11::glPopMatrix();
    }

    // a posed root draws now, a posed child only takes its pose and draws with its parent
    fn place(&self, index: usize, rotation: [f32; 3], pivot: [f32; 3], scale: f32)
    {
        let Some(part) = self.m_parts.get(index)
        else
        {
            return;
        };
        part.m_rotation.set(rotation);
        part.m_pivot.set(pivot);
        if part.m_root
        {
            self.drawPart(index, scale);
        }
    }

    fn run(&self, ops: &[ModelOp], scale: f32, overlay: bool)
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
                ModelOp::Color([red, green, blue, alpha]) if !overlay => GL11::glColor4f(*red, *green, *blue, *alpha),
                ModelOp::Texture(texture) if !overlay => bindTexture(texture),
                ModelOp::Blend(on) if !overlay => blend(*on),
                ModelOp::Part { part, rotation, pivot } => self.place(*part as usize, *rotation, *pivot, scale),
                ModelOp::FaceCamera =>
                {
                    let manager: Rc<RenderManager> = RenderManager::instance();
                    GL11::glRotatef(180.0 - manager.m_playerViewY.get(), 0.0, 1.0, 0.0);
                    GL11::glRotatef(-manager.m_playerViewX.get(), 1.0, 0.0, 0.0);
                }
                ModelOp::Quads(vertices) => drawQuads(vertices),
                ModelOp::Normal([x, y, z]) => GL11::glNormal3f(*x, *y, *z),
                ModelOp::Lighting(on) if !overlay =>
                {
                    if *on
                    {
                        GL11::glEnable(GL11::GL_LIGHTING);
                    }
                    else
                    {
                        GL11::glDisable(GL11::GL_LIGHTING);
                    }
                }
                ModelOp::Rest(part) =>
                {
                    if let Some(rest) = self.m_parts.get(*part as usize)
                    {
                        self.place(*part as usize, rest.m_spec.rotation, rest.m_spec.pivot, scale);
                    }
                }
                _ =>
                {}
            }
        }
    }

    fn resetPose(&self)
    {
        for part in &self.m_parts
        {
            part.m_rotation.set(part.m_spec.rotation);
            part.m_pivot.set(part.m_spec.pivot);
        }
    }
}

pub(crate) fn drawQuads(vertices: &[[f32; 5]])
{
    let tessellator: Rc<crate::render::Tessellator> = crate::render::Tessellator::instance();
    tessellator.startDrawingQuads();
    for [x, y, z, u, v] in vertices
    {
        tessellator.addVertexWithUV(*x as f64, *y as f64, *z as f64, *u as f64, *v as f64);
    }
    tessellator.draw();
}

fn bindTexture(texture: &str)
{
    let Some(path) = modloader::entities::modTexturePath(texture)
    else
    {
        return;
    };
    if let Some(engine) = RenderManager::instance().m_renderEngine.get()
    {
        engine.bindTexture(engine.getTexture(&path));
    }
}

fn blend(on: bool)
{
    if on
    {
        GL11::glEnable(GL11::GL_BLEND);
        GL11::glBlendFunc(GL11::GL_SRC_ALPHA, GL11::GL_ONE_MINUS_SRC_ALPHA);
    }
    else
    {
        GL11::glDisable(GL11::GL_BLEND);
    }
}

impl ModelBase for ModModel
{
    fn base(&self) -> &ModelBaseFields { &self.m_base }

    fn render(&self, _walk: f32, _walkSpeed: f32, _bob: f32, _yaw: f32, _pitch: f32, scale: f32)
    {
        let draws: u32 = self.m_draws.get();
        self.m_draws.set(draws + 1);
        let pose = self.m_pose.borrow();
        if pose.model.is_empty()
        {
            self.resetPose();
            for index in (0..self.m_parts.len()).filter(|index| self.m_parts[*index].m_root)
            {
                self.drawPart(index, scale);
            }
            return;
        }
        self.run(&pose.model, scale, draws > 0);
    }
}

fn lerp(previous: f32, current: f32, tickDelta: f32) -> f32 { previous + (current - previous) * tickDelta }

// RenderLiving.renderBody works these out before it calls the model, the pose callback needs them earlier
fn poseInput(entity: &Rc<Entity>, tickDelta: f32) -> PoseInput
{
    let fields: &entity_living::LivingFields = entity_living::fields(entity);
    let bodyYaw: f32 = lerp(fields.m_prevRenderYawOffset.get(), fields.m_renderYawOffset.get(), tickDelta);
    let headYaw: f32 = lerp(entity.m_prevRotationYaw.get(), entity.m_rotationYaw.get(), tickDelta);
    let walkSpeed: f32 =
        lerp(fields.m_lastWalkAnimationSpeed.get(), fields.m_walkAnimationSpeed.get(), tickDelta).min(1.0);
    let isLocal = |other: Rc<Entity>| {
        RenderManager::instance().m_livingPlayer.get().is_some_and(|local| Rc::ptr_eq(&other, &local))
    };
    let localRider: bool = entity.m_riddenByEntity.get().is_some_and(isLocal);
    let ridingLocal: bool = entity.m_ridingEntity.get().is_some_and(isLocal);
    PoseInput {
        entity: modloader::host::context::entityInfo(entity),
        walk: fields.m_walkAnimationProgress.get() - fields.m_walkAnimationSpeed.get() * (1.0 - tickDelta),
        walk_speed: walkSpeed,
        age: entity.m_ticksExisted.get() as f32 + tickDelta,
        head_yaw: headYaw - bodyYaw,
        head_pitch: lerp(entity.m_prevRotationPitch.get(), entity.m_rotationPitch.get(), tickDelta),
        partial: tickDelta,
        on_ground: entity.m_onGround.get(),
        riding: entity.m_ridingEntity.get().is_some(),
        ridden_by_local: localRider,
        riding_local: ridingLocal,
        rider_size: entity.m_riddenByEntity.get().map_or(0.0, |rider| rider.m_width.get() + rider.m_height.get()),
        hurt_time: fields.m_hurtTime.get(),
        synced: modloader::entities::synced(entity),
    }
}

thread_local! {
    static MODELS: RefCell<Vec<(Rc<Render>, Rc<ModModel>)>> = const { RefCell::new(Vec::new()) };
}

fn modelOf(render: &Render) -> Option<Rc<ModModel>>
{
    MODELS.with(|models| {
        models
            .borrow()
            .iter()
            .find(|(owner, _)| std::ptr::eq(Rc::as_ptr(owner), render))
            .map(|(_, model)| Rc::clone(model))
    })
}

const RENDER_MOD: RenderClass = RenderClass {
    m_name: "RenderModEntity",
    m_parent: Some(&render_living::RENDER_LIVING),
    preRenderCallback: |this, entity, tickDelta| {
        let Some(model) = modelOf(this)
        else
        {
            return;
        };
        let pose: Pose = modloader::entities::pose(entity, poseInput(entity, tickDelta)).unwrap_or_default();
        model.m_draws.set(0);
        model.resetPose();
        model.run(&pose.pre, MODEL_SCALE, false);
        *model.m_pose.borrow_mut() = pose;
    },
    ..render_living::RENDER_LIVING
};

// a projectile has no living fields, its pose input carries where it is and how it flies
fn objectPoseInput(entity: &Rc<Entity>, tickDelta: f32) -> PoseInput
{
    PoseInput {
        entity: modloader::host::context::entityInfo(entity),
        walk: 0.0,
        walk_speed: 0.0,
        age: entity.m_ticksExisted.get() as f32 + tickDelta,
        head_yaw: lerp(entity.m_prevRotationYaw.get(), entity.m_rotationYaw.get(), tickDelta),
        head_pitch: lerp(entity.m_prevRotationPitch.get(), entity.m_rotationPitch.get(), tickDelta),
        partial: tickDelta,
        on_ground: entity.m_onGround.get(),
        riding: entity.m_ridingEntity.get().is_some(),
        ridden_by_local: false,
        riding_local: false,
        rider_size: 0.0,
        hurt_time: 0,
        synced: modloader::entities::synced(entity),
    }
}

const RENDER_MOD_OBJECT: RenderClass = RenderClass {
    m_name: "RenderModObject",
    m_parent: Some(&crate::render::entity::RENDER),
    doRender: |this, entity, dx, dy, dz, _, tickDelta| {
        let Some(model) = modelOf(this)
        else
        {
            return;
        };
        let pose: Pose = modloader::entities::pose(entity, objectPoseInput(entity, tickDelta)).unwrap_or_default();
        GL11::glPushMatrix();
        GL11::glTranslatef(dx as f32, dy as f32, dz as f32);
        GL11::glEnable(lwjgl::opengl::GL12::GL_RESCALE_NORMAL);
        if let Some(texture) = entity.getEntityTexture()
        {
            this.loadTexture(&jvm::string::JString::display(Some(&texture)));
        }
        model.m_draws.set(0);
        model.resetPose();
        model.run(&pose.pre, MODEL_SCALE, false);
        if pose.model.is_empty()
        {
            model.render(0.0, 0.0, 0.0, 0.0, 0.0, MODEL_SCALE);
        }
        else
        {
            model.run(&pose.model, MODEL_SCALE, false);
        }
        GL11::glDisable(lwjgl::opengl::GL12::GL_RESCALE_NORMAL);
        GL11::glPopMatrix();
    },
    ..crate::render::entity::RENDER
};

// the renderer of a mod entity class whose spec carries a model, built once per class
pub fn rendererFor(className: &str) -> Option<Rc<Render>>
{
    let spec: EntityModelSpec = modloader::entities::modelByClass(className)?;
    let model: Rc<ModModel> = Rc::new(ModModel::new(&spec));
    let render: Rc<Render> = if modloader::entities::isLivingClass(className)
    {
        render_living::create(&RENDER_MOD, Rc::clone(&model) as Rc<dyn ModelBase>, spec.shadow, Box::new(()))
    }
    else
    {
        let render: Rc<Render> = Render::alloc(&RENDER_MOD_OBJECT, None, Box::new(()));
        render.m_shadowSize.set(spec.shadow);
        render
    };
    MODELS.with(|models| models.borrow_mut().push((Rc::clone(&render), model)));
    Some(render)
}
