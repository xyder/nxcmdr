use std::collections::HashMap;

pub mod auth;
pub mod models;

mod service;
mod constants;
mod sync;
mod utils;
mod store;

use security::models::{Decrypt, self as sec_models};


pub async fn get_by_name(name: &str, token: &models::TokenResponse)
        -> Result<HashMap<String, String>, reqwest::Error> {

    let key = token
        .master_key.clone()
        .expect("Key was not found on the token.");

    let key = base64::decode(key)
        .and_then(|k| Ok(sec_models::SymmetricKey::from(&sec_models::MasterKey { key: k, hash: "".to_string()})))
        .expect("Key has an invalid format.");

    let data = sync::load_data(&token).await.unwrap();
    let sym_key = sec_models::SymmetricKey::from(
        data.profile.key.decrypt(&key));

    // filter for a secure note with the specified name
    let mut found: Vec<&models::Cipher> = data
        .ciphers
        .iter()
        .filter_map(
            |c| if c.cipher_type == 2{
                if c.name.decrypt_string(&sym_key)
                        .unwrap()
                        .to_lowercase()
                        .contains(&name.to_lowercase()) {
                    Some(c)
                } else { None }
            } else { None })
        .collect();

    found.sort_by_cached_key(|c| c.name.decrypt_string(&sym_key).unwrap());

    let mut env_vars: HashMap<String, String> = HashMap::new();
    for cipher in found {
        for field in cipher
                .fields.as_ref()
                .unwrap_or(&Vec::<models::CipherField>::new()) {

            env_vars.insert(
                field.name.decrypt_string(&sym_key).unwrap(),
                field.value.decrypt_string(&sym_key).unwrap());
        }
    }

    Ok(env_vars)
}
