use std::{fs::File, path::Path};
use std::io::prelude::*;

use security::{models as sec_models, models::Decrypt};

use crate::models::{BoxedResult, Config};


pub fn store_data<T: serde::Serialize>(path: &Path, value: T) -> BoxedResult<()>{
    let config = Config::load(false);
    let path = Path::new(path);
    if path.exists() && !path.is_file() {
        return Err(format!("{} must be a file.", path.to_str().unwrap_or("")).into());
    }

    let data = serde_json::to_string(&value)?;
    let data = Vec::<u8>::from(data);
    let data = config.session_key.encrypt(&data);

    File::create(path)?.write_all(data.to_string().as_bytes())?;

    Ok(())
}

pub fn load_stored<T>(path: &Path) -> BoxedResult<T>
    where T: serde::de::DeserializeOwned
{
    let config = Config::load(false);
    let mut data = String::new();

    File::open(path)?.read_to_string(&mut data)?;
    let data = sec_models::CipherString::from(data.as_str())
        .decrypt(&config.session_key)?;

    let data = String::from_utf8(data)?;
    let data = serde_json::from_str(&data)?;

    Ok(data)
}
