use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result, anyhow};

pub fn get_env_vars(file_path: &str) -> Result<HashMap<String, String>> {
    let contents = std::fs::read_to_string(Path::new(file_path))
        .context(format!("Could not read file: {}", file_path))?;

    let envs = dotenv_parser::parse_dotenv(&contents)
        .map_err(|e| anyhow!(e))
        .context(format!("Could not parse file: {}", file_path))?;

    let mut out: HashMap<String, String> = HashMap::new();
    envs.iter().for_each(|(k, v)| {out.insert(k.to_string(), v.to_string());});

    Ok(out)
}
