use std::{collections::HashMap, sync::Arc};

use async_lock::{Lock, LockGuard};
use async_trait::async_trait;

use crate::{Session, SessionBeer, SessionStatus, State, Storable};

/// MemoryStore
///
/// Stores the session in an in-memory store.
#[derive(Clone, Debug)]
pub struct MemoryStore {
    inner: Lock<HashMap<String, State>>,
}

impl MemoryStore {
    /// Creates new Memory Store
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Lock::default(),
        }
    }

    async fn store(&self) -> LockGuard<HashMap<String, State>> {
        self.inner.lock().await
    }
}

#[async_trait]
impl Storable for MemoryStore {
    async fn get(&self, sid: &str) -> Session {
        let session = Session::new(Arc::new(self.clone()));

        if !self.verify_sid(sid).await {
            return session;
        }

        let store = self.store().await;

        if store.contains_key(sid) {
            if let Some(data) = store.get(sid).cloned() {
                let SessionBeer { id, state, status } = &mut *session.beer().await;
                *state = data;
                *status = SessionStatus::Existed;
                *id = sid.to_owned();
            }
        }

        session
    }

    async fn remove(&self, sid: &str) -> bool {
        self.store().await.remove(sid).is_some()
    }

    async fn save(&self, session: &Session) -> bool {
        self.store()
            .await
            .insert(session.id().await, session.state().await)
            .map_or_else(|| true, |_| true)
    }
}
