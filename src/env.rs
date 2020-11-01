use std::{collections::{BTreeMap, HashMap}, path::Path};

use crate::models;


pub fn get_env_vars(file_path: &str) -> models::BoxedResult<HashMap<String, String>> {
    let contents = std::fs::read_to_string(Path::new(file_path))?;

    let envs = dotenv_parser::parse_dotenv(&contents)
        .unwrap_or(BTreeMap::<String, String>::new());

    let mut out: HashMap<String, String> = HashMap::new();
    envs.iter().for_each(|(k, v)| {out.insert(k.to_string(), v.to_string());});

    Ok(out)
}
