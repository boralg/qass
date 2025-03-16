use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::ser::SerializeMap;
use serde::de::{self, MapAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;

pub struct ServiceEntry {
    pub username: String,
    pub password: String,
    pub extra_fields: Vec<(String, String)>,
}

impl<'de> Deserialize<'de> for ServiceEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ServiceEntryVisitor(PhantomData<ServiceEntry>);
        
        impl<'de> Visitor<'de> for ServiceEntryVisitor {
            type Value = ServiceEntry;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map containing at least 'username' and 'password' fields")
            }
            
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut username = None;
                let mut password = None;
                let mut extra_fields = Vec::new();
                
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "username" => {
                            if username.is_some() {
                                return Err(de::Error::duplicate_field("username"));
                            }
                            username = Some(map.next_value()?);
                        }
                        "password" => {
                            if password.is_some() {
                                return Err(de::Error::duplicate_field("password"));
                            }
                            password = Some(map.next_value()?);
                        }
                        _ => {
                            let value = map.next_value()?;
                            extra_fields.push((key, value));
                        }
                    }
                }
                
                let username = username.ok_or_else(|| de::Error::missing_field("username"))?;
                let password = password.ok_or_else(|| de::Error::missing_field("password"))?;
                
                Ok(ServiceEntry {
                    username,
                    password,
                    extra_fields,
                })
            }
        }
        
        deserializer.deserialize_map(ServiceEntryVisitor(PhantomData))
    }
}

impl Serialize for ServiceEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2 + self.extra_fields.len()))?;
        
        map.serialize_entry("username", &self.username)?;
        map.serialize_entry("password", &self.password)?;
        
        for (key, value) in &self.extra_fields {
            map.serialize_entry(key, value)?;
        }
        
        map.end()
    }
}