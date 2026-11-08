use crate::state;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

const WINDOW_TICKS: usize = 20;
const CONFIG_FILE: &str = "loader.toml";

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub enum WatchdogAction
{
    #[default]
    Warn,
    Disable,
    Stop,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Watchdog
{
    pub action: WatchdogAction,
    pub tick_budget_ms: f64,
    pub strikes: u32,
}

impl Default for Watchdog
{
    fn default() -> Self { Watchdog { action: WatchdogAction::Warn, tick_budget_ms: 20.0, strikes: 40 } }
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct LoaderConfig
{
    watchdog: Watchdog,
}

#[derive(Default)]
struct Profile
{
    m_window: VecDeque<Duration>,
    m_calls: u64,
    m_strikes: u32,
}

#[derive(Default)]
struct Profiler
{
    m_mods: HashMap<usize, Profile>,
    m_watchdog: Option<Watchdog>,
}

thread_local! {
    static PROFILER: RefCell<Profiler> = RefCell::new(Profiler::default());
}

// config/loader.toml [watchdog] action = "warn" | "disable" | "stop", tick_budget_ms, strikes
fn watchdog() -> Watchdog
{
    if let Some(watchdog) = PROFILER.with(|profiler| profiler.borrow().m_watchdog.clone())
    {
        return watchdog;
    }
    let loaded: Watchdog = crate::paths()
        .and_then(|paths| std::fs::read_to_string(paths.m_config.join(CONFIG_FILE)).ok())
        .and_then(|text| toml::from_str::<LoaderConfig>(&text).ok())
        .map(|config| config.watchdog)
        .unwrap_or_default();
    PROFILER.with(|profiler| profiler.borrow_mut().m_watchdog = Some(loaded.clone()));
    loaded
}

fn act(modIndex: usize, milliseconds: f64, rule: &Watchdog) -> bool
{
    let modId: String = state::modId(modIndex);
    let message: String = format!(
        "used {milliseconds:.1} ms of guest time per tick for {} ticks, budget {} ms",
        rule.strikes, rule.tick_budget_ms
    );
    match rule.action
    {
        WatchdogAction::Warn => crate::log::warn(&modId, &message),
        WatchdogAction::Disable =>
        {
            if let Some(runtime) = state::modAt(modIndex)
            {
                runtime.fail(format!("watchdog: {message}"));
                crate::events::removeMod(modIndex);
            }
        }
        WatchdogAction::Stop =>
        {
            crate::log::error(&modId, &format!("watchdog stops the game: {message}"));
            return true;
        }
    }
    false
}

// once per tick, collects guest time per mod; true asks the server to stop
pub fn tick() -> bool
{
    let rule: Watchdog = watchdog();
    let mut stop: bool = false;
    for runtime in state::mods()
    {
        let Some(instance) = runtime.m_instance.as_ref()
        else
        {
            continue;
        };
        let (time, calls): (Duration, u64) = instance.takeProfile();
        let over: Option<f64> = PROFILER.with(|profiler| {
            let mut profiler = profiler.borrow_mut();
            let profile: &mut Profile = profiler.m_mods.entry(runtime.m_index).or_default();
            profile.m_window.push_back(time);
            profile.m_calls = calls;
            if profile.m_window.len() > WINDOW_TICKS
            {
                profile.m_window.pop_front();
            }
            let milliseconds: f64 = time.as_secs_f64() * 1000.0;
            profile.m_strikes = if milliseconds > rule.tick_budget_ms
            {
                profile.m_strikes + 1
            }
            else
            {
                0
            };
            (profile.m_strikes == rule.strikes).then_some(milliseconds)
        });
        if let Some(milliseconds) = over
        {
            stop |= act(runtime.m_index, milliseconds, &rule);
        }
    }
    stop
}

// lines for the F3 screen, the average guest milliseconds per tick of each mod with code
pub fn lines() -> Vec<String>
{
    let mods = state::mods();
    PROFILER.with(|profiler| {
        let profiler = profiler.borrow();
        mods.iter()
            .filter_map(|runtime| {
                let profile: &Profile = profiler.m_mods.get(&runtime.m_index)?;
                let total: Duration = profile.m_window.iter().sum();
                let average: f64 = total.as_secs_f64() * 1000.0 / profile.m_window.len().max(1) as f64;
                Some(format!("{}: {average:.2} ms/tick, {} calls", runtime.id(), profile.m_calls))
            })
            .collect()
    })
}
