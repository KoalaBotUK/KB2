import os
import traceback
import httpx

from dislord.defer import DeferredThread


class LambdaDeferredThread(DeferredThread):
    def invocation_loop(self):
        runtime_api = os.environ.get('AWS_LAMBDA_RUNTIME_API')

        if not runtime_api:
            print("AWS_LAMBDA_RUNTIME_API is not set")
            return

        print("Starting LambdaDeferredThread")
        while True:
            response = httpx.get(f"http://{runtime_api}/2018-06-01/runtime/invocation/next",
                                 timeout=None)

            if response.status_code != 200:
                print(f"Failed to get next invocation. Status: {response.status_code}, Response: {response.text}")
                break

            request_id = response.headers["Lambda-Runtime-Aws-Request-Id"]

            try:
                print(f"Started invocation: {request_id}")
                self.client.defer_queue_interact()

                httpx.post(
                    f"http://{runtime_api}/2018-06-01/runtime/invocation/{request_id}/response",
                    content="SUCCESS")
            except Exception as e:
                httpx.post(
                    f"http://{runtime_api}/2018-06-01/runtime/invocation/{request_id}/error",
                    data={
                        "errorMessage": str(e),
                        "errorType": str(e.__class__.__name__),
                        "stackTrace": traceback.format_exc(),
                    },
                    headers={
                        "Lambda-Runtime-Function-Error-Type": f"Runtime.{e.__class__.__name__}"
                    }
                )
