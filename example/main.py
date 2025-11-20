import asyncio
import signal
import sys
from contextlib import suppress
from typing import Optional

import sounddevice as sd

from dianyaapi import *

TOKEN = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ1c2VyXzgzZTk5Y2YyIiwiZXhwIjoxNzY1MzU5Mjc4Ljk0ODk5fQ.JVL2o7u2IC-LhqFvSAmfE9oGVmnL7R4vfnxm_JA0V5k"

AUDIO_SAMPLE_RATE = 16_000
AUDIO_CHANNELS = 1
AUDIO_DTYPE = "int16"
QUEUE_MAX_CHUNKS = 50
CHUNK_DURATION_SECONDS = 0.2
BYTES_PER_SAMPLE = 2  # int16
CHUNK_SIZE_BYTES = int(AUDIO_SAMPLE_RATE * AUDIO_CHANNELS * BYTES_PER_SAMPLE * CHUNK_DURATION_SECONDS)


def log(msg: str) -> None:
    print(msg, file=sys.stderr, flush=True)


async def translate():
    api = TranscribeApi()
    result: TextTranslationResponse = await api.translate_text("你好, 你是谁?", "en", token=TOKEN)
    print(result)


async def transcribe_realtime():
    result = await TranscribeStream.create_session("speed", TOKEN)
    session_id = result.session_id
    task_id = result.task_id
    stream: Optional[TranscribeStream] = None
    stream_stop_task: Optional[asyncio.Task[None]] = None
    stop_event = asyncio.Event()
    audio_queue: asyncio.Queue[bytes] = asyncio.Queue(maxsize=QUEUE_MAX_CHUNKS)
    loop = asyncio.get_running_loop()
    queue_drop_count = 0
    ws_stopped = False

    async def _stop_stream():
        if stream is None:
            return
        await stream.stop()
        nonlocal ws_stopped
        ws_stopped = True
        log("已调用 stream.stop()")

    def schedule_stream_stop():
        nonlocal stream_stop_task
        if stream is None:
            return
        if stream_stop_task is None or stream_stop_task.done():
            stream_stop_task = asyncio.create_task(_stop_stream(), name="stream_stop")

    def _signal_handler():
        if not stop_event.is_set():
            log("检测到 Ctrl+C, 正在停止...")
            stop_event.set()
        schedule_stream_stop()

    def audio_callback(indata, frames, time_info, status):
        if status:
            log(f"录音状态: {status}")
        data = bytes(indata)
        if not data:
            return

        nonlocal queue_drop_count

        def _push():
            try:
                audio_queue.put_nowait(data)
            except asyncio.QueueFull:
                queue_drop_count += 1
                with suppress(asyncio.QueueEmpty):
                    audio_queue.get_nowait()
                with suppress(asyncio.QueueFull):
                    audio_queue.put_nowait(data)
                if queue_drop_count % 10 == 0:
                    log(f"音频队列溢出 {queue_drop_count} 次")

        loop.call_soon_threadsafe(_push)

    async def capture_audio():
        log("音频采集任务启动")
        blocksize = int(AUDIO_SAMPLE_RATE * CHUNK_DURATION_SECONDS)
        with sd.RawInputStream(
            samplerate=AUDIO_SAMPLE_RATE,
            channels=AUDIO_CHANNELS,
            dtype=AUDIO_DTYPE,
            blocksize=blocksize,
            callback=audio_callback,
        ):
            log("RawInputStream 已开启")
            while not stop_event.is_set():
                await asyncio.sleep(0.05)
        log("RawInputStream 已关闭")

    async def pump_audio():
        buffer = bytearray()
        next_flush = loop.time() + CHUNK_DURATION_SECONDS
        log("音频发送任务启动")
        while not stop_event.is_set():
            if ws_stopped:
                break
            try:
                chunk = await asyncio.wait_for(audio_queue.get(), timeout=CHUNK_DURATION_SECONDS)
                buffer.extend(chunk)
            except asyncio.TimeoutError:
                pass

            while len(buffer) >= CHUNK_SIZE_BYTES:
                if ws_stopped:
                    break
                segment = bytes(buffer[:CHUNK_SIZE_BYTES])
                del buffer[:CHUNK_SIZE_BYTES]
                await stream.send_bytes(segment)

            if ws_stopped:
                break

            now = loop.time()
            if buffer and now >= next_flush:
                await stream.send_bytes(bytes(buffer))
                buffer.clear()
                next_flush = now + CHUNK_DURATION_SECONDS

        if buffer and not ws_stopped:
            await stream.send_bytes(bytes(buffer))
            log(f"停止前 flush 剩余 {len(buffer)} 字节")

    async def receive_messages():
        log("接收任务启动")
        while not stop_event.is_set():
            message = await stream.read_next(timeout=None)
            if message is None:
                await asyncio.sleep(0.05)
                continue
            print(message)

    try:
        log(f"会话 {session_id} 初始化完成, 正在创建 TranscribeStream")
        stream = TranscribeStream(session_id)
        await stream.start()
        loop.add_signal_handler(signal.SIGINT, _signal_handler)

        capture_task = asyncio.create_task(capture_audio(), name="audio_capture")
        pump_task = asyncio.create_task(pump_audio(), name="audio_pump")
        recv_task = asyncio.create_task(receive_messages(), name="stream_reader")

        await asyncio.gather(capture_task, pump_task, recv_task)
    except asyncio.CancelledError:
        raise
    except KeyboardInterrupt:
        stop_event.set()
        schedule_stream_stop()
    finally:
        stop_event.set()
        schedule_stream_stop()
        if stream_stop_task is not None:
            with suppress(Exception):
                await stream_stop_task
        with suppress(Exception):
            loop.remove_signal_handler(signal.SIGINT)
        await TranscribeStream.close_session(task_id, token=TOKEN, timeout_seconds=None)


if __name__ == "__main__":
    asyncio.run(transcribe_realtime())
