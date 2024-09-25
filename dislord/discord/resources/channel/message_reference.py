from dislord.types import ObjDict
from dislord.discord.reference import Snowflake, Missing


class MessageReference(ObjDict):
    message_id: Snowflake | Missing = None
    channel_id: Snowflake | Missing = None
    guild_id: Snowflake | Missing = None
    fail_if_not_exists: bool | Missing = None
