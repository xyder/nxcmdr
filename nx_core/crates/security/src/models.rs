use std::string::FromUtf8Error;

use base64;
use serde::{Serialize, Deserialize};

use crate::crypt;


#[derive(Clone, Debug)]
pub struct CipherString {
    pub enc_type: i32,
    pub mac: Vec<u8>,
    pub iv: Vec<u8>,
    pub data: Vec<u8>,
    pub raw: String
}

impl From<&str> for CipherString {
    fn from(cipher_string: &str) -> Self {
        let raw = cipher_string.to_string();
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

impl Serialize for CipherString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.raw)
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
    fn decrypt(&self, key: T) -> Vec<u8>;

    fn decrypt_string(&self, key: T) -> Result<String, FromUtf8Error> {
        Ok(String::from_utf8(self.decrypt(key))?)
    }
}

impl Decrypt<&MasterKey> for CipherString {
    fn decrypt(&self, key: &MasterKey) -> Vec<u8> {
        crypt::decrypt_cipher_string(
            &SymmetricKey::from(key), self.clone())
    }
}

impl Decrypt<&SymmetricKey> for CipherString {
    fn decrypt(&self, key: &SymmetricKey) -> Vec<u8> {
        crypt::decrypt_cipher_string(
            key, self.clone())
    }
}

impl Decrypt<&Vec<u8>> for CipherString {
    fn decrypt(&self, key: &Vec<u8>) -> Vec<u8> {
        let key = SymmetricKey::from(key.clone());

        crypt::decrypt_cipher_string(
            &key, self.clone())
    }
}
