"""Dianya API Python SDK type stubs.

This module provides type hints for the Dianya API Python SDK, which is
a Rust + PyO3 based asynchronous Python SDK that fully encapsulates the
`transcribe` and `translate` modules.
"""

from __future__ import annotations

from typing import List, Literal, Sequence
from typing import TypedDict


# Type aliases for API parameters
ModelType = Literal["speed", "quality", "quality_v2"]
"""Transcription model type: speed, quality, or quality_v2."""

ExportTypeLiteral = Literal["transcript", "overview", "summary"]
"""Export type: transcript, overview, or summary."""

ExportFormatLiteral = Literal["pdf", "txt", "docx"]
"""Export format: pdf, txt, or docx."""

LanguageCode = Literal["zh", "en", "ja", "ko", "fr", "de"]
"""Language code for translation: zh (Chinese), en (English), ja (Japanese), ko (Korean), fr (French), or de (German)."""


class SessionCreateResult:
    """Response object returned from creating a real-time transcription session."""

    task_id: str
    session_id: str
    usage_id: str
    max_time: int


class _SessionCloseRequired(TypedDict):
    status: str


class SessionCloseResult:
    """Response from closing a transcription session."""

    status: str
    duration: int | None
    error_code: int | None
    message: str | None


class UploadResult:
    """Upload response for either normal or one-sentence mode.
    
    Attributes:
        kind: Literal ``"normal"`` or ``"one_sentence"``.
        task_id: Present when ``kind == "normal"``.
        status: Present when ``kind == "one_sentence"``.
        message: Optional status message.
        data: Transcription result when ``kind == "one_sentence"``.
    """

    kind: Literal["normal", "one_sentence"]
    task_id: str | None
    status: str | None
    message: str | None
    data: str | None


class UtterancePayload:
    """Utterance information for transcription or translation."""

    start_time: float
    end_time: float
    text: str
    speaker: int


class SummaryContent:
    """Summary content returned from transcription or translation."""

    short: str
    long: str
    all: str
    keywords: List[str]


class CallbackHistoryItem:
    """Callback history item."""

    timestamp: str
    status: str
    code: int


class CallbackRequestPayload(TypedDict, total=False):
    """Callback request payload for transcription status updates.
    
    Attributes:
        task_id: Task ID.
        status: Current task status as a string (e.g., "convert_pending", "transcribe_running").
        code: Status code (numeric form, corresponds to StateCode enum).
        utterances: List of transcription results (returned when transcription completes).
        summary: Summary content (returned when summary completes).
        duration: Audio file duration in milliseconds (returned when file length exceeds limit or conversion succeeds).
        message: Error message (returned when an error occurs).
    """
    task_id: str
    status: str
    code: int
    utterances: List[UtterancePayload]
    summary: SummaryContent
    duration: int
    message: str


class _StatusResponseRequired(TypedDict):
    status: str


class StatusResponse:
    """Response from getting transcription or summary task status."""

    status: str
    overview_md: str | None
    summary_md: str | None
    details: List[UtterancePayload]
    message: str | None
    usage_id: str | None
    task_id: str | None
    keywords: List[str]
    callback_history: List[CallbackHistoryItem]
    task_type: str | None


class CallbackResponse:
    """Response from handling a callback."""

    status: str


class ShareLinkResponse:
    """Response from getting a share link."""

    share_url: str
    expiration_time: int
    expired_at: str


class SummaryCreateResponse:
    """Response from creating a summary task."""

    task_id: str


class TextTranslationResponse:
    """Response from translating text."""

    status: str
    data: str


class UtteranceTranslationResponse:
    """Response from translating utterances."""

    status: str
    target_language: str
    details: List[UtterancePayload]


class TranslationDetail:
    """Translation detail combining utterance fields and translated text."""

    start_time: float
    end_time: float
    text: str
    speaker: int
    translations: dict[str, str]


class _TranscribeTranslationRequired(TypedDict):
    """Base required fields for transcription translation response."""
    task_id: str
    task_type: str
    status: str
    target_language: str


class TranscribeTranslationResponse:
    """Response from translating a transcription task."""

    task_id: str
    task_type: Literal["transcribe", "summary"]
    status: str
    target_language: str
    message: str | None
    details: List[TranslationDetail] | None
    overview_md: str | None
    summary_md: str | None
    keywords: List[str] | None


class TranscribeStream:
    """WebSocket-based real-time transcription stream client.
    
    This class provides a high-level interface for real-time transcription
    via WebSocket connection. It handles connection management, message
    sending, and result receiving.
    """
    def __init__(self, session_id: str) -> None:
        """Initialize a transcription stream with a session ID.
        
        Args:
            session_id: Session ID obtained from create_session.
        """
        ...

    @staticmethod
    async def create_session(
        model: ModelType, token: str
    ) -> SessionCreateResult:
        """Create a real-time transcription session.

        Args:
            model: Transcription model type (speed, quality, or quality_v2).
            token: Bearer token for authentication.

        Returns:
            Session creation result with task_id, session_id, usage_id, and max_time.
        """
        ...

    @staticmethod
    async def close_session(
        task_id: str, token: str, timeout_seconds: int | None = ...
    ) -> SessionCloseResult:
        """Close a real-time transcription session.

        This method automatically handles retries when the session is busy (error_code=4),
        retrying every 2 seconds until the session is successfully closed, timeout, or other error.

        Args:
            task_id: Task ID obtained from create_session.
            token: Bearer token for authentication.
            timeout_seconds: Timeout in seconds (default: 30).

        Returns:
            Session close result with status and optional duration/error information.
        """
        ...
    
    async def start(self) -> None:
        """Start the WebSocket connection and begin receiving transcription results."""
        ...
    
    async def stop(self) -> None:
        """Stop the WebSocket connection and send end signal to server."""
        ...
    
    async def send_text(self, message: str) -> None:
        """Send a text message (control command) to the server.
        
        Args:
            message: Text message to send (typically JSON string).
        """
        ...
    
    async def send_bytes(self, data: bytes | bytearray | memoryview) -> None:
        """Send binary audio data to the server.
        
        Args:
            data: Binary audio frame data (bytes, bytearray, or memoryview).
        """
        ...
    
    async def read_next(self, timeout: float | None = ...) -> str | None:
        """Read the next message from the stream.
        
        Args:
            timeout: Timeout in seconds. If None, wait indefinitely.
                    If specified, returns None if no message received within timeout.
        
        Returns:
            Received message as string, or None if timeout.
        """
        ...


