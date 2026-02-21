use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use rocket::{Request, State, http::Status};
use rocket::request::{FromRequest, Outcome};

/// A sliding-window rate limiter stored in Rocket managed state.
pub struct RateLimiter {
    /// Map of (ip + endpoint_tag) -> list of request timestamps.
    windows: Mutex<HashMap<String, Vec<Instant>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter { windows: Mutex::new(HashMap::new()) }
    }

    /// Returns `true` if the request should be allowed.
    /// - `key`: identifies the bucket (e.g. "register:1.2.3.4")
    /// - `limit`: max requests allowed within `window`
    /// - `window`: duration of the sliding window
    pub fn check(&self, key: &str, limit: usize, window: Duration) -> bool {
        let now = Instant::now();
        let mut map = self.windows.lock().unwrap();
        let timestamps = map.entry(key.to_string()).or_default();

        // Drop entries outside the window
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= limit {
            return false; // over limit
        }

        timestamps.push(now);
        true
    }
}

/// Extract the best client IP from the request headers.
pub fn client_ip(request: &Request<'_>) -> String {
    // Trust X-Real-IP first (set by nginx/caddy), then X-Forwarded-For, then socket addr
    if let Some(ip) = request.headers().get_one("X-Real-IP") {
        return ip.trim().to_string();
    }
    if let Some(fwd) = request.headers().get_one("X-Forwarded-For") {
        // Take the first (leftmost) IP — the original client
        if let Some(first) = fwd.split(',').next() {
            return first.trim().to_string();
        }
    }
    request.remote()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

// ── Request guards ────────────────────────────────────────────────────────────

/// Request guard: allow max 5 registrations per IP per hour.
pub struct RegisterRateLimit;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RegisterRateLimit {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let rl = match request.guard::<&State<RateLimiter>>().await {
            Outcome::Success(s) => s,
            _ => return Outcome::Success(RegisterRateLimit), // fail open if not set up
        };
        let ip = client_ip(request);
        let key = format!("register:{}", ip);
        if rl.check(&key, 5, Duration::from_secs(3600)) {
            Outcome::Success(RegisterRateLimit)
        } else {
            Outcome::Error((Status::TooManyRequests, ()))
        }
    }
}

/// Request guard: allow max 3 verify attempts per IP per 5 minutes (brute-force protection).
pub struct VerifyRateLimit;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VerifyRateLimit {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let rl = match request.guard::<&State<RateLimiter>>().await {
            Outcome::Success(s) => s,
            _ => return Outcome::Success(VerifyRateLimit),
        };
        let ip = client_ip(request);
        let key = format!("verify:{}", ip);
        if rl.check(&key, 3, Duration::from_secs(300)) {
            Outcome::Success(VerifyRateLimit)
        } else {
            Outcome::Error((Status::TooManyRequests, ()))
        }
    }
}

/// Request guard: allow max 10 challenge requests per IP per minute
/// (prevents challenge farming).
pub struct ChallengeRateLimit;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ChallengeRateLimit {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let rl = match request.guard::<&State<RateLimiter>>().await {
            Outcome::Success(s) => s,
            _ => return Outcome::Success(ChallengeRateLimit),
        };
        let ip = client_ip(request);
        let key = format!("challenge:{}", ip);
        if rl.check(&key, 10, Duration::from_secs(60)) {
            Outcome::Success(ChallengeRateLimit)
        } else {
            Outcome::Error((Status::TooManyRequests, ()))
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_under_limit() {
        let rl = RateLimiter::new();
        for _ in 0..5 {
            assert!(rl.check("test:1.2.3.4", 5, Duration::from_secs(60)));
        }
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let rl = RateLimiter::new();
        for _ in 0..5 {
            rl.check("test:5.6.7.8", 5, Duration::from_secs(60));
        }
        assert!(!rl.check("test:5.6.7.8", 5, Duration::from_secs(60)));
    }

    #[test]
    fn test_rate_limiter_different_keys_independent() {
        let rl = RateLimiter::new();
        for _ in 0..5 {
            rl.check("test:1.1.1.1", 5, Duration::from_secs(60));
        }
        // Different IP should still be allowed
        assert!(rl.check("test:2.2.2.2", 5, Duration::from_secs(60)));
    }

    #[test]
    fn test_rate_limiter_window_expiry() {
        let rl = RateLimiter::new();
        // Use a tiny window — immediately expired
        for _ in 0..5 {
            rl.check("test:expire", 5, Duration::from_nanos(1));
        }
        // After window expires (instant), should allow again
        std::thread::sleep(Duration::from_millis(1));
        assert!(rl.check("test:expire", 5, Duration::from_nanos(1)));
    }
}
