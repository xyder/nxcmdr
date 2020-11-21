use reqwest;
use chrono::{Utc, TimeZone};
use anyhow::{Context, Result, anyhow, bail};

use security::models as sec_models;

use crate::{constants::*, models};

fn make_get_request(url: &str, token: &models::TokenResponse) -> Result<reqwest::blocking::Response> {
    let client = reqwest::blocking::Client::new();
    let access_token = match &token.access_token {
        Some(v) => v,
        None => bail!("Could not get access token from token response")
    };

    Ok(client
        .get(url)
        .bearer_auth(access_token)
        .send()
        .context(format!("Request failed: {}", url))?)
}

pub fn get_full_sync(token: &models::TokenResponse) -> Result<models::SyncResponse> {
    let rev_date = get_revision_date(token)?;
    let mut res = make_get_request(&crate::SYNC_URL!(), &token)?
        .json::<models::SyncResponse>()?;

    res.rev_date = Some(rev_date);

    Ok(res)
}

pub fn get_revision_date(token: &models::TokenResponse) -> Result<chrono::DateTime<Utc>> {
    let res = make_get_request(&crate::REVISION_URL!(), token)?
        .json::<i64>()?;

    Ok(Utc.timestamp(res / 1000, 0))
}

pub fn get_iterations(email: &str) -> Result<u32> {
    let payload = models::IterationsRequest {
        email: email.to_string()
    };

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(&crate::PRELOGIN_URL!())
        .json(&payload)
        .send()?
        .json::<models::PreLoginResponse>()?;

    match &res.error {
        Some(e) => bail!("Could not retrieve iterations: {}", e),
        _ => ()
    };

    res.iterations.ok_or(anyhow!("Did not receive iterations for email: {}", email))
}

pub fn get_new_token(
    email: &str, master_key: &sec_models::MasterKey, tfa_code: &str,
) -> Result<models::TokenResponse> {

    let client = reqwest::blocking::Client::new();

    let payload = models::TokenRequest {
        grant_type: "password".into(),
        username: email.into(),
        password: master_key.hash.clone(),
        scope: "api offline_access".into(),
        client_id: "web".into(),
        device_type: 10,
        // todo: fetch from config
        device_id: uuid::Uuid::parse_str("7d52408d-883d-4ed1-8dbb-fc6ff1a16c38").unwrap(),
        device_name: "firefox".into(),
        two_factor_token: tfa_code.parse::<u32>().context("TFA code was not a number")?,
        two_factor_provider: 0,
        two_factor_remember: 0,
    };

    let mut res = client
        .post(&crate::TOKEN_URL!())
        .form(&payload)
        .send()?
        .json::<models::TokenResponse>()?;

    res.last_saved = Some(chrono::offset::Local::now().to_string());
    res.master_key = Some(base64::encode(&master_key.key));

    match &res.error_model {
        Some(e) => bail!(
            "Could not retrieve token: {}",
            match &e.message {
              Some(v) => v.clone(),
              None => "Unknown error".to_string()
            }),
        _ => ()
    };

    Ok(res)
}

pub fn refresh_token(token: &mut models::TokenResponse) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let refresh_token = match &token.refresh_token {
        Some(v) => v,
        None => bail!("Could not retrieve refresh token from token response")
    };
    let payload = models::RefreshTokenRequest {
        grant_type: "refresh_token".to_string(),
        client_id: "web".to_string(),
        refresh_token: refresh_token.into(),
    };

    let mut res = client
        .post(&crate::TOKEN_URL!())
        .form(&payload)
        .send()?
        .json::<models::TokenResponse>()?;

    match &res.error_model {
        Some(e) => bail!(
            "Could not retrieve token: {}",
            match &e.message {
                Some(v) => v.clone(),
                None => "Unknown error".to_string()
              }),
        _ => ()
    };

    res.last_saved = Some(chrono::offset::Local::now().to_string());
    token.access_token = res.access_token;
    token.expires_in = res.expires_in;
    token.token_type = res.token_type;
    token.refresh_token = res.refresh_token;
    token.scope = res.scope;
    token.last_saved = Some(chrono::offset::Local::now().to_string());

    Ok(())
}
