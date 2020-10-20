use std::env;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use bitwarden_service;


#[pyfunction]
pub fn get_token() -> PyResult<String> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut token: String = String::new();
    rt.block_on(async {
        let email = env::var("NXCMDR_BW_EMAIL").unwrap();
        let password = env::var("NXCMDR_BW_PASSWORD").unwrap();
        let response = bitwarden_service::get_token(&email, &password)
            .await
            .unwrap();
        token = format!("{:#?}", response)
    });
    Ok(token)
}

#[pymodule]
fn nx_core(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_token, m)?)?;

    Ok(())
}
