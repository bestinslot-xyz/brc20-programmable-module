//! Per-client rate limiting of the public transaction pool methods.
//!
//! [`ClientKeyExtractor`] runs as HTTP middleware and tags each request with the [`ClientKey`]
//! of its caller. [`RpcRateLimitMiddleware`] reads that key and counts calls to
//! [`RATE_LIMITED_METHODS`] in fixed windows, with a separate counter per client and method.

use std::collections::HashMap;
use std::future::Future;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use hyper::http::{Extensions, HeaderMap};
use jsonrpsee::core::middleware::{Batch, BatchEntry, BatchEntryErr, Notification, ResponseFuture};
use jsonrpsee::server::middleware::rpc::RpcServiceT;
use jsonrpsee::types::{ErrorObject, Request};
use jsonrpsee::MethodResponse;
use tower_http::validate_request::ValidateRequest;

/// Calls one client may make to each rate limited method per [`SEND_RAW_TX_WINDOW`].
pub const SEND_RAW_TX_LIMIT: u32 = 30;
/// Length of a rate limit window.
pub const SEND_RAW_TX_WINDOW: Duration = Duration::from_secs(60);
/// Methods counted against the rate limit, each with its own counter.
pub const RATE_LIMITED_METHODS: [&str; 2] =
    ["eth_sendRawTransaction", "brc20_unsentRawTransactions"];

/// JSON-RPC error code of a rate limited call.
const RATE_LIMITED_CODE: i32 = -32005;
const RATE_LIMITED_MESSAGE: &str = "Rate limit exceeded";
/// Number of windows the limiter tracks before it drops stale ones.
const MAX_TRACKED_WINDOWS: usize = 100_000;
/// Shortest time between two scans for stale windows, which bounds the cost of a full map.
const PRUNE_INTERVAL: Duration = Duration::from_secs(1);
/// Longest client key kept from an `X-Forwarded-For` value that is not an IP address.
const MAX_CLIENT_KEY_LEN: usize = 64;

/// Identifies the caller a rate limit window belongs to.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ClientKey {
    /// The first `X-Forwarded-For` hop, or its /64 prefix for an IPv6 address.
    Forwarded(String),
    /// Every caller without a trusted forwarded address.
    Shared,
}

/// HTTP middleware that inserts the caller's [`ClientKey`] into the request extensions.
///
/// With `trust_forwarded_for`, the first comma-separated `X-Forwarded-For` value is the key,
/// and an IPv6 address is keyed by its /64 prefix. A port is ignored, and an IPv4-mapped IPv6
/// address is keyed by its IPv4 address. Otherwise every request gets
/// [`ClientKey::Shared`]. It never rejects a request.
#[derive(Clone)]
pub struct ClientKeyExtractor {
    trust_forwarded_for: bool,
}

impl ClientKeyExtractor {
    pub fn new(trust_forwarded_for: bool) -> Self {
        Self {
            trust_forwarded_for,
        }
    }

    fn client_key(&self, headers: &HeaderMap) -> ClientKey {
        if !self.trust_forwarded_for {
            return ClientKey::Shared;
        }
        // `to_str` accepts visible ASCII only, so truncating by bytes keeps a valid string
        headers
            .get("X-Forwarded-For")
            .and_then(|header| header.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|first| !first.is_empty())
            .map(|first| {
                // A value with a port keys by its address, so the client keeps one key across
                // connections
                let address = first
                    .parse::<SocketAddr>()
                    .map(|socket| socket.ip())
                    .or_else(|_| first.parse::<IpAddr>());
                match address {
                    Ok(IpAddr::V4(v4)) => ClientKey::Forwarded(v4.to_string()),
                    Ok(IpAddr::V6(v6)) => match v6.to_ipv4_mapped() {
                        Some(v4) => ClientKey::Forwarded(v4.to_string()),
                        // One IPv6 subscriber usually holds a whole /64, so the prefix is the
                        // client
                        None => {
                            let prefix = Ipv6Addr::from(u128::from(v6) & !u128::from(u64::MAX));
                            ClientKey::Forwarded(format!("{prefix}/64"))
                        }
                    },
                    Err(_) => ClientKey::Forwarded(
                        first[..first.len().min(MAX_CLIENT_KEY_LEN)].to_string(),
                    ),
                }
            })
            .unwrap_or(ClientKey::Shared)
    }
}

impl<B> ValidateRequest<B> for ClientKeyExtractor {
    type ResponseBody = B;

