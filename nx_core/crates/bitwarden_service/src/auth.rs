use std::path::Path;

use security::models as sec_models;

use crate::{constants, models::{self, BoxedResult, Config}, service, store, utils::read_stdin};


async fn get_new_token() -> BoxedResult<models::TokenResponse> {
    let email = read_stdin("Bitwarden email: ");
    let iterations = service::get_iterations(&email).await?;

    let password = rpassword::prompt_password_stdout("Bitwarden password: ")?;
    let tfa_code = read_stdin("Enter TFA code: ");

    let master_key = sec_models::MasterKey::from(
        &sec_models::Credentials {
            email: email.to_string(), password: password.to_string(), iterations});

    Ok(service::get_new_token(&email, &master_key, &tfa_code).await?)
}

fn need_refresh(token: &models::TokenResponse) -> bool {
    let last_saved = token
        .last_saved.clone().unwrap()
        .parse::<chrono::DateTime<chrono::offset::Local>>().unwrap();

    let duration = chrono::Duration::seconds(
        token.expires_in.unwrap().into());

    last_saved + duration <= chrono::offset::Local::now()
}

pub async fn get_token() -> BoxedResult<models::TokenResponse> {
    let config = Config::load();
    let path = Path::new(&config.config_dir)
        .join(constants::TOKEN_FILENAME);

    let mut data = store::load_stored(&path).ok();

    let mut do_write = false;

    if data.is_none() {
        println!("Could not read file: {}", path.to_str().unwrap_or("<unknown>"));
        data = Some(get_new_token().await?);

        do_write = true;
    }

    let mut data = data.unwrap(); // safe unwrap

    // if this was a new token, it won't be refreshed
    if need_refresh(&data) {
        println!("Token expired. Refreshing token ..");
        service::refresh_token(&mut data).await?;

        do_write = true;
    }

    // at this point we have a valid token. we write it if it was modified
    if do_write {
        store::store_data(&path, &data).expect("Could not save token.");
    }

    Ok(data)
}
