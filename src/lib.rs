//! lib.rs

use pyo3::prelude::*;
use std::collections::HashMap;
use pyo3::exceptions;
use std::time::Duration;

#[pyclass]
struct Client {
    client: reqwest::Client,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new(timeout: u64, accept_invalid_certs: bool) -> PyResult<Self> {
        let client = match reqwest::Client::builder()
            .use_rustls_tls()
            .tls_built_in_root_certs(true)
            .danger_accept_invalid_certs(accept_invalid_certs)
            .http2_prior_knowledge()
            .timeout(Duration::from_secs(timeout))
            .build() {
            Ok(t) => t,
            Err(e) => return Err(PyErr::new::<exceptions::PyValueError, _>(
                format!("Failed to create http2 client: {}", e.to_string()))),
        };

        Ok(Client {
            client
        })
    }

    pub fn get(&self, py: Python, url: String, params: HashMap<String, String>) -> PyResult<PyObject> {
        let client = self.client.clone();

        pyo3_asyncio::tokio::into_coroutine(py, async move {
            let res = client.get(url)
                .query(&params)
                .send()
                .await;

            let resp = match res {
                Ok(t) => t,
                Err(e) => return Err(PyErr::new::<exceptions::PyValueError, _>(
                    format!("Request send failed: {}", e.to_string()))),
            };
            assert_eq!(resp.version(), reqwest::Version::HTTP_2);

            let body = match resp.text().await {
                Ok(t) => t,
                Err(e) => return Err(PyErr::new::<exceptions::PyValueError, _>(
                    format!("Request data read failed: {}", e.to_string()))),
            };

            Ok(Python::with_gil(|py| body.into_py(py)))
        })
    }
}


#[pymodule]
fn async_http2(py: Python, m: &PyModule) -> PyResult<()> {
    pyo3_asyncio::try_init(py)?;
    // Tokio needs explicit initialization before any pyo3-asyncio conversions.
    // The module import is a prime place to do this.
    pyo3_asyncio::tokio::init_multi_thread_once();

    m.add_class::<Client>()?;

    Ok(())
}
