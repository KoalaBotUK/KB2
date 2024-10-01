import json
import traceback
from http.client import UNAUTHORIZED, OK

from websockets.sync.client import connect, ClientConnection
from discord_interactions import verify_key

from kb2_entrypoint.env import PUBLIC_KEY
from kb2_entrypoint.log import logger

ws: ClientConnection


def connect_ws():
    global ws
    ws = connect("ws://127.0.0.1:8765/ws")


connect_ws()


def serverless_handler(event, context):
    logger.info(f"\nevent: {event}\ncontext: {context}")

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
            # try:
            #     ws.send(event["body"])
            # except Exception as e:
            #     logger.error(f"Failed to send. Error: {e.__class__.__name__} {e} {traceback.format_exc()}")
            #     connect_ws()
            #     ws.send(event["body"])

            return {"statusCode": OK,
                    "body": json.dumps({"type": 5, "data": {"flags": 64}}),
                    "headers": {
                        "Content-Type": "application/json"
                    }}


def server():
    from fastapi import FastAPI, Request, Response
    import uvicorn

    app = FastAPI()

    @app.post("/deferred-interactions")
    async def deferred_interactions(request: Request, response: Response):
        sl_response = serverless_handler({"httpMethod": request.method,
                                          "body": (await request.body()).decode("utf-8"),
                                          "headers": request.headers}, None)

        response.status_code = sl_response['statusCode']
        return sl_response['body']

    uvicorn.run(app, host="0.0.0.0", port=8123)


if __name__ == '__main__':
    server()
