use std::iter::repeat;

use rand_core::{OsRng, RngCore};
use sha2::Sha256;
use hmac::{Hmac, Mac, NewMac};
use aes::Aes256;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;

use crate::models;


type HmacSha256 = Hmac<Sha256>;
type Aes256Cbc = Cbc<Aes256, Pkcs7>;


fn hkdf_expand(key: &[u8], info: &str) -> Vec<u8> {
    let mut mac = HmacSha256::new_varkey(key).expect("Could not create hmac key.");

    let mut info = Vec::from(info.as_bytes());
    info.push(1 as u8);

    mac.update(&info);

    let res = mac.finalize();

    Vec::from(res.into_bytes().as_slice())
}

fn decrypt_aes(enc_key: &Vec<u8>, iv: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    Aes256Cbc::new_var(enc_key, iv)
        .unwrap()
        .decrypt_vec(data)
        .unwrap()
}

fn encrypt_aes(enc_key: &Vec<u8>, iv: &Vec<u8>, data: &Vec<u8>) -> Vec<u8> {
    Aes256Cbc::new_var(enc_key, iv)
        .unwrap()
        .encrypt_vec(data)
}

fn check_macs(mac_key: &[u8], cipher_string: &models::CipherString) -> bool {
    let mut mac = HmacSha256::new_varkey(mac_key).expect("Could not create hmac key.");

    let mut comp_data = cipher_string.iv.clone();
    let mut cs_data = cipher_string.data.clone();
    comp_data.append(&mut cs_data);

    mac.update(&comp_data);
    let comp_mac = mac.finalize_reset().into_bytes();

    mac.update(&cipher_string.mac);
    let hmac1 = mac.finalize_reset().into_bytes();

    mac.update(comp_mac.as_slice());
    let hmac2 = mac.finalize_reset().into_bytes();

    hmac1 == hmac2
}

pub fn expand_key(key: &[u8]) -> (Vec<u8>, Vec<u8>) {
    (hkdf_expand(&key, "enc"), hkdf_expand(&key, "mac"))
}

pub fn generate_pbkdf(password: &[u8], salt: &[u8], iterations: u32) -> Vec<u8> {
    let mut output: Vec<u8> = repeat(0).take(32).collect();

    pbkdf2::pbkdf2::<HmacSha256>(
        password, salt, iterations, &mut output);

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
    let mut hmac = HmacSha256::new_varkey(&key.mac).expect("Could not create hmac key.");

    let mut iv = [0u8; 16];
    OsRng.fill_bytes(&mut iv);
    let iv = Vec::from(iv);

    let data = encrypt_aes(&key.key, &iv, data);
    let mut mac_data = iv.clone();
    mac_data.append(&mut data.clone());

    hmac.update(&mac_data);
    let mac = hmac.finalize().into_bytes();
    let mac = Vec::from(mac.as_slice());

    models::CipherString {
        enc_type: 2,
        mac,
        iv,
        data,
        raw: None
    }
}