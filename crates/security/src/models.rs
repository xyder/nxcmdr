use base64;
use serde::{Serialize, Deserialize};
use rand_core::{OsRng, RngCore};

use crate::crypt;

pub type BoxedResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Clone, Debug)]
pub struct CipherString {
    pub enc_type: i32,
    pub mac: Vec<u8>,
    pub iv: Vec<u8>,
    pub data: Vec<u8>,
    pub raw: Option<String>
}

impl From<&str> for CipherString {
    fn from(cipher_string: &str) -> Self {
        let raw = Some(cipher_string.to_string());
        let mut parts = cipher_string.split('.');
        let enc_type = parts.next().unwrap_or("2");
        let enc_type = enc_type.parse::<i32>().unwrap_or(2);

        parts = parts.next().unwrap().split('|');
        let iv = base64::decode(parts.next().unwrap()).unwrap();
        let data = base64::decode(parts.next().unwrap()).unwrap();
        let mac = base64::decode(parts.next().unwrap()).unwrap();

        Self { enc_type, iv, data, mac, raw }
    }
}

impl ToString for CipherString {
    fn to_string(&self) -> String {
        match self.raw.clone() {
            Some(v) => v,
            None => format!(
                "{}.{}|{}|{}",
                self.enc_type,
                base64::encode(&self.iv),
                base64::encode(&self.data),
                base64::encode(&self.mac))
        }
    }
}

impl Serialize for CipherString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        match self.raw.clone() {
            Some(v) => serializer.serialize_some(&v),
            None => serializer.serialize_none()
        }
    }
}

struct CipherStringVisitor;

impl<'de> serde::de::Visitor<'de> for CipherStringVisitor {
    type Value = CipherString;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where E: serde::de::Error {
        Ok(CipherString::from(v))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a CipherString formatted str")
    }
}

impl<'de> Deserialize<'de> for CipherString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        deserializer.deserialize_string(CipherStringVisitor)
    }
}

#[derive(Clone, Debug)]
pub struct Credentials {
    pub email: String,
    pub password: String,
    pub iterations: u32
}

#[derive(Clone, Debug)]
pub struct MasterKey {
    pub key: Vec<u8>,
    pub hash: String
}

impl From<&Credentials> for MasterKey {
    fn from(input: &Credentials) -> Self {
        // derive master password using email as salt
        let key = crypt::generate_pbkdf(
            input.password.as_bytes(),
            input.email.as_bytes(),
            input.iterations);

        // run one iteration of derivation with the master password as salt
        let hash = crypt::generate_pbkdf(
            key.as_slice(),
            input.password.as_bytes(),
            1);

        let hash = base64::encode(hash);

        Self { key, hash }
    }
}

#[derive(Clone, Debug)]
pub struct SymmetricKey {
    pub mac: Vec<u8>,
    pub key: Vec<u8>
}

impl SymmetricKey {
    pub fn encrypt(&self, data: &Vec<u8>) -> CipherString {
        crypt::encrypt_cipher_string(self, data)
    }
}

impl ToString for SymmetricKey {
    fn to_string(&self) -> String {
        let mut out = self.key.clone();
        out.append(self.mac.clone().as_mut());
        base64::encode(out)
    }
}

impl From<String> for SymmetricKey {
    fn from(input: String) -> Self {
        Self::from(base64::decode(input).unwrap().to_vec())
    }
}

impl From<Option<String>> for SymmetricKey {
    fn from(input: Option<String>) -> Self {
        match input {
            Some(v) => Self::from(v),
            None => {
                let mut key = [0u8; 64];
                OsRng.fill_bytes(&mut key);
                let key = Vec::from(key);
                Self::from(key)
            }
        }

    }
}

impl From<Vec<u8>> for SymmetricKey {
    fn from(input: Vec<u8>) -> Self {
        let key = input[..32].to_vec();
        let mac = input[32..64].to_vec();

        Self { mac, key }
    }
}

impl From<&MasterKey> for SymmetricKey {
    fn from(input: &MasterKey) -> Self {
        let (key, mac) = crypt::expand_key(&input.key);

        Self { key, mac }
    }
}

pub trait Decrypt<T> {
    fn decrypt(&self, key: T) -> BoxedResult<Vec<u8>>;

    fn decrypt_string(&self, key: T) -> BoxedResult<String> {
        Ok(String::from_utf8(self.decrypt(key)?)?)
    }
}

impl Decrypt<&MasterKey> for CipherString {
    fn decrypt(&self, key: &MasterKey) -> BoxedResult<Vec<u8>> {
        crypt::decrypt_cipher_string(
            &SymmetricKey::from(key), self.clone())
    }
}

impl Decrypt<&SymmetricKey> for CipherString {
    fn decrypt(&self, key: &SymmetricKey) -> BoxedResult<Vec<u8>> {
        crypt::decrypt_cipher_string(
            key, self.clone())
    }
}

impl Decrypt<&Vec<u8>> for CipherString {
    fn decrypt(&self, key: &Vec<u8>) -> BoxedResult<Vec<u8>> {
        let key = SymmetricKey::from(key.clone());

        crypt::decrypt_cipher_string(
            &key, self.clone())
    }
}
