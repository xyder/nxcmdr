use std::iter::repeat;

// todo: remove either ring or openssl
use ring::{pbkdf2, hmac};
use openssl::symm::{decrypt, Cipher};

use crate::models;


fn hkdf_expand(key: &[u8], info: &str) -> Vec<u8> {
    // simulates one pass of HKDF SHA256 expand step
    let s_key = hmac::Key::new(hmac::HMAC_SHA256, key.as_ref());

    let mut info = Vec::from(info.as_bytes());
    info.push(1 as u8);

    let res = hmac::sign(&s_key, &info);

    Vec::from(res.as_ref())
}

fn decrypt_aes(enc_key: &Vec<u8>, iv: Vec<u8>, data: Vec<u8>) -> Vec<u8> {
    let plain = decrypt(
        Cipher::aes_256_cbc(),
        enc_key,
        Some(&iv),
        &data)
        .unwrap();

    plain
}

fn check_macs(mac_key: &[u8], cipher_string: models::CipherString) -> bool {
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

pub fn decrypt_cipher_string(key: &models::SymmetricKey, cipher_string: models::CipherString) -> Vec<u8> {

    if !check_macs(&key.mac, cipher_string.clone()) {
        panic!("Could not decrypt cipher string.")
    };

    decrypt_aes(&key.key, cipher_string.clone().iv, cipher_string.clone().data)
}
