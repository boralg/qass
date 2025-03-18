use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::service::{SaltEntry, ServiceEntry};

pub struct HiddenMapIndex {
    pub maps: HashMap<String, HiddenMap>,
}

impl HiddenMapIndex {
    pub fn new() -> Self {
        Self {
            maps: HashMap::new(),
        }
    }
}

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
