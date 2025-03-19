use std::{collections::HashMap, marker::PhantomData};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::service::{SaltEntry, ServiceEntry};

pub type HiddenMapIndex = IndexMap<String, EncryptedHiddenMap>;
type EncryptedHiddenMap = String;

#[derive(Serialize, Deserialize)]
pub struct HiddenMap {
    services: HashMap<String, HiddenEntry>,
    salt: SaltEntry,
}

#[derive(Serialize, Deserialize)]
pub struct UnsaltedHiddenMap {
    services: HashMap<String, HiddenEntry>,
}

impl UnsaltedHiddenMap {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        key: String,
        service: ServiceEntry,
        salt: SaltEntry,
    ) -> Option<HiddenEntry> {
        self.services.insert(key, HiddenEntry { service, salt })
    }

    pub fn add_salt(self, salt: SaltEntry) -> HiddenMap {
        HiddenMap {
            services: self.services,
            salt,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HiddenEntry {
    service: ServiceEntry,
    salt: SaltEntry,
}
