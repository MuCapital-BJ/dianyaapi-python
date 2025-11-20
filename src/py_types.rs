use std::collections::HashMap;

use pyo3::prelude::*;
use transcribe::{
    Utterance,
    transcribe::{
        CallbackHistory, CallbackResponse as InnerCallbackResponse, SessionCreator, SessionEnder,
        ShareLink, SummaryContent as InnerSummaryContent, SummaryCreator,
        TaskType as StatusTaskType, TranscribeStatus, UploadResponse,
    },
    translate::{TextTranslator, TranscribeTranslator, TranslateDetail, UtteranceTranslator},
};

fn status_task_type_to_str(value: StatusTaskType) -> &'static str {
    match value {
        StatusTaskType::NormalQuality => "normal_quality",
        StatusTaskType::NormalSpeed => "normal_speed",
        StatusTaskType::ShortAsrQuality => "short_asr_quality",
        StatusTaskType::ShortAsrSpeed => "short_asr_speed",
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct SessionCreateResult {
    #[pyo3(get)]
    task_id: String,
    #[pyo3(get)]
    session_id: String,
    #[pyo3(get)]
    usage_id: String,
    #[pyo3(get)]
    max_time: i32,
}

impl From<SessionCreator> for SessionCreateResult {
    fn from(value: SessionCreator) -> Self {
        Self {
            task_id: value.task_id,
            session_id: value.session_id,
            usage_id: value.usage_id,
            max_time: value.max_time,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct SessionCloseResult {
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    duration: Option<i32>,
    #[pyo3(get)]
    error_code: Option<i32>,
    #[pyo3(get)]
    message: Option<String>,
}

impl From<SessionEnder> for SessionCloseResult {
    fn from(value: SessionEnder) -> Self {
        Self {
            status: value.status,
            duration: value.duration,
            error_code: value.error_code,
            message: value.message,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct UploadResult {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    task_id: Option<String>,
    #[pyo3(get)]
    status: Option<String>,
    #[pyo3(get)]
    message: Option<String>,
    #[pyo3(get)]
    data: Option<String>,
}

impl From<UploadResponse> for UploadResult {
    fn from(value: UploadResponse) -> Self {
        match value {
            UploadResponse::Normal(normal) => Self {
                kind: "normal".into(),
                task_id: Some(normal.task_id),
                status: None,
                message: None,
                data: None,
            },
            UploadResponse::OneSentence(one_sentence) => Self {
                kind: "one_sentence".into(),
                task_id: None,
                status: Some(one_sentence.status),
                message: Some(one_sentence.message),
                data: Some(one_sentence.data),
            },
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct UtterancePayload {
    #[pyo3(get)]
    start_time: f64,
    #[pyo3(get)]
    end_time: f64,
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    speaker: i32,
}

impl From<Utterance> for UtterancePayload {
    fn from(value: Utterance) -> Self {
        Self {
            start_time: value.start_time,
            end_time: value.end_time,
            text: value.text,
            speaker: value.speaker,
        }
    }
}

impl From<&Utterance> for UtterancePayload {
    fn from(value: &Utterance) -> Self {
        Self {
            start_time: value.start_time,
            end_time: value.end_time,
            text: value.text.clone(),
            speaker: value.speaker,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct SummaryContent {
    #[pyo3(get)]
    short: String,
    #[pyo3(get)]
    long: String,
    #[pyo3(get)]
    all: String,
    #[pyo3(get)]
    keywords: Vec<String>,
}

impl From<InnerSummaryContent> for SummaryContent {
    fn from(value: InnerSummaryContent) -> Self {
        Self {
            short: value.short,
            long: value.long,
            all: value.all,
            keywords: value.keywords,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct CallbackHistoryItem {
    #[pyo3(get)]
    timestamp: String,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    code: u32,
}

impl From<CallbackHistory> for CallbackHistoryItem {
    fn from(value: CallbackHistory) -> Self {
        Self {
            timestamp: value.timestamp,
            status: value.status,
            code: value.code,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct StatusResponse {
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    overview_md: Option<String>,
    #[pyo3(get)]
    summary_md: Option<String>,
    #[pyo3(get)]
    details: Vec<UtterancePayload>,
    #[pyo3(get)]
    message: Option<String>,
    #[pyo3(get)]
    usage_id: Option<String>,
    #[pyo3(get)]
    task_id: Option<String>,
    #[pyo3(get)]
    keywords: Vec<String>,
    #[pyo3(get)]
    callback_history: Vec<CallbackHistoryItem>,
    #[pyo3(get)]
    task_type: Option<String>,
}

impl From<TranscribeStatus> for StatusResponse {
    fn from(value: TranscribeStatus) -> Self {
        Self {
            status: value.status,
            overview_md: value.overview_md,
            summary_md: value.summary_md,
            details: value.details.into_iter().map(Into::into).collect(),
            message: value.message,
            usage_id: value.usage_id,
            task_id: value.task_id,
            keywords: value.keywords,
            callback_history: value.callback_history.into_iter().map(Into::into).collect(),
            task_type: value
                .task_type
                .map(status_task_type_to_str)
                .map(str::to_string),
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct CallbackResponse {
    #[pyo3(get)]
    status: String,
}

impl From<InnerCallbackResponse> for CallbackResponse {
    fn from(value: InnerCallbackResponse) -> Self {
        Self {
            status: value.status,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct ShareLinkResponse {
    #[pyo3(get)]
    share_url: String,
    #[pyo3(get)]
    expiration_time: i32,
    #[pyo3(get)]
    expired_at: String,
}

impl From<ShareLink> for ShareLinkResponse {
    fn from(value: ShareLink) -> Self {
        Self {
            share_url: value.share_url,
            expiration_time: value.expiration_day,
            expired_at: value.expired_at,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct SummaryCreateResponse {
    #[pyo3(get)]
    task_id: String,
}

impl From<SummaryCreator> for SummaryCreateResponse {
    fn from(value: SummaryCreator) -> Self {
        Self {
            task_id: value.task_id,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct TextTranslationResponse {
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    data: String,
}

impl From<TextTranslator> for TextTranslationResponse {
    fn from(value: TextTranslator) -> Self {
        Self {
            status: value.status,
            data: value.data,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct UtteranceTranslationResponse {
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    target_language: String,
    #[pyo3(get)]
    details: Vec<UtterancePayload>,
}

impl From<UtteranceTranslator> for UtteranceTranslationResponse {
    fn from(value: UtteranceTranslator) -> Self {
        Self {
            status: value.status,
            target_language: value.lang.as_str().to_string(),
            details: value.details.into_iter().map(Into::into).collect(),
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct TranslationDetail {
    #[pyo3(get)]
    start_time: f64,
    #[pyo3(get)]
    end_time: f64,
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    speaker: i32,
    #[pyo3(get)]
    translations: HashMap<String, String>,
}

impl From<TranslateDetail> for TranslationDetail {
    fn from(value: TranslateDetail) -> Self {
        Self {
            start_time: value.utterance.start_time,
            end_time: value.utterance.end_time,
            text: value.utterance.text,
            speaker: value.utterance.speaker,
            translations: value.translations,
        }
    }
}

#[pyclass(module = "dianyaapi")]
#[derive(Clone, Debug)]
pub struct TranscribeTranslationResponse {
    #[pyo3(get)]
    task_id: String,
    #[pyo3(get)]
    task_type: String,
    #[pyo3(get)]
    status: String,
    #[pyo3(get)]
    target_language: String,
    #[pyo3(get)]
    message: Option<String>,
    #[pyo3(get)]
    details: Option<Vec<TranslationDetail>>,
    #[pyo3(get)]
    overview_md: Option<String>,
    #[pyo3(get)]
    summary_md: Option<String>,
    #[pyo3(get)]
    keywords: Option<Vec<String>>,
}

impl From<TranscribeTranslator> for TranscribeTranslationResponse {
    fn from(value: TranscribeTranslator) -> Self {
        Self {
            task_id: value.task_id,
            task_type: value.task_type.as_str().to_string(),
            status: value.status,
            target_language: value.lang.as_str().to_string(),
            message: value.message,
            details: value
                .details
                .map(|items| items.into_iter().map(Into::into).collect()),
            overview_md: value.overview_md,
            summary_md: value.summary_md,
            keywords: value.keywords,
        }
    }
}
