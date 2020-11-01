use std::{collections::HashMap, path::Path};

use crate::models;


pub fn get_env_vars(file_path: &str) -> models::BoxedResult<HashMap<String, String>> {
    let mut initial_envs: HashMap<String, String> = HashMap::new();
    std::env::vars().for_each(|v| {initial_envs.insert(v.0, v.1);});

    let path = Path::new(file_path);
    dotenv::from_path(path)?;

    let mut envs: HashMap<String, String> = HashMap::new();
    dotenv::vars().for_each(|v| {
        if !initial_envs.contains_key(&v.0) {
            envs.insert(v.0, v.1);
        }
    });

    Ok(envs)
}
