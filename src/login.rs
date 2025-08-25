use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SaltEntry {
    pub salt: String,
    pub nonce: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LoginEntry {
    pub username: String,
    pub password: String,
    #[serde(flatten)]
    pub extra_fields: IndexMap<String, String>,
}

pub struct UnencryptedLogin {
    pub login_name: String,
    pub username: String,
    pub password: Zeroizing<String>,
    pub extra_fields: IndexMap<String, String>,
}

#[derive(Default)]
pub struct LoginMap {
    pub logins: IndexMap<String, LoginEntry>,
}

impl LoginMap {
    pub fn new() -> Self {
        LoginMap {
            logins: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: LoginEntry) -> Option<LoginEntry> {
        self.logins.insert(key, value)
    }
}

impl From<IndexMap<String, LoginEntry>> for LoginMap {
    fn from(value: IndexMap<String, LoginEntry>) -> Self {
        LoginMap { logins: value }
    }
}

impl Serialize for LoginMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        NestedMap::from_entries(&self.logins).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LoginMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let nested = NestedMap::deserialize(deserializer)?;

        let mut login_map = LoginMap::new();
        nested.extract_entries("", &mut login_map);

        Ok(login_map)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum NestedMap {
    Map(IndexMap<String, Box<NestedMap>>),
    Leaf(LoginEntry),
}

impl NestedMap {
    pub fn from_entries(entries: &IndexMap<String, LoginEntry>) -> Self {
        let mut root = NestedMap::Map(IndexMap::new());

        for (path, entry) in entries {
            let segments: Vec<&str> = path.split('/').collect();
            insert_at_path(&mut root, &segments, entry.clone());
        }

        root
    }

    pub fn extract_entries(&self, prefix: &str, map: &mut LoginMap) {
        match self {
            NestedMap::Leaf(entry) => {
                map.logins.insert(prefix.to_string(), entry.clone());
            }
            NestedMap::Map(children) => {
                for (key, child) in children {
                    let new_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{}/{}", prefix, key)
                    };
                    child.extract_entries(&new_prefix, map);
                }
            }
        }
    }
}

fn insert_at_path(node: &mut NestedMap, segments: &[&str], entry: LoginEntry) {
    if segments.is_empty() {
        panic!("Path must not be empty.");
    }

    match node {
        NestedMap::Map(children) => {
            let current = segments[0];

            if segments.len() == 1 {
                children.insert(current.to_string(), Box::new(NestedMap::Leaf(entry)));
            } else {
                if let Some(mut child) = children.get_mut(current) {
                    insert_at_path(&mut child, &segments[1..], entry);
                } else {
                    let mut new_child = Box::new(NestedMap::Map(IndexMap::new()));
                    insert_at_path(&mut new_child, &segments[1..], entry);
                    children.insert(current.to_string(), new_child);
                }
            }
        }
        NestedMap::Leaf(_) => {
            panic!("Cannot insert to this path, as it is a login, not a collection. Turn it into a collection by making a new layer and naming it.")
        }
    }
}
