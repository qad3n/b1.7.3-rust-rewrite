use crate::client::Minecraft;
use crate::gui::Gui;
use crate::render::RenderEngine;
use jvm::random::Random;
use jvm::throwable::OrNpe;
use lwjgl::opengl::GL11;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    pub static RANDOM: Random = Random::new();
}

#[derive(Default)]
pub struct Particle
{
    pub m_x: f64,
    pub m_y: f64,
    pub m_lastX: f64,
    pub m_lastY: f64,
    pub m_velocityX: f64,
    pub m_velocityY: f64,
    pub m_velocityChange: f64,
    pub m_removed: bool,
    pub m_age: i32,
    pub m_lifetime: i32,
    pub m_r: f64,
    pub m_g: f64,
    pub m_b: f64,
    pub m_a: f64,
    pub m_lastR: f64,
    pub m_lastG: f64,
    pub m_lastB: f64,
    pub m_lastA: f64,
}

impl Particle
{
    pub fn tickPosition(&mut self, _gui: &GuiParticle)
    {
        self.m_x += self.m_velocityX;
        self.m_y += self.m_velocityY;
        self.m_velocityX *= self.m_velocityChange;
        self.m_velocityY *= self.m_velocityChange;
        self.m_velocityY += 0.1;
        self.m_age += 1;
        if self.m_age > self.m_lifetime
        {
            self.remove();
        }
        self.m_a = 2.0 - self.m_age as f64 / self.m_lifetime as f64 * 2.0;
        if self.m_a > 1.0
        {
            self.m_a = 1.0;
        }
        self.m_a *= self.m_a;
        self.m_a *= 0.5;
    }

    pub fn tickColor(&mut self)
    {
        self.m_lastR = self.m_r;
        self.m_lastG = self.m_g;
        self.m_lastB = self.m_b;
        self.m_lastA = self.m_a;
        self.m_lastX = self.m_x;
        self.m_lastY = self.m_y;
    }

    pub fn remove(&mut self) { self.m_removed = true; }
}

pub struct GuiParticle
{
    pub m_gui: Gui,
    m_particles: RefCell<Vec<Particle>>,
    m_minecraft: Rc<Minecraft>,
}

impl GuiParticle
{
    pub fn new(minecraft: Rc<Minecraft>) -> GuiParticle
    {
        GuiParticle { m_gui: Gui::new(), m_particles: RefCell::new(Vec::new()), m_minecraft: minecraft }
    }

    pub fn tick(&self)
    {
        let mut particles: Vec<Particle> = self.m_particles.take();
        let mut i: usize = 0;
        while i < particles.len()
        {
            let particle: &mut Particle = &mut particles[i];
            particle.tickColor();
            particle.tickPosition(self);
            if particle.m_removed
            {
                particles.remove(i);
                continue;
            }
            i += 1;
        }
        *self.m_particles.borrow_mut() = particles;
    }

    pub fn render(&self, tickDelta: f32)
    {
        let renderEngine: Rc<RenderEngine> = self.m_minecraft.m_renderEngine.get().orNpe();
        renderEngine.bindTexture(renderEngine.getTexture("/gui/particles.png"));
        let tickDelta: f64 = tickDelta as f64;
        for particle in self.m_particles.borrow().iter()
        {
            let j: i32 = (particle.m_lastX + (particle.m_x - particle.m_lastX) * tickDelta - 4.0) as i32;
            let k: i32 = (particle.m_lastY + (particle.m_y - particle.m_lastY) * tickDelta - 4.0) as i32;
            let f: f32 = (particle.m_lastA + (particle.m_a - particle.m_lastA) * tickDelta) as f32;
            let f1: f32 = (particle.m_lastR + (particle.m_r - particle.m_lastR) * tickDelta) as f32;
            let f2: f32 = (particle.m_lastG + (particle.m_g - particle.m_lastG) * tickDelta) as f32;
            let f3: f32 = (particle.m_lastB + (particle.m_b - particle.m_lastB) * tickDelta) as f32;
            GL11::glColor4f(f1, f2, f3, f);
            self.m_gui.drawTexturedModalRect(j, k, 40, 0, 8, 8);
        }
    }
}
