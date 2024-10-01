import json
import socket
import sys
import traceback
import datetime
from http.client import UNAUTHORIZED, OK

from discord_interactions import verify_key

from kb2_entrypoint.env import PUBLIC_KEY
from kb2_entrypoint.log import logger

logger.info("Starting Entrypoint")
SOCKET_PATH = "/tmp/kb2.sock"
RESPONSE_TIME_SLA = 2.5


def socket_connect() -> socket.socket:
    if sys.platform == "win32":
        client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        client.connect((socket.gethostname(), 8765))
        return client
    else:
        client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        client.connect(SOCKET_PATH)
        return client


def process_interact(event: dict):
    try:
        api_start_time = event['requestContext']['requestTimeEpoch']/1000
        respond_by = datetime.timedelta(seconds=RESPONSE_TIME_SLA) + datetime.datetime.fromtimestamp(api_start_time)
        client = socket_connect()

        # Send the payload to the extension for processing
        request = ('{"api_start_time":' + str(api_start_time)
                   + ',"interaction":' + event["body"] + '}')

        logger.debug(f"Sending interaction: {request.encode()}")
        client.sendall(request.encode())

        # Set a timeout of 3 seconds to receive the response
        client.settimeout((respond_by - datetime.datetime.now()).seconds)

        # Wait for the response from the extension
        try:
            data = client.recv(4096)
            if data:
                response = data.decode()
                # Check if the response came within 3 seconds
                if datetime.datetime.now() < respond_by:
                    logger.debug(f"Defer response for interaction: {response}")
                    return json.loads(response)
                else:
                    return {"statusCode": OK,
                            "body": json.dumps({"type": 5, "data": {"flags": 64}}),
                            "headers": {
                                "Content-Type": "application/json"
                            }}
        except socket.timeout:
            # If no response within 3 seconds, return "DEFER"
            return {"statusCode": OK,
                    "body": json.dumps({"type": 5, "data": {"flags": 64}}),
                    "headers": {
                        "Content-Type": "application/json"
                    }}
        finally:
            client.close()

    except Exception as e:
        logger.error("Error in kb2_entrypoint", exc_info=e)
        raise e

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
        if json.loads(event["body"]).get("type") == 1:
            return {"statusCode": OK,
                    "body": json.dumps({"type": 1}),
                    "headers": {
                        "Content-Type": "application/json"
                    }}
        else:
            return process_interact(event)



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
                                              "requestTimeEpoch": datetime.datetime.now().timestamp()*1000
                                          }}, None)

        response.status_code = sl_response['statusCode']
        return json.loads(sl_response['body'])

    uvicorn.run(app, host="0.0.0.0", port=8123)


if __name__ == '__main__':
    server()
