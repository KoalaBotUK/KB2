import logging
import os
import traceback
import httpx
import asyncio

from fastapi import FastAPI, WebSocket
from websockets.asyncio.server import serve


logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)

_FORMATTER = logging.Formatter("%(asctime)s %(levelname)-8s %(message)s")
stream_handler = logging.StreamHandler(sys.stdout)
stream_handler.setFormatter(_FORMATTER)
logger.addHandler(stream_handler)
app = FastAPI()


class WebsocketExtension:
    _host: str
    _port: int
    _runtime_api: str
    _ext_id: str
    _ext_req_id: str
    _async_httpx: httpx.AsyncClient

    def __init__(self, host: str = "localhost", port: int = 8765):
        self._host = host
        self._port = port
        self._runtime_api = os.environ.get('AWS_LAMBDA_RUNTIME_API')
        self._async_httpx = httpx.AsyncClient()

    async def register(self):
        try:
            register_response = await self._async_httpx.post(
                f"http://{self._runtime_api}/2020-01-01/extension/register",
                content='{"events": ["INVOKE"]}',
                headers={"Lambda-Extension-Name": "dislord",
                         "Content-Type": "application/json"}, )
            if register_response.status_code != 200:
                logger.error(
                    f"Failed to register. Status: {register_response.status_code}, Response: {register_response.text}")
                return
            else:
                logger.info("Registered Lambda extension")

            self._ext_id = register_response.headers.get("Lambda-Extension-Identifier")
            await self.next()

        except Exception as e:
            logger.error(f"Failed to register. Error: {e.__class__.__name__} {e} {traceback.format_exc()}")
            return

    async def next(self):
        next_response = await self._async_httpx.get(f"http://{self._runtime_api}/2020-01-01/extension/event/next",
                                                    headers={"Lambda-Extension-Identifier": self._ext_id},
                                                    timeout=None)

        if next_response.status_code != 200:
            logger.error(f"Failed to get next event. Status: {next_response.status_code}, Response: {next_response.text}")
        else:
            self._ext_req_id = next_response.headers.get("Lambda-Extension-Request-Id")

    async def error(self, e: Exception):
        await self._async_httpx.post(
            f"http://{self._runtime_api}/2020-01-01/extension/exit/error",
            data={
                "errorMessage": str(e),
                "errorType": str(e.__class__.__name__),
                "stackTrace": traceback.format_exc(),
            },
            headers={
                "Lambda-Extension-Identifier": self._ext_id,
                "Lambda-Runtime-Function-Error-Type": f"Runtime.{e.__class__.__name__}"
            }
        )

    async def process(self, websocket):
        async for message in websocket:
            logger.debug(message)
            await self.next()

    async def serve(self):
        async with serve(self.process, self._host, self._port):
            await asyncio.get_running_loop().create_future()  # run forever

    async def run(self):
        await self.register()
        await self.serve()


app = FastAPI()

ws_ext = WebsocketExtension()


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    while True:
        msg = await websocket.receive_text()
        await ws_ext.next()
        await websocket.send_text(f"Message text was: {msg}")


if __name__ == '__main__':
    import uvicorn
    asyncio.run(ws_ext.register())
    uvicorn.run(app, host="localhost", port=8765)
