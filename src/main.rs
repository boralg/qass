use anyhow::{anyhow, bail};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as b64, Engine as _};
use clap::{Parser, Subcommand};
use crypto::{decrypt_password, derive_key, encrypt_password, generate_salt};
use indexmap::IndexMap;
use io::{config_dir, load_from_file, save_to_file};
use service::{SaltEntry, ServiceEntry, ServiceMap};
use std::fs::{self, File};
use zeroize::Zeroizing;

pub mod crypto;
pub mod io;
pub mod service;

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::Add { service, username } => add(service, username),
    }
}

fn init() -> anyhow::Result<()> {
    let dir = io::config_dir()?;
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

    let mut salts: IndexMap<String, SaltEntry> = load_from_file(&salts_path)?;
    let load_from_file = load_from_file(&credentials_path);
    let mut credentials: ServiceMap = load_from_file?;

    // TODO: get rid of b64 in this module
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
