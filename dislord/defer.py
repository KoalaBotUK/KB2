from queue import Empty
from threading import Thread

from dislord import ApplicationClient


class DeferredThread:
    _instance: 'DeferredThread' = None
    client: ApplicationClient
    thread: Thread
    thread_started: bool = False

    def __init__(self, client: ApplicationClient):
        self.client = client
        self.thread = Thread(target=self.invocation_loop)

    @classmethod
    def instance(cls, client: ApplicationClient) -> 'DeferredThread':
        if cls._instance is None:
            cls._instance = cls(client)
        return cls._instance

    def invocation_loop(self):
        print("Starting DeferredThread")
        while True:
            try:
                self.client.defer_queue_interact()
            except Empty:
                continue
            except Exception as e:
                print(f"Failed to defer queue interact. Error: {e.__class__.__name__}")

    def start(self):
        if not self.thread_started:
            self.thread.start()
            self.thread_started = True
