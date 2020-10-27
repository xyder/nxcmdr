use std::path::Path;
use std::env;

use crate::{models::{self, BoxedResult}, service, store};


async fn needs_sync(token: &models::TokenResponse, data: &models::SyncResponse) -> BoxedResult<bool> {
    Ok(match data.rev_date {
        Some(v) => v < service::get_revision_date(token).await?,
        None => true
    })
}

pub async fn load_data(token: &models::TokenResponse) -> BoxedResult<models::SyncResponse> {
    let path = Path::new(
        &env::var("NXCMDR_CONFIG_DIR")
        .unwrap_or("~/.config/nxcmdr".to_string())
    ).join("data.json");

    let initial = store::load_stored::<models::SyncResponse>(&path).ok();
    let mut data: models::SyncResponse;

    match initial {
        Some(v) => data = v,
        None => {
            data = service::get_full_sync(&token).await?.into();
            return Ok(data);
        }
    }

    if needs_sync(&token, &data).await? {
        data = service::get_full_sync(&token).await?;
        store::store_data(&path, &data)?;
    }

    Ok(data)
}
