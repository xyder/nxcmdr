use std::{fs::File, path::Path};

use crate::models::BoxedResult;


pub fn store_data<T: serde::Serialize>(path: &Path, value: T) -> BoxedResult<()>{
    let path = Path::new(path);
    if path.exists() && !path.is_file() {
        // todo: convert to an error
        panic!(format!("{} must be a file.", path.to_str().unwrap_or("")));
    }

    serde_json::to_writer_pretty(File::create(path)?, &value)
        .map_err(|e| e.into())
}

pub fn load_stored<T>(path: &Path) -> BoxedResult<T>
    where T: serde::de::DeserializeOwned
{
    let file = File::open(path)?;

    serde_json::from_reader::<_, T>(&file)
        .map_err(|e| e.into())
}
