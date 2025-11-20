use common::Error;
use pyo3::{Bound, PyAny, PyResult, types::PyAnyMethods};
use transcribe::{
    Utterance,
    transcribe::{ExportFormat, ExportType, ModelType},
    translate::Language,
};

pub fn parse_model(value: &str) -> PyResult<ModelType> {
    match value.to_ascii_lowercase().as_str() {
        "speed" => Ok(ModelType::Speed),
        "quality" => Ok(ModelType::Quality),
        "quality_v2" => Ok(ModelType::QualityV2),
        invalid => Err(Error::InvalidInput(format!(
            "unsupported model '{invalid}' (expected 'speed' 'quality' or 'quality_v2')"
        ))
        .into()),
    }
}

pub fn parse_export_type(value: &str) -> PyResult<ExportType> {
    match value.to_ascii_lowercase().as_str() {
        "transcript" => Ok(ExportType::Transcript),
        "overview" => Ok(ExportType::Overview),
        "summary" => Ok(ExportType::Summary),
        invalid => Err(Error::InvalidInput(format!("unsupported export type '{invalid}'")).into()),
    }
}

pub fn parse_export_format(value: &str) -> PyResult<ExportFormat> {
    match value.to_ascii_lowercase().as_str() {
        "pdf" => Ok(ExportFormat::Pdf),
        "txt" => Ok(ExportFormat::Txt),
        "docx" => Ok(ExportFormat::Docx),
        invalid => {
            Err(Error::InvalidInput(format!("unsupported export format '{invalid}'")).into())
        }
    }
}

pub fn parse_language(value: &str) -> PyResult<Language> {
    match value.to_ascii_lowercase().as_str() {
        "zh" | "zh-cn" => Ok(Language::ChineseSimplified),
        "en" | "en-us" => Ok(Language::EnglishUS),
        "ja" => Ok(Language::Japanese),
        "ko" | "kr" | "jp" => Ok(Language::Korean),
        "fr" => Ok(Language::French),
        "de" => Ok(Language::German),
        invalid => {
            Err(Error::InvalidInput(format!("unsupported language code '{invalid}'")).into())
        }
    }
}

pub fn extract_utterances(value: Bound<'_, PyAny>) -> PyResult<Vec<Utterance>> {
    let py = value.py();
    let dumped = py
        .import("json")?
        .call_method1("dumps", (value,))?
        .extract::<String>()?;
    serde_json::from_str(&dumped)
        .map_err(|err| Error::InvalidInput(format!("invalid utterances payload: {err}")).into())
}