class TranscribeApi:
    """Main API client for Dianya transcription and translation services.
    
    This class provides asynchronous methods for all transcription and
    translation operations, including session management, file upload,
    status queries, export, and translation.
    """
    def __init__(self) -> None:
        """Initialize the API client."""
        ...

    async def transcribe_upload(
        self,
        filepath: str,
        transcribe_only: bool,
        short_asr: bool,
        model: ModelType,
        token: str,
    ) -> UploadResult:
        """Upload an audio file for transcription.
        
        Args:
            filepath: Path to the audio file.
            transcribe_only: Whether to transcribe only (no summary).
            short_asr: Whether to use one-sentence ASR mode (duration <= 3 minutes, file <= 50MB).
            model: Transcription model type.
            token: Bearer token for authentication.
        
        Returns:
            Upload result: Normal mode returns task_id, one-sentence mode returns transcription directly.
        """
        ...

    async def transcribe_status(
        self, task_id: str | None = ..., share_id: str | None = ..., *, token: str
    ) -> StatusResponse:
        """Get transcription or summary task status.
        
        Args:
            task_id: Task ID (optional, mutually exclusive with share_id).
            share_id: Share link ID (optional, mutually exclusive with task_id).
            token: Bearer token for authentication (keyword-only).
        
        Returns:
            Task status with transcription results, summary, overview, etc.
        """
        ...

    async def transcribe_callback(
        self, request: str | CallbackRequestPayload, *, token: str
    ) -> CallbackResponse:
        """Handle transcription task status callback.
        
        Note: This endpoint is typically used on the server side to receive
        callbacks from the Suth service. For client SDKs, this may not be applicable.
        
        Args:
            request: Callback request data (JSON string or dict).
            token: Bearer token for authentication (keyword-only).
        
        Returns:
            Callback response with status.
        """
        ...

    async def transcribe_share_link(
        self, task_id: str, expiration_days: int | None = ..., *, token: str
    ) -> ShareLinkResponse:
        """Get a share link for a transcription task.
        
        Args:
            task_id: Task ID.
            expiration_days: Expiration time in days (optional, default: 7).
            token: Bearer token for authentication (keyword-only).
        
        Returns:
            Share link response with share_url, expiration_time, and expired_at.
        """
        ...

    async def transcribe_create_summary(
        self, utterances: Sequence[UtterancePayload], *, token: str
    ) -> SummaryCreateResponse:
        """Create a summary task from utterances.
        
        Args:
            utterances: Sequence of utterance dictionaries with start_time, end_time, text, and speaker.
            token: Bearer token for authentication (keyword-only).
        
        Returns:
            Summary creation result with task_id.
        """
        ...

    async def transcribe_export(
        self,
        task_id: str,
        type: ExportTypeLiteral,
        format: ExportFormatLiteral,
        token: str,
    ) -> bytes:
        """Export transcription or summary content.
        
        Args:
            task_id: Task ID (required).
            type: Export type: transcript (note: summary tasks don't support this),
                  overview, or summary.
            format: Export format: pdf (default), txt, or docx.
            token: Bearer token for authentication.
        
        Returns:
            Binary data of the exported file, which can be saved as the corresponding format.
        """
        ...

    async def translate_text(
        self, text: str, language: LanguageCode, *, token: str
    ) -> TextTranslationResponse:
        """Translate a text string.
        
        Args:
            text: Text to translate.
            language: Target language code.
            token: Bearer token for authentication (keyword-only).
        
        Returns:
            Translation response with status and translated data.
        """
        ...

    async def translate_utterances(
        self, utterances: Sequence[UtterancePayload], language: LanguageCode, *, token: str
    ) -> UtteranceTranslationResponse:
        """Translate a list of utterances.
        
        Args:
            utterances: Sequence of utterance dictionaries to translate.
            language: Target language code.
            token: Bearer token for authentication (keyword-only).
        
        Returns:
            Translation response with status, target_language, and translated details.
        """
        ...

    async def translate_transcribe(
        self, task_id: str, language: LanguageCode, *, token: str
    ) -> TranscribeTranslationResponse:
        """Get translation result for a transcription task.
        
        Args:
            task_id: Task ID.
            language: Target language code.
            token: Bearer token for authentication (keyword-only).
        
        Returns:
            Translation response with task information and translated results including
            details, overview_md, summary_md, and keywords.
        """
        ...


__all__ = [
    "TranscribeApi",
    "TranscribeStream",
    "ModelType",
    "ExportTypeLiteral",
    "ExportFormatLiteral",
    "LanguageCode",
    "SessionCreateResult",
    "SessionCloseResult",
    "UploadResult",
    "UtterancePayload",
    "SummaryContent",
    "CallbackHistoryItem",
    "StatusResponse",
    "CallbackResponse",
    "ShareLinkResponse",
    "SummaryCreateResponse",
    "TextTranslationResponse",
    "UtteranceTranslationResponse",
    "TranslationDetail",
    "TranscribeTranslationResponse",
]

