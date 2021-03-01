use std::env;

use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};

use security::models as sec_models;


#[derive(Deserialize, Debug)]
pub struct PreLoginResponse {
    #[serde(rename = "Kdf")]
    pub kdf: Option<u32>,
    #[serde(rename = "KdfIterations")]
    pub iterations: Option<u32>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BWErrorModel {
    #[serde(rename = "Message")]
    pub message: Option<String>,
    #[serde(rename = "Object")]
    pub object: Option<String>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenResponse {
    pub access_token: Option<String>, // jwt with at least the following keys: nbf, exp, iss, sub, email, name, premium
    pub expires_in: Option<u32>, // in seconds
    pub token_type: Option<String>,   // Bearer
    pub refresh_token: Option<String>, // used after `expires_in` seconds to get a new access token
    pub scope: Option<String>,
    #[serde(rename = "PrivateKey")]
    pub private_key: Option<String>,
    #[serde(rename = "Key")]
    pub key: Option<String>,
    #[serde(rename = "ResetMasterPassword")]
    pub reset_password: Option<bool>,
    #[serde(rename = "Kdf")]
    pub kdf: Option<u32>,
    #[serde(rename = "KdfIterations")]
    pub iterations: Option<u32>,
    pub error: Option<String>,
    pub error_description: Option<String>,
    #[serde(rename = "ErrorModel")]
    pub error_model: Option<BWErrorModel>,

    // additional
    pub last_saved: Option<String>,
    pub master_key: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "Key")]
    pub key: sec_models::CipherString
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CipherField {
    #[serde(rename = "Name")]
    pub name: sec_models::CipherString,
    // todo: write what types exist
    #[serde(rename = "Type")]
    pub field_type: u8,
    #[serde(rename = "Value")]
    pub value: sec_models::CipherString
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cipher {
    #[serde(rename = "Id")]
    pub id: uuid::Uuid,

    #[serde(rename = "Name")]
    pub name: sec_models::CipherString,

    /**
    Cipher types:
        0 - ???
        1 - Login
        2 - Secure Note
        3 - Card
        4 - Identity
    */
    #[serde(rename = "Type")]
    pub cipher_type: u8,

    #[serde(rename = "Fields")]
    pub fields: Option<Vec<CipherField>>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyncResponse {
    pub rev_date: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "Profile")]
    pub profile: Profile,
    #[serde(rename = "Ciphers")]
    pub ciphers: Vec<Cipher>,
    #[serde(rename = "Collections")]
    pub collections: Vec<serde_json::Value>,
    #[serde(rename = "Domains")]
    pub domains: serde_json::Value,
    #[serde(rename = "Folders")]
    pub folders: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct  IterationsRequest {
    pub email: String
}

// todo: implement defaults for this
#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshTokenRequest {
    pub grant_type: String,
    pub client_id: String, // todo: fetch this from config
    pub refresh_token: String
}

// todo: implement defaults for this
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenRequest {
    pub grant_type: String,
    pub username: String,
    pub password: String,
    pub scope: String,
    pub client_id: String, // todo: fetch this from config
    #[serde(rename = "deviceType")]
    pub device_type: u8,
    #[serde(rename = "deviceIdentifier")]
    pub device_id: uuid::Uuid,  // todo: generate this into a config file
    #[serde(rename = "deviceName")]
    pub device_name: String,  // todo: fetch this from config
    #[serde(rename = "twoFactorToken")]
    pub two_factor_token: u32,
    #[serde(rename = "twoFactorProvider")]
    pub two_factor_provider: u8,
    #[serde(rename = "twoFactorRemember")]
    pub two_factor_remember: u8
}

#[derive(Clone, Debug)]
pub struct Config {
    pub config_dir: String,
    pub session_key: sec_models::SymmetricKey,
    pub bw_user: Option<String>,
    pub bw_pass: Option<String>,
    pub bw_tfa: Option<String>
}

impl Config {
    pub fn load(reset_session: bool) -> Result<Self> {
        Ok(Self {
            config_dir: {
                let config_dir = match env::var("NXCMDR_CONFIG_DIR") {
                    Ok(v) => v,
                    Err(_) => format!("{}/.config/nxcmdr", env::var("HOME")
                        .context("HOME environment variable is not set")?)
                };

                std::fs::create_dir_all(&config_dir)
                    .context("Could not create config directory")?;

                config_dir
            },
            session_key: {
                let skip_session_gen = env::var("NXCMDR_SKIP_SESSION_GEN").is_ok();

                let session_key = match reset_session && !skip_session_gen {
                    true => None,
                    false => env::var("NXCMDR_SESSION_KEY").ok()
                };

                let was_none = session_key.is_none();
                let session_key = sec_models::SymmetricKey::from(session_key);

                if was_none {
                    let session_key_str = session_key.to_string();
                    println!("Run this command to skip login next time:\n\
                        export NXCMDR_SESSION_KEY={}", session_key_str);

                    env::set_var("NXCMDR_SESSION_KEY", session_key_str);
                    // todo: using env vars as a global. I know, ugly.
                    env::set_var("NXCMDR_SKIP_SESSION_GEN", "generated");
                }

                session_key
            },
            bw_user: env::var("NXCMDR_BW_USER").ok(),
            bw_pass: env::var("NXCMDR_BW_PASS").ok(),
            bw_tfa: env::var("NXCMDR_BW_TFA").ok()
        })
    }
}
