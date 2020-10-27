use reqwest;
use chrono::{Utc, TimeZone};

use security::models as sec_models;

use crate::{constants::*, models};


pub async fn get_full_sync(token: &models::TokenResponse) -> Result<models::SyncResponse, reqwest::Error> {
    let rev_date = get_revision_date(token).await?;

    let client = reqwest::Client::new();
    let mut res = client
        .get(&crate::SYNC_URL!())
        .bearer_auth(token.access_token.as_ref().unwrap())
        .send()
        .await?
        .json::<models::SyncResponse>()
        .await?;

    res.rev_date = Some(rev_date);

    Ok(res)
}

pub async fn get_revision_date(token: &models::TokenResponse) -> Result<chrono::DateTime<Utc>, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(&crate::REVISION_URL!())
        .bearer_auth(token.access_token.as_ref().unwrap())
        .send()
        .await?
        .json::<i64>()
        .await?;

    Ok(Utc.timestamp(res / 1000, 0))
}

pub async fn get_iterations(email: &str) -> Result<u32, reqwest::Error> {
    let client = reqwest::Client::new();
    let payload = models::IterationsRequest {
        email: email.into()
    };

    let res = client
        .post(&crate::PRELOGIN_URL!())
        .json(&payload)
        .send()
        .await?
        .json::<models::PreLoginResponse>()
        .await?;

    match res.error.clone() {
        // todo: bubble error upstream
        Some(_) => panic!(format!("Error received: {:#?}", res)),
        _ => ()
    };

    Ok(res.iterations.unwrap())
}

pub async fn get_new_token(
    email: &str,
    master_key: &sec_models::MasterKey,
    tfa_code: &str,
) -> Result<models::TokenResponse, reqwest::Error> {
    let client = reqwest::Client::new();

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
        two_factor_token: tfa_code.parse::<u32>().unwrap(),
        two_factor_provider: 0,
        two_factor_remember: 0,
    };

    let mut res = client
        .post(&crate::TOKEN_URL!())
        .form(&payload)
        .send()
        .await?
        .json::<models::TokenResponse>()
        .await?;

    res.last_saved = Some(chrono::offset::Local::now().to_string());
    res.master_key = Some(base64::encode(master_key.key.clone()));

    match res.error.clone() {
        // todo: bubble error upstream
        Some(_) => panic!(format!("Error received: {:#?}", res)),
        _ => ()
    };

    Ok(res)
}

pub async fn refresh_token(
    token: &mut models::TokenResponse,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let payload = models::RefreshTokenRequest {
        grant_type: "refresh_token".into(),
        client_id: "web".into(),
        refresh_token: token.refresh_token.clone().unwrap().into(),
    };

    let mut res = client
        .post(&crate::TOKEN_URL!())
        .form(&payload)
        .send()
        .await?
        .json::<models::TokenResponse>()
        .await?;

    match res.error.clone() {
        // todo: bubble error upstream
        Some(_) => panic!(format!("Error received: {:#?}", res)),
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
