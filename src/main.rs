use api::State;
use clap::{Parser, Subcommand};
use device_query::{DeviceEvents, DeviceEventsHandler, Keycode};
use enigo::{Enigo, Keyboard, Settings};
use std::{
    fs::{self, File},
    io::Write,
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
#[cfg(feature = "gui")]
pub mod gui;
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
    Add {
        service: String,
        username: String,
    },
    Type {
        service: String,
    },
    Hide {
        path: String,
    },
    Unhide {
        path: String,
    },
    TypeHidden {
        service: String,
    },
    Import {
        path: String,
    },
    List,
    Sync {
        #[clap(default_value = "/")]
        path: String,
    },
    Unlock {
        path: String,
    },
    #[cfg(feature = "gui")]
    Gui,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::Add { service, username } => add(service, username),
        Commands::Type { service } => type_password(service),
        Commands::Hide { path } => hide(path),
        Commands::Unhide { path } => unhide(path),
        Commands::TypeHidden { service } => type_hidden_password(service),
        Commands::Import { path } => import_csv(path),
        Commands::List => list_services(),
        Commands::Unlock { path } => unlock(path),
        Commands::Sync { path } => sync(path),
        #[cfg(feature = "gui")]
        Commands::Gui => gui::run(),
    }
}

// TODO: move to api
fn init() -> anyhow::Result<()> {
    let dir = io::config_dir()?;
    fs::create_dir_all(&dir)?;

    for file in &["credentials.yml", "salts.yml", "hidden.yml"] {
        let path = dir.join(file);
        if !path.exists() {
            File::create(path)?;
        }
    }

    Ok(())
}

fn add(service: String, username: String) -> anyhow::Result<()> {
    let mut state = State::load()?;

    let password = Zeroizing::new(rpassword::prompt_password("Password: ")?);
    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    state.add(service, username, password, master_pwd)?;
    state.save()
}

fn type_password(service: String) -> anyhow::Result<()> {
    let state = State::load()?;

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);
    let password = state.get(service, master_pwd)?;

    type_password_text(&password)
}

fn type_password_text(password: &str) -> anyhow::Result<()> {
    println!("Focus the target field and press CONTROL to type password (5s timeout)...");

    let start_time = Instant::now();
    let timeout = Duration::from_secs(5);

    let pressed = Arc::new(AtomicBool::new(false));
    {
        let pressed_clone = pressed.clone();

        let event_handler = DeviceEventsHandler::new(Duration::from_millis(10))
            .expect("Could not initialize event loop");
        let _keypress_guard = event_handler.on_key_up(move |keycode| {
            if matches!(keycode, Keycode::LControl | Keycode::RControl) {
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

        // enigo.key(enigo::Key::Backspace, enigo::Direction::Press)?;
        // thread::sleep(Duration::from_millis(50));
        // enigo.key(enigo::Key::Backspace, enigo::Direction::Release)?;
        enigo.text(password)?;
    }

    Ok(())
}

fn hide(path: String) -> anyhow::Result<()> {
    let mut state = State::load()?;

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    state.hide(path, master_pwd)?;
    state.save()
}

fn unhide(path: String) -> anyhow::Result<()> {
    let mut state = State::load()?;

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    state.unhide(path, master_pwd)?;
    state.save()
}

fn type_hidden_password(service: String) -> anyhow::Result<()> {
    let state = State::load()?;

    let master_pwd_unhide =
        Zeroizing::new(rpassword::prompt_password("Master Password (Unhide): ")?);
    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);
    let password = state.get_hidden(service, master_pwd_unhide, master_pwd)?;

    type_password_text(&password)?;

    Ok(())
}

fn import_csv(path: String) -> anyhow::Result<()> {
    let mut state = State::load()?;

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    println!("Importing services...");

    // TODO: progress bar
    let count = state.import_csv(path, master_pwd)?;
    state.save()?;

    println!("Successfully imported {} services", count);

    Ok(())
}

fn list_services() -> anyhow::Result<()> {
    let state = State::load()?;

    for path in state.list() {
        println!("{}", path);
    }

    Ok(())
}

fn sync(path: String) -> anyhow::Result<()> {
    let mut state = State::load()?;

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    let count = state.sync(path, master_pwd)?;
    state.save()?;

    println!("Successfully synced {} entries", count);

    Ok(())
}

fn unlock(path: String) -> anyhow::Result<()> {
    let mut state = State::load()?;

    println!("WARNING: This will decrypt passwords and store them in cleartext.");
    println!("Anyone with access to your store directory will be able to see these passwords.");
    println!("You can re-encrypt them later using the 'sync' command.");
    print!("Are you sure you want to continue? [y/N]: ");
    std::io::stdout().flush()?;

    let mut response = String::new();
    std::io::stdin().read_line(&mut response)?;

    if response.trim().to_lowercase() != "y" {
        println!("Operation canceled.");
        return Ok(());
    }

    let master_pwd = Zeroizing::new(rpassword::prompt_password("Master Password: ")?);

    let count = state.unlock(path, master_pwd);
    state.save()?;

    println!("Successfully unlocked {} entries", count);

    Ok(())
}
