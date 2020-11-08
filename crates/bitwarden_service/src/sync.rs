use std::path::Path;

use crate::{constants, models::{self, BoxedResult}, service, store};


fn needs_sync(token: &models::TokenResponse, data: &models::SyncResponse) -> BoxedResult<bool> {
    Ok(match data.rev_date {
        Some(v) => v < service::get_revision_date(token)?,
        None => true
    })
}

pub fn load_data(token: &models::TokenResponse) -> BoxedResult<models::SyncResponse> {
    let config = models::Config::load(false);
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

    if needs_sync(&token, &data)? {
        data = service::get_full_sync(&token)?;
        store::store_data(&path, &data)?;
    }

    Ok(data)
}
