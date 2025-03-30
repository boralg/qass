use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::service::{SaltEntry, ServiceEntry};

pub type HiddenMapIndex = IndexMap<String, HiddenMap>;

#[derive(Serialize, Deserialize)]
pub struct HiddenMap {
    pub services: EncryptedHiddenMap,
    pub salt: SaltEntry,
}

type EncryptedHiddenMap = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsaltedHiddenMap {
    pub services: IndexMap<String, HiddenEntry>,
}

impl UnsaltedHiddenMap {
    pub fn new() -> Self {
        Self {
            services: IndexMap::new(),
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
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HiddenEntry {
    pub service: ServiceEntry,
    pub salt: SaltEntry,
}
