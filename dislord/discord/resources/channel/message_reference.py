from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake, Missing


class MessageReference(BaseModel):
    message_id: Snowflake | Missing
    channel_id: Snowflake | Missing
    guild_id: Snowflake | Missing
    fail_if_not_exists: bool | Missing
