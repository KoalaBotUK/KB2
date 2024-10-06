#!/usr/bin/env python3
import asyncio
import json
import logging
import os
import socket
import sys
import traceback
from datetime import datetime

import httpx
from dotenv import load_dotenv
from pydantic import TypeAdapter

from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from dislord.discord.interactions.receiving_and_responding.interaction_response import MessagesInteractionCallbackData
from dislord.model.api import HttpResponse
from dislord.types import ObjDict
from kb2.client import client

load_dotenv()

PUBLIC_KEY = os.environ.get("PUBLIC_KEY")
BOT_TOKEN = os.environ.get("DISCORD_TOKEN")
SOCKET_PATH = "/tmp/kb2.sock"
RESPONSE_TIME_SLA_MS = 2500

logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)

_FORMATTER = logging.Formatter("%(asctime)s %(levelname)-8s %(message)s")
stream_handler = logging.StreamHandler(sys.stdout)
stream_handler.setFormatter(_FORMATTER)
logger.addHandler(stream_handler)


def socket_connect() -> socket.socket:
    if sys.platform == "win32":
        sock_client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        global SOCKET_PATH
        SOCKET_PATH = (socket.gethostname(), 8765)
        return sock_client
    else:
        sock_client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        return sock_client


class DeferredRequest(ObjDict):
    api_start_time_ms: float
    interaction: Interaction


class LambdaExtension:
    _host: str
    _port: int
    _runtime_api: str
    _ext_id: str
    _ext_req_id: str
    _async_httpx: httpx.AsyncClient

    def __init__(self, host: str = "127.0.0.1", port: int = 8765):
        self._host = host
        self._port = port
        self._runtime_api = os.environ.get('AWS_LAMBDA_RUNTIME_API')
        self._async_httpx = httpx.AsyncClient()

    async def register(self):
        logger.info("Registering Lambda extension")
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
        logger.info("Getting next Lambda extension event")
        if self._runtime_api is None:
            return

        next_response = await self._async_httpx.get(f"http://{self._runtime_api}/2020-01-01/extension/event/next",
                                                    headers={"Lambda-Extension-Identifier": self._ext_id},
                                                    timeout=None)

        if next_response.status_code != 200:
            logger.error(
                f"Failed to get next event. Status: {next_response.status_code}, Response: {next_response.text}")
        else:
            logger.info(f"Next Lambda extension event {next_response.json()}")
            self._ext_req_id = next_response.headers.get("Lambda-Extension-Request-Id")

    async def error(self, e: Exception):
        logger.error(f"Error: {e.__class__.__name__} {e} {traceback.format_exc()}")
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


l_ext = LambdaExtension()


def update_original_response(interaction: Interaction, response: HttpResponse):
    interact_response: MessagesInteractionCallbackData = (TypeAdapter(MessagesInteractionCallbackData)
                                                          .validate_python(response.body["data"]))
    if interact_response.flags is None:
        interact_response.flags = 0

    logger.debug(f"/defer/process Sending Response {interaction.id}")
    success = False
    failures = 0
    while not success and failures < 3:
        try:
            client.edit_original_response(interaction.token, interact_response)
            success = True
        except Exception as e:
            logger.error(f"Failed to edit original response. Error: {e.__class__.__name__} {e}")
            failures += 1


async def socket_process():
    server = socket_connect()
    await l_ext.register()
    try:
        server.bind(SOCKET_PATH)
        server.listen(1)

        while True:
            await l_ext.next()
            conn, _ = server.accept()
            conn.setblocking(True)

            with conn:
                # Receive data from the entrypoint Lambda function
                data = conn.recv(4096)
                if data:
                    deferred_request: DeferredRequest = TypeAdapter(DeferredRequest).validate_json(data.decode())
                    interaction = deferred_request.interaction
                    logger.debug(f"/defer/process Received Interaction {interaction.id}")
                    defer = client.defer(interaction)
                    if defer:
                        conn.sendall(json.dumps(defer.as_serverless_response()).encode())
                    else:
                        conn.sendall("None".encode())
                    interact_http_response: HttpResponse = client.interact(interaction)

                    if datetime.now().timestamp()*1000 - deferred_request.api_start_time_ms < RESPONSE_TIME_SLA_MS:
                        # If processing finishes within 3 seconds, send the result back to the function
                        conn.sendall(json.dumps(interact_http_response.as_serverless_response()).encode())
                    else:
                        # If processing takes longer, return nothing and handle deferred API call
                        update_original_response(interaction, interact_http_response)

                conn.close()

    except Exception as e:
        print(f"Error in extension: {str(e)}")
        await l_ext.error(e)

    finally:
        server.close()


if __name__ == '__main__':
    logger.info("Starting kb2_defer")
    asyncio.run(socket_process())
