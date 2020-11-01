use std::path::Path;

use crate::{constants, models::{self, BoxedResult}, service, store};


async fn needs_sync(token: &models::TokenResponse, data: &models::SyncResponse) -> BoxedResult<bool> {
    Ok(match data.rev_date {
        Some(v) => v < service::get_revision_date(token).await?,
        None => true
    })
}

pub async fn load_data(token: &models::TokenResponse) -> BoxedResult<models::SyncResponse> {
    let config = models::Config::load(false);
    let path = Path::new(&config.config_dir)
        .join(constants::DATA_FILENAME);

    let initial = store::load_stored::<models::SyncResponse>(&path).ok();
    let mut data: models::SyncResponse;

    match initial {
        Some(v) => data = v,
        None => {
            data = service::get_full_sync(&token).await?.into();
            store::store_data(&path, &data)?;
            return Ok(data);
        }
    }

    if needs_sync(&token, &data).await? {
        data = service::get_full_sync(&token).await?;
        store::store_data(&path, &data)?;
    }

    Ok(data)
}
