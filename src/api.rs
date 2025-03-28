use anyhow::{anyhow, bail};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as b64, Engine as _};
use indexmap::IndexMap;
use zeroize::Zeroizing;

use crate::{
    crypto::{decrypt, derive_key, encrypt, generate_salt},
    hidden::{HiddenMap, HiddenMapIndex, UnsaltedHiddenMap},
    io::{config_dir, load_from_yaml, save_to_file},
    service::{SaltEntry, ServiceEntry, ServiceMap},
};

pub fn add(
    service: String,
    username: String,
    password: Zeroizing<String>,
    master_password: Zeroizing<String>,
) -> anyhow::Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let salts_path = dir.join("salts.yml");
    let credentials_path = dir.join("credentials.yml");

    let salt = generate_salt();
    let key = derive_key(&master_password, &salt)?;
    let (nonce, ciphertext) = encrypt(&password, &key)?;

    let mut salts: IndexMap<String, SaltEntry> = load_from_yaml(&salts_path)?;
    let mut credentials: ServiceMap = load_from_yaml(&credentials_path)?;

    credentials.insert(
        service.clone(),
        ServiceEntry {
            username,
            password: b64.encode(ciphertext),
            extra_fields: IndexMap::new(),
        },
    );
    salts.insert(
        service,
        SaltEntry {
            nonce: b64.encode(nonce),
            salt: salt,
        },
    );

    save_to_file(&credentials_path, &credentials)?;
    save_to_file(&salts_path, &salts)?;

    Ok(())
}

pub fn get(service: String, master_password: Zeroizing<String>) -> anyhow::Result<Zeroizing<String>> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let salts_path = dir.join("salts.yml");
    let credentials_path = dir.join("credentials.yml");

    let salts: IndexMap<String, SaltEntry> = load_from_yaml(&salts_path)?;
    let credentials: ServiceMap = load_from_yaml(&credentials_path)?;

    let service_entry = credentials
        .services
        .get(&service)
        .ok_or_else(|| anyhow!("Service '{}' not found in credentials", service))?;
    let salt_entry = salts
        .get(&service)
        .ok_or_else(|| anyhow!("Service '{}' not found in salts", service))?;

    let key = derive_key(&master_password, &salt_entry.salt)?;

    let ciphertext = b64.decode(&service_entry.password)?;
    let nonce = b64.decode(&salt_entry.nonce)?;

    Ok(Zeroizing::new(decrypt(&ciphertext, &key, &nonce)?))
}

pub fn hide(path: String, master_password: Zeroizing<String>) -> anyhow::Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let salts_path = dir.join("salts.yml");
    let credentials_path = dir.join("credentials.yml");
    let hidden_path = dir.join("hidden.yml");

    let salts: IndexMap<String, SaltEntry> = load_from_yaml(&salts_path)?;
    let credentials: ServiceMap = load_from_yaml(&credentials_path)?;
    let mut hidden_credentials: HiddenMapIndex = load_from_yaml(&hidden_path)?;

    let (to_hide, rest): (Vec<(_, _)>, Vec<(_, _)>) =
        credentials.services.into_iter().partition(|(k, _)| {
            k.starts_with(&path)
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

    hidden_credentials.insert(
        path,
        HiddenMap {
            services: b64.encode(ciphertext),
            salt: SaltEntry {
                nonce: b64.encode(nonce),
                salt,
            },
        },
    );

    save_to_file(&credentials_path, &ServiceMap::from(rest))?;
    save_to_file(&salts_path, &salts_rest)?;
    save_to_file(&hidden_path, &hidden_credentials)?;

    Ok(())
}

pub fn unhide(path: String, master_password: Zeroizing<String>) -> anyhow::Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let salts_path = dir.join("salts.yml");
    let credentials_path = dir.join("credentials.yml");
    let hidden_path = dir.join("hidden.yml");

    let mut salts: IndexMap<String, SaltEntry> = load_from_yaml(&salts_path)?;
    let mut credentials: ServiceMap = load_from_yaml(&credentials_path)?;
    let mut hidden_credentials: HiddenMapIndex = load_from_yaml(&hidden_path)?;

    let hidden_map = hidden_credentials
        .get(&path)
        .ok_or_else(|| anyhow!("Path '{}' not found", path))?;

    let key = derive_key(&master_password, &hidden_map.salt.salt)?;
    let nonce = b64.decode(&hidden_map.salt.nonce)?;
    let ciphertext = b64.decode(&hidden_map.services)?;

    let hidden_str = decrypt(&ciphertext, &key, &nonce)?;
    let hidden: UnsaltedHiddenMap = serde_yaml::from_str(&hidden_str)?;

    for (service_key, entry) in hidden.services {
        credentials.insert(service_key.clone(), entry.service);
        salts.insert(service_key, entry.salt);
    }

    hidden_credentials.shift_remove(&path);

    save_to_file(&credentials_path, &credentials)?;
    save_to_file(&salts_path, &salts)?;
    save_to_file(&hidden_path, &hidden_credentials)?;

    Ok(())
}

pub fn get_hidden() {

}