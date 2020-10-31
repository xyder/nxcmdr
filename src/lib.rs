use std::{collections::HashMap};

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use bitwarden_service;

#[pyfunction]
pub fn get_by_name(name: &str) -> PyResult<HashMap<String, String>> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut envs: HashMap<String, String> = HashMap::new();

    rt.block_on(async {
        let token = bitwarden_service::auth::get_token()
            .await
            .expect("Could not get Bitwarden token.");
        envs = bitwarden_service::get_by_name(name, &token)
            .await
            .expect("Could not retrieve keys.");
    });
    Ok(envs)
}

#[pymodule]
fn nx_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_by_name, m)?)?;

    Ok(())
}
