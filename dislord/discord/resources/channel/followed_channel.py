from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class FollowedChannel(BaseModel):
    channel_id: Snowflake
    webhook_id: Snowflake
