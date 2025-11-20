# Dianya API Python SDK

基于 Rust + PyO3 实现的异步 Python SDK，完整封装了 `transcribe` 与 `translate` 模块，并提供统一的错误类型、流式转写能力与类型提示。

## 环境要求

- Python ≥ 3.10
- 已安装 [`uv`](https://github.com/astral-sh/uv) 作为 Python 包管理与执行工具
- Rust 工具链（与 workspace 版本保持一致）

## 快速开始

```bash
# 开发模式安装本地扩展
uv run maturin develop --release
```

```python
import asyncio
from dianya import TranscribeApi, DianyaApiError


async def main() -> None:
    token = "YOUR_BEARER_TOKEN"
    client = TranscribeApi()

    try:
        session = await client.transcribe_create_session(token=token)
        print("session:", session["task_id"])

        upload = await client.transcribe_upload(
            filepath="sample.wav",
            transcribe_only=True,
            token=token,
        )
        print("upload:", upload)

        status = await client.transcribe_status(task_id=session["task_id"], token=token)
        print("status:", status["status"])
    except DianyaApiError as exc:
        # SDK 会把错误码挂载到异常的 code 字段
        print(f"接口调用失败: {exc.code} -> {exc}")


asyncio.run(main())
```

## 流式转写

```python
from dianya import TranscribeApi, TranscribeStream


async def consume_stream(session_id: str) -> None:
    stream = TranscribeStream(session_id)
    await stream.start()

    try:
        while message := await stream.read_next():
            print("WS message:", message)
    finally:
        await stream.stop()
```

- `TranscribeStream.send_text` / `TranscribeStream.send_bytes` 向服务端发送控制命令或音频帧
- `TranscribeStream.read_next` 支持可选超时（秒），无消息时返回 `None`
- `TranscribeStream.stop` 会主动关闭连接，实例释放时也会尝试收尾

## 错误处理

- 所有错误统一包装为 `DianyaApiError`，`code` 字段与 `common::Error` 定义保持一致，例如 `WS_ERROR`、`INVALID_INPUT`
- 底层 JSON 解析失败会抛出 `JSON_ERROR`
- 运行时未初始化的流式连接会触发 `UNEXPECTED_ERROR`

## 可用方法概览

| 分类 | 方法 |
| ---- | ---- |
| 会话 | `transcribe_create_session`, `transcribe_close_session` |
| 上传 | `transcribe_upload` |
| 状态 | `transcribe_status`, `transcribe_callback`, `transcribe_share_link` |
| 总结 | `transcribe_create_summary`, `transcribe_export` |
| 翻译 | `translate_text`, `translate_utterances`, `translate_transcribe` |
| 流式 | `TranscribeStream.start`, `TranscribeStream.send_text`, `TranscribeStream.send_bytes`, `TranscribeStream.read_next`, `TranscribeStream.stop` |

所有入参与返回值的结构均在 `dianya/__init__.pyi` 中给出显式类型提示，可直接用于 IDE 补全与类型检查。

## 开发与测试

```bash
# 格式化 & lint（依赖 workspace 工具链）
cargo fmt
cargo clippy

# 安装开发依赖（包括 pytest 和 pytest-asyncio）
uv pip install -e ".[dev]"

# 运行 PyO3 扩展的示例/单测
uv run maturin develop --release
uv run pytest tests/ -v

# 运行单个测试文件
uv run pytest tests/test_session.py -v
uv run pytest tests/test_upload.py -v
uv run pytest tests/test_status.py -v

# 运行单个测试用例
uv run pytest tests/test_session.py::test_create_session -v
uv run pytest tests/test_upload.py::test_upload -v
uv run pytest tests/test_export.py::test_export_overview_as_pdf -v
```

> 参考测试策略：
> - Rust 端补充关键路径的单元测试（例如错误转换、WS 封装）
> - Python 侧可通过 `pytest` + `asyncio` 编写使用示例测试，覆盖正常返回与异常路径

## 常见问题

- **如何传入自定义结构？** `transcribe_create_summary`、`translate_utterances` 等接口接受任何 `Seq[dict]`，字段需与类型提示一致。
- **是否可以同步调用？** SDK 当前仅提供 `async` API，建议在 Python 应用层自行封装同步包装，如果确实需要可用 `asyncio.run`.
- **如何查看原始响应？** 所有返回值都保持与 `transcribe` crate 一致的 JSON 结构，可直接访问字典字段。

