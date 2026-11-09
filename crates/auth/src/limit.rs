use crate::AuthResult;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

mod limits
{
    use std::time::Duration;

    pub const ATTEMPTS: usize = 5;
    pub const ATTEMPT_WINDOW: Duration = Duration::from_secs(60);
    pub const REGISTRATIONS: usize = 3;
    pub const REGISTRATION_WINDOW: Duration = Duration::from_secs(60 * 60);
    pub const CONNECTIONS_PER_IP: u32 = 3;
    pub const PENDING: u32 = 32;
    pub const BACKOFF_START: Duration = Duration::from_secs(1);
    pub const BACKOFF_MAX: Duration = Duration::from_secs(5 * 60);
}

struct Backoff
{
    m_delay: Duration,
    m_until: Instant,
}

// a failed login delays the next try for that account instead of locking it so nobody can lock a player out
#[derive(Default)]
pub struct Limiter
{
    m_attempts: HashMap<IpAddr, Vec<Instant>>,
    m_registrations: HashMap<IpAddr, Vec<Instant>>,
    m_open: HashMap<IpAddr, u32>,
    m_pending: u32,
    m_failures: HashMap<String, Backoff>,
}

impl Limiter
{
    pub fn new() -> Limiter { Limiter::default() }

    pub fn open(&mut self, ip: IpAddr) -> AuthResult<()>
    {
        if self.m_pending >= limits::PENDING
        {
            return Err(String::from("The server is busy logging players in, try again shortly"));
        }

        let open: &mut u32 = self.m_open.entry(ip).or_insert(0);
        if *open >= limits::CONNECTIONS_PER_IP
        {
            return Err(String::from("Too many connections from your address"));
        }

        *open += 1;
        self.m_pending += 1;
        Ok(())
    }

    pub fn close(&mut self, ip: IpAddr)
    {
        if let Some(open) = self.m_open.get_mut(&ip)
        {
            *open -= 1;
            if *open == 0
            {
                self.m_open.remove(&ip);
            }
            self.m_pending -= 1;
        }
    }

    pub fn attempt(&mut self, ip: IpAddr, name: &str, now: Instant) -> AuthResult<()>
    {
        self.m_failures.retain(|_, backoff| backoff.m_until + limits::BACKOFF_MAX > now);
        if let Some(backoff) = self.m_failures.get(name).filter(|backoff| backoff.m_until > now)
        {
            return Err(retry((backoff.m_until - now).as_secs() + 1));
        }

        window(&mut self.m_attempts, ip, now, limits::ATTEMPT_WINDOW, limits::ATTEMPTS)
    }

    pub fn register(&mut self, ip: IpAddr, now: Instant) -> AuthResult<()>
    {
        window(&mut self.m_registrations, ip, now, limits::REGISTRATION_WINDOW, limits::REGISTRATIONS)
    }

    pub fn failed(&mut self, name: &str, now: Instant)
    {
        let delay: Duration = self
            .m_failures
            .get(name)
            .map_or(limits::BACKOFF_START, |backoff| (backoff.m_delay * 2).min(limits::BACKOFF_MAX));
        self.m_failures.insert(name.to_string(), Backoff { m_delay: delay, m_until: now + delay });
    }

    pub fn succeeded(&mut self, name: &str) { self.m_failures.remove(name); }
}

fn retry(secs: u64) -> String { format!("Too many attempts, try again in {secs} seconds") }

fn window(
    entries: &mut HashMap<IpAddr, Vec<Instant>>,
    ip: IpAddr,
    now: Instant,
    span: Duration,
    max: usize,
) -> AuthResult<()>
{
    entries.retain(|_, times| {
        times.retain(|time| *time + span > now);
        !times.is_empty()
    });
    let times: &mut Vec<Instant> = entries.entry(ip).or_default();
    if times.len() >= max
    {
        return Err(retry((times[0] + span - now).as_secs() + 1));
    }

    times.push(now);
    Ok(())
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::net::Ipv4Addr;

    const IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

    #[test]
    fn failed_logins_back_off_and_reset_on_success()
    {
        let mut limiter: Limiter = Limiter::new();
        let start: Instant = Instant::now();
        limiter.failed("notch", start);
        assert!(limiter.attempt(IP, "notch", start).is_err());
        assert!(limiter.attempt(IP, "jeb_", start).is_ok());
        assert!(limiter.attempt(IP, "notch", start + Duration::from_secs(2)).is_ok());

        limiter.failed("notch", start);
        limiter.failed("notch", start);
        assert!(limiter.attempt(IP, "notch", start + Duration::from_secs(3)).is_err());
        limiter.succeeded("notch");
        assert!(limiter.attempt(IP, "notch", start + Duration::from_secs(3)).is_ok());
    }

    #[test]
    fn attempts_per_address_are_capped_per_window()
    {
        let mut limiter: Limiter = Limiter::new();
        let start: Instant = Instant::now();
        for _ in 0..limits::ATTEMPTS
        {
            assert!(limiter.attempt(IP, "notch", start).is_ok());
        }
        assert!(limiter.attempt(IP, "notch", start).is_err());
        assert!(limiter.attempt(IP, "notch", start + limits::ATTEMPT_WINDOW).is_ok());
    }

    #[test]
    fn connections_per_address_are_capped()
    {
        let mut limiter: Limiter = Limiter::new();
        for _ in 0..limits::CONNECTIONS_PER_IP
        {
            assert!(limiter.open(IP).is_ok());
        }
        assert!(limiter.open(IP).is_err());
        limiter.close(IP);
        assert!(limiter.open(IP).is_ok());
    }
}
