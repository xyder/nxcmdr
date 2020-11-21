use base64;
use serde::{Serialize, Deserialize};
use rand_core::{OsRng, RngCore};

use anyhow::{Context, Result};

use crate::crypt;

#[derive(Clone, Debug)]
pub struct CipherString {
    pub enc_type: i32,
    pub mac: Vec<u8>,
    pub iv: Vec<u8>,
    pub data: Vec<u8>,
    pub raw: Option<String>
}

// need this to implement own traits
pub struct StringWrapper(String);

impl From<StringWrapper> for String {
    fn from(input: StringWrapper) -> Self {
        input.0
    }
}

impl From<&StringWrapper> for String {
    fn from(input: &StringWrapper) -> Self {
        input.0.clone()
    }
}

impl From<&str> for StringWrapper {
    fn from(input: &str) -> Self {
        Self(input.to_string())
    }
}

impl From<&StringWrapper> for Result<CipherString> {
    fn from(cipher_string: &StringWrapper) -> Self {
        let cipher_string: String = cipher_string.into();
        let raw = Some(cipher_string.clone());

        let mut parts = cipher_string.split('.');
        let enc_type = parts.next().unwrap_or("2");
        let enc_type = enc_type.parse::<i32>().unwrap_or(2);

        parts = parts.next()
            .context("Missing composite data part on CipherString")?
            .split('|');

        let iv = base64::decode(parts
                .next()
                .context("Missing iv part on CipherString")?
            )
            .context("Could not decode iv on CipherString")?;

        let data = base64::decode(parts
                .next()
                .context("Missing data part on CipherString")?
            )
            .context("Could not decode data on CipherString")?;

        let mac = base64::decode(parts
                .next()
                .context("Missing mac part on CipherString")?
            )
            .context("Could not decode mac on CipherString")?;

        Ok(CipherString { enc_type, iv, data, mac, raw })
    }
}

impl ToString for CipherString {
    fn to_string(&self) -> String {
        match &self.raw {
            Some(v) => v.clone(),
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
        match &self.raw {
            Some(v) => serializer.serialize_some(v),
            None => serializer.serialize_none()
        }
    }
}

struct CipherStringVisitor;

impl<'de> serde::de::Visitor<'de> for CipherStringVisitor {
    type Value = CipherString;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where E: serde::de::Error {
        let v_wrapper = StringWrapper::from(v);
        let cs: Result<CipherString> = (&v_wrapper).into();
        // let cs = cs?;
        match cs {
            Ok(v) => Ok(v),
            Err(_) => Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self))
        }
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
    pub fn encrypt(&self, data: &Vec<u8>) -> Result<CipherString> {
        Ok(crypt::encrypt_cipher_string(self, data)?)
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

impl From<&Vec<u8>> for SymmetricKey {
    fn from(input: &Vec<u8>) -> Self {
        input.into()
    }
}

impl From<MasterKey> for Result<SymmetricKey> {
    fn from(input: MasterKey) -> Self {
        let (key, mac) = crypt::expand_key(&input.key)?;

        Ok(SymmetricKey { key, mac })
    }
}

impl From<&MasterKey> for Result<SymmetricKey> {
    fn from(input: &MasterKey) -> Self {
        let (key, mac) = crypt::expand_key(&input.key)?;

        Ok(SymmetricKey { key, mac })
    }
}


pub trait Decrypt<T> {
    fn decrypt(&self, key: T) -> anyhow::Result<Vec<u8>>;

    fn decrypt_string(&self, key: T) -> anyhow::Result<String> {
        Ok(String::from_utf8(self.decrypt(key)?)?)
    }
}

impl Decrypt<&MasterKey> for CipherString {
    fn decrypt(&self, key: &MasterKey) -> anyhow::Result<Vec<u8>> {
        let key: Result<SymmetricKey> = key.into();
        crypt::decrypt_cipher_string(
            &key?, self)
    }
}

impl Decrypt<&SymmetricKey> for CipherString {
    fn decrypt(&self, key: &SymmetricKey) -> anyhow::Result<Vec<u8>> {
        crypt::decrypt_cipher_string(
            key, self)
    }
}

impl Decrypt<&Vec<u8>> for CipherString {
    fn decrypt(&self, key: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let key = SymmetricKey::from(key);

        crypt::decrypt_cipher_string(
            &key, self)
    }
}
