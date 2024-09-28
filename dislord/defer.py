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
            self.client.defer_queue_interact()

    def start(self):
        if not self.thread_started:
            self.thread.start()
            self.thread_started = True
