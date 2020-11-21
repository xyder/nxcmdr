use std::collections::HashMap;

use anyhow::{bail, Context, Result};

pub mod auth;
pub mod models;

mod service;
mod constants;
mod sync;
mod utils;
mod store;

use security::models::{Decrypt, self as sec_models};


pub fn get_by_name(name: &str, token: &models::TokenResponse)
        -> Result<HashMap<String, String>> {

    let key = match &token.master_key {
        Some(v) => v,
        None => bail!("Could not retrieve key from token response")
    };

    let key = base64::decode(key)
        .context("Could not decode base64 string.")?;
    let master_key = sec_models::MasterKey { key, hash: "".to_string()};
    let key: Result<sec_models::SymmetricKey> = master_key.into();
    let key = key?;

    let data = sync::load_data(&token)?;
    let sym_key = sec_models::SymmetricKey::from(
        data.profile.key.decrypt(&key)?);

    // filter for a secure note with the specified name
    let mut found: Vec<&models::Cipher> = data
        .ciphers
        .iter()
        .filter_map(
            |c| if c.cipher_type == 2{
                if c.name.decrypt_string(&sym_key)
                        .unwrap_or("".to_string()) // empty string contains only empty string
                        .to_lowercase()
                        .contains(&name.to_lowercase()) {
                    Some(c)
                } else { None }
            } else { None })
        .collect();

    found.sort_by_cached_key(|c| c.name
        .decrypt_string(&sym_key)
        .unwrap_or("".to_string())  // don't order strings that can't be decrypted :)
    );

    let mut env_vars: HashMap<String, String> = HashMap::new();
    for cipher in found {
        for field in cipher
                .fields.as_ref()
                .unwrap_or(&Vec::<models::CipherField>::new()) {

            env_vars.insert(
                field.name.decrypt_string(&sym_key).unwrap_or("".to_string()),
                field.value.decrypt_string(&sym_key).unwrap_or("".to_string()));
        }
    }

    Ok(env_vars)
}
