import json
from http.client import UNAUTHORIZED, OK

from websockets.sync.client import connect
from discord_interactions import verify_key

from .env import PUBLIC_KEY
from .log import logger

ws = connect("ws://localhost:8765/ws")


def serverless_handler(event, context):
    logger.info(f"\nevent: {event}\ncontext: {context}")

    if event['httpMethod'] == "POST":
        raw_headers = event["headers"]
        signature = raw_headers.get('x-signature-ed25519')
        timestamp = raw_headers.get('x-signature-timestamp')
        if signature is None or timestamp is None or not verify_key(event["body"], signature, timestamp, PUBLIC_KEY):
            return {"statusCode": UNAUTHORIZED,
                    "body": "Bad request signature",
                    "headers": None}
        if json.loads(event["body"]).get("type") == 1:
            return {"statusCode": OK,
                    "body": {"type": 1},
                    "headers": None}
        else:
            ws.send(event["body"])

            return {"statusCode": OK,
                    "body": {"type": 5, "data": {"flags": 64}},
                    "headers": None}
