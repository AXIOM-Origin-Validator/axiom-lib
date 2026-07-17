//! Per-IP sliding-window rate limiter for AXIOM TCP listeners.
//!
//! Tracks request counts per IP within a configurable time window.
//! Stale entries are pruned periodically to bound memory usage.
//!
//! Single source of truth shared by Lambda (`server.rs` / `admin.rs`)
//! and ANTIE — both previously carried byte-identical copies of this
//! module. Zero AXIOM dependencies, std-only, so it sits at the bottom
//! of the dependency graph.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Instant;

/// Memory bound: reject new IPs once this many are tracked.
/// Prevents DoS via IP-spoofing that grows the HashMap without bound.
const MAX_TRACKED_IPS: usize = 10_000;

/// Per-IP rate limiter with sliding-window counters.
pub struct RateLimiter {
    /// Max requests per window per IP
    max_requests: u32,
    /// Window duration
    window: std::time::Duration,
    /// IP → (count, window_start)
    entries: HashMap<IpAddr, (u32, Instant)>,
    /// Check counter for periodic pruning
    check_count: u32,
}

impl RateLimiter {
    /// Create a new rate limiter.
    /// `max_requests`: max requests per IP per window.
    /// `window_secs`: window duration in seconds.
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window: std::time::Duration::from_secs(window_secs),
            entries: HashMap::new(),
            check_count: 0,
        }
    }

    /// Check if `ip` is allowed. Returns true if under limit, false if rate-limited.
    pub fn check(&mut self, ip: IpAddr) -> bool {
        self.check_count += 1;
        if self.check_count.is_multiple_of(100) {
            self.prune();
        }

        // Memory bound: reject unknown IPs when at capacity
        if !self.entries.contains_key(&ip) && self.entries.len() >= MAX_TRACKED_IPS {
            return false;
        }

        let now = Instant::now();
        let entry = self.entries.entry(ip).or_insert((0, now));

        // Reset window if expired
        if now.duration_since(entry.1) >= self.window {
            entry.0 = 0;
            entry.1 = now;
        }

        if entry.0 >= self.max_requests {
            false
        } else {
            entry.0 += 1;
            true
        }
    }

    /// Remove entries whose window has expired.
    fn prune(&mut self) {
        let now = Instant::now();
        let window = self.window;
        self.entries.retain(|_, (_, start)| now.duration_since(*start) < window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_allows_under_limit() {
        let mut rl = RateLimiter::new(3, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        assert!(rl.check(ip));
        assert!(rl.check(ip));
        assert!(rl.check(ip));
    }

    #[test]
    fn test_blocks_over_limit() {
        let mut rl = RateLimiter::new(2, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        assert!(rl.check(ip));
        assert!(rl.check(ip));
        assert!(!rl.check(ip)); // 3rd request blocked
    }

    #[test]
    fn test_different_ips_independent() {
        let mut rl = RateLimiter::new(1, 60);
        let ip1 = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        let ip2 = IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8));
        assert!(rl.check(ip1));
        assert!(!rl.check(ip1)); // ip1 blocked
        assert!(rl.check(ip2)); // ip2 still ok
    }

    #[test]
    fn test_max_tracked_ips_rejects_new() {
        let mut rl = RateLimiter::new(100, 60);
        // Fill to capacity with distinct IPs
        for i in 0..MAX_TRACKED_IPS as u32 {
            let ip = IpAddr::V4(Ipv4Addr::new(
                ((i >> 16) & 0xFF) as u8,
                ((i >> 8) & 0xFF) as u8,
                (i & 0xFF) as u8,
                1,
            ));
            assert!(rl.check(ip), "IP {} should be allowed", i);
        }
        // New IP rejected at capacity
        let new_ip = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255));
        assert!(!rl.check(new_ip), "new IP should be rejected at MAX_TRACKED_IPS");
        // Existing IP still works
        let existing = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 1));
        assert!(rl.check(existing), "existing IP should still be allowed");
    }

    #[test]
    fn test_prune_removes_stale() {
        let mut rl = RateLimiter::new(10, 0); // 0-second window = always expired
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        for _ in 0..200 {
            assert!(rl.check(ip)); // always allowed since window is 0s
        }
        // After 200 checks (2 prunes), stale entries removed
        assert!(rl.entries.len() <= 1);
    }
}
