import asyncio
from queue import Empty
from threading import Thread

import websockets

from dislord import ApplicationClient
from dislord.log import logger


class DeferredThread:
    _instance: 'DeferredThread' = None
    client: ApplicationClient
    thread: Thread
    thread_started: bool = False
    _ws_host: str
    _ws_port: int
    _ws_connection: websockets.WebSocketClientProtocol

    def __init__(self, client: ApplicationClient, ws_host: str = None, ws_port: int = None):
        self.client = client
        self.thread = Thread(target=self.invocation_loop)
        self._ws_host = ws_host
        self._ws_port = ws_port

    @classmethod
    def instance(cls, client: ApplicationClient, ws_host: str = None, ws_port: int = None) -> 'DeferredThread':
        if cls._instance is None:
            cls._instance = cls(client, ws_host, ws_port)
        return cls._instance

    def invocation_loop(self):
        logger.info("Starting DeferredThread")
        asyncio.run(self.ws_connect())
        while True:
            try:
                self.client.defer_queue_interact()
                asyncio.run(self.ws_send())
            except Empty:
                continue
            except Exception as e:
                logger.error(f"Failed to defer queue interact. Error: {e.__class__.__name__}")

    def start(self):
        if not self.thread_started:
            self.thread.start()
            self.thread_started = True

    async def ws_connect(self):
        self._ws_connection = await websockets.connect(f"ws://{self._ws_host}:{self._ws_port}/ws")

    async def ws_send(self):
        await self._ws_connection.send("next")
