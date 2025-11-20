use crate::{
    py_types::{
        CallbackResponse, ShareLinkResponse, StatusResponse, SummaryCreateResponse,
        TextTranslationResponse, TranscribeTranslationResponse, UploadResult,
        UtteranceTranslationResponse,
    },
    types::{
        extract_utterances, parse_export_format, parse_export_type, parse_language, parse_model,
    },
};
use common::Error;
use pyo3::{
    prelude::*,
    types::{PyAnyMethods, PyBytes, PyString},
};
use transcribe::{
    Utterance,
    transcribe::{
        CallbackRequest, callback as transcribe_callback, create_summary,
        export as transcribe_export, get_share_link, status as transcribe_status, upload,
    },
    translate::{translate_text, translate_transcribe, translate_utterance},
};

#[pyclass]
#[derive(Debug, Clone, Default)]
pub struct TranscribeApi;

#[pymethods]
impl TranscribeApi {
    #[new]
    pub fn new() -> Self {
        Self
    }

    pub fn transcribe_upload<'py>(
        &self,
        py: Python<'py>,
        filepath: Bound<'py, PyAny>,
        transcribe_only: bool,
        short_asr: bool,
        model: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let filepath: String = filepath.extract()?;
        let model: String = model.extract()?;
        let model = parse_model(&model)?;
        let token: String = token.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = upload(&filepath, transcribe_only, short_asr, model, &token).await?;
            Ok(UploadResult::from(response))
        })
    }

    pub fn transcribe_status<'py>(
        &self,
        py: Python<'py>,
        task_id: Option<Bound<'py, PyAny>>,
        share_id: Option<Bound<'py, PyAny>>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let task_id_str = if let Some(value) = task_id {
            Some(value.extract::<String>()?)
        } else {
            None
        };
        let share_id_str = if let Some(value) = share_id {
            Some(value.extract::<String>()?)
        } else {
            None
        };

        let token: String = token.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response =
                transcribe_status(task_id_str.as_deref(), share_id_str.as_deref(), &token).await?;
            Ok(StatusResponse::from(response))
        })
    }

    pub fn transcribe_callback<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let token: String = token.extract()?;
        let payload: CallbackRequest = if request.is_instance_of::<PyBytes>()
            || request.is_instance_of::<PyString>()
        {
            let text: String = request.extract()?;
            match serde_json::from_str(&text) {
                Ok(value) => value,
                Err(err) => {
                    return Err(
                        Error::InvalidInput(format!("invalid callback payload: {err}")).into(),
                    );
                }
            }
        } else {
            return Err(Error::InvalidInput("callback payload must be str or bytes".into()).into());
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = transcribe_callback(&payload, &token).await?;
            Ok(CallbackResponse::from(response))
        })
    }

    pub fn transcribe_share_link<'py>(
        &self,
        py: Python<'py>,
        task_id: Bound<'py, PyAny>,
        expiration_days: Option<i32>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let task_id: String = task_id.extract()?;
        let token: String = token.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = get_share_link(&task_id, expiration_days, &token).await?;
            Ok(ShareLinkResponse::from(response))
        })
    }

    pub fn transcribe_create_summary<'py>(
        &self,
        py: Python<'py>,
        utterances: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let token: String = token.extract()?;
        let utterances: Vec<Utterance> = extract_utterances(utterances)?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = create_summary(utterances, &token).await?;
            Ok(SummaryCreateResponse::from(response))
        })
    }

    pub fn transcribe_export<'py>(
        &self,
        py: Python<'py>,
        task_id: Bound<'py, PyAny>,
        r#type: Bound<'py, PyAny>,
        format: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let task_id: String = task_id.extract()?;
        let r#type: String = r#type.extract()?;
        let format: String = format.extract()?;
        let token: String = token.extract()?;

        let export_type = parse_export_type(&r#type)?;
        let export_format = parse_export_format(&format)?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let data = transcribe_export(&task_id, export_type, export_format, &token).await?;
            Ok(data.to_vec())
        })
    }

    pub fn translate_text<'py>(
        &self,
        py: Python<'py>,
        text: Bound<'py, PyAny>,
        language: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let text: String = text.extract()?;
        let language_str: String = language.extract()?;
        let language = parse_language(&language_str)?;
        let token: String = token.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = translate_text(&text, language, &token).await?;
            Ok(TextTranslationResponse::from(response))
        })
    }

    pub fn translate_utterances<'py>(
        &self,
        py: Python<'py>,
        utterances: Bound<'py, PyAny>,
        language: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let utterances: Vec<Utterance> = extract_utterances(utterances)?;
        let language_str: String = language.extract()?;
        let language = parse_language(&language_str)?;
        let token: String = token.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = translate_utterance(utterances, language, &token).await?;
            Ok(UtteranceTranslationResponse::from(response))
        })
    }

    pub fn translate_transcribe<'py>(
        &self,
        py: Python<'py>,
        task_id: Bound<'py, PyAny>,
        language: Bound<'py, PyAny>,
        token: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let task_id: String = task_id.extract()?;
        let language_str: String = language.extract()?;
        let language = parse_language(&language_str)?;
        let token: String = token.extract()?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let response = translate_transcribe(&task_id, language, &token).await?;
            Ok(TranscribeTranslationResponse::from(response))
        })
    }
}
