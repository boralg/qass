use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::login::{LoginEntry, SaltEntry};

pub type HiddenMapIndex = IndexMap<String, HiddenMap>;

#[derive(Serialize, Deserialize)]
pub struct HiddenMap {
    pub logins: EncryptedHiddenMap,
    pub salt: SaltEntry,
}

type EncryptedHiddenMap = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsaltedHiddenMap {
    pub logins: IndexMap<String, HiddenEntry>,
}

impl UnsaltedHiddenMap {
    pub fn new() -> Self {
        Self {
            logins: IndexMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        key: String,
        login: LoginEntry,
        salt: SaltEntry,
    ) -> Option<HiddenEntry> {
        self.logins.insert(key, HiddenEntry { login, salt })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct HiddenEntry {
    pub login: LoginEntry,
    pub salt: SaltEntry,
}
