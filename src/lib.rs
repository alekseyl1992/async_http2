//! lib.rs

use pyo3::prelude::*;
use std::collections::HashMap;
use pyo3::exceptions;
use pyo3::create_exception;
use std::time::Duration;
use reqwest::Error;
use std::error::Error as StdError;

#[pyclass]
struct Client {
    client: reqwest::Client,
}

#[pymethods]
impl Client {
    #[new]
    pub fn new(py: Python, timeout: u64) -> PyResult<Self> {
        pyo3_asyncio::try_init(py)?;
        pyo3_asyncio::tokio::init_multi_thread_once();

        let client = match reqwest::Client::builder()
            .use_rustls_tls()
            .http2_prior_knowledge()
            .timeout(Duration::from_secs(timeout))
            .build() {
            Ok(t) => t,
            Err(e) => return Err(make_exception(
                "failed to create http2 client", e)),
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
                Err(e) => return Err(make_exception(
                    "request send failed", e)),
            };
            assert_eq!(resp.version(), reqwest::Version::HTTP_2);

            let body = match resp.text().await {
                Ok(t) => t,
                Err(e) => return Err(make_exception(
                    "request data read failed", e)),
            };

            Ok(Python::with_gil(|py| body.into_py(py)))
        })
    }
}

fn make_exception(desc: &str, e: Error) -> PyErr {
    let source_desc = match e.source() {
        Some(s) => s.to_string(),
        None => "".to_string(),
    };
    let msg = format!("[{}] {}: {}", source_desc, desc, e.to_string());

    // check if it is NO_ERROR(0) error
    // see:
    // https://github.com/seanmonstar/reqwest/issues/1276
    // https://github.com/hyperium/hyper/issues/2500
    let no_error = match find_source::<h2::Error>(&e) {
        Some(h2_error) => match h2_error.reason() {
            Some(reason) => u32::from(reason) == 0,
            None => false,
        },
        None => false,
    };
    if no_error {
        return PyErr::new::<RetryError, _>(msg);
    }

    if e.is_timeout() {
        PyErr::new::<TimeoutError, _>(msg)
    } else if e.is_connect() {
        PyErr::new::<ConnectError, _>(msg)
    } else if e.is_request() {
        PyErr::new::<RequestError, _>(msg)
    } else if e.is_status() {
        PyErr::new::<BadStatusCodeError, _>(msg)
    } else if e.is_body() {
        PyErr::new::<BodyError, _>(msg)
    } else if e.is_builder() {
        PyErr::new::<BuilderError, _>(msg)
    } else if e.is_decode() {
        PyErr::new::<DecodeError, _>(msg)
    } else if e.is_redirect() {
        PyErr::new::<RedirectError, _>(msg)
    } else {
        PyErr::new::<exceptions::PyException, _>(msg)
    }
}

fn find_source<E: StdError + 'static>(e: &Error) -> Option<&E> {
    let mut cause = e.source();
    while let Some(err) = cause {
        if let Some(ref typed) = err.downcast_ref() {
            return Some(typed);
        }
        cause = err.source();
    }

    // else
    None
}

create_exception!(async_http2, TimeoutError, pyo3::exceptions::PyTimeoutError);
create_exception!(async_http2, ConnectError, pyo3::exceptions::PyException);
create_exception!(async_http2, RequestError, pyo3::exceptions::PyException);
create_exception!(async_http2, BadStatusCodeError, pyo3::exceptions::PyException);
create_exception!(async_http2, BodyError, pyo3::exceptions::PyException);
create_exception!(async_http2, BuilderError, pyo3::exceptions::PyException);
create_exception!(async_http2, DecodeError, pyo3::exceptions::PyException);
create_exception!(async_http2, RedirectError, pyo3::exceptions::PyException);
create_exception!(async_http2, RetryError, pyo3::exceptions::PyException);

#[pymodule]
fn async_http2(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add("TimeoutError", py.get_type::<TimeoutError>())?;
    m.add("ConnectError", py.get_type::<ConnectError>())?;
    m.add("RequestError", py.get_type::<RequestError>())?;
    m.add("BadStatusCodeError", py.get_type::<BadStatusCodeError>())?;
    m.add("BodyError", py.get_type::<BodyError>())?;
    m.add("BuilderError", py.get_type::<BuilderError>())?;
    m.add("DecodeError", py.get_type::<DecodeError>())?;
    m.add("RedirectError", py.get_type::<RedirectError>())?;
    m.add("RetryError", py.get_type::<RetryError>())?;

    Ok(())
}
