from enum import IntEnum

from dislord.types import ObjDict
from dislord.discord.resources.channel.channel import PartialChannel
from dislord.discord.resources.guild.guild import PartialGuild
from dislord.discord.resources.user.user import User
from dislord.discord.reference import Snowflake, Missing


class WebhookType(IntEnum):
    INCOMING = 1
    CHANNEL_FOLLOWER = 2
    APPLICATION = 3


class Webhook(ObjDict):
    id: Snowflake
    type: WebhookType
    guild_id: Snowflake | Missing | None = None
    channel_id: Snowflake | None = None
    user: User | None = None
    avatar: str | None = None
    token: str | Missing = None
    application_id: Snowflake | None = None
    source_guild: PartialGuild | Missing = None
    source_channel: PartialChannel | Missing = None
    url: str | Missing = None
