from enum import IntEnum

from dislord.discord.base import BaseModel
from dislord.discord.resources.channel.channel import PartialChannel
from dislord.discord.resources.guild.guild import PartialGuild
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, Missing


class WebhookType(IntEnum):
    INCOMING = 1
    CHANNEL_FOLLOWER = 2
    APPLICATION = 3


class Webhook(BaseModel):
    id: Snowflake
    type: WebhookType
    guild_id: Snowflake | Missing | None
    channel_id: Snowflake | None
    user: User | None
    avatar: str | None
    token: str | Missing
    application_id: Snowflake | None
    source_guild: PartialGuild | Missing
    source_channel: PartialChannel | Missing
    url: str | Missing
