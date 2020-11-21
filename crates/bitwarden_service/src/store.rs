use std::{fs::File, path::Path};
use std::io::prelude::*;

use anyhow::{Context, Result, bail};

use security::{models as sec_models, models::Decrypt};

use crate::models::Config;


pub fn store_data<T: serde::Serialize>(path: &Path, value: T) -> Result<()>{
    let config = Config::load(false)?;
    let path = Path::new(path);
    let path_str = path.to_str().unwrap_or("");

    if path.exists() && !path.is_file() {
        bail!("{} must be a file.", path_str)
    }

    let data = serde_json::to_string(&value)?;
    let data = Vec::<u8>::from(data);
    let data = config.session_key.encrypt(&data)?;

    File::create(path)
        .context(format!("Could not create file {}", path_str))?
        .write_all(data.to_string().as_bytes())
        .context(format!("Could not write to file {}", path_str))?;

    Ok(())
}

pub fn load_stored<T>(path: &Path) -> Result<T>
    where T: serde::de::DeserializeOwned
{
    let config = Config::load(false)?;
    let mut data = String::new();
    let path_str = path.to_str().unwrap_or("");

    File::open(path)
        .context(format!("Could not open file {}", path_str))?
        .read_to_string(&mut data)
        .context(format!("Could not read file {}", path_str))?;

    let data_wrapper = sec_models::StringWrapper::from(data.as_str());
    let data_cs: Result<sec_models::CipherString> = (&data_wrapper).into();
    let data = data_cs?.decrypt(&config.session_key)?;

    let data = String::from_utf8(data)?;
    let data = serde_json::from_str(&data)?;

    Ok(data)
}
