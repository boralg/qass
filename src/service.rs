use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaltEntry {
    pub salt: String,
    pub nonce: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub username: String,
    pub password: String,
    #[serde(flatten)]
    pub extra_fields: IndexMap<String, String>,
}

#[derive(Default)]
pub struct ServiceMap {
    pub services: IndexMap<String, ServiceEntry>,
}

impl ServiceMap {
    pub fn new() -> Self {
        ServiceMap {
            services: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: ServiceEntry) -> Option<ServiceEntry> {
        self.services.insert(key, value)
    }
}

impl From<IndexMap<String, ServiceEntry>> for ServiceMap {
    fn from(value: IndexMap<String, ServiceEntry>) -> Self {
        ServiceMap { services: value }
    }
}

impl Serialize for ServiceMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        NestedMap::from_entries(&self.services).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ServiceMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let nested = NestedMap::deserialize(deserializer)?;

        let mut service_map = ServiceMap::new();
        nested.extract_entries("", &mut service_map);

        Ok(service_map)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum NestedMap {
    Map(IndexMap<String, Box<NestedMap>>),
    Leaf(ServiceEntry),
}

impl NestedMap {
    pub fn from_entries(entries: &IndexMap<String, ServiceEntry>) -> Self {
        let mut root = NestedMap::Map(IndexMap::new());

        for (path, entry) in entries {
            let segments: Vec<&str> = path.split('/').collect();
            insert_at_path(&mut root, &segments, entry.clone());
        }

        root
    }

    pub fn extract_entries(&self, prefix: &str, map: &mut ServiceMap) {
        match self {
            NestedMap::Leaf(entry) => {
                map.services.insert(prefix.to_string(), entry.clone());
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

fn insert_at_path(node: &mut NestedMap, segments: &[&str], entry: ServiceEntry) {
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
            panic!("Cannot insert to this path, as it is not a list. Turn it into a list by naming the current entry in the store.")
        }
    }
}
