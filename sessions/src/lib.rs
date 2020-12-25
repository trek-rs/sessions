//! Sessions

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

mod config;
mod cookie_options;
mod storage;

pub use config::Config;
pub use cookie_options::CookieOptions;
pub use storage::Storage;

#[derive(Debug)]
pub struct Session<S: Storage, F> {
    pub id: String,
    pub fresh: AtomicBool,
    //store: Arc<S>,
    config: Arc<Config<S, F>>,
}

impl<S: Storage, F> Session<S, F>
where
    F: Fn() -> String,
{
    /// Gets the session id
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Gets the session status
    pub fn fresh(&self) -> bool {
        self.fresh.load(Ordering::Relaxed)
    }

    /// Gets a value by the key
    pub async fn get(&self, key: &str) {}

    /// Sets a value by the key
    pub async fn set<T>(&self, key: &str, value: T) {}

    /// Removes a value
    pub async fn remove(&self, key: &str) {}

    /// Clears the state
    pub async fn clear(&self) {}

    /// Saves the current state to the store
    pub async fn save(&self) {}

    /// Renews the new state
    pub async fn renew(&mut self) {
        //self.id = self.config.generate_id();
    }

    /// Destroys the current state from store
    pub async fn destroy(&self) {}
}
