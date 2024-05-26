from dislord.discord.base import BaseModel
from dislord.discord.resources.channel.channel import ChannelType
from dislord.discord.type import Snowflake


class ChannelMention(BaseModel):
    id: Snowflake
    guild_id: Snowflake
    type: ChannelType
    name: str
