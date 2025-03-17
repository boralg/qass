use anyhow::{anyhow, bail};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as b64, Engine as _};
use indexmap::IndexMap;
use zeroize::Zeroizing;

use crate::{
    crypto::{decrypt, derive_key, encrypt, generate_salt},
    io::{config_dir, load_from_file, save_to_file},
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

    let mut salts: IndexMap<String, SaltEntry> = load_from_file(&salts_path)?;
    let mut credentials: ServiceMap = load_from_file(&credentials_path)?;

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

pub fn get(service: String) -> anyhow::Result<Zeroizing<String>> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let salts_path = dir.join("salts.yml");
    let credentials_path = dir.join("credentials.yml");

    let salts: IndexMap<String, SaltEntry> = load_from_file(&salts_path)?;
    let credentials: ServiceMap = load_from_file(&credentials_path)?;

    let service_entry = credentials
        .services
        .get(&service)
        .ok_or_else(|| anyhow!("Service '{}' not found in credentials", service))?;
    let salt_entry = salts
        .get(&service)
        .ok_or_else(|| anyhow!("Service '{}' not found in salts", service))?;

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);
    let key = derive_key(&master_pwd, &salt_entry.salt)?;

    let ciphertext = b64.decode(&service_entry.password)?;
    let nonce = b64.decode(&salt_entry.nonce)?;

    Ok(Zeroizing::new(decrypt(&ciphertext, &key, &nonce)?))
}
