use std::cell::Cell;

pub struct Timer
{
    pub m_ticksPerSecond: Cell<f32>,
    m_lastHRTime: Cell<f64>,
    pub m_elapsedTicks: Cell<i32>,
    pub m_renderPartialTicks: Cell<f32>,
    pub m_timerSpeed: Cell<f32>,
    pub m_elapsedPartialTicks: Cell<f32>,
    m_lastSyncSysClock: Cell<i64>,
    m_lastSyncHRClock: Cell<i64>,
    m_cumTickTime: Cell<i64>,
    m_timeSyncAdjustment: Cell<f64>,
}

impl Timer
{
    pub fn new(tps: f32) -> Timer
    {
        Timer {
            m_ticksPerSecond: Cell::new(tps),
            m_lastHRTime: Cell::new(0.0),
            m_elapsedTicks: Cell::new(0),
            m_renderPartialTicks: Cell::new(0.0),
            m_timerSpeed: Cell::new(1.0),
            m_elapsedPartialTicks: Cell::new(0.0),
            m_lastSyncSysClock: Cell::new(jvm::system::currentTimeMillis()),
            m_lastSyncHRClock: Cell::new(jvm::system::nanoTime() / 1000000),
            m_cumTickTime: Cell::new(0),
            m_timeSyncAdjustment: Cell::new(1.0),
        }
    }

    pub fn updateTimer(&self)
    {
        let l: i64 = jvm::system::currentTimeMillis();
        let l2: i64 = l.wrapping_sub(self.m_lastSyncSysClock.get());
        let l3: i64 = jvm::system::nanoTime() / 1000000;
        let d: f64 = l3 as f64 / 1000.0;
        if !(0..=1000).contains(&l2)
        {
            self.m_lastHRTime.set(d);
        }
        else
        {
            self.m_cumTickTime.set(self.m_cumTickTime.get() + l2);
            if self.m_cumTickTime.get() > 1000
            {
                let l4: i64 = l3 - self.m_lastSyncHRClock.get();
                let d2: f64 = self.m_cumTickTime.get() as f64 / l4 as f64;
                let adjustment: f64 = self.m_timeSyncAdjustment.get();
                self.m_timeSyncAdjustment.set(adjustment + (d2 - adjustment) * 0.2f32 as f64);
                self.m_lastSyncHRClock.set(l3);
                self.m_cumTickTime.set(0);
            }
            if self.m_cumTickTime.get() < 0
            {
                self.m_lastSyncHRClock.set(l3);
            }
        }
        self.m_lastSyncSysClock.set(l);
        let d3: f64 = ((d - self.m_lastHRTime.get()) * self.m_timeSyncAdjustment.get()).clamp(0.0, 1.0);
        self.m_lastHRTime.set(d);
        self.m_elapsedPartialTicks.set(
            (self.m_elapsedPartialTicks.get() as f64
                + d3 * self.m_timerSpeed.get() as f64 * self.m_ticksPerSecond.get() as f64) as f32,
        );
        self.m_elapsedTicks.set(self.m_elapsedPartialTicks.get() as i32);
        self.m_elapsedPartialTicks.set(self.m_elapsedPartialTicks.get() - self.m_elapsedTicks.get() as f32);
        if self.m_elapsedTicks.get() > 10
        {
            self.m_elapsedTicks.set(10);
        }
        self.m_renderPartialTicks.set(self.m_elapsedPartialTicks.get());
    }
}
