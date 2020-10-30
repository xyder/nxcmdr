use std::iter::repeat;

// todo: remove either ring or openssl, and maybe rand_core
use rand_core::{OsRng, RngCore};
use ring::{pbkdf2, hmac};
use openssl::symm::{decrypt, encrypt, Cipher};

use crate::models;


fn hkdf_expand(key: &[u8], info: &str) -> Vec<u8> {
    // simulates one pass of HKDF SHA256 expand step
    let s_key = hmac::Key::new(hmac::HMAC_SHA256, key.as_ref());

    let mut info = Vec::from(info.as_bytes());
    info.push(1 as u8);

    let res = hmac::sign(&s_key, &info);

    Vec::from(res.as_ref())
}

fn decrypt_aes(enc_key: &Vec<u8>, iv: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    decrypt(
        Cipher::aes_256_cbc(),
        enc_key,
        Some(iv),
        data)
        .unwrap()
}

fn encrypt_aes(enc_key: &Vec<u8>, iv: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    encrypt(
        Cipher::aes_256_cbc(),
        enc_key,
        Some(iv),
        data
    ).unwrap()
}

fn check_macs(mac_key: &[u8], cipher_string: &models::CipherString) -> bool {
    let s_key = hmac::Key::new(hmac::HMAC_SHA256, mac_key);

    let mut comp_data = cipher_string.iv.clone();
    let mut cs_data = cipher_string.data.clone();
    comp_data.append(&mut cs_data);

    let comp_mac = hmac::sign(&s_key, &comp_data);
    let comp_mac = comp_mac.as_ref();
    let hmac1 = hmac::sign(&s_key, &cipher_string.mac);
    let hmac1 = hmac1.as_ref();
    let hmac2 = hmac::sign(&s_key, comp_mac);
    let hmac2 = hmac2.as_ref();

    hmac1 == hmac2
}

pub fn expand_key(key: &[u8]) -> (Vec<u8>, Vec<u8>) {
    (hkdf_expand(&key, "enc"), hkdf_expand(&key, "mac"))
}

pub fn generate_pbkdf(password: &[u8], salt: &[u8], iterations: u32) -> Vec<u8> {
    let mut output: Vec<u8> = repeat(0).take(32).collect();

    let nz_iterations =
        std::num::NonZeroU32::new(iterations)
        .expect("Non-zero number of iterations expected.");

    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        nz_iterations,
        salt,
        password,
        &mut output,
    );

    output
}

pub fn decrypt_cipher_string(
        key: &models::SymmetricKey, cipher_string: models::CipherString)
        -> models::BoxedResult<Vec<u8>> {

    if !check_macs(&key.mac, &cipher_string) {
        return Err("Decrypt failed".into());
    };

    Ok(decrypt_aes(&key.key, &cipher_string.iv, &cipher_string.data))
}

pub fn encrypt_cipher_string(key: &models::SymmetricKey, data: &Vec<u8>) -> models::CipherString {
    let s_key = hmac::Key::new(hmac::HMAC_SHA256, &key.mac);

    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);
    let iv = Vec::from(iv);

    let data = encrypt_aes(&key.key, &iv, data);
    let mut mac_data = iv.clone();
    mac_data.append(&mut data.clone());

    let mac = hmac::sign(&s_key, &mac_data);
    let mac = Vec::from(mac.as_ref());

    models::CipherString {
        enc_type: 2,
        mac,
        iv,
        data,
        raw: None
    }
}