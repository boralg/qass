use std::{collections::HashMap, marker::PhantomData};

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

#[derive(Serialize, Deserialize)]
pub struct UnsaltedHiddenMap {
    pub services: HashMap<String, HiddenEntry>,
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
}

#[derive(Serialize, Deserialize)]
pub struct HiddenEntry {
    pub service: ServiceEntry,
    pub salt: SaltEntry,
}
