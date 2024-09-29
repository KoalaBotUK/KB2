#!/usr/bin/env python3

import logging
import os
import sys
import traceback
from contextlib import asynccontextmanager

from fastapi import FastAPI, WebSocket
from pydantic import TypeAdapter
import httpx
import asyncio

from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from dislord.discord.interactions.receiving_and_responding.interaction_response import MessagesInteractionCallbackData
from dislord.model.api import HttpResponse
from kb2.main import client

logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)

_FORMATTER = logging.Formatter("%(asctime)s %(levelname)-8s %(message)s")
stream_handler = logging.StreamHandler(sys.stdout)
stream_handler.setFormatter(_FORMATTER)
logger.addHandler(stream_handler)


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
        if self._runtime_api is None:
            return

        try:
            register_response = await self._async_httpx.post(
                f"http://{self._runtime_api}/2020-01-01/extension/register",
                content='{"events": ["INVOKE"]}',
                headers={"Lambda-Extension-Name": "kb2_defer",
                         "Content-Type": "application/json"}, )
            if register_response.status_code != 200:
                logger.error(
                    f"Failed to register. Status: {register_response.status_code}, Response: {register_response.text}")
                return
            else:
                logger.info("Registered Lambda extension")

            self._ext_id = register_response.headers.get("Lambda-Extension-Identifier")

        except Exception as e:
            logger.error(f"Failed to register. Error: {e.__class__.__name__} {e} {traceback.format_exc()}")
            return

    async def next(self):
        if self._runtime_api is None:
            return

        next_response = await self._async_httpx.get(f"http://{self._runtime_api}/2020-01-01/extension/event/next",
                                                    headers={"Lambda-Extension-Identifier": self._ext_id},
                                                    timeout=None)

        if next_response.status_code != 200:
            logger.error(
                f"Failed to get next event. Status: {next_response.status_code}, Response: {next_response.text}")
        else:
            self._ext_req_id = next_response.headers.get("Lambda-Extension-Request-Id")

    async def error(self, e: Exception):
        if self._runtime_api is None:
            return

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


ws_ext = WebsocketExtension()


@asynccontextmanager
async def lifespan(app: FastAPI):
    # Load the ML model
    await ws_ext.register()
    yield


app = FastAPI(lifespan=lifespan)


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    logger.info(f"Client connected: {websocket.client.host}:{websocket.client.port}")
    await websocket.accept()
    await ws_ext.next()
    while True:
        msg = await websocket.receive_text()
        interaction = TypeAdapter(Interaction).validate_json(msg)
        logger.debug(f"DEFER QUEUE REQUEST: {interaction}")
        interact_http_response: HttpResponse = client.interact(interaction)
        logger.debug(f"DEFER QUEUE RESPONSE: {interact_http_response.body}")
        interact_response: MessagesInteractionCallbackData = (TypeAdapter(MessagesInteractionCallbackData)
                                                              .validate_python(interact_http_response.body["data"]))
        if interact_response.flags is None:
            interact_response.flags = 0
        client.edit_original_response(interaction.token, interact_response)
        await ws_ext.next()
        logger.debug(f"Message text was: {msg}")


if __name__ == '__main__':
    import uvicorn

    try:
        asyncio.get_running_loop()
    except RuntimeError:
        asyncio.new_event_loop()

    logger.info("Starting uvicorn")
    uvicorn.run(app, host="localhost", port=8765)