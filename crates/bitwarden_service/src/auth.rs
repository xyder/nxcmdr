use std::path::Path;

use anyhow::{Context, bail, Result};
use security::models as sec_models;

use crate::{constants, models::{self, Config}, service, store, utils::read_from_stdin};


fn get_new_token(config: &Config) -> anyhow::Result<models::TokenResponse> {
    let email = read_from_stdin(&config.bw_user, "Bitwarden email: ", false)?;
    if email == "" {
        bail!("Email was not provided.");
    }

    let iterations = service::get_iterations(&email)?;

    let password = read_from_stdin(&config.bw_pass, "Bitwarden password: ", true)?;
    if password == "" {
        bail!("Password was not provided.");
    }

    let master_key = sec_models::MasterKey::from(
        &sec_models::Credentials {
            email: email.to_string(), password: password.to_string(), iterations});

    let tfa_code = read_from_stdin(&config.bw_tfa, "TFA code: ", false)?;
    if tfa_code == "" {
        bail!("TFA code was not provided.");
    }

    Ok(service::get_new_token(&email, &master_key, &tfa_code)?)
}

fn need_refresh(token: &models::TokenResponse) -> Result<bool> {
    let last_saved = match &token.last_saved {
        Some(v) => v.parse::<chrono::DateTime<chrono::offset::Local>>()?,
        None => bail!("Could not find last saved time on token response")
    };


    let duration = chrono::Duration::seconds(
        match &token.expires_in {
            Some(v) => v.clone().into(),
            None => bail!("Could not find expired time on token response")
        });

    Ok(last_saved + duration <= chrono::offset::Local::now())
}

pub fn get_token(quiet: bool) -> anyhow::Result<models::TokenResponse> {
    let config = Config::load(false)?;
    let path = Path::new(&config.config_dir)
        .join(constants::TOKEN_FILENAME);

    let data = store::load_stored(&path).ok();

    let mut do_write = false;

    let mut data = match data {
        Some(v) => v,
        None => {
            if !quiet {
                println!("Could not read file: {}", path.to_str().unwrap_or("<unknown>"));
            }
            Config::load(true)?;
            do_write = true;
            get_new_token(&config)?
        }
    };

    // if this was a new token, it won't be refreshed
    if need_refresh(&data)? {
        if !quiet {
            println!("Token expired. Refreshing token ..");
        }
        service::refresh_token(&mut data)?;

        do_write = true;
    }

    // at this point we have a valid token. we write it if it was modified
    if do_write {
        store::store_data(&path, &data)
            .context("Could not save token")?;
    }

    Ok(data)
}
