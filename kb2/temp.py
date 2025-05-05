import json

from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionResponse, \
    InteractionCallbackType
from dislord.log import logger
from dislord.model.api import HttpOk


def temp_bot_handler(event, context):  # Not needed if using server
    logger.info(f"\nevent: {event}\ncontext: {context}")
    return HttpOk(json.loads(InteractionResponse(type=InteractionCallbackType.DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE).model_dump_json()), headers={"Content-Type": "application/json"}).as_serverless_response()

