use anyhow::{anyhow, bail};
use clap::{Parser, Subcommand};
use device_query::{DeviceEvents, DeviceEventsHandler, Keycode};
use enigo::{Enigo, Keyboard, Settings};
use io::config_dir;
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

pub mod api;
pub mod crypto;
pub mod hidden;
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
    Hide { path: String },
    Unhide { path: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::Add { service, username } => add(service, username),
        Commands::Type { service } => type_password(service),
        Commands::Hide { path } => hide(path),
        Commands::Unhide { path } => unhide(path),
    }
}

fn init() -> anyhow::Result<()> {
    let dir = io::config_dir()?;
    fs::create_dir_all(&dir)?;

    for file in &["credentials.yml", "salts.yml", "hidden.bin"] {
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

    let password = Zeroizing::new(rpassword::prompt_password("Password: ")?);
    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    api::add(service, username, password, master_pwd)
}

fn type_password(service: String) -> anyhow::Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let password = api::get(service)?;

    println!("Focus the target field and press SPACEBAR to type password (5s timeout)...");

    let start_time = Instant::now();
    let timeout = Duration::from_secs(5);

    let pressed = Arc::new(AtomicBool::new(false));
    {
        let pressed_clone = pressed.clone();

        let event_handler = DeviceEventsHandler::new(Duration::from_millis(10))
            .expect("Could not initialize event loop");
        let _keypress_guard = event_handler.on_key_up(move |keycode| {
            if matches!(keycode, Keycode::Space) {
                pressed_clone.store(true, Ordering::SeqCst);
            }
        });

        while start_time.elapsed() < timeout {
            if pressed.load(Ordering::SeqCst) {
                break;
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    if pressed.load(Ordering::SeqCst) {
        let mut enigo = Enigo::new(&Settings::default()).expect("Could not create keypresses");

        enigo.key(enigo::Key::Backspace, enigo::Direction::Press);
        thread::sleep(Duration::from_millis(50));
        enigo.key(enigo::Key::Backspace, enigo::Direction::Press);
        enigo.text(&password);
    }

    Ok(())
}

fn hide(path: String) -> anyhow::Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    api::hide(path, master_pwd)
}

fn unhide(path: String) -> anyhow::Result<()> {
    let dir = config_dir()?;
    if !dir.exists() {
        bail!("Config not found. Run 'qass init' first");
    }

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    api::unhide(path, master_pwd)
}

