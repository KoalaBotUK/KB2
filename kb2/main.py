import json

from mangum import Mangum

import dislord
import kb2.env as env
from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionResponse, \
    InteractionCallbackType
from dislord.model.api import HttpOk
from kb2.api import app
from kb2.client import client
from kb2.log import logger
from kb2 import ext

ext.register_all()
logger.info("Starting main.py")


def bot_handler(event, context):  # Not needed if using server
    logger.info(f"\nevent: {event}\ncontext: {context}")
    response = dislord.server.serverless_handler(client, event, context, root_path=env.API_GATEWAY_BASE_PATH)
    logger.info(f"\nresponse: {response}")
    return response


def sync_bot_handler(event, context):
    client.sync_commands()
    client.sync_commands(guild_ids=[g.id for g in client.guilds])
    return {"statusCode": 200}


api_handler = Mangum(app, api_gateway_base_path=env.API_GATEWAY_BASE_PATH)

if __name__ == '__main__':
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)

