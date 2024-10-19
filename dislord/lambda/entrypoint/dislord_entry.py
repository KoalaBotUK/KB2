import json
import logging
import os
import socket
import sys
import datetime
from http.client import UNAUTHORIZED, OK

from discord_interactions import verify_key
from dotenv import load_dotenv

# Constants
BASE_SOCKET_PATH = "/tmp/kb2-%s.sock"
RESPONSE_TIME_SLA_MS = 2500
SOCKET_SIZE = 2 ** 14

load_dotenv()
PUBLIC_KEY = os.environ.get("PUBLIC_KEY")
BOT_TOKEN = os.environ.get("DISCORD_TOKEN")

# Logging
_LOG_LEVEL = logging.DEBUG
_FORMATTER = logging.Formatter("%(asctime)s %(levelname)-8s %(message)s")
stdout_handler = logging.StreamHandler(sys.stdout)
stdout_handler.setFormatter(_FORMATTER)
stdout_handler.setLevel(_LOG_LEVEL)

logger = logging.getLogger(__name__)
logger.setLevel(_LOG_LEVEL)
logger.addHandler(stdout_handler)
logger.info("Starting Entrypoint")


# Code


def get_socket_address() -> tuple[any, ...] | str:
    if sys.platform == "win32":
        return socket.gethostname(), 8765
    else:
        return BASE_SOCKET_PATH.format(os.environ.get("_X_AMZN_TRACE_ID"))


def now_ms() -> float:
    return datetime.datetime.now().timestamp() * 1000


def socket_connect() -> socket.socket:
    if sys.platform == "win32":
        client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    else:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    logger.debug(f"Binding to socket {get_socket_address()}")
    client.connect(get_socket_address())
    return client


def process_interact(event: dict):
    try:
        api_start_time_ms = event['requestContext']['requestTimeEpoch']
        respond_by_ms = api_start_time_ms + RESPONSE_TIME_SLA_MS
        client = socket_connect()

        # Send the payload to the extension for processing
        request = ('{"api_start_time_ms":' + str(api_start_time_ms)
                   + ',"interaction":' + event["body"] + '}')

        logger.debug(f"Sending interaction: {request.encode()}")
        client.sendall(request.encode())

        # Set a timeout of 3 seconds to receive the response
        client.settimeout((respond_by_ms - now_ms()) / 1000)
        logger.debug(f"Setting timeout to: {respond_by_ms} - {now_ms()} = {client.gettimeout()}")

        defer = json.loads(client.recv(SOCKET_SIZE).decode())

        # Wait for the response from the extension
        try:
            data = client.recv(SOCKET_SIZE)
            if data:
                response = json.loads(data.decode())
                # Check if the response came within 3 seconds
                logger.debug(f"Defer response for interaction: {response}")
                return response

        except socket.timeout:
            # If no response within 3 seconds, return "DEFER"
            return defer
        finally:
            client.close()

    except Exception as e:
        logger.error("Error in kb2_entrypoint", exc_info=e)
        raise e


def serverless_handler_no_auth(event, context):
    if json.loads(event["body"]).get("type") == 1:
        return {"statusCode": OK,
                "body": json.dumps({"type": 1}),
                "headers": {
                    "Content-Type": "application/json"
                }}
    else:
        return process_interact(event)


def serverless_handler(event, context):
    logger.debug(f"Recieved Event\nevent: {event}\ncontext: {context}")

    if event['httpMethod'] == "POST":
        raw_headers = event["headers"]
        signature = raw_headers.get('x-signature-ed25519')
        timestamp = raw_headers.get('x-signature-timestamp')
        if signature is None or timestamp is None or not verify_key(event["body"].encode("utf-8"), signature, timestamp,
                                                                    PUBLIC_KEY):
            return {"statusCode": UNAUTHORIZED,
                    "body": "Bad request signature",
                    "headers": {
                        "Content-Type": "application/json"
                    }}
    return serverless_handler_no_auth(event, context)


def server():
    from fastapi import FastAPI, Request, Response
    import uvicorn

    app = FastAPI()

    @app.post("/deferred-interactions")
    async def deferred_interactions(request: Request, response: Response):
        sl_response = serverless_handler({"httpMethod": request.method,
                                          "body": (await request.body()).decode("utf-8"),
                                          "headers": request.headers,
                                          "requestContext": {
                                              "requestTimeEpoch": datetime.datetime.now().timestamp() * 1000
                                          }}, None)

        response.status_code = sl_response['statusCode']
        return json.loads(sl_response['body'])

    @app.post("/deferred-interactions-no-auth")
    async def deferred_interactions(request: Request, response: Response):
        sl_response = serverless_handler_no_auth({"httpMethod": request.method,
                                                  "body": (await request.body()).decode("utf-8"),
                                                  "headers": request.headers,
                                                  "requestContext": {
                                                      "requestTimeEpoch": datetime.datetime.now().timestamp() * 1000
                                                  }}, None)

        response.status_code = sl_response['statusCode']
        return json.loads(sl_response['body'])

    uvicorn.run(app, host="0.0.0.0", port=8123)


if __name__ == '__main__':
    server()
