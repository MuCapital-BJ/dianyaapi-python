use crate::{
    py_types::{SessionCloseResult, SessionCreateResult},
    types::parse_model,
};
use common::Error;
use pyo3::prelude::*;
use std::{pin::Pin, sync::Arc, time::Duration};
use stream_cancel::Valved;
use tokio::sync::Mutex;
use tokio_stream::{Stream, StreamExt};
use transcribe::transcribe::{TranscribeWs, close_session, create_session};
use tungstenite::{Message, Utf8Bytes};

#[pyclass]
pub struct TranscribeStream {
    ws: Arc<Mutex<TranscribeWs>>,
    stream: Arc<Mutex<Valved<Pin<Box<dyn Stream<Item = Utf8Bytes> + Send>>>>>,
}

#[pymethods]
impl TranscribeStream {
    #[new]
    pub fn new(session_id: String) -> PyResult<Self> {
        let mut ws = TranscribeWs::new(&session_id);
        let stream = ws.subscribe()?;
        Ok(Self {
            ws: Arc::new(Mutex::new(ws)),
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    #[staticmethod]
    pub fn create_session<'py>(
        py: Python<'py>,
        model: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let model: String = model.extract()?;
        let model = parse_model(&model)?;
        let token: String = token.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = create_session(model, &token).await?;
            Ok(SessionCreateResult::from(response))
        })
    }

    #[staticmethod]
    pub fn close_session<'py>(
        py: Python<'py>,
        task_id: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
        timeout_seconds: Option<u64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let task_id: String = task_id.extract()?;
        let token: String = token.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = close_session(&task_id, &token, timeout_seconds).await?;
            Ok(SessionCloseResult::from(response))
        })
    }

    pub fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let ws = self.ws.clone();
        // let stream = self.stream.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut ws_guard = ws.lock().await;
            ws_guard.start().await?;
            // let mut stream_guard = stream.lock().await;
            // let stream = ws_guard.subscribe().await?;
            // *stream_guard = Some(stream);
            Ok(())
        })
    }

    pub fn stop<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let ws = self.ws.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = ws.lock().await;
            guard.stop();
            Ok(())
        })
    }

    pub fn send_text<'py>(
        &self,
        py: Python<'py>,
        message: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let ws = self.ws.clone();
        let payload: String = message.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = ws.lock().await;
            guard.write(Message::Text(payload.into())).await?;
            Ok(())
        })
    }

    pub fn send_bytes<'py>(
        &self,
        py: Python<'py>,
        data: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let ws = self.ws.clone();
        let payload: Vec<u8> = match data.extract() {
            Ok(bytes) => bytes,
            Err(err) => {
                return Err(Error::InvalidInput(format!("data must be bytes-like: {err}")).into());
            }
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut guard = ws.lock().await;
            guard.write(Message::Binary(payload.into())).await?;
            Ok(())
        })
    }

    pub fn read_next<'py>(
        &self,
        py: Python<'py>,
        timeout: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let stream = self.stream.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let duration = timeout
                .filter(|value| *value >= 0.0)
                .map(Duration::from_secs_f64);

            let mut guard = stream.lock().await;
            if let Some(duration) = duration {
                match tokio::time::timeout(duration, guard.next()).await {
                    Ok(Some(message)) => Ok(Some(message.to_string())),
                    _ => Ok(None),
                }
            } else {
                match guard.next().await {
                    Some(message) => Ok(Some(message.to_string())),
                    None => Ok(None),
                }
            }
        })
    }
}
