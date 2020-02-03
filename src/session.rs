use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value};
use std::{
    error::Error as ErrorExt,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::State;
use crate::Storable;

#[derive(Debug)]
pub struct Session {
    store: Arc<dyn Storable>,
    /// Why not use `Rc<RefCell<Map<String, Value>>>`?
    /// See: https://github.com/hyperium/http/blob/master/src/extensions.rs
    state: Arc<RwLock<State>>,
    name: String,
    is_new: bool,
}

impl Session {
    #[inline]
    pub fn new(name: &str, store: Arc<impl Storable>) -> Self {
        Self {
            store,
            state: Arc::default(),
            name: name.to_owned(),
            is_new: false,
        }
    }

    pub fn name(&self) -> String {
        self.name.to_owned()
    }

    pub fn store(&self) -> Arc<dyn Storable> {
        self.store.clone()
    }

    pub fn state(&self) -> Result<RwLockReadGuard<'_, State>, Error> {
        self.state
            .read()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    pub fn state_mut(&self) -> Result<RwLockWriteGuard<'_, State>, Error> {
        self.state
            .write()
            .map_err(|e| Error::new(ErrorKind::Other, e.description()))
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        Ok(if let Some(val) = self.state()?.get(key).cloned() {
            from_value(val)?
        } else {
            None
        })
    }

    pub fn set<T: DeserializeOwned + Serialize>(
        &self,
        key: &str,
        val: T,
    ) -> Result<Option<T>, Error> {
        Ok(
            if let Some(prev) = self.state_mut()?.insert(key.to_owned(), to_value(val)?) {
                from_value(prev)?
            } else {
                None
            },
        )
    }

    pub fn remove<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        Ok(if let Some(val) = self.state_mut()?.remove(key) {
            from_value(val)?
        } else {
            None
        })
    }

    pub fn clear(&self) -> Result<(), Error> {
        Ok(self.state_mut()?.clear())
    }

    pub async fn save(&self) -> Result<(), Error> {
        let name = self.name();
        let data = self.state().unwrap().clone();
        self.store.save(name, data).await
    }
}
