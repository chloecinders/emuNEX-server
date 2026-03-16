use dashmap::DashMap;
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use rocket::{
    Data, Request,
    fairing::{Fairing, Info, Kind},
    http::Status,
};
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

type Limiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

fn make_limiter(per_minute: u32) -> Limiter {
    let quota = Quota::per_minute(NonZeroU32::new(per_minute).unwrap());
    Arc::new(RateLimiter::direct(quota))
}

pub struct RateLimitFairing {
    auth_limiters: Arc<DashMap<IpAddr, Limiter>>,
    api_limiters: Arc<DashMap<IpAddr, Limiter>>,
}

impl RateLimitFairing {
    pub fn new() -> Self {
        RateLimitFairing {
            auth_limiters: Arc::new(DashMap::new()),
            api_limiters: Arc::new(DashMap::new()),
        }
    }

    fn get_ip(req: &Request<'_>) -> Option<IpAddr> {
        req.headers()
            .get_one("X-Forwarded-For")
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse().ok())
            .or_else(|| req.remote().map(|a| a.ip()))
    }
}

fn is_auth_route(path: &str) -> bool {
    path == "/api/v1/login" || path == "/api/v1/register"
}

fn is_api_route(path: &str) -> bool {
    path.starts_with("/api/")
}

#[rocket::async_trait]
impl Fairing for RateLimitFairing {
    fn info(&self) -> Info {
        Info {
            name: "Rate Limiter",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let path = req.uri().path().to_string();
        let ip = match Self::get_ip(req) {
            Some(ip) => ip,
            None => return,
        };

        if is_auth_route(&path) {
            let limiter = self
                .auth_limiters
                .entry(ip)
                .or_insert_with(|| make_limiter(5))
                .clone();
            if limiter.check().is_err() {
                req.local_cache(|| RateLimited(true));
            }
        } else if is_api_route(&path) {
            let limiter = self
                .api_limiters
                .entry(ip)
                .or_insert_with(|| make_limiter(120))
                .clone();
            if limiter.check().is_err() {
                req.local_cache(|| RateLimited(true));
            }
        }
    }
}

#[derive(Clone)]
struct RateLimited(bool);

pub struct RateLimitGuard;

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for RateLimitGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        if req.local_cache(|| RateLimited(false)).0 {
            rocket::request::Outcome::Error((Status::TooManyRequests, ()))
        } else {
            rocket::request::Outcome::Success(RateLimitGuard)
        }
    }
}
