use std::collections::HashMap;
use std::io;
// bring flush() into scope
use std::io::Write;
use std::fs::File;

use serde_json::json;

mod models;
mod constants;

use security::build_password;
use constants::*;

use models::{PreLoginResponse, TokenResponse};


fn get_prelogin_url() -> String {
    format!("{base}/accounts/prelogin", base = BW_API_BASE)
}

fn get_token_url() -> String {
    format!("{base}/connect/token", base = BW_IDENTITY_BASE)
}

async fn get_iterations(client: &reqwest::Client, email: &str) -> Result<u32, reqwest::Error> {
    let mut payload = HashMap::new();
    payload.insert("email", email);

    let res = client
        .post(&get_prelogin_url())
        .json(&payload)
        .send()
        .await?;

    let json_content = res.json::<PreLoginResponse>().await?;

    match json_content.error.clone() {
        Some(_) => panic!(format!("Error received: {:#?}", json_content)),  // todo: bubble error upstream
        _ => ()
    };

    Ok(json_content.KdfIterations)
}

async fn get_new_token(
    client: &reqwest::Client,
    email: &str,
    password: &str,
    tfa_code: &str,
) -> Result<TokenResponse, reqwest::Error> {
    let iterations = get_iterations(&client, &email).await?;
    let password = build_password(&email, &password, iterations);


    let payload = json!({
        "grant_type": "password",
        "username": email,
        "password": password,
        "scope": "api offline_access",
        "client_id": "web",
        "deviceType": 10,
        "deviceIdentifier": "7d52408d-883d-4ed1-8dbb-fc6ff1a16c38",
        "deviceName": "firefox",
        "twoFactorToken": tfa_code,
        "twoFactorProvider": 0,  // 1 for email, 0 for TOTP
        "twoFactorRemember": 0  // set to 1 to remember two factor for this device
    });

    let res = client
        .post(&get_token_url())
        .form(&payload)
        .send()
        .await?;

    let mut token_content = res.json::<TokenResponse>().await?;

    token_content.last_saved = Some(chrono::offset::Local::now().to_string());

    match token_content.error.clone() {
        Some(_) => panic!(format!("Error received: {:#?}", token_content)),  // todo: bubble error upstream
        _ => ()
    };

    println!("{:?}", token_content);

    Ok(token_content)
}

async fn refresh_token(
    client: &reqwest::Client,
    refresh_token: &str,
) -> Result<TokenResponse, reqwest::Error> {
    let payload = json!({
        "grant_type": "refresh_token",
        "client_id": "web",
        "refresh_token": refresh_token
    });

    let res = client.post(&get_token_url()).form(&payload).send().await?;

    let mut token_content = res.json::<TokenResponse>().await?;

    token_content.last_saved = Some(chrono::offset::Local::now().to_string());

    match token_content.error.clone() {
        Some(_) => panic!(format!("Error received: {:#?}", token_content)),  // todo: bubble error upstream
        _ => ()
    };

    Ok(token_content)
}

fn read_tfa_code() -> String {
    print!("Enter the 2FA code: ");
    io::stdout().flush().unwrap();

    let mut tfa_code = String::new();
    io::stdin()
        .read_line(&mut tfa_code)
        .expect("Could not read 2FA code.");
    tfa_code.retain(|c| !c.is_whitespace());

    tfa_code
}

pub async fn get_token(
    email: &str,
    password: &str,
) -> Result<TokenResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let path = TOKEN_FILE;

    let token_content: TokenResponse = match File::open(path) {
        Ok(file) => match serde_json::from_reader::<_, TokenResponse>(&file) {
            Ok(content) => {
                println!("Successfully read from file.");

                let last_saved = content
                    .last_saved
                    .as_ref()
                    .unwrap()
                    .parse::<chrono::DateTime<chrono::offset::Local>>()
                    .unwrap();

                let duration = chrono::Duration::seconds(content.expires_in.unwrap().into());

                match last_saved + duration > chrono::offset::Local::now() {
                    true => content,

                    // token expired. fetching a new one
                    false => {
                        println!("Token expired. Refreshing token ..");

                        refresh_token(&client, &content.refresh_token.unwrap()).await?
                    }
                }
            }

            // could not read json
            Err(_) => {
                println!("Could not parse token json file. Re-fetching ..");
                let tfa_code = read_tfa_code();
                get_new_token(&client, email, password, &tfa_code).await?
            }
        },

        // could not open file or file does not exist
        Err(_) => {
            println!("Could not read token json file. Re-fetching ..");
            let tfa_code = read_tfa_code();
            get_new_token(&client, email, password, &tfa_code).await?
        }
    };

    match token_content.error.clone() {
        Some(_) => panic!(format!("Error received: {:#?}", token_content)),  // todo: bubble error upstream
        _ => ()
    };

    serde_json::to_writer_pretty(
        &File::create(TOKEN_FILE).expect("Could not create file"),
        &token_content
    ).expect("Could not save token.");

    Ok(token_content)
}
