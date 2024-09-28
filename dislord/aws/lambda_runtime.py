import os
import traceback
import httpx

from dislord.defer import DeferredThread


class LambdaDeferredThread(DeferredThread):
    def invocation_loop(self):
        print("Starting LambdaDeferredThread")
        runtime_api = os.environ.get('AWS_LAMBDA_RUNTIME_API')
        print(f"Runtime API: {runtime_api}")

        if not runtime_api:
            print("AWS_LAMBDA_RUNTIME_API is not set")
            return

        try:
            register_response = httpx.post(f"http://{runtime_api}/2020-01-01/extension/register",
                                           content='{"events": ["INVOKE"]}',
                                           headers={"Lambda-Extension-Name": "dislord",
                                                    "Content-Type": "application/json"},
                                           timeout=0.1)
        except Exception as e:
            print(f"Failed to register. Error: {e.__class__.__name__} {e} {traceback.format_exc()}")
            return

        if register_response.status_code != 200:
            print(f"Failed to register. Status: {register_response.status_code}, Response: {register_response.text}")
            return
        else:
            print("Registered Lambda extension")

        ext_id = register_response.headers.get("Lambda-Extension-Identifier")

        while True:
            next_response = httpx.get(f"http://{runtime_api}/2020-01-01/extension/event/next",
                                      headers={"Lambda-Extension-Identifier": ext_id},
                                      timeout=None)

            if next_response.status_code != 200:
                print(f"Failed to get next event. Status: {next_response.status_code}, Response: {next_response.text}")
                break

            ext_req_id = next_response.headers.get("Lambda-Extension-Request-Id")

            try:
                print(f"Started invocation: {ext_req_id}")
                self.client.defer_queue_interact()
            except Exception as e:
                httpx.post(
                    f"http://{runtime_api}/2020-01-01/extension/exit/error",
                    data={
                        "errorMessage": str(e),
                        "errorType": str(e.__class__.__name__),
                        "stackTrace": traceback.format_exc(),
                    },
                    headers={
                        "Lambda-Extension-Identifier": ext_id,
                        "Lambda-Runtime-Function-Error-Type": f"Runtime.{e.__class__.__name__}"
                    }
                )
