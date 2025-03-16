use anyhow::{anyhow, bail};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as b64, Engine as _};
use clap::{Parser, Subcommand};
use crypto::{decrypt_password, derive_key, encrypt_password, generate_salt};
use device_query::{DeviceEvents, DeviceEventsHandler, DeviceQuery, DeviceState, Keycode};
use enigo::{Enigo, Keyboard, Settings};
use indexmap::IndexMap;
use io::{config_dir, load_from_file, save_to_file};
use service::{SaltEntry, ServiceEntry, ServiceMap};
use std::{
    fs::{self, File},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
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
    Type { service: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::Add { service, username } => add(service, username),
        Commands::Type { service } => type_password(service),
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

fn type_password(service: String) -> anyhow::Result<()> {
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

    let password = Zeroizing::new(decrypt_password(&ciphertext, &key, &nonce)?);

    println!("Focus the target field and press SPACEBAR to type password (5s timeout)...");

    let start_time = Instant::now();
    let timeout = Duration::from_secs(5);

    let pressed = Arc::new(AtomicBool::new(false));
    let pressed_clone = pressed.clone();

    let event_handler = DeviceEventsHandler::new(Duration::from_millis(10))
        .expect("Could not initialize event loop");
    let _mouse_move_guard = event_handler.on_key_up(move |keycode| {
        if !matches!(keycode, Keycode::Space) {
            return;
        }

        let mut enigo = Enigo::new(&Settings::default()).expect("Could not create keypresses");

        enigo.key(enigo::Key::Backspace, enigo::Direction::Press);
        thread::sleep(Duration::from_millis(50));
        enigo.key(enigo::Key::Backspace, enigo::Direction::Press);
        enigo.text(&password);

        pressed_clone.store(true, Ordering::SeqCst);
    });

    while start_time.elapsed() < timeout {
        if pressed.load(Ordering::SeqCst) {
            break;
        }

        thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
