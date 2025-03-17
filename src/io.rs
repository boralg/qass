use anyhow::anyhow;
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub fn config_dir() -> anyhow::Result<PathBuf> {
    UserDirs::new()
        .and_then(|ud| Some(ud.home_dir().to_path_buf().join(".qass")))
        .ok_or(anyhow!("Could not determine home directory"))
}

pub fn load_from_yaml<E>(path: &PathBuf) -> anyhow::Result<E>
where
    E: for<'a> Deserialize<'a> + Default,
{
    if !path.exists() {
        return Ok(Default::default());
    }

    let content = fs::read_to_string(path)?;
    Ok(if content.trim().is_empty() {
        Default::default()
    } else {
        serde_yaml::from_str(&content)?
    })
}

pub fn save_to_file<E>(path: &PathBuf, data: &E) -> anyhow::Result<()>
where
    E: Serialize,
{
    let yaml = serde_yaml::to_string(data)?;
    fs::write(path, yaml)?;
    Ok(())
}
