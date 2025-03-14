use aes_gcm_siv::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256GcmSiv,
};
use anyhow::{anyhow, bail};
use argon2::Argon2;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as b64, Engine as _};
use clap::{Parser, Subcommand};
use directories::UserDirs;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    path::PathBuf,
};
use zeroize::Zeroizing;

#[derive(Parser)]
#[command(name = "qass")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add { service: String, username: String },
}

#[derive(Serialize, Deserialize)]
struct SaltEntry {
    salt: String,
    nonce: String,
}

#[derive(Serialize, Deserialize)]
struct ServiceEntry {
    username: String,
    password: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::Add { service, username } => add(service, username),
    }
}

fn config_dir() -> anyhow::Result<PathBuf> {
    UserDirs::new()
        .and_then(|ud| Some(ud.home_dir().to_path_buf().join(".qass")))
        .ok_or(anyhow!("Could not determine home directory"))
}

fn init() -> anyhow::Result<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;

    for file in &["credentials.yml", "salts.yml"] {
        let path = dir.join(file);
        if !path.exists() {
            File::create(path)?;
        }
    }

    Ok(())
}

fn add(service: String, username: String) -> anyhow::Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let salts_path = dir.join("salts.yml");
    let credentials_path = dir.join("credentials.yml");

    let password = Zeroizing::new(rpassword::prompt_password("Password: ")?);
    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    let salt = generate_salt();
    let key = derive_key(&master_pwd, &salt)?;
    let (nonce, ciphertext) = encrypt_password(&password, &key)?;

    let mut salts = load_from_file::<SaltEntry>(&salts_path)?;
    let mut credentials = load_from_file(&credentials_path)?;

    credentials.insert(
        service.clone(),
        ServiceEntry {
            username,
            password: b64.encode(ciphertext),
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

fn generate_salt() -> String {
    let mut salt = [0u8; 16];
    rand::rng().fill_bytes(&mut salt);
    b64.encode(salt)
}

fn derive_key(master_pwd: &str, base64_salt: &str) -> anyhow::Result<[u8; 32]> {
    let salt_bytes = b64.decode(base64_salt)?;
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(master_pwd.as_bytes(), &salt_bytes, &mut key)
        .map_err(|_| anyhow!("Failed to derive key from master password"))?;
    Ok(key)
}

fn encrypt_password(password: &str, key: &[u8; 32]) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    let cipher = Aes256GcmSiv::new_from_slice(key)
        .map_err(|_| anyhow!("Failed to initialize cipher from derived key"))?;
    let nonce = Aes256GcmSiv::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, password.as_bytes())
        .map_err(|_| anyhow!("Failed to enrypt password"))?;

    Ok((nonce.to_vec(), ciphertext))
}

fn load_from_file<E>(path: &PathBuf) -> anyhow::Result<HashMap<String, E>>
where
    E: for<'a> Deserialize<'a>,
{
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path)?;
    Ok(if content.trim().is_empty() {
        HashMap::new()
    } else {
        serde_yaml::from_str(&content)?
    })
}

fn save_to_file<E>(path: &PathBuf, data: &HashMap<String, E>) -> anyhow::Result<()>
where
    E: Serialize,
{
    let yaml = serde_yaml::to_string(data)?;
    fs::write(path, yaml)?;
    Ok(())
}
