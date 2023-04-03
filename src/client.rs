use governor::{
    clock::MonotonicClock,
    state::{direct::NotKeyed, InMemoryState},
    Quota, RateLimiter,
};
use mangadex_api::v5::MangaDexClient;
use std::{num::NonZeroU32, sync::Arc};

/// A rate-limited MangaDex client.
#[derive(Clone)]
pub struct Client {
    client: MangaDexClient,
    http_client: reqwest::Client,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, MonotonicClock>>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            client: MangaDexClient::default(),
            http_client: reqwest::Client::new(),
            // See https://api.mangadex.org/docs/rate-limits/
            limiter: Arc::new(RateLimiter::direct(Quota::per_second(
                NonZeroU32::new(5).expect("quota"),
            ))),
        }
    }
}

impl Client {
    pub async fn get(&self) -> &MangaDexClient {
        self.limiter.until_ready().await;
        &self.client
    }

    pub async fn http_client(&self) -> reqwest::Client {
        self.limiter.until_ready().await;
        self.http_client.clone()
    }
}
