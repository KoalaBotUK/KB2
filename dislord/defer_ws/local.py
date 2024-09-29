import logging
import sys

from fastapi import FastAPI, WebSocket

logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)

_FORMATTER = logging.Formatter("%(asctime)s %(levelname)-8s %(message)s")
stream_handler = logging.StreamHandler(sys.stdout)
stream_handler.setFormatter(_FORMATTER)
logger.addHandler(stream_handler)
app = FastAPI()


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    logger.info(f"Client connected: {websocket.client.host}:{websocket.client.port}")
    await websocket.accept()
    while True:
        msg = await websocket.receive_text()
        logger.debug(f"Message text was: {msg}")


if __name__ == '__main__':
    import uvicorn
    logger.info("Starting uvicorn")
    uvicorn.run(app, host="localhost", port=8765)
