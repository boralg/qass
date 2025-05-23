use std::path::PathBuf;

use anyhow::{anyhow, bail};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as b64, Engine as _};
use indexmap::IndexMap;
use zeroize::Zeroizing;

use crate::{
    crypto::{decrypt, derive_key, encrypt, generate_salt},
    hidden::{HiddenMap, HiddenMapIndex, UnsaltedHiddenMap},
    io::{config_dir, load_from_yaml, save_to_file},
    service::{SaltEntry, ServiceEntry, ServiceMap, UnencryptedService},
};

pub struct State {
    credentials: ServiceMap,
    salts: IndexMap<String, SaltEntry>,
    hidden_credentials: HiddenMapIndex,
}

impl State {
    fn paths() -> anyhow::Result<(PathBuf, PathBuf, PathBuf)> {
        let dir = config_dir()?;
        if !dir.exists() {
            bail!("Config not found. Run 'qass init' first");
        }

        let credentials_path = dir.join("credentials.yml");
        let salts_path = dir.join("salts.yml");
        let hidden_path = dir.join("hidden.yml");

        Ok((credentials_path, salts_path, hidden_path))
    }

    pub fn load() -> anyhow::Result<State> {
        let (credentials_path, salts_path, hidden_path) = State::paths()?;

        let credentials: ServiceMap = load_from_yaml(&credentials_path)?;
        let salts: IndexMap<String, SaltEntry> = load_from_yaml(&salts_path)?;
        let hidden_credentials: HiddenMapIndex = load_from_yaml(&hidden_path)?;

        Ok(State {
            credentials,
            salts,
            hidden_credentials,
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let (credentials_path, salts_path, hidden_path) = State::paths()?;

        save_to_file(&credentials_path, &self.credentials)?;
        save_to_file(&salts_path, &self.salts)?;
        save_to_file(&hidden_path, &self.hidden_credentials)?;

        Ok(())
    }

    pub fn add(
        &mut self,
        service: String,
        username: String,
        password: Zeroizing<String>,
        master_password: Zeroizing<String>,
    ) -> anyhow::Result<()> {
        self.add_many(
            vec![UnencryptedService {
                service,
                username,
                password,
                extra_fields: IndexMap::new(),
            }],
            master_password,
        )
    }

    pub fn add_many(
        &mut self,
        services: Vec<UnencryptedService>,
        master_password: Zeroizing<String>,
    ) -> anyhow::Result<()> {
        for UnencryptedService {
            service,
            username,
            password,
            ..
        } in services
        {
            let salt = generate_salt();
            let key = derive_key(&master_password, &salt)?;
            let (nonce, ciphertext) = encrypt(&password, &key)?;

            self.credentials.insert(
                service.clone(),
                ServiceEntry {
                    username,
                    password: b64.encode(ciphertext),
                    extra_fields: IndexMap::new(),
                },
            );
            self.salts.insert(
                service,
                SaltEntry {
                    nonce: b64.encode(nonce),
                    salt: salt,
                },
            );
        }

        Ok(())
    }

    pub fn get(
        &self,
        service: String,
        master_password: Zeroizing<String>,
    ) -> anyhow::Result<Zeroizing<String>> {
        let service_entry = self
            .credentials
            .services
            .get(&service)
            .ok_or_else(|| anyhow!("Service '{}' not found in credentials", service))?;
        let salt_entry = self
            .salts
            .get(&service)
            .ok_or_else(|| anyhow!("Service '{}' not found in salts", service))?;

        let key = derive_key(&master_password, &salt_entry.salt)?;

        let ciphertext = b64.decode(&service_entry.password)?;
        let nonce = b64.decode(&salt_entry.nonce)?;

        Ok(Zeroizing::new(decrypt(&ciphertext, &key, &nonce)?))
    }

    pub fn hide(&mut self, path: String, master_password: Zeroizing<String>) -> anyhow::Result<()> {
        let credentials = std::mem::take(&mut self.credentials);
        let salts = std::mem::take(&mut self.salts);

        let (to_hide, rest): (Vec<(_, _)>, Vec<(_, _)>) =
            credentials.services.into_iter().partition(|(k, _)| {
                path == "/"
                    || k.starts_with(&path)
                        && (k.len() == path.len() || k.chars().nth(path.len()).unwrap() == '/')
            });
        let (to_hide, rest): (IndexMap<_, _>, IndexMap<_, _>) =
            (IndexMap::from_iter(to_hide), IndexMap::from_iter(rest));

        let mut salts_rest = salts;

        let mut hidden = UnsaltedHiddenMap::new();
        for (k, v) in to_hide {
            if let Some(salt) = salts_rest.shift_remove(&k) {
                hidden.insert(k, v, salt);
            } else {
                bail!("No salt found for {k}.");
            }
        }

        let hidden_str = serde_yaml::to_string(&hidden)?;

        let salt = generate_salt();
        let key = derive_key(&master_password, &salt)?;
        let (nonce, ciphertext) = encrypt(&hidden_str, &key)?;

        self.hidden_credentials.insert(
            path,
            HiddenMap {
                services: b64.encode(ciphertext),
                salt: SaltEntry {
                    nonce: b64.encode(nonce),
                    salt,
                },
            },
        );

        self.credentials = ServiceMap::from(rest);
        self.salts = salts_rest;

        Ok(())
    }

    fn decrypt_hidden(
        hidden_credentials: &HiddenMapIndex,
        path: &String,
        master_password: &str,
    ) -> anyhow::Result<UnsaltedHiddenMap> {
        let hidden_map = hidden_credentials
            .get(path)
            .ok_or_else(|| anyhow!("Path '{}' not found", path))?;

        let key = derive_key(&master_password, &hidden_map.salt.salt)?;
        let nonce = b64.decode(&hidden_map.salt.nonce)?;
        let ciphertext = b64.decode(&hidden_map.services)?;

        let hidden_str = decrypt(&ciphertext, &key, &nonce)?;
        let hidden: UnsaltedHiddenMap = serde_yaml::from_str(&hidden_str)?;

        Ok(hidden)
    }

    pub fn unhide(
        &mut self,
        path: String,
        master_password: Zeroizing<String>,
    ) -> anyhow::Result<()> {
        let hidden = State::decrypt_hidden(&self.hidden_credentials, &path, &master_password)?;

        for (service_key, entry) in hidden.services {
            self.credentials.insert(service_key.clone(), entry.service);
            self.salts.insert(service_key, entry.salt);
        }

        self.hidden_credentials.shift_remove(&path);

        Ok(())
    }

    pub fn get_hidden(
        &self,
        path: String,
        master_password_unhide: Zeroizing<String>,
        master_password: Zeroizing<String>,
    ) -> anyhow::Result<Zeroizing<String>> {
        let hidden = self
            .hidden_credentials
            .keys()
            .filter(|p| path.starts_with(*p))
            .find_map(|p| {
                let h = State::decrypt_hidden(&self.hidden_credentials, p, &master_password_unhide);
                h.ok().and_then(|h| h.services.get(&path).cloned())
            })
            .ok_or_else(|| anyhow!("Service '{}' not found in credentials", path))?;

        let key = derive_key(&master_password, &hidden.salt.salt)?;
        let ciphertext = b64.decode(&hidden.service.password)?;
        let nonce = b64.decode(&hidden.salt.nonce)?;

        Ok(Zeroizing::new(decrypt(&ciphertext, &key, &nonce)?))
    }

    pub fn import_csv(
        &mut self,
        path: String,
        master_password: Zeroizing<String>,
    ) -> anyhow::Result<usize> {
        let file = std::fs::File::open(&path)?;
        let mut reader = csv::Reader::from_reader(file);

        let headers = reader.headers()?.clone();

        let url_idx = headers
            .iter()
            .position(|h| h == "url")
            .ok_or_else(|| anyhow!("Missing 'url' column"))?;
        let username_idx = headers
            .iter()
            .position(|h| h == "username")
            .ok_or_else(|| anyhow!("Missing 'username' column"))?;
        let password_idx = headers
            .iter()
            .position(|h| h == "password")
            .ok_or_else(|| anyhow!("Missing 'password' column"))?;

        let mut services = vec![];
        for result in reader.records() {
            let record = result?;

            if record.len() <= url_idx
                || record.len() <= username_idx
                || record.len() <= password_idx
            {
                continue;
            }

            let url = &record[url_idx];
            let username = &record[username_idx];
            let password = &record[password_idx];

            let mut service = url.to_string();
            service.push('/');
            service.push_str(username);

            services.push(UnencryptedService {
                service,
                username: username.to_string(),
                password: Zeroizing::new(password.to_string()),
                extra_fields: IndexMap::new(),
            });
        }

        let count = services.len();
        self.add_many(services, master_password)?;

        Ok(count)
    }

    pub fn list(&self) -> anyhow::Result<Vec<String>> {
        let services: Vec<String> = self
            .credentials
            .services
            .keys()
            .filter(|key| self.salts.contains_key(*key))
            .cloned()
            .collect();

        Ok(services)
    }

    pub fn unlock(
        &mut self,
        path: String,
        master_password: Zeroizing<String>
    ) -> anyhow::Result<usize> {
        let services: Vec<String> = self.salts.keys().cloned()
            .filter(|p| {
                path == "/"
                    || p.starts_with(&path)
                        && (p.len() == path.len() || p.chars().nth(path.len()).unwrap() == '/')
            })
            .collect();
        
        let mut unlocked_count = 0;

        for service in services {
            match self.get(service.clone(), master_password.clone()) {
                Ok(cleartext) => {
                    if let Some(entry) = self.credentials.services.get_mut(&service) {
                        entry.password = cleartext.to_string();
                        self.salts.shift_remove(&service);
                        unlocked_count += 1;
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(unlocked_count)
    }

    pub fn sync(
        &mut self,
        path: String,
        master_password: Zeroizing<String>,
    ) -> anyhow::Result<usize> {
        self.salts = std::mem::take(&mut self.salts)
            .into_iter()
            .filter(|(p, _)| self.credentials.services.contains_key(p))
            .collect();

        let to_add: Vec<UnencryptedService> = self
            .credentials
            .services
            .iter()
            .filter(|(p, _)| !self.salts.contains_key(*p))
            .filter(|(p, _)| {
                path == "/"
                    || p.starts_with(&path)
                        && (p.len() == path.len() || p.chars().nth(path.len()).unwrap() == '/')
            })
            .map(|(p, s)| UnencryptedService {
                service: p.to_owned(),
                username: s.username.to_owned(),
                password: Zeroizing::new(s.password.to_owned()),
                extra_fields: s.extra_fields.to_owned(),
            })
            .collect();

        let count = to_add.len();
        self.add_many(to_add, master_password)?;

        Ok(count)
    }
}