    fn validate(
        &mut self,
        request: &mut hyper::Request<B>,
    ) -> Result<(), hyper::Response<Self::ResponseBody>> {
        let key = self.client_key(request.headers());
        request.extensions_mut().insert(key);
        Ok(())
    }
}

struct Windows {
    /// Start and call count of each window, keyed by client and method.
    counts: HashMap<(ClientKey, &'static str), (Instant, u32)>,
    /// Earliest time a full map scans for stale windows again.
    next_prune: Instant,
}

/// Fixed window call counter, per client and method.
pub struct RateLimiter {
    limit: u32,
    window: Duration,
    windows: Mutex<Windows>,
}

impl RateLimiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        Self {
            limit,
            window,
            windows: Mutex::new(Windows {
                counts: HashMap::new(),
                next_prune: Instant::now(),
            }),
        }
    }

    /// Counts a call by `key` to `method` at `now`, and returns whether it is within the limit.
    ///
    /// When the map holds [`MAX_TRACKED_WINDOWS`] windows, a new key first drops the stale
    /// windows, at most once per [`PRUNE_INTERVAL`]. If the map is still full, the call counts
    /// against [`ClientKey::Shared`], so the map stays bounded.
    fn allow(&self, key: &ClientKey, method: &'static str, now: Instant) -> bool {
        // A poisoned lock is recovered, since the counters hold no invariant across entries
        let mut windows = self
            .windows
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut map_key = (key.clone(), method);
        if windows.counts.len() >= MAX_TRACKED_WINDOWS && !windows.counts.contains_key(&map_key) {
            if now >= windows.next_prune {
                let window = self.window;
                windows
                    .counts
                    .retain(|_, (start, _)| now.saturating_duration_since(*start) < window);
                let oldest = windows.counts.values().map(|(start, _)| *start).min();
                windows.next_prune = oldest
                    .map_or(now, |oldest| oldest + window)
                    .max(now + PRUNE_INTERVAL);
            }
            if windows.counts.len() >= MAX_TRACKED_WINDOWS {
                map_key = (ClientKey::Shared, method);
            }
        }

        let (start, count) = windows.counts.entry(map_key).or_insert((now, 0));
        if now.saturating_duration_since(*start) >= self.window {
            *start = now;
            *count = 0;
        }
        if *count >= self.limit {
            return false;
        }
        *count += 1;
        true
    }

    fn allow_call(&self, method_name: &str, extensions: &Extensions) -> bool {
        let Some(method) = RATE_LIMITED_METHODS
            .iter()
            .copied()
            .find(|method| *method == method_name)
        else {
            return true;
        };
        let key = extensions.get::<ClientKey>().unwrap_or(&ClientKey::Shared);
        self.allow(key, method, Instant::now())
    }
}

fn rate_limited_error() -> ErrorObject<'static> {
    ErrorObject::borrowed(RATE_LIMITED_CODE, RATE_LIMITED_MESSAGE, None)
}

/// RPC middleware that rejects calls to [`RATE_LIMITED_METHODS`] beyond the client's limit
/// with error `-32005`. It reads the [`ClientKey`] that [`ClientKeyExtractor`] inserts, and
/// counts each call in a batch as one call. Notifications pass through uncounted. Without a
/// limiter it passes every call through.
#[derive(Clone)]
pub struct RpcRateLimitMiddleware<S> {
    service: S,
    limiter: Option<Arc<RateLimiter>>,
}

impl<S> RpcRateLimitMiddleware<S> {
    pub fn new(service: S, limiter: Option<Arc<RateLimiter>>) -> Self {
        Self { service, limiter }
    }

    fn allow_call(&self, method_name: &str, extensions: &Extensions) -> bool {
        self.limiter
            .as_ref()
            .map_or(true, |limiter| limiter.allow_call(method_name, extensions))
    }
}

impl<S> RpcServiceT for RpcRateLimitMiddleware<S>
where
    S: RpcServiceT<
        MethodResponse = MethodResponse,
        BatchResponse = MethodResponse,
        NotificationResponse = MethodResponse,
    >,
{
    type MethodResponse = S::MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn call<'a>(&self, req: Request<'a>) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        if self.allow_call(req.method_name(), req.extensions()) {
            ResponseFuture::future(self.service.call(req))
        } else {
            ResponseFuture::ready(MethodResponse::error(req.id(), rate_limited_error()))
        }
    }

    fn notification<'a>(
        &self,
        notification: Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        // The server runs no method for a notification, so it costs no budget
        self.service.notification(notification)
    }

    fn batch<'a>(
        &self,
        mut batch: Batch<'a>,
    ) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        // Notifications pass through uncounted, as in `notification`
        for entry in batch.iter_mut() {
            if let Ok(BatchEntry::Call(req)) = entry {
                if !self.allow_call(req.method_name(), req.extensions()) {
                    *entry = Err(BatchEntryErr::new(req.id(), rate_limited_error()));
                }
            }
        }
        self.service.batch(batch)
    }
}

