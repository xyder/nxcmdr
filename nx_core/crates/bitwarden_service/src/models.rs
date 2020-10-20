use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct PreLoginResponse {
    pub Kdf: u32,
    pub KdfIterations: u32,
    pub error: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct BWErrorModel {
    pub Message: Option<String>,
    pub Object: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub refresh_token: Option<String>, // used after `expires_in` seconds to get a new access token
    pub access_token: Option<String>, // jwt with at least the following keys: nbf, exp, iss, sub, email, name, premium
    pub token_type: Option<String>,   // Bearer
    pub last_saved: Option<String>,
    pub expires_in: Option<u32>, // in seconds
    pub error: Option<String>,
    pub error_description: Option<String>,
    pub ErrorModel: Option<BWErrorModel>,
}
