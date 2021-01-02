use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
    time::Duration,
};

use crate::{
    anyhow,
    data::{from_value, to_value, DeserializeOwned, Serialize},
    Config, Data, Result, Storage,
};

/// Session
#[derive(Clone)]
pub struct Session {
    /// Session's id
    pub id: String,
    /// Session's status, 0: inited, 1: saved, 2: renewed, 3: destroyed
    pub status: Arc<AtomicUsize>,
    /// Session's Data status, false: unchanged, true: changed
    data_status: Arc<AtomicBool>,
    /// Session's Data
    data: Arc<RwLock<Data>>,
    /// Session's Config
    config: Arc<Config>,
}

impl Session {
    /// Creates new `Session` with `id` `status` and `Config`
    pub fn new(id: &str, status: usize, config: Arc<Config>) -> Self {
        Self {
            config,
            id: id.into(),
            data: Arc::new(RwLock::new(Data::new())),
            data_status: Arc::new(AtomicBool::new(false)),
            status: Arc::new(AtomicUsize::new(status)),
        }
    }

    /// Reads the session expires or cookie max_age
    pub fn max_age(&self) -> Duration {
        self.config.max_age()
    }

    /// Reads the session state
    pub fn data(&self) -> Result<RwLockReadGuard<'_, Data>> {
        self.data.read().map_err(|e| anyhow!(e.to_string()))
    }

    /// Writes the session state
    pub fn data_mut(&self) -> Result<RwLockWriteGuard<'_, Data>> {
        self.data.write().map_err(|e| anyhow!(e.to_string()))
    }

    /// Gets the session id
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Gets the session data status
    pub fn data_status(&self) -> bool {
        self.data_status.load(Ordering::Relaxed)
    }

    /// Gets the session status
    pub fn status(&self) -> usize {
        self.status.load(Ordering::Relaxed)
    }

    /// Gets a value by the key
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        from_value(self.data().ok()?.get(key).cloned()?).ok()
    }

    /// Sets a value by the key
    pub fn set<T: DeserializeOwned + Serialize>(&self, key: &str, val: T) -> Option<T> {
        let prev = self
            .data_mut()
            .ok()?
            .insert(key.into(), to_value(val).ok()?);
        self.data_status.store(true, Ordering::SeqCst);
        from_value(prev?).ok()
    }

    /// Removes a value
    pub fn remove<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let prev = self.data_mut().ok()?.remove(key)?;
        self.data_status.store(true, Ordering::SeqCst);
        from_value(prev).ok()
    }

    /// Clears the state
    pub fn clear(&self) -> Result<()> {
        self.data_mut()?.clear();
        self.data_status.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Saves the current state to the store
    pub async fn save(&self) -> Result<()> {
        if self.status.compare_and_swap(0, 1, Ordering::SeqCst) == 0 {
            let data = self.data()?.clone();
            self.config.set(&self.id, data, self.max_age()).await?;
        }
        Ok(())
    }

    /// Renews the new state
    pub async fn renew(&mut self) -> Result<()> {
        if self.status.load(Ordering::Relaxed) < 2 {
            self.config.remove(&self.id).await?;
            self.id = self.config.generate();
            self.data_mut()?.clear();
            self.status.store(2, Ordering::SeqCst);
        }
        Ok(())
    }

    /// Destroys the current state from store
    pub async fn destroy(&self) -> Result<()> {
        if self.status.load(Ordering::Relaxed) < 3 {
            self.config.remove(&self.id).await?;
            self.status.store(3, Ordering::SeqCst);
        }
        Ok(())
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("status", &self.status)
            .field("data", &self.data)
            .field("config", &self.config)
            .finish()
    }
}