use std::iter::repeat;

use ring::pbkdf2;


fn generate_pbkdf(password: &[u8], salt: &[u8], iterations: u32) -> Vec<u8> {
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

pub fn build_password(email: &str, password: &str, iterations: u32) -> String {
    // derive master password using email as salt
    let e_password = generate_pbkdf(
        password.as_bytes(),
        email.as_bytes(),
        iterations);

    // run one iteration of deriving with the master password as salt
    let d_password = generate_pbkdf(
        e_password.as_slice(),
        password.as_bytes(),
        1);

    base64::encode(d_password)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
