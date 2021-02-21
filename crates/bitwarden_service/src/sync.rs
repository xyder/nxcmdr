use std::path::Path;

use anyhow::Result;

use crate::{constants, models, service, store, utils::process_conn_errors};



fn needs_sync(token: &models::TokenResponse, data: &models::SyncResponse) -> Result<bool> {
    Ok(match data.rev_date {
        Some(v) => v < service::get_revision_date(token)?,
        None => true
    })
}

pub fn load_data(token: &models::TokenResponse, ignore_conn_errors: bool, quiet: bool) -> anyhow::Result<models::SyncResponse> {
    let config = models::Config::load(false)?;
    let path = Path::new(&config.config_dir)
        .join(constants::DATA_FILENAME);

    let initial = store::load_stored::<models::SyncResponse>(&path).ok();
    let mut data: models::SyncResponse;

    match initial {
        Some(v) => data = v,
        None => {
            data = service::get_full_sync(&token)?.into();
            store::store_data(&path, &data)?;
            return Ok(data);
        }
    }

    if process_conn_errors(
            needs_sync(&token, &data), false, ignore_conn_errors, quiet)? {
        data = service::get_full_sync(&token)?;
        store::store_data(&path, &data)?;
    }

    Ok(data)
}
