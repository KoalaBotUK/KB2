from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionCallbackType
from dislord.discord.interactions.receiving_and_responding.message_interaction import InteractionType
from dislord.discord.reference import Snowflake, Missing
from dislord.discord.resources.channel.message import Message
from dislord.types import ObjDict


class InteractionCallbackActivityInstanceResource(ObjDict):
    id: str


class InteractionCallbackResource(ObjDict):
    type: InteractionCallbackType
    activity_instance: InteractionCallbackActivityInstanceResource | Missing = None
    message: Message | Missing = None


class InteractionCallback(ObjDict):
    id: Snowflake
    type: InteractionType
    activity_instance_id: str | Missing = None
    response_message_id: Snowflake | Missing = None
    response_message_loading: bool | Missing = None
    response_message_ephemeral: bool | Missing = None


class InteractionCallbackResponse(ObjDict):
    interaction: InteractionCallback
    resource: InteractionCallbackResource