#[cfg(test)]
mod tests {
    use hyper::http::HeaderValue;
    use jsonrpsee::types::Id;

    use super::*;

    const SEND: &str = RATE_LIMITED_METHODS[0];
    const UNSENT: &str = RATE_LIMITED_METHODS[1];

    fn forwarded(address: &str) -> ClientKey {
        ClientKey::Forwarded(address.to_string())
    }

    fn headers(forwarded_for: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Forwarded-For",
            HeaderValue::from_str(forwarded_for).unwrap(),
        );
        headers
    }

    #[test]
    fn client_key_is_trimmed_first_hop_when_trusted() {
        let extractor = ClientKeyExtractor::new(true);
        assert_eq!(
            extractor.client_key(&headers(" 1.1.1.1 , 10.0.0.1")),
            forwarded("1.1.1.1")
        );
        assert_eq!(
            extractor.client_key(&headers(" , 10.0.0.1")),
            ClientKey::Shared
        );
        assert_eq!(extractor.client_key(&HeaderMap::new()), ClientKey::Shared);

        let long = "a".repeat(200);
        assert_eq!(
            extractor.client_key(&headers(&long)),
            forwarded(&long[..MAX_CLIENT_KEY_LEN])
        );
    }

    #[test]
    fn ipv4_mapped_address_keys_by_its_ipv4() {
        let extractor = ClientKeyExtractor::new(true);
        let first = extractor.client_key(&headers("::ffff:1.2.3.4"));
        let second = extractor.client_key(&headers("::ffff:5.6.7.8"));
        assert_eq!(first, forwarded("1.2.3.4"));
        assert_ne!(first, second);
        assert_eq!(first, extractor.client_key(&headers("1.2.3.4")));
    }

    #[test]
    fn trailing_port_is_ignored() {
        let extractor = ClientKeyExtractor::new(true);
        assert_eq!(
            extractor.client_key(&headers("1.2.3.4:5678")),
            forwarded("1.2.3.4")
        );
        assert_eq!(
            extractor.client_key(&headers("[2001:db8::1]:443")),
            extractor.client_key(&headers("2001:db8::1"))
        );
        assert_eq!(
            extractor.client_key(&headers("[2001:db8::1]:443")),
            forwarded("2001:db8::/64")
        );
        assert_eq!(
            extractor.client_key(&headers("[::ffff:1.2.3.4]:80")),
            forwarded("1.2.3.4")
        );
    }

    #[test]
    fn client_key_is_shared_when_untrusted() {
        let extractor = ClientKeyExtractor::new(false);
        assert_eq!(extractor.client_key(&headers("1.1.1.1")), ClientKey::Shared);
    }

    #[test]
    fn window_resets_after_it_elapses() {
        let limiter = RateLimiter::new(2, SEND_RAW_TX_WINDOW);
        let key = forwarded("1.1.1.1");
        let start = Instant::now();

        assert!(limiter.allow(&key, SEND, start));
        assert!(limiter.allow(&key, SEND, start));
        assert!(!limiter.allow(&key, SEND, start + SEND_RAW_TX_WINDOW / 2));
        assert!(limiter.allow(&key, UNSENT, start));
        assert!(limiter.allow(&key, SEND, start + SEND_RAW_TX_WINDOW));
    }

    #[test]
    fn full_map_drops_stale_windows() {
        let limiter = RateLimiter::new(1, SEND_RAW_TX_WINDOW);
        let start = Instant::now();
        for n in 0..MAX_TRACKED_WINDOWS {
            assert!(limiter.allow(&forwarded(&n.to_string()), SEND, start));
        }

        let later = start + SEND_RAW_TX_WINDOW;
        assert!(limiter.allow(&forwarded("new"), SEND, later));
        let windows = limiter.windows.lock().unwrap();
        assert_eq!(windows.counts.len(), 1);
    }

    #[test]
    fn full_map_of_live_windows_falls_back_to_shared_key() {
        let limiter = RateLimiter::new(1, SEND_RAW_TX_WINDOW);
        let start = Instant::now();
        for n in 0..MAX_TRACKED_WINDOWS {
            assert!(limiter.allow(&forwarded(&n.to_string()), SEND, start));
        }

        assert!(limiter.allow(&forwarded("new"), SEND, start));
        assert!(!limiter.allow(&forwarded("other"), SEND, start));
        // A tracked client keeps its own window
        assert!(!limiter.allow(&forwarded("0"), SEND, start));
        let windows = limiter.windows.lock().unwrap();
        assert_eq!(windows.counts.len(), MAX_TRACKED_WINDOWS + 1);
        assert!(windows.counts.contains_key(&(ClientKey::Shared, SEND)));
    }

    #[test]
    fn prune_waits_at_least_the_prune_interval() {
        let limiter = RateLimiter::new(1, SEND_RAW_TX_WINDOW);
        let start = Instant::now();
        let step = Duration::from_micros(10);
        for n in 0..MAX_TRACKED_WINDOWS {
            let at = start + step * n as u32;
            assert!(limiter.allow(&forwarded(&n.to_string()), SEND, at));
        }

        // Only the first window is stale here, so the oldest survivor expires 5 µs later
        let first_call = start + SEND_RAW_TX_WINDOW + step / 2;
        assert!(limiter.allow(&forwarded("new"), SEND, first_call));
        let next_prune = limiter.windows.lock().unwrap().next_prune;
        assert!(next_prune >= first_call + PRUNE_INTERVAL);

        let second_call = first_call + Duration::from_millis(1);
        limiter.allow(&forwarded("newer"), SEND, second_call);
        let windows = limiter.windows.lock().unwrap();
        assert_eq!(windows.next_prune, next_prune);
        // No second scan ran, so the windows that went stale in that 1 ms are still tracked
        assert!(windows.counts.contains_key(&(forwarded("1"), SEND)));
    }

    #[test]
    fn client_key_groups_ipv6_by_64_prefix() {
        let extractor = ClientKeyExtractor::new(true);
        let first = extractor.client_key(&headers("2001:db8:1:2::1"));
        let same_prefix = extractor.client_key(&headers("2001:db8:1:2:ffff:ffff:ffff:5"));
        let other_prefix = extractor.client_key(&headers("2001:db8:1:3::1"));

        assert_eq!(first, forwarded("2001:db8:1:2::/64"));
        assert_eq!(first, same_prefix);
        assert_ne!(first, other_prefix);
        assert_eq!(
            extractor.client_key(&headers("1.2.3.4")),
            forwarded("1.2.3.4")
        );
        assert_eq!(
            extractor.client_key(&headers("not-an-ip")),
            forwarded("not-an-ip")
        );
    }

    /// Answers every call with success, and a batch with success only when no entry is an error.
    struct MockRpcService;

    impl RpcServiceT for MockRpcService {
        type MethodResponse = MethodResponse;
        type NotificationResponse = MethodResponse;
        type BatchResponse = MethodResponse;

        fn call<'a>(
            &self,
            req: Request<'a>,
        ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
            std::future::ready(MethodResponse::response(
                req.id(),
                jsonrpsee::ResponsePayload::success(true),
                usize::MAX,
            ))
        }

        fn batch<'a>(
            &self,
            batch: Batch<'a>,
        ) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
            let response = if batch.iter().all(|entry| entry.is_ok()) {
                MethodResponse::response(
                    Id::Number(1),
                    jsonrpsee::ResponsePayload::success(true),
                    usize::MAX,
                )
            } else {
                MethodResponse::error(Id::Number(1), rate_limited_error())
            };
            std::future::ready(response)
        }

        fn notification<'a>(
            &self,
            _: Notification<'a>,
        ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
            std::future::ready(MethodResponse::notification())
        }
    }

    #[tokio::test]
    async fn notifications_pass_through_uncounted() {
        let limiter = Arc::new(RateLimiter::new(1, SEND_RAW_TX_WINDOW));
        let middleware = RpcRateLimitMiddleware::new(MockRpcService, Some(limiter.clone()));
        let notification = || Notification::new(SEND.into(), None);

        for _ in 0..3 {
            middleware.notification(notification()).await;
        }
        let entries = (0..3)
            .map(|_| Ok(BatchEntry::Notification(notification())))
            .collect();
        let response = middleware.batch(Batch::from(entries)).await;

        assert!(response.is_success());
        assert!(limiter.windows.lock().unwrap().counts.is_empty());
    }
}
