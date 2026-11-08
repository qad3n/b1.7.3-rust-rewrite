use std::cell::Cell;

#[derive(Default)]
pub struct MouseFilter
{
    m_actualSum: Cell<f32>,
    m_smoothedSum: Cell<f32>,
    m_movementLatency: Cell<f32>,
}

impl MouseFilter
{
    pub fn new() -> MouseFilter { MouseFilter::default() }

    pub fn smooth(&self, mut original: f32, smoother: f32) -> f32
    {
        self.m_actualSum.set(self.m_actualSum.get() + original);
        original = (self.m_actualSum.get() - self.m_smoothedSum.get()) * smoother;
        self.m_movementLatency.set(self.m_movementLatency.get() + (original - self.m_movementLatency.get()) * 0.5);
        let latency: f32 = self.m_movementLatency.get();
        if original > 0.0 && original > latency || original < 0.0 && original < latency
        {
            original = latency;
        }
        self.m_smoothedSum.set(self.m_smoothedSum.get() + original);
        original
    }
}
